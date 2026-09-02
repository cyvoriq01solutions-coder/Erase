//! USB controller topology for Advance scan.
//!
//! Report A counted SMBIOS port-connector labels, which on real PCs often
//! miss HDMI and invent USB counts that do not match the plastic connectors.
//! This probe instead walks what Windows actually enumerates: USB controllers,
//! hubs and attached devices, with negotiated speed when firmware reports it.
//!
//! Empty plastic connectors are invisible to this method. Report D therefore
//! prints controller topology separately from "physically verified ports",
//! which stay not-attempted until a technician inserts a device (A7).

use crate::collector_runtime::{
    CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
};
use crate::{CollectorErrorKind, CollectorName};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_STDOUT_BYTES: usize = 64 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROTOCOL_LINES: usize = 768;
const MAX_VALUE_BYTES: usize = 2 * 1024;
const MAX_INDEX: usize = 48;

const USB_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
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

function Speed-FromText([string]$text) {
    if ([string]::IsNullOrWhiteSpace($text)) { return $null }
    if ($text -match 'SuperSpeedPlus|USB 3\.2|USB3\.2') { return 'USB 3.2' }
    if ($text -match 'SuperSpeed|USB 3\.0|USB 3\.1|USB3|USB 3') { return 'USB 3.0' }
    if ($text -match 'USB 2\.0|USB2|High-Speed|USB 2') { return 'USB 2.0' }
    if ($text -match 'USB 1\.1|USB 1\.0|Full-Speed|Low-Speed') { return 'USB 1.x' }
    if ($text -match 'USB30_HUB|USB\\USB30') { return 'USB 3.0' }
    if ($text -match 'USB20_HUB|USB\\USB20') { return 'USB 2.0' }
    return $null
}

try {
    $items = @(Get-CimInstance -ClassName Win32_USBController -Property Name,Manufacturer,Status,DeviceID -ErrorAction Stop)
    Emit-Value 'controller' 0 'source_status' 'reported'
    Emit-Value 'controller' 0 'record_count' $items.Count
    for ($i = 0; $i -lt $items.Count; $i++) {
        $c = $items[$i]
        Emit-Value 'controller' $i 'present' $true
        Emit-Value 'controller' $i 'name' $c.Name
        Emit-Value 'controller' $i 'manufacturer' $c.Manufacturer
        Emit-Value 'controller' $i 'status' $c.Status
        $speed = Speed-FromText ([string]$c.Name)
        if ($speed) { Emit-Value 'controller' $i 'speed' $speed }
    }
} catch { Emit-Status 'controller' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_USBHub -Property Name,Status,DeviceID,Description -ErrorAction Stop)
    Emit-Value 'hub' 0 'source_status' 'reported'
    Emit-Value 'hub' 0 'record_count' $items.Count
    for ($i = 0; $i -lt $items.Count; $i++) {
        $h = $items[$i]
        Emit-Value 'hub' $i 'present' $true
        Emit-Value 'hub' $i 'name' $h.Name
        Emit-Value 'hub' $i 'description' $h.Description
        $speed = Speed-FromText ("$($h.Name) $($h.Description) $($h.DeviceID)")
        if ($speed) { Emit-Value 'hub' $i 'speed' $speed }
        $root = $false
        if ("$($h.Name) $($h.Description)" -match 'Root Hub') { $root = $true }
        Emit-Value 'hub' $i 'root_hub' $root
    }
} catch { Emit-Status 'hub' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_PnPEntity -Filter "PNPClass='USB'" -Property Name,Caption,Manufacturer,PNPDeviceID,HardwareID,CompatibleID,PNPClass,Status -ErrorAction Stop)
    Emit-Value 'device' 0 'source_status' 'reported'
    $index = 0
    foreach ($d in $items) {
        $label = if (-not [string]::IsNullOrWhiteSpace([string]$d.Caption)) { [string]$d.Caption } else { [string]$d.Name }
        if ([string]::IsNullOrWhiteSpace($label)) { continue }
        if ($label -match 'Root Hub|USB Composite Device|Generic USB Hub|USB Hub') { continue }
        $hw = @()
        if ($null -ne $d.HardwareID) { $hw += @($d.HardwareID) }
        if ($null -ne $d.CompatibleID) { $hw += @($d.CompatibleID) }
        $joined = ($hw | ForEach-Object { [string]$_ }) -join ' '
        Emit-Value 'device' $index 'present' $true
        Emit-Value 'device' $index 'occupied' $true
        Emit-Value 'device' $index 'name' $label
        Emit-Value 'device' $index 'manufacturer' $d.Manufacturer
        $speed = Speed-FromText ("$label $joined")
        if ($speed) { Emit-Value 'device' $index 'speed' $speed }
        try {
            $addr = Get-PnpDeviceProperty -InstanceId $d.PNPDeviceID -KeyName 'DEVPKEY_Device_Address' -ErrorAction Stop
            if ($null -ne $addr.Data -and [int]$addr.Data -gt 0) {
                Emit-Value 'device' $index 'port_index' $addr.Data
            }
        } catch {}
        $index++
        if ($index -ge 48) { break }
    }
    Emit-Value 'device' 0 'record_count' $index
} catch { Emit-Status 'device' $_ }
"#,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UsbSource {
    Controller,
    Hub,
    Device,
}

impl UsbSource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Controller => "USB controllers",
            Self::Hub => "USB hubs",
            Self::Device => "Attached USB devices",
        }
    }

    const fn section(self) -> &'static str {
        match self {
            Self::Controller => "controller",
            Self::Hub => "hub",
            Self::Device => "device",
        }
    }

    const ALL: [Self; 3] = [Self::Controller, Self::Hub, Self::Device];
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
    pub source: UsbSource,
    pub outcome: SourceOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbNamed {
    pub name: String,
    pub speed: Option<String>,
    pub root_hub: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbAttached {
    pub name: String,
    pub manufacturer: Option<String>,
    pub speed: Option<String>,
    pub port_index: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbProbe {
    pub controllers: Vec<UsbNamed>,
    pub hubs: Vec<UsbNamed>,
    pub devices: Vec<UsbAttached>,
    pub sources: Vec<SourceStatus>,
    pub probe_error: Option<&'static str>,
}

impl UsbProbe {
    #[must_use]
    pub fn unavailable(reason: &'static str) -> Self {
        Self {
            controllers: Vec::new(),
            hubs: Vec::new(),
            devices: Vec::new(),
            sources: UsbSource::ALL
                .iter()
                .map(|source| SourceStatus {
                    source: *source,
                    outcome: SourceOutcome::NotQueried,
                })
                .collect(),
            probe_error: Some(reason),
        }
    }

    /// True when Windows answered the controller class, which is enough to
    /// score the two topology points in rubric CG-1.0.
    #[must_use]
    pub fn topology_enumerated(&self) -> bool {
        self.probe_error.is_none()
            && self.sources.iter().any(|status| {
                status.source == UsbSource::Controller && status.outcome == SourceOutcome::Reported
            })
    }

    #[must_use]
    pub fn speed_summary(&self) -> Option<String> {
        let mut speeds: BTreeSet<String> = BTreeSet::new();
        for device in &self.devices {
            if let Some(speed) = &device.speed {
                speeds.insert(speed.clone());
            }
        }
        if speeds.is_empty() {
            return None;
        }
        Some(speeds.into_iter().collect::<Vec<_>>().join(", "))
    }

    #[must_use]
    pub fn outcome_for(&self, source: UsbSource) -> SourceOutcome {
        self.sources
            .iter()
            .find(|status| status.source == source)
            .map_or(SourceOutcome::NotQueried, |status| status.outcome)
    }
}

#[must_use]
pub fn collect(cancellation: &CancellationToken) -> UsbProbe {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );

    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        USB_SCRIPT,
        limits,
        cancellation,
    ) {
        Ok(output) => match std::str::from_utf8(output.stdout()) {
            Ok(text) => parse_probe(text),
            Err(_) => UsbProbe::unavailable("The USB probe returned unreadable output."),
        },
        Err(error) => UsbProbe::unavailable(match error.kind {
            CollectorErrorKind::Unsupported => {
                "USB topology collection is only available on Windows."
            }
            CollectorErrorKind::PermissionDenied => {
                "Windows refused the USB topology query on this account."
            }
            CollectorErrorKind::TimedOut => "The USB topology probe exceeded its time limit.",
            CollectorErrorKind::Cancelled => "The USB topology probe was cancelled.",
            CollectorErrorKind::OutputLimitExceeded => {
                "The USB topology probe returned more data than allowed."
            }
            _ => "The USB topology probe could not be completed on this PC.",
        }),
    }
}

#[must_use]
pub fn parse_probe(text: &str) -> UsbProbe {
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

    let sources = UsbSource::ALL
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

    UsbProbe {
        controllers: named_list(&values, "controller"),
        hubs: named_list(&values, "hub"),
        devices: attached_list(&values),
        sources,
        probe_error: None,
    }
}

fn named_list(values: &BTreeMap<(String, usize, String), String>, section: &str) -> Vec<UsbNamed> {
    (0..=MAX_INDEX)
        .filter(|index| flag(values, section, *index, "present"))
        .filter_map(|index| {
            let name = text(values, section, index, "name")?;
            Some(UsbNamed {
                name,
                speed: text(values, section, index, "speed"),
                root_hub: flag(values, section, index, "root_hub"),
            })
        })
        .collect()
}

fn attached_list(values: &BTreeMap<(String, usize, String), String>) -> Vec<UsbAttached> {
    (0..=MAX_INDEX)
        .filter(|index| flag(values, "device", *index, "present"))
        .filter_map(|index| {
            let name = text(values, "device", index, "name")?;
            Some(UsbAttached {
                name,
                manufacturer: text(values, "device", index, "manufacturer"),
                speed: text(values, "device", index, "speed"),
                port_index: number(values, "device", index, "port_index")
                    .and_then(|value| u32::try_from(value).ok()),
            })
        })
        .collect()
}

fn text(
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
    matches!(section, "controller" | "hub" | "device")
        && matches!(
            name,
            "source_status"
                | "record_count"
                | "present"
                | "name"
                | "manufacturer"
                | "status"
                | "description"
                | "speed"
                | "root_hub"
                | "occupied"
                | "port_index"
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

        fn probe(&self) -> UsbProbe {
            parse_probe(&self.0)
        }
    }

    #[test]
    fn controllers_and_a_superspeed_device_are_printed() {
        let probe = Fixture::new()
            .line("controller", 0, "source_status", "reported")
            .line("controller", 0, "present", "True")
            .line(
                "controller",
                0,
                "name",
                "Intel USB 3.0 eXtensible Host Controller",
            )
            .line("hub", 0, "source_status", "reported")
            .line("hub", 0, "present", "True")
            .line("hub", 0, "name", "USB Root Hub (USB 3.0)")
            .line("hub", 0, "root_hub", "True")
            .line("hub", 0, "speed", "USB 3.0")
            .line("device", 0, "source_status", "reported")
            .line("device", 0, "present", "True")
            .line("device", 0, "name", "USB Mass Storage Device")
            .line("device", 0, "speed", "USB 3.0")
            .line("device", 0, "port_index", "3")
            .probe();

        assert!(probe.topology_enumerated());
        assert_eq!(probe.controllers.len(), 1);
        assert!(probe.hubs[0].root_hub);
        assert_eq!(probe.devices[0].port_index, Some(3));
        assert_eq!(probe.speed_summary().as_deref(), Some("USB 3.0"));
    }

    #[test]
    fn an_empty_but_answered_controller_class_still_counts_as_enumerated() {
        let probe = Fixture::new()
            .line("controller", 0, "source_status", "reported")
            .line("controller", 0, "record_count", "0")
            .probe();

        assert!(probe.topology_enumerated());
        assert!(probe.controllers.is_empty());
        assert!(probe.devices.is_empty());
    }

    #[test]
    fn a_refused_query_is_not_treated_as_enumerated() {
        let probe = Fixture::new()
            .line("controller", 0, "source_status", "permission_denied")
            .probe();

        assert!(!probe.topology_enumerated());
        assert_eq!(
            probe.outcome_for(UsbSource::Controller),
            SourceOutcome::PermissionDenied
        );
    }

    #[test]
    fn unavailable_off_windows_claims_nothing() {
        let probe = UsbProbe::unavailable("USB topology collection is only available on Windows.");

        assert!(!probe.topology_enumerated());
        assert!(probe.controllers.is_empty());
        assert_eq!(
            probe.probe_error,
            Some("USB topology collection is only available on Windows.")
        );
    }
}
