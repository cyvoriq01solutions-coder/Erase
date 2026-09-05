//! Dual confirmation and Mode S target planning.

use super::media::{classify, method_for, MediaClass, MethodClass};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedTarget {
    pub letter: String,
    pub disk_index: u32,
    pub media_class: MediaClass,
    pub method: MethodClass,
    pub model: String,
    pub serial: String,
    pub bus: String,
    pub size_bytes: u64,
    pub allowed: bool,
    pub refuse_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VolumeHint {
    pub letter: String,
    pub drive_kind: String,
    pub is_system: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct DiskHint {
    pub index: u32,
    pub model: String,
    pub serial: String,
    pub size_bytes: u64,
    pub interface_type: String,
    pub media_type: String,
}

pub fn hostname_matches(expected: &str, typed: &str) -> bool {
    let left = expected.trim().to_ascii_lowercase();
    let right = typed.trim().to_ascii_lowercase();
    !left.is_empty() && left == right
}

pub fn erase_confirmed(typed: &str) -> bool {
    typed.trim() == "ERASE"
}

pub fn plan_targets(
    selected_letters: &[String],
    volumes: &[VolumeHint],
    disks: &[DiskHint],
    usb_opt_in: bool,
) -> Vec<PlannedTarget> {
    selected_letters
        .iter()
        .map(|raw| {
            let letter = raw.trim().trim_end_matches(':').to_ascii_uppercase();
            let volume = volumes.iter().find(|item| item.letter == letter);
            let is_system = volume.map(|item| item.is_system).unwrap_or(false);
            let drive_kind = volume
                .map(|item| item.drive_kind.as_str())
                .unwrap_or("unknown");
            let size_hint = volume.map(|item| item.size_bytes).unwrap_or(0);
            let disk = match_disk(disks, size_hint, drive_kind);
            let interface = disk
                .map(|item| item.interface_type.as_str())
                .unwrap_or("");
            let media_type = disk.map(|item| item.media_type.as_str()).unwrap_or("");
            let model = disk.map(|item| item.model.as_str()).unwrap_or("unknown");
            let class = classify(is_system, drive_kind, interface, media_type, model);
            let method = method_for(class);
            let usb_media = matches!(class, MediaClass::UsbHdd | MediaClass::UsbFlash);
            let _ = usb_opt_in;
            let allowed = class.mode_s_allowed() && method != MethodClass::Refused && !usb_media;
            let refuse_reason = if class == MediaClass::SystemDisk {
                Some(
                    "The Windows system disk cannot be sanitised while this application is running on it (Mode S)."
                        .to_string(),
                )
            } else if usb_media {
                Some(
                    "Attached USB or removable media cannot be sanitised by this application."
                        .to_string(),
                )
            } else if class == MediaClass::Optical {
                Some("Optical drives cannot be sanitised by this application.".to_string())
            } else if class == MediaClass::Network {
                Some("Network locations cannot be sanitised by this application.".to_string())
            } else if allowed {
                None
            } else {
                Some("Unknown or unsupported media. Mode S fails closed.".to_string())
            };
            PlannedTarget {
                letter,
                disk_index: disk.map(|item| item.index).unwrap_or(u32::MAX),
                media_class: class,
                method,
                model: model.to_string(),
                serial: disk
                    .map(|item| item.serial.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                bus: if interface.is_empty() {
                    drive_kind.to_string()
                } else {
                    interface.to_string()
                },
                size_bytes: disk.map(|item| item.size_bytes).unwrap_or(size_hint),
                allowed,
                refuse_reason,
            }
        })
        .collect()
}

fn match_disk<'a>(disks: &'a [DiskHint], size_hint: u64, drive_kind: &str) -> Option<&'a DiskHint> {
    if disks.is_empty() {
        return None;
    }
    if disks.len() == 1 {
        return disks.first();
    }
    let removable = drive_kind == "removable";
    disks.iter().find(|disk| {
        let usb = disk.interface_type.to_ascii_lowercase().contains("usb");
        if removable {
            usb || disk.media_type.to_ascii_lowercase().contains("removable")
        } else {
            !usb && (size_hint == 0 || disk.size_bytes.abs_diff(size_hint) < 64 * 1024 * 1024)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(letter: &str, kind: &str, system: bool) -> VolumeHint {
        VolumeHint {
            letter: letter.to_string(),
            drive_kind: kind.to_string(),
            is_system: system,
            size_bytes: 8_000_000_000,
        }
    }

    fn usb_disk() -> DiskHint {
        DiskHint {
            index: 1,
            model: "TOSHIBA USB".to_string(),
            serial: "ABC123".to_string(),
            size_bytes: 8_000_000_000,
            interface_type: "USB".to_string(),
            media_type: "External hard disk media".to_string(),
        }
    }

    #[test]
    fn dual_confirm_tokens() {
        assert!(hostname_matches("LENOVO-PC", "lenovo-pc"));
        assert!(!hostname_matches("LENOVO-PC", "other"));
        assert!(erase_confirmed("ERASE"));
        assert!(erase_confirmed(" ERASE "));
        assert!(!erase_confirmed("erase"));
        assert!(!erase_confirmed("ERASE NOW"));
    }

    #[test]
    fn system_letter_is_planned_but_not_allowed() {
        let planned = plan_targets(
            &["C".to_string()],
            &[volume("C", "internal", true)],
            &[],
            true,
        );
        assert_eq!(planned.len(), 1);
        assert!(!planned[0].allowed);
        assert_eq!(planned[0].media_class, MediaClass::SystemDisk);
    }

    #[test]
    fn usb_hdd_is_always_refused() {
        let planned = plan_targets(
            &["E".to_string()],
            &[volume("E", "removable", false)],
            &[usb_disk()],
            true,
        );
        assert!(!planned[0].allowed);
        assert_eq!(planned[0].media_class, MediaClass::UsbHdd);
        assert!(planned[0]
            .refuse_reason
            .as_deref()
            .unwrap_or("")
            .contains("cannot be sanitised by this application"));
    }

    #[test]
    fn usb_opt_in_is_ignored() {
        let planned = plan_targets(
            &["E".to_string()],
            &[volume("E", "removable", false)],
            &[usb_disk()],
            false,
        );
        assert!(!planned[0].allowed);
        assert!(planned[0]
            .refuse_reason
            .as_deref()
            .unwrap_or("")
            .contains("USB or removable"));
    }

    #[test]
    fn extra_internal_hdd_is_allowed_clear() {
        let planned = plan_targets(
            &["D".to_string()],
            &[volume("D", "internal", false)],
            &[DiskHint {
                index: 1,
                model: "ST1000DM".to_string(),
                serial: "INT001".to_string(),
                size_bytes: 8_000_000_000,
                interface_type: "SATA".to_string(),
                media_type: "HDD".to_string(),
            }],
            false,
        );
        assert!(planned[0].allowed);
        assert_eq!(planned[0].media_class, MediaClass::MagneticHdd);
        assert_eq!(planned[0].method, MethodClass::OverwriteClear);
    }
}
