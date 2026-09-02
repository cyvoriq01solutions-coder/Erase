//! Camera and microphone enumeration for Advance scan.
//!
//! Report A asked a single Camera ClassGuid and an AudioEndpoint filter, and
//! on at least one laptop that printed "None enumerated by Windows" even though
//! a UVC webcam was present. This probe therefore unions several PnP classes
//! Windows actually uses for capture devices, records *which* class answered,
//! and still captures nothing: no frame, no sample, no file.
//!
//! Live preview and microphone record/playback stay in a later consented
//! slice (A10). A3 only lists what is plugged in.

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
const MAX_DEVICE_INDEX: usize = 16;

const CAPTURE_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
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

function Is-CameraName([string]$label) {
    if ([string]::IsNullOrWhiteSpace($label)) { return $false }
    if ($label -match 'Scanner|WIA|Print|Fax|Still Capture') { return $false }
    return ($label -match 'Camera|Webcam|UVC|Imaging Device|Integrated Camera|IR Camera|RGB Camera')
}

function Is-MicrophoneName([string]$label) {
    if ([string]::IsNullOrWhiteSpace($label)) { return $false }
    if ($label -match 'Speaker|Headphone|HDMI|Display Audio|Stereo Mix') { return $false }
    return ($label -match 'Microphone|Mic Array|\bMic\b|Headset')
}

$cameraSeen = @{}
$cameraIndex = 0
function Add-Camera($item, [string]$via) {
    $id = [string]$item.PNPDeviceID
    if ([string]::IsNullOrWhiteSpace($id)) { $id = [string]$item.DeviceID }
    if ([string]::IsNullOrWhiteSpace($id)) { $id = [string]$item.Name }
    if ([string]::IsNullOrWhiteSpace($id) -or $cameraSeen.ContainsKey($id)) { return }
    $label = if (-not [string]::IsNullOrWhiteSpace([string]$item.Caption)) { [string]$item.Caption } else { [string]$item.Name }
    if (-not (Is-CameraName $label) -and $via -eq 'media_class') { return }
    if ($label -match 'Scanner|WIA|Print') { return }
    $script:cameraSeen[$id] = $true
    Emit-Value 'camera' $script:cameraIndex 'present' $true
    Emit-Value 'camera' $script:cameraIndex 'name' $label
    Emit-Value 'camera' $script:cameraIndex 'manufacturer' $item.Manufacturer
    Emit-Value 'camera' $script:cameraIndex 'enumerated_by' $via
    $script:cameraIndex++
}

$micSeen = @{}
$micIndex = 0
function Add-Microphone($item, [string]$via) {
    $id = [string]$item.PNPDeviceID
    if ([string]::IsNullOrWhiteSpace($id)) { $id = [string]$item.DeviceID }
    if ([string]::IsNullOrWhiteSpace($id)) { $id = [string]$item.Name }
    if ([string]::IsNullOrWhiteSpace($id) -or $micSeen.ContainsKey($id)) { return }
    $label = if (-not [string]::IsNullOrWhiteSpace([string]$item.Caption)) { [string]$item.Caption } else { [string]$item.Name }
    if (-not (Is-MicrophoneName $label)) { return }
    $script:micSeen[$id] = $true
    Emit-Value 'microphone' $script:micIndex 'present' $true
    Emit-Value 'microphone' $script:micIndex 'name' $label
    Emit-Value 'microphone' $script:micIndex 'manufacturer' $item.Manufacturer
    Emit-Value 'microphone' $script:micIndex 'enumerated_by' $via
    $script:micIndex++
}

try {
    $guid = '{ca3e7ab9-b4c3-4ae6-8251-579689032b24}'
    $items = @(Get-CimInstance -ClassName Win32_PnPEntity -Filter "ClassGuid='$guid'" -Property Name,Caption,Manufacturer,PNPDeviceID -ErrorAction Stop)
    Emit-Value 'camera_class' 0 'source_status' 'reported'
    foreach ($item in $items) { Add-Camera $item 'PnP Camera class' }
} catch { Emit-Status 'camera_class' $_ }

try {
    $guid = '{6bdd1fc6-810f-11d0-bec7-08002be2092f}'
    $items = @(Get-CimInstance -ClassName Win32_PnPEntity -Filter "ClassGuid='$guid'" -Property Name,Caption,Manufacturer,PNPDeviceID -ErrorAction Stop)
    Emit-Value 'image_class' 0 'source_status' 'reported'
    foreach ($item in $items) { Add-Camera $item 'PnP Image class' }
} catch { Emit-Status 'image_class' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_PnPEntity -Filter "Service='usbvideo'" -Property Name,Caption,Manufacturer,PNPDeviceID,Service -ErrorAction Stop)
    Emit-Value 'usbvideo' 0 'source_status' 'reported'
    foreach ($item in $items) { Add-Camera $item 'USB video service' }
} catch { Emit-Status 'usbvideo' $_ }

try {
    $guid = '{4d36e96c-e325-11ce-bfc1-08002be10318}'
    $items = @(Get-CimInstance -ClassName Win32_PnPEntity -Filter "ClassGuid='$guid'" -Property Name,Caption,Manufacturer,PNPDeviceID -ErrorAction Stop)
    Emit-Value 'media_class' 0 'source_status' 'reported'
    foreach ($item in $items) { Add-Camera $item 'media_class' }
} catch { Emit-Status 'media_class' $_ }

if ($cameraIndex -eq 0) {
    Emit-Value 'camera' 0 'none_enumerated' $true
}

try {
    $guid = '{c166523c-fe0c-4a94-a586-f1a80cfbbf32}'
    $items = @(Get-CimInstance -ClassName Win32_PnPEntity -Filter "ClassGuid='$guid'" -Property Name,Caption,Manufacturer,PNPDeviceID -ErrorAction Stop)
    Emit-Value 'audio_endpoint' 0 'source_status' 'reported'
    foreach ($item in $items) { Add-Microphone $item 'Audio endpoint class' }
} catch { Emit-Status 'audio_endpoint' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_SoundDevice -Property Name,Manufacturer,Status,ProductName -ErrorAction Stop)
    Emit-Value 'sound_device' 0 'source_status' 'reported'
    foreach ($item in $items) { Add-Microphone $item 'Windows sound device' }
} catch { Emit-Status 'sound_device' $_ }

if ($micIndex -eq 0) {
    Emit-Value 'microphone' 0 'none_enumerated' $true
}

Emit-Value 'capture' 0 'frames_captured' $false
Emit-Value 'capture' 0 'audio_recorded' $false
"#,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CaptureSource {
    CameraClass,
    ImageClass,
    UsbVideo,
    MediaClass,
    AudioEndpoint,
    SoundDevice,
}

impl CaptureSource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::CameraClass => "PnP Camera class",
            Self::ImageClass => "PnP Image class",
            Self::UsbVideo => "USB video service",
            Self::MediaClass => "PnP Media class",
            Self::AudioEndpoint => "Audio endpoint class",
            Self::SoundDevice => "Windows sound device",
        }
    }

    const fn section(self) -> &'static str {
        match self {
            Self::CameraClass => "camera_class",
            Self::ImageClass => "image_class",
            Self::UsbVideo => "usbvideo",
            Self::MediaClass => "media_class",
            Self::AudioEndpoint => "audio_endpoint",
            Self::SoundDevice => "sound_device",
        }
    }

    const ALL: [Self; 6] = [
        Self::CameraClass,
        Self::ImageClass,
        Self::UsbVideo,
        Self::MediaClass,
        Self::AudioEndpoint,
        Self::SoundDevice,
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
    pub source: CaptureSource,
    pub outcome: SourceOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureDevice {
    pub kind: &'static str,
    pub present: bool,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub enumerated_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureProbe {
    pub cameras: Vec<CaptureDevice>,
    pub microphones: Vec<CaptureDevice>,
    pub sources: Vec<SourceStatus>,
    pub frames_captured: bool,
    pub audio_recorded: bool,
    pub probe_error: Option<&'static str>,
}

impl CaptureProbe {
    #[must_use]
    pub fn unavailable(reason: &'static str) -> Self {
        Self {
            cameras: Vec::new(),
            microphones: Vec::new(),
            sources: CaptureSource::ALL
                .iter()
                .map(|source| SourceStatus {
                    source: *source,
                    outcome: SourceOutcome::NotQueried,
                })
                .collect(),
            frames_captured: false,
            audio_recorded: false,
            probe_error: Some(reason),
        }
    }

    #[must_use]
    pub fn camera_names(&self) -> Vec<String> {
        self.cameras
            .iter()
            .filter(|device| device.present)
            .filter_map(|device| device.name.clone())
            .collect()
    }

    #[must_use]
    pub fn microphone_names(&self) -> Vec<String> {
        self.microphones
            .iter()
            .filter(|device| device.present)
            .filter_map(|device| device.name.clone())
            .collect()
    }

    #[must_use]
    pub fn reports_no_cameras(&self) -> bool {
        self.cameras.is_empty()
            && self.probe_error.is_none()
            && self.sources.iter().any(|status| {
                matches!(
                    status.source,
                    CaptureSource::CameraClass | CaptureSource::UsbVideo
                ) && status.outcome == SourceOutcome::Reported
            })
    }

    #[must_use]
    pub fn reports_no_microphones(&self) -> bool {
        self.microphones.is_empty()
            && self.probe_error.is_none()
            && self.sources.iter().any(|status| {
                matches!(
                    status.source,
                    CaptureSource::AudioEndpoint | CaptureSource::SoundDevice
                ) && status.outcome == SourceOutcome::Reported
            })
    }
}

#[must_use]
pub fn collect(cancellation: &CancellationToken) -> CaptureProbe {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );

    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        CAPTURE_SCRIPT,
        limits,
        cancellation,
    ) {
        Ok(output) => match std::str::from_utf8(output.stdout()) {
            Ok(text) => parse_probe(text),
            Err(_) => CaptureProbe::unavailable("The capture probe returned unreadable output."),
        },
        Err(error) => CaptureProbe::unavailable(match error.kind {
            CollectorErrorKind::Unsupported => {
                "Camera and microphone collection is only available on Windows."
            }
            CollectorErrorKind::PermissionDenied => {
                "Windows refused the camera and microphone query on this account."
            }
            CollectorErrorKind::TimedOut => {
                "The camera and microphone probe exceeded its time limit."
            }
            CollectorErrorKind::Cancelled => "The camera and microphone probe was cancelled.",
            CollectorErrorKind::OutputLimitExceeded => {
                "The camera and microphone probe returned more data than allowed."
            }
            _ => "The camera and microphone probe could not be completed on this PC.",
        }),
    }
}

#[must_use]
pub fn parse_probe(text: &str) -> CaptureProbe {
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
        if index > MAX_DEVICE_INDEX {
            continue;
        }
        let Some(value) = decode_hex(encoded) else {
            continue;
        };
        values.insert((section.to_string(), index, name.to_string()), value);
    }

    let sources = CaptureSource::ALL
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

    CaptureProbe {
        cameras: devices(&values, "camera", "camera"),
        microphones: devices(&values, "microphone", "microphone"),
        sources,
        frames_captured: flag(&values, "capture", 0, "frames_captured"),
        audio_recorded: flag(&values, "capture", 0, "audio_recorded"),
        probe_error: None,
    }
}

fn devices(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    kind: &'static str,
) -> Vec<CaptureDevice> {
    (0..=MAX_DEVICE_INDEX)
        .filter(|index| flag(values, section, *index, "present"))
        .map(|index| CaptureDevice {
            kind,
            present: true,
            name: text(values, section, index, "name"),
            manufacturer: text(values, section, index, "manufacturer"),
            enumerated_by: text(values, section, index, "enumerated_by"),
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
    matches!(
        section,
        "camera"
            | "microphone"
            | "camera_class"
            | "image_class"
            | "usbvideo"
            | "media_class"
            | "audio_endpoint"
            | "sound_device"
            | "capture"
    ) && matches!(
        name,
        "source_status"
            | "present"
            | "name"
            | "manufacturer"
            | "enumerated_by"
            | "none_enumerated"
            | "frames_captured"
            | "audio_recorded"
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

        fn probe(&self) -> CaptureProbe {
            parse_probe(&self.0)
        }
    }

    #[test]
    fn a_uvc_webcam_is_listed_even_when_the_camera_class_is_empty() {
        let probe = Fixture::new()
            .line("camera_class", 0, "source_status", "reported")
            .line("usbvideo", 0, "source_status", "reported")
            .line("camera", 0, "present", "True")
            .line("camera", 0, "name", "USB Video Device")
            .line("camera", 0, "enumerated_by", "USB video service")
            .line("capture", 0, "frames_captured", "False")
            .probe();

        assert_eq!(probe.camera_names(), vec!["USB Video Device".to_string()]);
        assert!(!probe.frames_captured);
        assert_eq!(
            probe.cameras[0].enumerated_by.as_deref(),
            Some("USB video service")
        );
    }

    #[test]
    fn scanners_are_not_turned_into_cameras() {
        let probe = Fixture::new()
            .line("image_class", 0, "source_status", "reported")
            .probe();

        assert!(probe.cameras.is_empty());
    }

    #[test]
    fn a_microphone_array_is_kept_and_speakers_are_dropped() {
        let probe = Fixture::new()
            .line("audio_endpoint", 0, "source_status", "reported")
            .line("microphone", 0, "present", "True")
            .line("microphone", 0, "name", "Microphone Array")
            .line("capture", 0, "audio_recorded", "False")
            .probe();

        assert_eq!(
            probe.microphone_names(),
            vec!["Microphone Array".to_string()]
        );
        assert!(!probe.audio_recorded);
    }

    #[test]
    fn none_enumerated_is_an_empty_list_not_a_fake_device() {
        let probe = Fixture::new()
            .line("camera_class", 0, "source_status", "reported")
            .line("usbvideo", 0, "source_status", "reported")
            .line("camera", 0, "none_enumerated", "True")
            .probe();

        assert!(probe.cameras.is_empty());
        assert!(probe.reports_no_cameras());
    }

    #[test]
    fn enumeration_never_claims_a_captured_frame() {
        let probe = Fixture::new()
            .line("camera", 0, "present", "True")
            .line("camera", 0, "name", "Integrated Camera")
            .line("capture", 0, "frames_captured", "False")
            .line("capture", 0, "audio_recorded", "False")
            .probe();

        assert!(!probe.frames_captured);
        assert!(!probe.audio_recorded);
    }

    #[test]
    fn an_unavailable_probe_states_the_reason() {
        let probe = CaptureProbe::unavailable(
            "Camera and microphone collection is only available on Windows.",
        );

        assert!(probe.cameras.is_empty());
        assert!(probe.microphones.is_empty());
        assert!(!probe.reports_no_cameras());
        assert_eq!(
            probe.probe_error,
            Some("Camera and microphone collection is only available on Windows.")
        );
    }
}
