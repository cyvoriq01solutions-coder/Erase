//! Media class for Mode S. Unknown, optical and network are refused.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaClass {
    MagneticHdd,
    SataSsd,
    Nvme,
    UsbHdd,
    UsbFlash,
    Optical,
    Network,
    SystemDisk,
    Unknown,
}

impl MediaClass {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::MagneticHdd => "Magnetic HDD",
            Self::SataSsd => "SATA SSD",
            Self::Nvme => "NVMe SSD",
            Self::UsbHdd => "USB hard disk",
            Self::UsbFlash => "USB flash",
            Self::Optical => "Optical drive",
            Self::Network => "Network location",
            Self::SystemDisk => "Windows system disk",
            Self::Unknown => "Unknown media",
        }
    }

    pub fn mode_s_allowed(self) -> bool {
        matches!(
            self,
            Self::MagneticHdd | Self::SataSsd | Self::Nvme | Self::UsbHdd | Self::UsbFlash
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodClass {
    AtaSecureErase,
    NvmeSanitize,
    OverwriteClear,
    Refused,
}

impl MethodClass {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::AtaSecureErase => "ATA SECURITY ERASE UNIT",
            Self::NvmeSanitize => "NVMe Sanitize (Block Erase)",
            Self::OverwriteClear => "Single-pass overwrite (NIST Clear) — not Purge",
            Self::Refused => "Refused",
        }
    }

    pub fn standard(self) -> &'static str {
        match self {
            Self::AtaSecureErase => "NIST SP 800-88 Rev. 1 Purge",
            Self::NvmeSanitize => "NIST SP 800-88 Rev. 1 Purge / IEEE 2883-2022",
            Self::OverwriteClear => "NIST SP 800-88 Rev. 1 Clear",
            Self::Refused => "None",
        }
    }

    pub fn is_purge(self) -> bool {
        matches!(self, Self::AtaSecureErase | Self::NvmeSanitize)
    }
}

pub fn classify(
    is_system: bool,
    drive_kind: &str,
    interface_type: &str,
    media_type: &str,
    model: &str,
) -> MediaClass {
    if is_system {
        return MediaClass::SystemDisk;
    }
    let kind = drive_kind.to_ascii_lowercase();
    if kind == "optical" {
        return MediaClass::Optical;
    }
    if kind == "network" {
        return MediaClass::Network;
    }

    let iface = interface_type.to_ascii_lowercase();
    let media = media_type.to_ascii_lowercase();
    let model_l = model.to_ascii_lowercase();
    let looks_nvme = iface.contains("nvme") || model_l.contains("nvme");
    let looks_ssd = media.contains("ssd")
        || model_l.contains("ssd")
        || media.contains("solid")
        || iface.contains("nvme");
    let looks_usb = iface.contains("usb") || kind == "removable";
    let looks_flash =
        model_l.contains("flash") || model_l.contains("usb disk") || media.contains("removable");

    if looks_nvme {
        return MediaClass::Nvme;
    }
    if looks_usb && looks_ssd && !looks_flash {
        return MediaClass::SataSsd;
    }
    if looks_usb && looks_flash && !media.contains("hard") {
        return MediaClass::UsbFlash;
    }
    if looks_usb {
        return if looks_ssd {
            MediaClass::SataSsd
        } else {
            MediaClass::UsbHdd
        };
    }
    if looks_ssd {
        return MediaClass::SataSsd;
    }
    if kind == "internal"
        || media.contains("hard")
        || iface.contains("ide")
        || iface.contains("sata")
    {
        return MediaClass::MagneticHdd;
    }
    if kind == "removable" {
        return MediaClass::UsbHdd;
    }
    MediaClass::Unknown
}

pub fn method_for(class: MediaClass) -> MethodClass {
    match class {
        MediaClass::MagneticHdd | MediaClass::UsbHdd | MediaClass::UsbFlash => {
            MethodClass::OverwriteClear
        }
        MediaClass::SataSsd => MethodClass::AtaSecureErase,
        MediaClass::Nvme => MethodClass::NvmeSanitize,
        MediaClass::Optical
        | MediaClass::Network
        | MediaClass::SystemDisk
        | MediaClass::Unknown => MethodClass::Refused,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_disk_is_never_mode_s() {
        let class = classify(true, "internal", "SATA", "HDD", "ST1000");
        assert_eq!(class, MediaClass::SystemDisk);
        assert!(!class.mode_s_allowed());
        assert_eq!(method_for(class), MethodClass::Refused);
    }

    #[test]
    fn optical_and_network_are_refused() {
        assert_eq!(
            classify(false, "optical", "", "", "DVD"),
            MediaClass::Optical
        );
        assert_eq!(
            classify(false, "network", "", "", "share"),
            MediaClass::Network
        );
        assert_eq!(method_for(MediaClass::Unknown), MethodClass::Refused);
    }

    #[test]
    fn usb_hdd_is_clear_overwrite() {
        let class = classify(
            false,
            "removable",
            "USB",
            "External hard disk media",
            "TOSHIBA",
        );
        assert_eq!(class, MediaClass::UsbHdd);
        assert_eq!(method_for(class), MethodClass::OverwriteClear);
        assert!(!method_for(class).is_purge());
    }

    #[test]
    fn nvme_uses_sanitize_not_overwrite() {
        let class = classify(false, "internal", "NVMe", "SSD", "Samsung SSD 980");
        assert_eq!(class, MediaClass::Nvme);
        assert_eq!(method_for(class), MethodClass::NvmeSanitize);
        assert!(method_for(class).is_purge());
    }
}
