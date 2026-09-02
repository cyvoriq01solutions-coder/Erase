//! Processor and memory identity for Advance scan.
//!
//! One bounded PowerShell script reads Win32_Processor, Win32_PhysicalMemory
//! and Win32_OperatingSystem. Active benchmarks live in `advance_bench` and
//! run in-process only after the operator consents, so default Advance scan
//! tests do not spawn a seventh collector process.

use crate::collector_runtime::{
    CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
};
use crate::{CollectorErrorKind, CollectorName};
use std::collections::BTreeMap;
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_STDOUT_BYTES: usize = 32 * 1024;
const MAX_STDERR_BYTES: usize = 8 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROTOCOL_LINES: usize = 256;
const MAX_VALUE_BYTES: usize = 1024;
const MAX_INDEX: usize = 8;

const IDENTITY_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
    r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Emit-Value([string]$section, [int]$index, [string]$name, $value) {
    if ($null -eq $value) { return }
    $text = [Convert]::ToString($value, [Globalization.CultureInfo]::InvariantCulture)
    if ($null -eq $text) { return }
    $text = $text.Trim()
    if ($text.Length -eq 0) { return }
    $bytes = [Text.Encoding]::UTF8.GetBytes($text)
    $hex = -join ($bytes | ForEach-Object { $_.ToString('x2') })
    [Console]::Out.WriteLine("$section`t$index`t$name`t$hex")
}

function Emit-Status([string]$section, $errorRecord) {
    $status = 'collection_error'
    if ($null -ne $errorRecord) {
        $hresult = $errorRecord.Exception.HResult
        $native = $errorRecord.Exception.NativeErrorCode
        if (($hresult -eq -2147024891) -or ($native -eq 5) -or ("$($errorRecord.Exception.Message)" -match 'Access is denied|UnauthorizedAccess')) { $status = 'permission_denied' }
        elseif ($errorRecord.Exception -is [Management.Automation.CommandNotFoundException]) { $status = 'unsupported' }
        elseif ("$($errorRecord.Exception.Message)" -match 'not supported|Invalid class|Not found') { $status = 'unsupported' }
    }
    Emit-Value $section 0 'source_status' $status
}

try {
    $cpu = Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'cpu' 0 'source_status' 'reported'
    Emit-Value 'cpu' 0 'present' $true
    Emit-Value 'cpu' 0 'name' $cpu.Name
    Emit-Value 'cpu' 0 'manufacturer' $cpu.Manufacturer
    if ($cpu.NumberOfCores) { Emit-Value 'cpu' 0 'cores' $cpu.NumberOfCores }
    if ($cpu.NumberOfLogicalProcessors) { Emit-Value 'cpu' 0 'logical' $cpu.NumberOfLogicalProcessors }
    if ($cpu.MaxClockSpeed) { Emit-Value 'cpu' 0 'max_mhz' $cpu.MaxClockSpeed }
    if ($cpu.CurrentClockSpeed) { Emit-Value 'cpu' 0 'current_mhz' $cpu.CurrentClockSpeed }
    if ($cpu.L2CacheSize) { Emit-Value 'cpu' 0 'l2_kb' $cpu.L2CacheSize }
    if ($cpu.L3CacheSize) { Emit-Value 'cpu' 0 'l3_kb' $cpu.L3CacheSize }
} catch { Emit-Status 'cpu' $_ }

try {
    $os = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop
    Emit-Value 'memory' 0 'source_status' 'reported'
    if ($os.TotalVisibleMemorySize) { Emit-Value 'memory' 0 'total_kb' $os.TotalVisibleMemorySize }
    if ($os.FreePhysicalMemory) { Emit-Value 'memory' 0 'available_kb' $os.FreePhysicalMemory }
} catch { Emit-Status 'memory' $_ }

try {
    $modules = @(Get-CimInstance -ClassName Win32_PhysicalMemory -ErrorAction Stop)
    Emit-Value 'module' 0 'source_status' 'reported'
    Emit-Value 'module' 0 'record_count' $modules.Count
    for ($i = 0; $i -lt $modules.Count; $i++) {
        if ($i -ge 8) { break }
        $m = $modules[$i]
        Emit-Value 'module' $i 'present' $true
        Emit-Value 'module' $i 'locator' $m.DeviceLocator
        if ($m.Capacity) { Emit-Value 'module' $i 'capacity_bytes' $m.Capacity }
        if ($m.Speed) { Emit-Value 'module' $i 'speed_mhz' $m.Speed }
        if ($m.ConfiguredClockSpeed) { Emit-Value 'module' $i 'configured_mhz' $m.ConfiguredClockSpeed }
        Emit-Value 'module' $i 'part_number' $m.PartNumber
    }
} catch { Emit-Status 'module' $_ }
"#,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdentitySource {
    Processor,
    MemorySummary,
    MemoryModule,
}

impl IdentitySource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Processor => "Processor class",
            Self::MemorySummary => "Operating-system memory",
            Self::MemoryModule => "Physical memory modules",
        }
    }

    const fn section(self) -> &'static str {
        match self {
            Self::Processor => "cpu",
            Self::MemorySummary => "memory",
            Self::MemoryModule => "module",
        }
    }

    const ALL: [Self; 3] = [Self::Processor, Self::MemorySummary, Self::MemoryModule];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceOutcome {
    Reported,
    NotQueried,
    PermissionDenied,
    Unsupported,
    CollectionError,
}

impl SourceOutcome {
    #[must_use]
    pub const fn customer_label(self) -> &'static str {
        match self {
            Self::Reported => "Answered",
            Self::NotQueried => "Not queried on this PC",
            Self::PermissionDenied => "Refused without administrator rights",
            Self::Unsupported => "Not available on this PC",
            Self::CollectionError => "Windows returned an error",
        }
    }

    const fn from_wire(value: &str) -> Self {
        match value.as_bytes() {
            b"reported" => Self::Reported,
            b"permission_denied" => Self::PermissionDenied,
            b"unsupported" => Self::Unsupported,
            _ => Self::CollectionError,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceStatus {
    pub source: IdentitySource,
    pub outcome: SourceOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorIdentity {
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub cores: Option<u32>,
    pub logical_processors: Option<u32>,
    pub max_mhz: Option<u32>,
    pub current_mhz: Option<u32>,
    pub l2_kb: Option<u32>,
    pub l3_kb: Option<u32>,
}

impl ProcessorIdentity {
    #[must_use]
    pub fn identity_complete(&self) -> bool {
        self.name.is_some()
            && self.cores.is_some()
            && (self.l2_kb.is_some() || self.l3_kb.is_some())
    }

    #[must_use]
    pub fn cache_summary(&self) -> Option<String> {
        match (self.l2_kb, self.l3_kb) {
            (Some(l2), Some(l3)) => Some(format!("L2 {l2} KB, L3 {l3} KB")),
            (Some(l2), None) => Some(format!("L2 {l2} KB")),
            (None, Some(l3)) => Some(format!("L3 {l3} KB")),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryModuleReading {
    pub locator: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub speed_mhz: Option<u32>,
    pub part_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuMemoryProbe {
    pub processor: Option<ProcessorIdentity>,
    pub modules: Vec<MemoryModuleReading>,
    pub installed_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub sources: Vec<SourceStatus>,
    pub probe_error: Option<&'static str>,
}

impl CpuMemoryProbe {
    #[must_use]
    pub fn unavailable(reason: &'static str) -> Self {
        Self {
            processor: None,
            modules: Vec::new(),
            installed_bytes: None,
            available_bytes: None,
            sources: IdentitySource::ALL
                .iter()
                .map(|source| SourceStatus {
                    source: *source,
                    outcome: SourceOutcome::NotQueried,
                })
                .collect(),
            probe_error: Some(reason),
        }
    }

    #[must_use]
    pub fn inventory_complete(&self) -> bool {
        let has_capacity = self.installed_bytes.is_some()
            || self
                .modules
                .iter()
                .any(|module| module.capacity_bytes.is_some());
        let has_speed = self.modules.iter().any(|module| module.speed_mhz.is_some());
        let has_slots = !self.modules.is_empty() || self.installed_bytes.is_some();
        has_slots && has_capacity && has_speed
    }
}

#[must_use]
pub fn collect(cancellation: &CancellationToken) -> CpuMemoryProbe {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );

    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        IDENTITY_SCRIPT,
        limits,
        cancellation,
    ) {
        Ok(output) => match std::str::from_utf8(output.stdout()) {
            Ok(text) => parse_probe(text),
            Err(_) => CpuMemoryProbe::unavailable(
                "The processor and memory identity probe returned unreadable output.",
            ),
        },
        Err(error) => CpuMemoryProbe::unavailable(match error.kind {
            CollectorErrorKind::Unsupported => {
                "Processor and memory identity collection is only available on Windows."
            }
            CollectorErrorKind::PermissionDenied => {
                "Windows refused the processor and memory query on this account."
            }
            CollectorErrorKind::TimedOut => {
                "The processor and memory identity probe exceeded its time limit."
            }
            CollectorErrorKind::Cancelled => {
                "The processor and memory identity probe was cancelled."
            }
            CollectorErrorKind::OutputLimitExceeded => {
                "The processor and memory identity probe returned more data than allowed."
            }
            _ => "The processor and memory identity probe could not be completed on this PC.",
        }),
    }
}

#[must_use]
pub fn parse_probe(text: &str) -> CpuMemoryProbe {
    let mut values: BTreeMap<(String, usize, String), String> = BTreeMap::new();

    for (line_number, line) in text.lines().enumerate() {
        if line_number >= MAX_PROTOCOL_LINES || line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let (Some(section), Some(index), Some(name), Some(encoded)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if parts.next().is_some() || !is_allowed(section, name) {
            continue;
        }
        let Ok(index) = index.parse::<usize>() else {
            continue;
        };
        if index > MAX_INDEX {
            continue;
        }
        let Some(value) = decode_hex(encoded) else {
            continue;
        };
        values.insert((section.to_string(), index, name.to_string()), value);
    }

    let sources = IdentitySource::ALL
        .iter()
        .map(|source| SourceStatus {
            source: *source,
            outcome: values
                .get(&(source.section().to_string(), 0, "source_status".to_string()))
                .map_or(SourceOutcome::NotQueried, |value| {
                    SourceOutcome::from_wire(value)
                }),
        })
        .collect();

    let processor = flag(&values, "cpu", 0, "present").then(|| ProcessorIdentity {
        name: text_field(&values, "cpu", 0, "name"),
        manufacturer: text_field(&values, "cpu", 0, "manufacturer"),
        cores: number_u32(&values, "cpu", 0, "cores"),
        logical_processors: number_u32(&values, "cpu", 0, "logical"),
        max_mhz: number_u32(&values, "cpu", 0, "max_mhz"),
        current_mhz: number_u32(&values, "cpu", 0, "current_mhz"),
        l2_kb: number_u32(&values, "cpu", 0, "l2_kb"),
        l3_kb: number_u32(&values, "cpu", 0, "l3_kb"),
    });

    CpuMemoryProbe {
        processor,
        modules: module_list(&values),
        installed_bytes: number_u64(&values, "memory", 0, "total_kb")
            .map(|kb| kb.saturating_mul(1024)),
        available_bytes: number_u64(&values, "memory", 0, "available_kb")
            .map(|kb| kb.saturating_mul(1024)),
        sources,
        probe_error: None,
    }
}

fn module_list(values: &BTreeMap<(String, usize, String), String>) -> Vec<MemoryModuleReading> {
    (0..=MAX_INDEX)
        .filter(|index| flag(values, "module", *index, "present"))
        .map(|index| MemoryModuleReading {
            locator: text_field(values, "module", index, "locator"),
            capacity_bytes: number_u64(values, "module", index, "capacity_bytes"),
            speed_mhz: number_u32(values, "module", index, "speed_mhz")
                .or_else(|| number_u32(values, "module", index, "configured_mhz")),
            part_number: text_field(values, "module", index, "part_number"),
        })
        .collect()
}

fn text_field(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<String> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && !is_placeholder(value))
}

fn number_u32(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<u32> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value as u32)
        .filter(|value| *value > 0)
}

fn number_u64(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<u64> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value as u64)
        .filter(|value| *value > 0)
}

fn flag(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> bool {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn is_placeholder(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    compact.is_empty()
        || matches!(
            compact.as_str(),
            "unknown" | "none" | "na" | "notavailable" | "tobefilledbyoem" | "defaultstring"
        )
}

fn is_allowed(section: &str, name: &str) -> bool {
    matches!(section, "cpu" | "memory" | "module")
        && matches!(
            name,
            "source_status"
                | "record_count"
                | "present"
                | "name"
                | "manufacturer"
                | "cores"
                | "logical"
                | "max_mhz"
                | "current_mhz"
                | "l2_kb"
                | "l3_kb"
                | "total_kb"
                | "available_kb"
                | "locator"
                | "capacity_bytes"
                | "speed_mhz"
                | "configured_mhz"
                | "part_number"
        )
}

fn decode_hex(encoded: &str) -> Option<String> {
    if !encoded.len().is_multiple_of(2) || encoded.len() / 2 > MAX_VALUE_BYTES {
        return None;
    }
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        decoded.push((high << 4) | low);
    }
    let value = String::from_utf8(decoded).ok()?;
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    struct Fixture(String);

    impl Fixture {
        fn line(mut self, section: &str, index: usize, name: &str, value: &str) -> Self {
            self.0
                .push_str(&format!("{section}\t{index}\t{name}\t{}\n", hex(value)));
            self
        }

        fn probe(&self) -> CpuMemoryProbe {
            parse_probe(&self.0)
        }
    }

    impl Fixture {
        fn new() -> Self {
            Self(String::new())
        }
    }

    #[test]
    fn identity_is_complete_with_model_cores_and_cache() {
        let probe = Fixture::new()
            .line("cpu", 0, "source_status", "reported")
            .line("cpu", 0, "present", "True")
            .line("cpu", 0, "name", "Intel Core i7-6500U")
            .line("cpu", 0, "cores", "2")
            .line("cpu", 0, "l3_kb", "4096")
            .line("cpu", 0, "max_mhz", "2500")
            .line("memory", 0, "source_status", "reported")
            .line("memory", 0, "total_kb", "8388608")
            .line("module", 0, "source_status", "reported")
            .line("module", 0, "present", "True")
            .line("module", 0, "locator", "ChannelA-DIMM0")
            .line("module", 0, "capacity_bytes", "4294967296")
            .line("module", 0, "speed_mhz", "2133")
            .probe();

        assert!(probe.processor.as_ref().unwrap().identity_complete());
        assert!(probe.inventory_complete());
        assert_eq!(probe.installed_bytes, Some(8_388_608 * 1024));
    }

    #[test]
    fn missing_cache_is_not_a_complete_identity() {
        let probe = Fixture::new()
            .line("cpu", 0, "source_status", "reported")
            .line("cpu", 0, "present", "True")
            .line("cpu", 0, "name", "Intel")
            .line("cpu", 0, "cores", "2")
            .probe();
        assert!(!probe.processor.as_ref().unwrap().identity_complete());
    }

    #[test]
    fn the_script_is_identity_only() {
        let script = IDENTITY_SCRIPT.as_str();
        assert!(script.contains("Win32_Processor"));
        assert!(script.contains("Win32_PhysicalMemory"));
        assert!(!script.contains("Format-Volume"));
        assert!(!script.contains("Clear-Disk"));
    }
}
