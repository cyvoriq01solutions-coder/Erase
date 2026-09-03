//! Live technician intake for USB insertion and charger sensing.
//!
//! The Interactive Checks overlay polls this while a technician inserts a USB
//! stick or a charger. It is read-only: it never writes to a volume.
//!
//! Charging is not a CG-1.0 scoring domain. A sensed USB volume is not a
//! substitute for the four physical-port attestation points. USB 1–USB 4 ticks
//! award those points. Reported USB speed is telemetry only.

use crate::collector_runtime::{
    CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
};
use crate::list_scan_targets;
use crate::{CollectorName, ScanTarget};
use std::collections::BTreeMap;
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_STDOUT_BYTES: usize = 8 * 1024;
const MAX_STDERR_BYTES: usize = 4 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

const POWER_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
    r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
try {
    $items = @(Get-CimInstance -ClassName Win32_Battery -ErrorAction Stop)
    Write-Output ("count=" + $items.Count)
    if ($items.Count -gt 0) {
        $b = $items[0]
        Write-Output ("present=1")
        Write-Output ("status=" + $b.BatteryStatus)
        Write-Output ("percent=" + $b.EstimatedChargeRemaining)
    }
} catch {
    Write-Output "count=0"
    Write-Output "status=error"
}
"#,
);

const VOLUME_SPEED_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
    r#"
$ErrorActionPreference = 'SilentlyContinue'
$ProgressPreference = 'SilentlyContinue'
Get-CimInstance Win32_LogicalDisk | Where-Object { $_.DriveType -eq 2 } | ForEach-Object {
    $letter = $_.DeviceID.TrimEnd(':')
    $speed = 'Not reported by Windows'
    try {
        $part = Get-CimInstance -Query ("ASSOCIATORS OF {Win32_LogicalDisk.DeviceID='" + $_.DeviceID + "'} WHERE AssocClass=Win32_LogicalDiskToPartition")
        if ($part) {
            $disk = Get-CimInstance -Query ("ASSOCIATORS OF {Win32_DiskPartition.DeviceID='" + $part.DeviceID + "'} WHERE AssocClass=Win32_DiskDriveToDiskPartition")
            if ($disk) {
                $blob = [string]$disk.Caption + ' ' + [string]$disk.PNPDeviceID + ' ' + [string]$disk.InterfaceType + ' ' + [string]$disk.Description
                $speed = $blob
            }
        }
    } catch {
        $speed = 'Not reported by Windows'
    }
    Write-Output ("letter=" + $letter)
    Write-Output ("descriptor=" + $speed)
}
"#,
);

/// Customer-safe notes printed on Report D. Empty means the check was not opened.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LiveIntakeNotes {
    pub usb_listed: String,
    pub power_status: String,
    pub camera_session: String,
    /// One line per USB 1–USB 4 tick, in order.
    pub usb_ports: Vec<String>,
}

impl LiveIntakeNotes {
    #[must_use]
    pub fn usb_or_default(&self) -> String {
        if self.usb_listed.trim().is_empty() {
            "Not attempted in this scan. A technician records this at physical verification."
                .to_string()
        } else {
            self.usb_listed.trim().to_string()
        }
    }

    #[must_use]
    pub fn power_or_default(&self) -> String {
        if self.power_status.trim().is_empty() {
            "Not attempted in this scan. A technician records this at physical verification."
                .to_string()
        } else {
            self.power_status.trim().to_string()
        }
    }

    #[must_use]
    pub fn camera_or_default(&self) -> String {
        if self.camera_session.trim().is_empty() {
            "Not attempted in this scan. A technician records this at physical verification."
                .to_string()
        } else {
            self.camera_session.trim().to_string()
        }
    }

    #[must_use]
    pub fn usb_port_or_default(&self, index: usize) -> String {
        self.usb_ports
            .get(index)
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                "Not attempted in this scan. A technician records this at physical verification."
                    .to_string()
            })
    }

    #[must_use]
    pub fn has_camera_capture(&self) -> bool {
        let text = self.camera_session.to_ascii_lowercase();
        text.contains("snapshot") || text.contains("clip")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRemovableVolume {
    pub letter: String,
    pub label: String,
    pub size_label: String,
    pub speed_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivePowerStatus {
    pub present: bool,
    pub on_mains: bool,
    pub charging: bool,
    pub status_code: Option<i32>,
    pub status_label: String,
    pub charge_percent: Option<u8>,
    pub available: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveIntakeProbe {
    pub removable: Vec<LiveRemovableVolume>,
    pub power: LivePowerStatus,
}

/// Map a Windows battery status code. Used by the overlay and by tests.
#[must_use]
pub fn interpret_battery_status(code: i32, charge_percent: Option<u8>) -> LivePowerStatus {
    let (on_mains, charging, label) = match code {
        1 => (false, false, "Discharging"),
        2 => (true, false, "On mains power"),
        3 => (true, false, "Fully charged"),
        4 => (false, false, "Low"),
        5 => (false, false, "Critical"),
        6 => (true, true, "Charging"),
        7 => (true, true, "Charging and high"),
        8 => (true, true, "Charging and low"),
        9 => (true, true, "Charging and critical"),
        11 => (false, false, "Partially charged"),
        _ => (false, false, "Windows did not name this power state"),
    };
    let percent_text = charge_percent
        .map(|value| format!("{value}%"))
        .unwrap_or_else(|| "charge level not reported".to_string());
    let detail = if charging {
        format!("Windows reports the pack is charging ({percent_text}). BatteryStatus {code}.")
    } else if on_mains {
        format!(
            "Windows reports AC is available ({percent_text}). The pack is not necessarily charging. BatteryStatus {code}."
        )
    } else {
        format!(
            "Windows reports the pack is not on a charger ({percent_text}). BatteryStatus {code}."
        )
    };
    LivePowerStatus {
        present: true,
        on_mains,
        charging,
        status_code: Some(code),
        status_label: label.to_string(),
        charge_percent,
        available: true,
        detail,
    }
}

/// Map a Windows disk caption / PnP string to a customer USB speed label.
#[must_use]
pub fn classify_usb_speed(descriptor: &str) -> String {
    let upper = descriptor.to_ascii_uppercase();
    if upper.contains("NOT REPORTED BY WINDOWS") || descriptor.trim().is_empty() {
        return "Not reported by Windows".to_string();
    }
    if upper.contains("SSPLUS")
        || upper.contains("SUPERSPEEDPLUS")
        || upper.contains("SUPER SPEED PLUS")
        || upper.contains("USB 3.2")
        || upper.contains("USB3.2")
        || upper.contains("20GB")
        || upper.contains("10GB")
    {
        return "USB 3.2 SuperSpeed+".to_string();
    }
    if upper.contains("SUPERSPEED")
        || upper.contains("SUPER SPEED")
        || upper.contains("USB 3.1")
        || upper.contains("USB3.1")
        || upper.contains("USB 3.0")
        || upper.contains("USB3.0")
        || upper.contains("USB3")
    {
        return "USB 3.0 SuperSpeed".to_string();
    }
    if upper.contains("USB 2.0")
        || upper.contains("USB2.0")
        || upper.contains("HIGH SPEED")
        || upper.contains("USB2")
    {
        return "USB 2.0 High Speed".to_string();
    }
    if upper.contains("USB 1.1")
        || upper.contains("USB1.1")
        || upper.contains("FULL SPEED")
        || upper.contains("USB1")
    {
        return "USB 1.1 Full Speed".to_string();
    }
    "Not reported by Windows".to_string()
}

fn letter_key(letter: &str) -> String {
    letter.trim().trim_end_matches(':').to_ascii_uppercase()
}

fn removable_from_targets(targets: Vec<ScanTarget>) -> Vec<LiveRemovableVolume> {
    targets
        .into_iter()
        .filter(|target| target.kind == "Removable or USB")
        .map(|target| LiveRemovableVolume {
            letter: target.letter,
            label: target.label,
            size_label: target.size_label,
            speed_label: "Not reported by Windows".to_string(),
        })
        .collect()
}

fn parse_volume_speeds(stdout: &str) -> BTreeMap<String, String> {
    let mut speeds = BTreeMap::new();
    let mut current_letter = String::new();
    for line in stdout.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "letter" => current_letter = letter_key(value),
            "descriptor" | "speed" if !current_letter.is_empty() => {
                let label = if key.trim() == "speed" {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        "Not reported by Windows".to_string()
                    } else {
                        trimmed.to_string()
                    }
                } else {
                    classify_usb_speed(value)
                };
                speeds.insert(current_letter.clone(), label);
            }
            _ => {}
        }
    }
    speeds
}

fn apply_volume_speeds(volumes: &mut [LiveRemovableVolume], speeds: &BTreeMap<String, String>) {
    for volume in volumes {
        if let Some(speed) = speeds.get(&letter_key(&volume.letter)) {
            volume.speed_label = speed.clone();
        }
    }
}

fn parse_power_stdout(stdout: &str) -> LivePowerStatus {
    let mut count = 0_i32;
    let mut status_code: Option<i32> = None;
    let mut percent: Option<u8> = None;
    let mut saw_error = false;
    for line in stdout.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "count" => count = value.trim().parse().unwrap_or(0),
            "status" => {
                if value.trim().eq_ignore_ascii_case("error") {
                    saw_error = true;
                } else {
                    status_code = value.trim().parse().ok();
                }
            }
            "percent" => percent = value.trim().parse().ok(),
            _ => {}
        }
    }
    if saw_error && status_code.is_none() {
        return LivePowerStatus {
            present: false,
            on_mains: false,
            charging: false,
            status_code: None,
            status_label: "Not collected".to_string(),
            charge_percent: None,
            available: false,
            detail: "Windows did not return a battery status for this check.".to_string(),
        };
    }
    if count <= 0 {
        return LivePowerStatus {
            present: false,
            on_mains: false,
            charging: false,
            status_code: None,
            status_label: "No battery reported".to_string(),
            charge_percent: None,
            available: true,
            detail:
                "Windows listed no battery pack. A desktop without a pack cannot show charging."
                    .to_string(),
        };
    }
    match status_code {
        Some(code) => interpret_battery_status(code, percent),
        None => LivePowerStatus {
            present: true,
            on_mains: false,
            charging: false,
            status_code: None,
            status_label: "Present".to_string(),
            charge_percent: percent,
            available: true,
            detail: "Windows listed a battery but did not name BatteryStatus.".to_string(),
        },
    }
}

fn probe_live_power() -> LivePowerStatus {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );
    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        POWER_SCRIPT,
        limits,
        &CancellationToken::new(),
    ) {
        Ok(output) => parse_power_stdout(&String::from_utf8_lossy(output.stdout())),
        Err(_) => LivePowerStatus {
            present: false,
            on_mains: false,
            charging: false,
            status_code: None,
            status_label: "Not available on this PC".to_string(),
            charge_percent: None,
            available: false,
            detail: if cfg!(target_os = "windows") {
                "Windows did not return a battery status for this check.".to_string()
            } else {
                "Live charger sensing runs only on Windows.".to_string()
            },
        },
    }
}

fn probe_volume_speeds() -> BTreeMap<String, String> {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );
    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        VOLUME_SPEED_SCRIPT,
        limits,
        &CancellationToken::new(),
    ) {
        Ok(output) => parse_volume_speeds(&String::from_utf8_lossy(output.stdout())),
        Err(_) => BTreeMap::new(),
    }
}

/// Removable volumes Windows currently lists, plus a WMI-only power read.
#[must_use]
pub fn probe_live_intake() -> LiveIntakeProbe {
    let mut removable = removable_from_targets(list_scan_targets());
    let speeds = probe_volume_speeds();
    apply_volume_speeds(&mut removable, &speeds);
    LiveIntakeProbe {
        removable,
        power: probe_live_power(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LiveIntakeNotes, apply_volume_speeds, classify_usb_speed, interpret_battery_status,
        parse_power_stdout, parse_volume_speeds, removable_from_targets,
    };
    use crate::ScanTarget;
    use std::collections::BTreeMap;

    fn target(letter: &str, kind: &str) -> ScanTarget {
        ScanTarget {
            letter: letter.to_string(),
            label: "Stick".to_string(),
            kind: kind.to_string(),
            size_label: "8 GB".to_string(),
            default_selected: false,
            hint: String::new(),
        }
    }

    #[test]
    fn charging_codes_are_on_mains_and_charging() {
        for code in [6, 7, 8, 9] {
            let status = interpret_battery_status(code, Some(41));
            assert!(status.charging, "code {code}");
            assert!(status.on_mains, "code {code}");
            assert_eq!(status.charge_percent, Some(41));
            assert!(status.detail.contains("charging"));
        }
    }

    #[test]
    fn ac_online_is_not_claimed_as_charging() {
        let status = interpret_battery_status(2, Some(88));
        assert!(status.on_mains);
        assert!(!status.charging);
        assert!(status.detail.contains("not necessarily charging"));
    }

    #[test]
    fn discharging_is_not_on_mains() {
        let status = interpret_battery_status(1, Some(12));
        assert!(!status.on_mains);
        assert!(!status.charging);
        assert_eq!(status.status_label, "Discharging");
    }

    #[test]
    fn empty_battery_list_is_honest() {
        let status = parse_power_stdout("count=0\n");
        assert!(!status.present);
        assert!(status.available);
        assert!(status.detail.contains("no battery pack"));
    }

    #[test]
    fn protocol_line_maps_status_six() {
        let status = parse_power_stdout("count=1\npresent=1\nstatus=6\npercent=41\n");
        assert!(status.charging);
        assert_eq!(status.charge_percent, Some(41));
        assert_eq!(status.status_code, Some(6));
    }

    #[test]
    fn removable_filter_drops_internal_disks() {
        let listed = removable_from_targets(vec![
            target("C", "Internal disk"),
            target("E", "Removable or USB"),
        ]);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].letter, "E");
        assert_eq!(listed[0].speed_label, "Not reported by Windows");
    }

    #[test]
    fn empty_notes_print_not_attempted() {
        let notes = LiveIntakeNotes::default();
        assert!(notes.usb_or_default().contains("Not attempted"));
        assert!(notes.usb_port_or_default(0).contains("Not attempted"));
        assert!(!notes.has_camera_capture());
    }

    #[test]
    fn snapshot_note_is_a_capture() {
        let notes = LiveIntakeNotes {
            camera_session:
                "Live preview opened. A snapshot was taken in this session and was not stored."
                    .to_string(),
            ..LiveIntakeNotes::default()
        };
        assert!(notes.has_camera_capture());
    }

    #[test]
    fn classify_usb_speed_maps_common_windows_strings() {
        assert_eq!(
            classify_usb_speed("USB SuperSpeedPlus Root Hub"),
            "USB 3.2 SuperSpeed+"
        );
        assert_eq!(
            classify_usb_speed("USB 3.0 SuperSpeed Root Hub"),
            "USB 3.0 SuperSpeed"
        );
        assert_eq!(
            classify_usb_speed("USB Mass Storage Device USB2"),
            "USB 2.0 High Speed"
        );
        assert_eq!(
            classify_usb_speed("Full-Speed USB Composite Device USB1"),
            "USB 1.1 Full Speed"
        );
        assert_eq!(
            classify_usb_speed("Generic USB Flash Disk"),
            "Not reported by Windows"
        );
    }

    #[test]
    fn volume_speed_protocol_maps_the_letter() {
        let speeds = parse_volume_speeds(
            "letter=E\ndescriptor=USB 3.0 SuperSpeed Root Hub\nletter=F\nspeed=USB 2.0 High Speed\n",
        );
        assert_eq!(
            speeds.get("E").map(String::as_str),
            Some("USB 3.0 SuperSpeed")
        );
        assert_eq!(
            speeds.get("F").map(String::as_str),
            Some("USB 2.0 High Speed")
        );
        let mut volumes = removable_from_targets(vec![target("E", "Removable or USB")]);
        apply_volume_speeds(&mut volumes, &speeds);
        assert_eq!(volumes[0].speed_label, "USB 3.0 SuperSpeed");
        let unused: BTreeMap<String, String> = BTreeMap::new();
        apply_volume_speeds(&mut volumes, &unused);
        assert_eq!(volumes[0].speed_label, "USB 3.0 SuperSpeed");
    }
}
