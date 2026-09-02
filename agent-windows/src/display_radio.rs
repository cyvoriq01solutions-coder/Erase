//! Display panel identity and radio telemetry for Advance scan.
//!
//! One bounded PowerShell script covers both A5 surfaces so Windows CI does
//! not spawn a sixth collector process beside battery, SMART, USB and capture.
//!
//! Display: `WmiMonitorID` plus the first 128 bytes of the raw EDID block when
//! Windows exposes it. Native width/height come from the EDID preferred timing,
//! never from the current desktop mode. HDR is not guessed.
//!
//! Radios: Wi-Fi, Bluetooth and Ethernet adapters. MAC addresses are never
//! emitted. Link state is printed as Windows reports it.
//!
//! Screen-domain points stay 0 until a technician attests a colour wash (A7).

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
const MAX_INDEX: usize = 8;

const DISPLAY_RADIO_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
    r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$btAdapter = $false

function Emit-Value([string]$section, [int]$index, [string]$name, $value) {
    if ($null -eq $value) { return }
    $text = [Convert]::ToString($value, [Globalization.CultureInfo]::InvariantCulture)
    if ($null -eq $text) { return }
    $text = $text.Trim()
    if ($text.Length -eq 0) { return }
    if ($text -match '(?i)([0-9A-F]{2}[:-]){5}[0-9A-F]{2}') { return }
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

function From-UInt16Chars($values) {
    if ($null -eq $values) { return $null }
    $chars = @()
    foreach ($v in @($values)) {
        if ([int]$v -gt 0) { $chars += [char]([int]$v) }
    }
    if ($chars.Count -eq 0) { return $null }
    return -join $chars
}

try {
    $items = @(Get-CimInstance -Namespace 'root/wmi' -ClassName WmiMonitorID -ErrorAction Stop)
    Emit-Value 'panel' 0 'source_status' 'reported'
    Emit-Value 'panel' 0 'record_count' $items.Count
    for ($i = 0; $i -lt $items.Count; $i++) {
        if ($i -ge 8) { break }
        $p = $items[$i]
        Emit-Value 'panel' $i 'present' $true
        Emit-Value 'panel' $i 'manufacturer' (From-UInt16Chars $p.ManufacturerName)
        Emit-Value 'panel' $i 'name' (From-UInt16Chars $p.UserFriendlyName)
        Emit-Value 'panel' $i 'product_code' (From-UInt16Chars $p.ProductCodeID)
        Emit-Value 'panel' $i 'serial_number' (From-UInt16Chars $p.SerialNumberID)
        if ($p.WeekOfManufacture -and ([int]$p.WeekOfManufacture) -gt 0) { Emit-Value 'panel' $i 'manufacture_week' $p.WeekOfManufacture }
        if ($p.YearOfManufacture -and ([int]$p.YearOfManufacture) -gt 0) { Emit-Value 'panel' $i 'manufacture_year' $p.YearOfManufacture }
        $instance = [string]$p.InstanceName
        if ($instance -match 'DISPLAY\\' -and $instance -notmatch 'Default_Monitor') { Emit-Value 'panel' $i 'internal_panel' $true }
    }
} catch { Emit-Status 'panel' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/wmi' -ClassName WmiMonitorRawEEdidV1Block -ErrorAction Stop)
    Emit-Value 'edid' 0 'source_status' 'reported'
    for ($i = 0; $i -lt $items.Count; $i++) {
        if ($i -ge 8) { break }
        $block = $items[$i].UserData
        if ($null -eq $block -or $block.Length -lt 128) { continue }
        $slice = $block[0..127]
        $hex = -join ($slice | ForEach-Object { $_.ToString('x2') })
        Emit-Value 'edid' $i 'present' $true
        Emit-Value 'edid' $i 'block_hex' $hex
    }
} catch { Emit-Status 'edid' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_VideoController -Property Name,CurrentHorizontalResolution,CurrentVerticalResolution,CurrentRefreshRate,CurrentBitsPerPixel,AdapterCompatibility,PNPDeviceID -ErrorAction Stop)
    Emit-Value 'video' 0 'source_status' 'reported'
    $index = 0
    foreach ($v in $items) {
        $name = [string]$v.Name
        if ([string]::IsNullOrWhiteSpace($name)) { continue }
        if ($name -match 'Basic Display|Remote Desktop|Meta|Virtual') { continue }
        Emit-Value 'video' $index 'present' $true
        Emit-Value 'video' $index 'name' $name
        if ($v.CurrentHorizontalResolution) { Emit-Value 'video' $index 'current_width' $v.CurrentHorizontalResolution }
        if ($v.CurrentVerticalResolution) { Emit-Value 'video' $index 'current_height' $v.CurrentVerticalResolution }
        if ($v.CurrentRefreshRate) { Emit-Value 'video' $index 'refresh_hz' $v.CurrentRefreshRate }
        if ($v.CurrentBitsPerPixel) { Emit-Value 'video' $index 'bit_depth' $v.CurrentBitsPerPixel }
        $index++
        if ($index -ge 8) { break }
    }
    Emit-Value 'video' 0 'record_count' $index
} catch { Emit-Status 'video' $_ }

try {
    $adapters = @(Get-NetAdapter -ErrorAction Stop)
    Emit-Value 'adapter' 0 'source_status' 'reported'
    $wifiIndex = 0
    $ethIndex = 0
    $btAdapter = $false
    foreach ($a in $adapters) {
        $desc = [string]$a.InterfaceDescription
        $name = [string]$a.Name
        $status = [string]$a.Status
        $medium = [string]$a.PhysicalMediaType
        $joined = "$name $desc $medium"
        if ($joined -match 'Wi-?Fi|Wireless|802\.11') {
            if ($wifiIndex -eq 0) { Emit-Value 'wifi' 0 'source_status' 'reported' }
            Emit-Value 'wifi' $wifiIndex 'present' $true
            Emit-Value 'wifi' $wifiIndex 'name' $name
            Emit-Value 'wifi' $wifiIndex 'description' $desc
            Emit-Value 'wifi' $wifiIndex 'state' $status
            if ($a.LinkSpeed) { Emit-Value 'wifi' $wifiIndex 'link_mbps' ([int64]([double]$a.LinkSpeed / 1000000)) }
            $wifiIndex++
        } elseif ($joined -match 'Bluetooth') {
            $btAdapter = $true
        } elseif ($joined -match '802\.3|Ethernet' -or $medium -match '802\.3') {
            if ($ethIndex -eq 0) { Emit-Value 'ethernet' 0 'source_status' 'reported' }
            Emit-Value 'ethernet' $ethIndex 'present' $true
            Emit-Value 'ethernet' $ethIndex 'name' $name
            Emit-Value 'ethernet' $ethIndex 'description' $desc
            Emit-Value 'ethernet' $ethIndex 'state' $status
            if ($a.LinkSpeed) { Emit-Value 'ethernet' $ethIndex 'link_mbps' ([int64]([double]$a.LinkSpeed / 1000000)) }
            $ethIndex++
        }
    }
    if ($wifiIndex -eq 0) { Emit-Value 'wifi' 0 'source_status' 'reported'; Emit-Value 'wifi' 0 'record_count' 0 }
    if ($ethIndex -eq 0) { Emit-Value 'ethernet' 0 'source_status' 'reported'; Emit-Value 'ethernet' 0 'record_count' 0 }
    if ($btAdapter) {
        Emit-Value 'bluetooth' 0 'source_status' 'reported'
        Emit-Value 'bluetooth' 0 'present' $true
        Emit-Value 'bluetooth' 0 'state' 'adapter'
    }
} catch { Emit-Status 'adapter' $_; Emit-Status 'wifi' $_; Emit-Status 'ethernet' $_ }

try {
    $wlan = netsh wlan show interfaces 2>$null | Out-String
    if ($wlan -and $wlan -match 'SSID') {
        if ($wlan -match 'Signal\s*:\s*(\d+)\s*%') { Emit-Value 'wifi' 0 'signal_quality' $Matches[1] }
        if ($wlan -match 'Radio type\s*:\s*(.+)') { Emit-Value 'wifi' 0 'radio_standards' $Matches[1].Trim() }
        if ($wlan -match 'Receive rate.+:\s*([\d.]+)') { Emit-Value 'wifi' 0 'receive_mbps' $Matches[1] }
        if ($wlan -match 'Transmit rate.+:\s*([\d.]+)') { Emit-Value 'wifi' 0 'transmit_mbps' $Matches[1] }
        if ($wlan -match 'State\s*:\s*(.+)') { Emit-Value 'wifi' 0 'state' $Matches[1].Trim() }
    }
} catch {}

try {
    $items = @(Get-PnpDevice -Class Bluetooth -Status OK -ErrorAction Stop)
    if ($items.Count -gt 0) {
        Emit-Value 'bluetooth' 0 'source_status' 'reported'
        Emit-Value 'bluetooth' 0 'present' $true
        $label = [string]$items[0].FriendlyName
        if (-not [string]::IsNullOrWhiteSpace($label)) { Emit-Value 'bluetooth' 0 'name' $label }
        Emit-Value 'bluetooth' 0 'state' 'present'
    } elseif (-not $btAdapter) {
        Emit-Value 'bluetooth' 0 'source_status' 'reported'
        Emit-Value 'bluetooth' 0 'record_count' 0
    }
} catch {
    if (-not $btAdapter) { Emit-Status 'bluetooth' $_ }
}
"#,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DisplayRadioSource {
    Panel,
    Edid,
    Video,
    Wifi,
    Bluetooth,
    Ethernet,
}

impl DisplayRadioSource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Panel => "Monitor identity",
            Self::Edid => "EDID block",
            Self::Video => "Video controller",
            Self::Wifi => "Wi-Fi adapter",
            Self::Bluetooth => "Bluetooth radio",
            Self::Ethernet => "Ethernet adapter",
        }
    }

    const fn section(self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::Edid => "edid",
            Self::Video => "video",
            Self::Wifi => "wifi",
            Self::Bluetooth => "bluetooth",
            Self::Ethernet => "ethernet",
        }
    }

    const ALL: [Self; 6] = [
        Self::Panel,
        Self::Edid,
        Self::Video,
        Self::Wifi,
        Self::Bluetooth,
        Self::Ethernet,
    ];
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
    pub source: DisplayRadioSource,
    pub outcome: SourceOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelReading {
    pub manufacturer: Option<String>,
    pub name: Option<String>,
    pub product_code: Option<String>,
    pub serial_number: Option<String>,
    pub manufacture_week: Option<u32>,
    pub manufacture_year: Option<u32>,
    pub native_width: Option<u32>,
    pub native_height: Option<u32>,
    pub current_width: Option<u32>,
    pub current_height: Option<u32>,
    pub refresh_hz: Option<u32>,
    pub bit_depth: Option<u32>,
    pub internal_panel: bool,
}

impl PanelReading {
    #[must_use]
    pub fn identified(&self) -> bool {
        self.manufacturer.is_some() || self.name.is_some() || self.native_width.is_some()
    }

    #[must_use]
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.manufacturer.clone())
            .unwrap_or_else(|| "Display panel".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiReading {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub signal_quality_percent: Option<u32>,
    pub receive_mbps: Option<u32>,
    pub transmit_mbps: Option<u32>,
    pub link_mbps: Option<u32>,
    pub radio_standards: Option<String>,
}

impl WifiReading {
    #[must_use]
    pub fn present_and_reporting(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.state.is_some()
            || self.signal_quality_percent.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothReading {
    pub name: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthernetReading {
    pub name: Option<String>,
    pub description: Option<String>,
    pub state: Option<String>,
    pub link_mbps: Option<u32>,
}

impl EthernetReading {
    #[must_use]
    pub fn link_state_readable(&self) -> bool {
        self.state.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayRadioProbe {
    pub panels: Vec<PanelReading>,
    pub wifi: Option<WifiReading>,
    pub bluetooth: Option<BluetoothReading>,
    pub ethernet: Vec<EthernetReading>,
    pub sources: Vec<SourceStatus>,
    pub probe_error: Option<&'static str>,
}

impl DisplayRadioProbe {
    #[must_use]
    pub fn unavailable(reason: &'static str) -> Self {
        Self {
            panels: Vec::new(),
            wifi: None,
            bluetooth: None,
            ethernet: Vec::new(),
            sources: DisplayRadioSource::ALL
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
    pub fn outcome_for(&self, source: DisplayRadioSource) -> SourceOutcome {
        self.sources
            .iter()
            .find(|status| status.source == source)
            .map_or(SourceOutcome::NotQueried, |status| status.outcome)
    }

    #[must_use]
    pub fn wifi_reporting(&self) -> bool {
        self.wifi
            .as_ref()
            .is_some_and(WifiReading::present_and_reporting)
    }

    #[must_use]
    pub fn bluetooth_present(&self) -> bool {
        self.bluetooth.is_some()
    }

    #[must_use]
    pub fn ethernet_link_readable(&self) -> bool {
        self.ethernet
            .iter()
            .any(EthernetReading::link_state_readable)
    }

    #[must_use]
    pub fn panel_identified(&self) -> bool {
        self.panels.iter().any(PanelReading::identified)
    }
}

#[must_use]
pub fn collect(cancellation: &CancellationToken) -> DisplayRadioProbe {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );

    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        DISPLAY_RADIO_SCRIPT,
        limits,
        cancellation,
    ) {
        Ok(output) => match std::str::from_utf8(output.stdout()) {
            Ok(text) => parse_probe(text),
            Err(_) => DisplayRadioProbe::unavailable(
                "The display and radio probe returned unreadable output.",
            ),
        },
        Err(error) => DisplayRadioProbe::unavailable(match error.kind {
            CollectorErrorKind::Unsupported => {
                "Display and radio collection is only available on Windows."
            }
            CollectorErrorKind::PermissionDenied => {
                "Windows refused the display and radio query on this account."
            }
            CollectorErrorKind::TimedOut => "The display and radio probe exceeded its time limit.",
            CollectorErrorKind::Cancelled => "The display and radio probe was cancelled.",
            CollectorErrorKind::OutputLimitExceeded => {
                "The display and radio probe returned more data than allowed."
            }
            _ => "The display and radio probe could not be completed on this PC.",
        }),
    }
}

#[must_use]
pub fn parse_probe(text: &str) -> DisplayRadioProbe {
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
        if looks_like_mac(&value) {
            continue;
        }
        values.insert((section.to_string(), index, name.to_string()), value);
    }

    let sources = DisplayRadioSource::ALL
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

    DisplayRadioProbe {
        panels: panel_list(&values),
        wifi: wifi_reading(&values),
        bluetooth: bluetooth_reading(&values),
        ethernet: ethernet_list(&values),
        sources,
        probe_error: None,
    }
}

fn panel_list(values: &BTreeMap<(String, usize, String), String>) -> Vec<PanelReading> {
    let mut panels = Vec::new();
    for index in 0..=MAX_INDEX {
        if !flag(values, "panel", index, "present") && !flag(values, "edid", index, "present") {
            continue;
        }
        let edid = text(values, "edid", index, "block_hex")
            .as_deref()
            .and_then(parse_edid_native);
        panels.push(PanelReading {
            manufacturer: text(values, "panel", index, "manufacturer")
                .or_else(|| edid.as_ref().and_then(|parsed| parsed.manufacturer.clone())),
            name: text(values, "panel", index, "name"),
            product_code: text(values, "panel", index, "product_code"),
            serial_number: text(values, "panel", index, "serial_number"),
            manufacture_week: number_u32(values, "panel", index, "manufacture_week")
                .or_else(|| edid.as_ref().and_then(|parsed| parsed.week)),
            manufacture_year: number_u32(values, "panel", index, "manufacture_year")
                .or_else(|| edid.as_ref().and_then(|parsed| parsed.year)),
            native_width: edid.as_ref().and_then(|parsed| parsed.width),
            native_height: edid.as_ref().and_then(|parsed| parsed.height),
            current_width: number_u32(values, "video", index, "current_width")
                .or_else(|| number_u32(values, "video", 0, "current_width")),
            current_height: number_u32(values, "video", index, "current_height")
                .or_else(|| number_u32(values, "video", 0, "current_height")),
            refresh_hz: number_u32(values, "video", index, "refresh_hz")
                .or_else(|| number_u32(values, "video", 0, "refresh_hz")),
            bit_depth: number_u32(values, "video", index, "bit_depth")
                .or_else(|| number_u32(values, "video", 0, "bit_depth")),
            internal_panel: flag(values, "panel", index, "internal_panel"),
        });
    }
    panels
}

struct EdidTiming {
    manufacturer: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    week: Option<u32>,
    year: Option<u32>,
}

fn parse_edid_native(hex: &str) -> Option<EdidTiming> {
    if hex.len() < 256 {
        return None;
    }
    let mut bytes = [0_u8; 128];
    for (index, chunk) in hex.as_bytes().chunks_exact(2).take(128).enumerate() {
        bytes[index] = (hex_nibble(chunk[0])? << 4) | hex_nibble(chunk[1])?;
    }
    if bytes[0] != 0x00 || bytes[7] != 0x00 {
        return None;
    }
    let manufacturer = edid_manufacturer(&bytes);
    let week = (bytes[16] > 0 && bytes[16] <= 54).then_some(u32::from(bytes[16]));
    let year = (bytes[17] > 0).then_some(1990 + u32::from(bytes[17]));
    let (width, height) = edid_preferred_timing(&bytes);
    Some(EdidTiming {
        manufacturer,
        width,
        height,
        week,
        year,
    })
}

fn edid_manufacturer(bytes: &[u8; 128]) -> Option<String> {
    let packed = (u16::from(bytes[8]) << 8) | u16::from(bytes[9]);
    if packed == 0 {
        return None;
    }
    let c1 = (((packed >> 10) & 0x1F) as u8).wrapping_add(b'@');
    let c2 = (((packed >> 5) & 0x1F) as u8).wrapping_add(b'@');
    let c3 = ((packed & 0x1F) as u8).wrapping_add(b'@');
    if !c1.is_ascii_uppercase() || !c2.is_ascii_uppercase() || !c3.is_ascii_uppercase() {
        return None;
    }
    String::from_utf8(vec![c1, c2, c3]).ok()
}

fn edid_preferred_timing(bytes: &[u8; 128]) -> (Option<u32>, Option<u32>) {
    let dtd = &bytes[54..72];
    if dtd.iter().all(|byte| *byte == 0) {
        return (None, None);
    }
    let width = u32::from(dtd[2]) + (u32::from(dtd[4] >> 4) << 8);
    let height = u32::from(dtd[5]) + (u32::from(dtd[7] >> 4) << 8);
    (
        (width >= 640).then_some(width),
        (height >= 480).then_some(height),
    )
}

fn wifi_reading(values: &BTreeMap<(String, usize, String), String>) -> Option<WifiReading> {
    if !flag(values, "wifi", 0, "present")
        && text(values, "wifi", 0, "state").is_none()
        && number_u32(values, "wifi", 0, "signal_quality").is_none()
    {
        return None;
    }
    Some(WifiReading {
        name: text(values, "wifi", 0, "name"),
        description: text(values, "wifi", 0, "description"),
        state: text(values, "wifi", 0, "state"),
        signal_quality_percent: number_u32(values, "wifi", 0, "signal_quality"),
        receive_mbps: number_u32(values, "wifi", 0, "receive_mbps"),
        transmit_mbps: number_u32(values, "wifi", 0, "transmit_mbps"),
        link_mbps: number_u32(values, "wifi", 0, "link_mbps"),
        radio_standards: text(values, "wifi", 0, "radio_standards"),
    })
}

fn bluetooth_reading(
    values: &BTreeMap<(String, usize, String), String>,
) -> Option<BluetoothReading> {
    if !flag(values, "bluetooth", 0, "present") {
        return None;
    }
    Some(BluetoothReading {
        name: text(values, "bluetooth", 0, "name"),
        state: text(values, "bluetooth", 0, "state"),
    })
}

fn ethernet_list(values: &BTreeMap<(String, usize, String), String>) -> Vec<EthernetReading> {
    (0..=MAX_INDEX)
        .filter(|index| flag(values, "ethernet", *index, "present"))
        .map(|index| EthernetReading {
            name: text(values, "ethernet", index, "name"),
            description: text(values, "ethernet", index, "description"),
            state: text(values, "ethernet", index, "state"),
            link_mbps: number_u32(values, "ethernet", index, "link_mbps"),
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
        .filter(|value| !value.is_empty() && !is_placeholder(value) && !looks_like_mac(value))
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

fn looks_like_mac(value: &str) -> bool {
    let separators = if value.contains(':') {
        ':'
    } else if value.contains('-') {
        '-'
    } else {
        return false;
    };
    let parts: Vec<&str> = value.split(separators).collect();
    parts.len() == 6
        && parts.iter().all(|part| {
            part.len() == 2 && part.chars().all(|character| character.is_ascii_hexdigit())
        })
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
    matches!(
        section,
        "panel" | "edid" | "video" | "adapter" | "wifi" | "bluetooth" | "ethernet"
    ) && matches!(
        name,
        "source_status"
            | "record_count"
            | "present"
            | "manufacturer"
            | "name"
            | "product_code"
            | "serial_number"
            | "manufacture_week"
            | "manufacture_year"
            | "internal_panel"
            | "block_hex"
            | "current_width"
            | "current_height"
            | "refresh_hz"
            | "bit_depth"
            | "description"
            | "state"
            | "signal_quality"
            | "receive_mbps"
            | "transmit_mbps"
            | "link_mbps"
            | "radio_standards"
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

        fn probe(&self) -> DisplayRadioProbe {
            parse_probe(&self.0)
        }
    }

    fn sample_edid_hex() -> String {
        let mut bytes = [0_u8; 128];
        bytes[0] = 0x00;
        bytes[7] = 0x00;
        // LEN = L E N -> 12,5,14
        let packed: u16 = (12 << 10) | (5 << 5) | 14;
        bytes[8] = (packed >> 8) as u8;
        bytes[9] = packed as u8;
        bytes[16] = 12;
        bytes[17] = 34; // 2024
        // 1920x1080 preferred timing
        bytes[54] = 0x02;
        bytes[55] = 0x3A;
        bytes[56] = 0x80; // 1920 low
        bytes[58] = 0x70; // 1920 high nibble
        bytes[59] = 0x38; // 1080 low
        bytes[61] = 0x40; // 1080 high nibble
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn edid_preferred_timing_is_native_not_current_mode() {
        let probe = Fixture::new()
            .line("panel", 0, "source_status", "reported")
            .line("panel", 0, "present", "True")
            .line("panel", 0, "manufacturer", "LEN")
            .line("panel", 0, "name", "LENovo LCD")
            .line("edid", 0, "source_status", "reported")
            .line("edid", 0, "present", "True")
            .line("edid", 0, "block_hex", &sample_edid_hex())
            .line("video", 0, "source_status", "reported")
            .line("video", 0, "present", "True")
            .line("video", 0, "current_width", "1280")
            .line("video", 0, "current_height", "720")
            .line("video", 0, "refresh_hz", "60")
            .probe();

        assert_eq!(probe.panels[0].native_width, Some(1920));
        assert_eq!(probe.panels[0].native_height, Some(1080));
        assert_eq!(probe.panels[0].current_width, Some(1280));
        assert_eq!(probe.panels[0].manufacturer.as_deref(), Some("LEN"));
        assert!(probe.panel_identified());
    }

    #[test]
    fn wifi_bluetooth_and_ethernet_score_inputs_are_separate() {
        let probe = Fixture::new()
            .line("wifi", 0, "source_status", "reported")
            .line("wifi", 0, "present", "True")
            .line("wifi", 0, "name", "Wi-Fi")
            .line("wifi", 0, "state", "Up")
            .line("wifi", 0, "signal_quality", "72")
            .line("bluetooth", 0, "source_status", "reported")
            .line("bluetooth", 0, "present", "True")
            .line("bluetooth", 0, "name", "Intel Wireless Bluetooth")
            .line("ethernet", 0, "source_status", "reported")
            .line("ethernet", 0, "present", "True")
            .line("ethernet", 0, "name", "Ethernet")
            .line("ethernet", 0, "state", "Disconnected")
            .probe();

        assert!(probe.wifi_reporting());
        assert_eq!(
            probe.wifi.as_ref().unwrap().signal_quality_percent,
            Some(72)
        );
        assert!(probe.bluetooth_present());
        assert!(probe.ethernet_link_readable());
    }

    #[test]
    fn a_mac_address_is_dropped_and_never_printed() {
        let probe = Fixture::new()
            .line("wifi", 0, "source_status", "reported")
            .line("wifi", 0, "present", "True")
            .line("wifi", 0, "name", "AA:BB:CC:DD:EE:FF")
            .line("wifi", 0, "state", "Up")
            .probe();

        assert!(probe.wifi_reporting());
        assert!(probe.wifi.as_ref().unwrap().name.is_none());
    }

    #[test]
    fn unavailable_off_windows_claims_nothing() {
        let probe = DisplayRadioProbe::unavailable(
            "Display and radio collection is only available on Windows.",
        );
        assert!(probe.panels.is_empty());
        assert!(!probe.wifi_reporting());
        assert!(!probe.bluetooth_present());
        assert!(!probe.ethernet_link_readable());
    }

    #[test]
    fn the_script_never_emits_mac_fields() {
        let script = DISPLAY_RADIO_SCRIPT.as_str();
        assert!(!script.contains("MacAddress"));
        assert!(script.contains("WmiMonitorID"));
        assert!(script.contains("Get-NetAdapter"));
        assert!(script.contains("netsh wlan show interfaces"));
    }
}
