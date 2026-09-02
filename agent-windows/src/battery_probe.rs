//! Battery and power collection for Advance scan.
//!
//! Report A already asks Windows for a battery through the shared hardware
//! snapshot, and on at least one real laptop that came back empty without
//! saying why. This probe therefore does three things the earlier attempt did
//! not: it queries every source Windows offers, it records *why* each source
//! failed, and it falls back to the firmware battery report only when the
//! management classes produced no capacity.
//!
//! Two transports, both through the existing bounded runtime:
//!   1. `Win32_Battery` plus the `root/WMI` capacity classes.
//!   2. `powercfg /batteryreport /xml`, Windows' own firmware report, used only
//!      when transport 1 yielded no design capacity. It writes one temporary
//!      file, which is deleted and disclosed on Report D.
//!
//! Nothing here estimates. A battery whose firmware withholds design capacity
//! stays unknown, and a percentage is never manufactured from a charge level.

use crate::collector_runtime::{
    CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
};
use crate::{CollectorErrorKind, CollectorName};
use std::collections::BTreeMap;
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_STDOUT_BYTES: usize = 64 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROTOCOL_LINES: usize = 512;
const MAX_VALUE_BYTES: usize = 2 * 1024;
const MAX_BATTERY_INDEX: usize = 8;

/// Windows reports relative units instead of mWh for some packs. When that is
/// the case a capacity number must never be labelled as energy.
const RELATIVE_CAPACITY_NOTE: &str = "Firmware reports this pack in relative units, not milliwatt-hours, so capacities are shown without an energy unit.";

const BATTERY_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
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
        if (($hresult -eq -2147024891) -or ($native -eq 5)) { $status = 'permission_denied' }
        elseif ($errorRecord.Exception -is [Management.Automation.CommandNotFoundException]) { $status = 'unsupported' }
        elseif ("$($errorRecord.Exception.Message)" -match 'not supported|Invalid class|Not found') { $status = 'unsupported' }
    }
    Emit-Value $section 0 'source_status' $status
}

$designSeen = $false

try {
    $items = @(Get-CimInstance -ClassName Win32_Battery -ErrorAction Stop)
    Emit-Value 'battery' 0 'source_status' 'reported'
    Emit-Value 'battery' 0 'record_count' $items.Count
    for ($i = 0; $i -lt $items.Count; $i++) {
        $b = $items[$i]
        Emit-Value 'battery' $i 'present' $true
        Emit-Value 'battery' $i 'name' $b.Name
        Emit-Value 'battery' $i 'manufacturer' $b.Manufacturer
        Emit-Value 'battery' $i 'device_id' $b.DeviceID
        Emit-Value 'battery' $i 'chemistry_code' $b.Chemistry
        Emit-Value 'battery' $i 'battery_status_code' $b.BatteryStatus
        Emit-Value 'battery' $i 'charge_percent' $b.EstimatedChargeRemaining
        Emit-Value 'battery' $i 'design_voltage_mv' $b.DesignVoltage
        if ($b.DesignCapacity -and ([int64]$b.DesignCapacity) -gt 0) {
            Emit-Value 'battery' $i 'designed_capacity' $b.DesignCapacity
            $designSeen = $true
        }
        if ($b.FullChargeCapacity -and ([int64]$b.FullChargeCapacity) -gt 0) {
            Emit-Value 'battery' $i 'full_charge_capacity' $b.FullChargeCapacity
        }
    }
} catch { Emit-Status 'battery' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/WMI' -ClassName BatteryStaticData -ErrorAction Stop)
    Emit-Value 'static' 0 'source_status' 'reported'
    for ($i = 0; $i -lt $items.Count; $i++) {
        $s = $items[$i]
        if ($s.DesignedCapacity -and ([int64]$s.DesignedCapacity) -gt 0) {
            Emit-Value 'static' $i 'designed_capacity' $s.DesignedCapacity
            $designSeen = $true
        }
        Emit-Value 'static' $i 'capabilities' $s.Capabilities
        Emit-Value 'static' $i 'chemistry' $s.Chemistry
        Emit-Value 'static' $i 'manufacture_date' $s.ManufactureDate
        Emit-Value 'static' $i 'serial_number' $s.SerialNumber
        Emit-Value 'static' $i 'device_name' $s.DeviceName
    }
} catch { Emit-Status 'static' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/WMI' -ClassName BatteryFullChargedCapacity -ErrorAction Stop)
    Emit-Value 'full' 0 'source_status' 'reported'
    for ($i = 0; $i -lt $items.Count; $i++) {
        $f = $items[$i]
        if ($f.FullChargedCapacity -and ([int64]$f.FullChargedCapacity) -gt 0) {
            Emit-Value 'full' $i 'full_charge_capacity' $f.FullChargedCapacity
        }
    }
} catch { Emit-Status 'full' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/WMI' -ClassName BatteryCycleCount -ErrorAction Stop)
    Emit-Value 'cycle' 0 'source_status' 'reported'
    for ($i = 0; $i -lt $items.Count; $i++) {
        $c = $items[$i]
        if ($c.CycleCount -and ([int64]$c.CycleCount) -gt 0) {
            Emit-Value 'cycle' $i 'cycle_count' $c.CycleCount
        }
    }
} catch { Emit-Status 'cycle' $_ }

if (-not $designSeen) {
    try {
        $report = Join-Path $env:TEMP 'cyvra-battery-report.xml'
        if (Test-Path $report) { Remove-Item $report -Force -ErrorAction SilentlyContinue }
        $null = & powercfg.exe /batteryreport /xml /output $report 2>$null
        if (Test-Path $report) {
            Emit-Value 'firmware' 0 'temporary_file_written' $true
            [xml]$xml = Get-Content -LiteralPath $report -Raw -ErrorAction Stop
            $packs = @($xml.BatteryReport.Batteries.Battery)
            Emit-Value 'firmware' 0 'source_status' 'reported'
            for ($i = 0; $i -lt $packs.Count; $i++) {
                $p = $packs[$i]
                if ($p.DesignCapacity -and ([int64]$p.DesignCapacity) -gt 0) {
                    Emit-Value 'firmware' $i 'designed_capacity' $p.DesignCapacity
                }
                if ($p.FullChargeCapacity -and ([int64]$p.FullChargeCapacity) -gt 0) {
                    Emit-Value 'firmware' $i 'full_charge_capacity' $p.FullChargeCapacity
                }
                if ($p.CycleCount -and ([int64]$p.CycleCount) -gt 0) {
                    Emit-Value 'firmware' $i 'cycle_count' $p.CycleCount
                }
                Emit-Value 'firmware' $i 'manufacturer' $p.Manufacturer
                Emit-Value 'firmware' $i 'serial_number' $p.SerialNumber
                Emit-Value 'firmware' $i 'chemistry' $p.Chemistry
                Emit-Value 'firmware' $i 'device_name' $p.Id
            }
            Remove-Item $report -Force -ErrorAction SilentlyContinue
            if (-not (Test-Path $report)) {
                Emit-Value 'firmware' 0 'temporary_file_removed' $true
            }
        } else {
            Emit-Value 'firmware' 0 'source_status' 'unsupported'
        }
    } catch { Emit-Status 'firmware' $_ }
}
"#,
);

/// Where a battery value came from, and whether that source answered at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BatterySource {
    ManagementClass,
    StaticData,
    FullChargedCapacity,
    CycleCount,
    FirmwareReport,
}

impl BatterySource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ManagementClass => "Windows battery class",
            Self::StaticData => "Firmware static data",
            Self::FullChargedCapacity => "Firmware full-charge capacity",
            Self::CycleCount => "Firmware cycle count",
            Self::FirmwareReport => "Windows battery report",
        }
    }

    const fn section(self) -> &'static str {
        match self {
            Self::ManagementClass => "battery",
            Self::StaticData => "static",
            Self::FullChargedCapacity => "full",
            Self::CycleCount => "cycle",
            Self::FirmwareReport => "firmware",
        }
    }

    const ALL: [Self; 5] = [
        Self::ManagementClass,
        Self::StaticData,
        Self::FullChargedCapacity,
        Self::CycleCount,
        Self::FirmwareReport,
    ];
}

/// Why a source did or did not answer. Mirrors the inventory vocabulary so the
/// report can explain a gap instead of hiding it.
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
    pub source: BatterySource,
    pub outcome: SourceOutcome,
}

/// One battery pack, as reported. Every field is optional on purpose.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatteryReading {
    pub present: bool,
    pub device_name: Option<String>,
    pub manufacturer: Option<String>,
    pub serial_number: Option<String>,
    pub chemistry: Option<String>,
    pub manufacture_date: Option<String>,
    pub designed_capacity: Option<u64>,
    pub full_charge_capacity: Option<u64>,
    pub cycle_count: Option<u32>,
    pub charge_percent: Option<u32>,
    pub design_voltage_mv: Option<u32>,
    pub status_text: Option<String>,
    /// Capacities are unitless relative values rather than mWh.
    pub relative_capacity: bool,
    pub capacity_source: Option<BatterySource>,
}

impl BatteryReading {
    /// Wear in percent, and only when both capacities are real and sane.
    /// A full-charge capacity above design is clamped to zero wear rather than
    /// printed as a negative number.
    #[must_use]
    pub fn wear_percent(&self) -> Option<f64> {
        let designed = self.designed_capacity?;
        let full = self.full_charge_capacity?;
        if designed == 0 || full == 0 {
            return None;
        }
        let ratio = full as f64 / designed as f64;
        if !(0.05..=1.5).contains(&ratio) {
            return None;
        }
        Some(((1.0 - ratio) * 100.0).max(0.0))
    }

    #[must_use]
    pub fn health_percent(&self) -> Option<f64> {
        self.wear_percent().map(|wear| 100.0 - wear)
    }

    /// Good / Degraded / Critical, from the rubric. `None` when unknown.
    #[must_use]
    pub fn health_band(&self) -> Option<&'static str> {
        let health = self.health_percent()?;
        Some(if health >= 80.0 {
            "Good"
        } else if health >= 50.0 {
            "Degraded"
        } else {
            "Critical"
        })
    }

    #[must_use]
    pub fn capacity_unit(&self) -> &'static str {
        if self.relative_capacity { "" } else { " mWh" }
    }
}

/// Everything Advance scan learned about power, including the misses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryProbe {
    pub readings: Vec<BatteryReading>,
    pub sources: Vec<SourceStatus>,
    pub temporary_file_written: bool,
    pub temporary_file_removed: bool,
    /// Set when the whole probe could not run, for example off Windows.
    pub probe_error: Option<&'static str>,
}

impl BatteryProbe {
    #[must_use]
    pub fn unavailable(reason: &'static str) -> Self {
        Self {
            readings: Vec::new(),
            sources: BatterySource::ALL
                .iter()
                .map(|source| SourceStatus {
                    source: *source,
                    outcome: SourceOutcome::NotQueried,
                })
                .collect(),
            temporary_file_written: false,
            temporary_file_removed: false,
            probe_error: Some(reason),
        }
    }

    /// True when at least one pack produced both capacities, which is the only
    /// state in which wear may be printed.
    #[must_use]
    pub fn has_capacity(&self) -> bool {
        self.readings
            .iter()
            .any(|reading| reading.wear_percent().is_some())
    }

    #[must_use]
    pub fn primary(&self) -> Option<&BatteryReading> {
        self.readings
            .iter()
            .find(|reading| reading.wear_percent().is_some())
            .or_else(|| self.readings.first())
    }

    /// Battery is genuinely absent, as opposed to unreadable: the management
    /// class answered and reported no packs.
    #[must_use]
    pub fn reports_no_battery(&self) -> bool {
        self.readings.is_empty()
            && self.sources.iter().any(|status| {
                status.source == BatterySource::ManagementClass
                    && status.outcome == SourceOutcome::Reported
            })
    }

    #[must_use]
    pub fn outcome_for(&self, source: BatterySource) -> SourceOutcome {
        self.sources
            .iter()
            .find(|status| status.source == source)
            .map_or(SourceOutcome::NotQueried, |status| status.outcome)
    }

    #[must_use]
    pub fn relative_capacity_note(&self) -> Option<&'static str> {
        self.readings
            .iter()
            .any(|reading| reading.relative_capacity)
            .then_some(RELATIVE_CAPACITY_NOTE)
    }
}

/// Run the bounded battery probe. Never panics and never returns a guess.
#[must_use]
pub fn collect(cancellation: &CancellationToken) -> BatteryProbe {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );

    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        BATTERY_SCRIPT,
        limits,
        cancellation,
    ) {
        Ok(output) => match std::str::from_utf8(output.stdout()) {
            Ok(text) => parse_probe(text),
            Err(_) => BatteryProbe::unavailable("The battery probe returned unreadable output."),
        },
        Err(error) => BatteryProbe::unavailable(match error.kind {
            CollectorErrorKind::Unsupported => "Battery collection is only available on Windows.",
            CollectorErrorKind::PermissionDenied => {
                "Windows refused the battery probe on this account."
            }
            CollectorErrorKind::TimedOut => "The battery probe exceeded its time limit.",
            CollectorErrorKind::Cancelled => "The battery probe was cancelled.",
            CollectorErrorKind::OutputLimitExceeded => {
                "The battery probe returned more data than allowed."
            }
            _ => "The battery probe could not be completed on this PC.",
        }),
    }
}

/// Parse the probe protocol. Pure, so the whole decision tree is unit-tested
/// off Windows with fixtures taken from real shapes.
#[must_use]
pub fn parse_probe(text: &str) -> BatteryProbe {
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
        if index > MAX_BATTERY_INDEX {
            continue;
        }
        let Some(value) = decode_hex(encoded) else {
            continue;
        };
        values.insert((section.to_string(), index, name.to_string()), value);
    }

    let sources = BatterySource::ALL
        .iter()
        .map(|source| SourceStatus {
            source: *source,
            outcome: values
                .get(&(source.section().to_string(), 0, "source_status".to_string()))
                .map_or(SourceOutcome::NotQueried, |value| {
                    SourceOutcome::from_wire(value)
                }),
        })
        .collect::<Vec<_>>();

    let readings = build_readings(&values);

    BatteryProbe {
        readings,
        temporary_file_written: flag(&values, "firmware", 0, "temporary_file_written"),
        temporary_file_removed: flag(&values, "firmware", 0, "temporary_file_removed"),
        sources,
        probe_error: None,
    }
}

fn build_readings(values: &BTreeMap<(String, usize, String), String>) -> Vec<BatteryReading> {
    let pack_count = (0..=MAX_BATTERY_INDEX)
        .filter(|index| {
            flag(values, "battery", *index, "present")
                || has_any(values, *index, "designed_capacity")
                || has_any(values, *index, "full_charge_capacity")
        })
        .count();

    (0..pack_count)
        .map(|index| {
            let capabilities = number(values, "static", index, "capabilities").unwrap_or(0);
            let relative_capacity = capabilities & 0x4000_0000 != 0;

            let (designed_capacity, capacity_source) =
                first_number(values, index, "designed_capacity");
            let (full_charge_capacity, _) = first_number(values, index, "full_charge_capacity");
            let (cycle_count, _) = first_number(values, index, "cycle_count");

            BatteryReading {
                present: true,
                device_name: text(values, index, "device_name").or_else(|| {
                    values
                        .get(&("battery".to_string(), index, "name".to_string()))
                        .cloned()
                }),
                manufacturer: text(values, index, "manufacturer"),
                serial_number: text(values, index, "serial_number"),
                chemistry: text(values, index, "chemistry")
                    .or_else(|| chemistry_name(values, index)),
                manufacture_date: text(values, index, "manufacture_date"),
                designed_capacity,
                full_charge_capacity,
                cycle_count: cycle_count.and_then(|value| u32::try_from(value).ok()),
                charge_percent: number(values, "battery", index, "charge_percent")
                    .and_then(|value| u32::try_from(value).ok())
                    .filter(|percent| *percent <= 100),
                design_voltage_mv: number(values, "battery", index, "design_voltage_mv")
                    .and_then(|value| u32::try_from(value).ok()),
                status_text: status_name(values, index),
                relative_capacity,
                capacity_source,
            }
        })
        .collect()
}

/// Capacity search order: the management class first because it is the most
/// widely available, then firmware static data, then Windows' own report.
fn first_number(
    values: &BTreeMap<(String, usize, String), String>,
    index: usize,
    name: &str,
) -> (Option<u64>, Option<BatterySource>) {
    let order = [
        (BatterySource::ManagementClass, "battery"),
        (BatterySource::StaticData, "static"),
        (BatterySource::FullChargedCapacity, "full"),
        (BatterySource::CycleCount, "cycle"),
        (BatterySource::FirmwareReport, "firmware"),
    ];
    for (source, section) in order {
        if let Some(value) = number(values, section, index, name) {
            return (Some(value), Some(source));
        }
    }
    (None, None)
}

fn has_any(values: &BTreeMap<(String, usize, String), String>, index: usize, name: &str) -> bool {
    first_number(values, index, name).0.is_some()
}

fn text(
    values: &BTreeMap<(String, usize, String), String>,
    index: usize,
    name: &str,
) -> Option<String> {
    for section in ["battery", "static", "firmware"] {
        if let Some(value) = values.get(&(section.to_string(), index, name.to_string())) {
            let trimmed = value.trim();
            if !trimmed.is_empty() && !is_placeholder(trimmed) {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn number(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<u64> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .and_then(|value| value.parse::<u64>().ok())
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

fn chemistry_name(
    values: &BTreeMap<(String, usize, String), String>,
    index: usize,
) -> Option<String> {
    let code = number(values, "battery", index, "chemistry_code")?;
    Some(
        match code {
            1 => "Other",
            2 => "Unknown",
            3 => "Lead acid",
            4 => "Nickel cadmium",
            5 => "Nickel metal hydride",
            6 => "Lithium-ion",
            7 => "Zinc air",
            8 => "Lithium polymer",
            _ => return None,
        }
        .to_string(),
    )
}

fn status_name(values: &BTreeMap<(String, usize, String), String>, index: usize) -> Option<String> {
    let code = number(values, "battery", index, "battery_status_code")?;
    Some(
        match code {
            1 => "Discharging",
            2 => "On mains power",
            3 => "Fully charged",
            4 => "Low",
            5 => "Critical",
            6 => "Charging",
            7 => "Charging and high",
            8 => "Charging and low",
            9 => "Charging and critical",
            11 => "Partially charged",
            _ => return None,
        }
        .to_string(),
    )
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
        || compact.chars().all(|character| character == '0')
}

fn is_allowed(section: &str, name: &str) -> bool {
    matches!(
        section,
        "battery" | "static" | "full" | "cycle" | "firmware"
    ) && matches!(
        name,
        "source_status"
            | "record_count"
            | "present"
            | "name"
            | "device_name"
            | "manufacturer"
            | "device_id"
            | "serial_number"
            | "chemistry"
            | "chemistry_code"
            | "manufacture_date"
            | "battery_status_code"
            | "charge_percent"
            | "design_voltage_mv"
            | "designed_capacity"
            | "full_charge_capacity"
            | "cycle_count"
            | "capabilities"
            | "temporary_file_written"
            | "temporary_file_removed"
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
        fn new() -> Self {
            Self(String::new())
        }

        fn line(mut self, section: &str, index: usize, name: &str, value: &str) -> Self {
            self.0
                .push_str(&format!("{section}\t{index}\t{name}\t{}\n", hex(value)));
            self
        }

        fn probe(&self) -> BatteryProbe {
            parse_probe(&self.0)
        }
    }

    #[test]
    fn healthy_pack_reports_real_wear() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("battery", 0, "name", "Primary")
            .line("battery", 0, "chemistry_code", "6")
            .line("battery", 0, "battery_status_code", "2")
            .line("battery", 0, "charge_percent", "97")
            .line("battery", 0, "designed_capacity", "45000")
            .line("battery", 0, "full_charge_capacity", "39150")
            .probe();

        let reading = probe.primary().expect("one pack");
        assert!(probe.has_capacity());
        assert_eq!(reading.chemistry.as_deref(), Some("Lithium-ion"));
        assert_eq!(reading.status_text.as_deref(), Some("On mains power"));
        assert_eq!(reading.charge_percent, Some(97));
        assert!((reading.wear_percent().expect("wear") - 13.0).abs() < 0.01);
        assert!((reading.health_percent().expect("health") - 87.0).abs() < 0.01);
        assert_eq!(reading.health_band(), Some("Good"));
        assert_eq!(reading.capacity_unit(), " mWh");
        assert_eq!(
            reading.capacity_source,
            Some(BatterySource::ManagementClass)
        );
    }

    #[test]
    fn firmware_report_supplies_capacity_when_the_class_will_not() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("static", 0, "source_status", "permission_denied")
            .line("firmware", 0, "source_status", "reported")
            .line("firmware", 0, "temporary_file_written", "True")
            .line("firmware", 0, "temporary_file_removed", "True")
            .line("firmware", 0, "designed_capacity", "45000")
            .line("firmware", 0, "full_charge_capacity", "27000")
            .line("firmware", 0, "cycle_count", "412")
            .probe();

        let reading = probe.primary().expect("one pack");
        assert_eq!(reading.designed_capacity, Some(45_000));
        assert_eq!(reading.cycle_count, Some(412));
        assert_eq!(reading.capacity_source, Some(BatterySource::FirmwareReport));
        assert_eq!(reading.health_band(), Some("Degraded"));
        assert!(probe.temporary_file_written);
        assert!(probe.temporary_file_removed);
        assert_eq!(
            probe.outcome_for(BatterySource::StaticData),
            SourceOutcome::PermissionDenied
        );
    }

    #[test]
    fn a_missing_design_capacity_never_becomes_a_percentage() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("battery", 0, "charge_percent", "64")
            .line("battery", 0, "full_charge_capacity", "39150")
            .probe();

        let reading = probe.primary().expect("one pack");
        assert_eq!(reading.designed_capacity, None);
        assert_eq!(reading.wear_percent(), None);
        assert_eq!(reading.health_percent(), None);
        assert_eq!(reading.health_band(), None);
        assert!(!probe.has_capacity());
        // A charge level is not health, and must never be reused as one.
        assert_eq!(reading.charge_percent, Some(64));
    }

    #[test]
    fn a_desktop_is_reported_as_having_no_battery_not_as_unreadable() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "record_count", "0")
            .probe();

        assert!(probe.readings.is_empty());
        assert!(probe.reports_no_battery());
        assert_eq!(
            probe.outcome_for(BatterySource::ManagementClass),
            SourceOutcome::Reported
        );
    }

    #[test]
    fn a_refused_query_is_distinguished_from_an_absent_battery() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "permission_denied")
            .probe();

        assert!(probe.readings.is_empty());
        assert!(!probe.reports_no_battery());
        assert_eq!(
            probe.outcome_for(BatterySource::ManagementClass),
            SourceOutcome::PermissionDenied
        );
        assert_eq!(
            probe
                .outcome_for(BatterySource::ManagementClass)
                .customer_label(),
            "Refused without administrator rights"
        );
    }

    #[test]
    fn relative_capacity_packs_do_not_claim_milliwatt_hours() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("battery", 0, "designed_capacity", "100")
            .line("battery", 0, "full_charge_capacity", "88")
            .line("static", 0, "source_status", "reported")
            .line("static", 0, "capabilities", "1073741824")
            .probe();

        let reading = probe.primary().expect("one pack");
        assert!(reading.relative_capacity);
        assert_eq!(reading.capacity_unit(), "");
        assert_eq!(reading.health_band(), Some("Good"));
        assert!(probe.relative_capacity_note().is_some());
    }

    #[test]
    fn an_impossible_capacity_ratio_is_refused_rather_than_printed() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("battery", 0, "designed_capacity", "1000")
            .line("battery", 0, "full_charge_capacity", "9000000")
            .probe();

        assert_eq!(probe.primary().expect("pack").wear_percent(), None);
    }

    #[test]
    fn a_slightly_over_design_capacity_reads_as_zero_wear_not_negative() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("battery", 0, "designed_capacity", "45000")
            .line("battery", 0, "full_charge_capacity", "45900")
            .probe();

        let reading = probe.primary().expect("pack");
        assert_eq!(reading.wear_percent(), Some(0.0));
        assert_eq!(reading.health_band(), Some("Good"));
    }

    #[test]
    fn placeholder_identifiers_are_dropped() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("static", 0, "serial_number", "0000000")
            .line("static", 0, "manufacturer", "To Be Filled By O.E.M.")
            .probe();

        let reading = probe.primary().expect("pack");
        assert_eq!(reading.serial_number, None);
        assert_eq!(reading.manufacturer, None);
    }

    #[test]
    fn malformed_and_unknown_protocol_lines_are_ignored_safely() {
        let probe = Fixture::new()
            .line("battery", 0, "source_status", "reported")
            .line("battery", 0, "present", "True")
            .line("battery", 0, "designed_capacity", "45000")
            .line("battery", 0, "full_charge_capacity", "36000")
            .probe();
        let mut text = String::new();
        text.push_str("battery\t0\tsource_status\t");
        text.push_str(&hex("reported"));
        text.push('\n');
        text.push_str("battery\t0\tpresent\tzz\n");
        text.push_str("battery\tNOPE\tpresent\t54727565\n");
        text.push_str("evil\t0\tdesigned_capacity\t3435303030\n");
        text.push_str("battery\t0\tsecret_field\t3435303030\n");

        let hostile = parse_probe(&text);
        assert!(hostile.readings.is_empty());
        assert!(probe.has_capacity());
    }

    #[test]
    fn an_unavailable_probe_states_the_reason_and_claims_nothing() {
        let probe = BatteryProbe::unavailable("Battery collection is only available on Windows.");

        assert!(probe.readings.is_empty());
        assert!(!probe.has_capacity());
        assert!(!probe.reports_no_battery());
        assert_eq!(
            probe.probe_error,
            Some("Battery collection is only available on Windows.")
        );
        for source in BatterySource::ALL {
            assert_eq!(probe.outcome_for(source), SourceOutcome::NotQueried);
        }
    }
}
