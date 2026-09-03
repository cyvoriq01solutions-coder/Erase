pub mod advance_bench;
pub mod application_data;
pub mod assessment;
pub mod battery_probe;
pub mod capture_probe;
pub mod collector_runtime;
pub mod cpu;
pub mod cpu_memory;
pub mod device;
pub mod diagnostics;
pub mod display_radio;
pub mod encryption;
pub mod evidence;
pub mod hardware_diagnostics_v1;
pub mod hardware_inventory_v1;
pub mod hardware_validation;
pub mod live_intake;
pub mod os;
pub mod pdem;
pub mod personal_data;
pub mod report;
pub mod report_signing;
pub mod storage;
pub mod storage_health;
pub mod usb_topology;
pub mod user_profiles;
pub mod volume;
pub mod windows_hardware;

mod platform;

pub use platform::{NativePlatformAdapter, PlatformAdapter};

/// Stable typed boundary consumed by the engineering CLI and, later, the Tauri shell.
/// `hardware_inventory` is populated only by a platform adapter with an implemented,
/// separately tested passive collector. Unsupported adapters keep it explicitly absent.
#[derive(Debug)]
pub struct ScanResult {
    pub device: device::DeviceIdentity,
    pub operating_system: os::OsProfile,
    pub cpu: cpu::CpuProfile,
    pub storage: storage::StorageProfile,
    pub volumes: Vec<volume::VolumeProfile>,
    pub encryption: encryption::EncryptionProfile,
    pub user_profiles: user_profiles::UserProfileInventory,
    pub personal_data: personal_data::PersonalDataInventory,
    pub application_data: application_data::ApplicationDataInventory,
    pub pdem: pdem::PdemProfile,
    pub evidence: evidence::EvidenceRecord,
    pub assessment: assessment::AssessmentResult,
    pub hardware_inventory: Option<hardware_inventory_v1::HardwareInventoryV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollectorName {
    Device,
    OperatingSystem,
    Processor,
    Storage,
    Volumes,
    Encryption,
    UserProfiles,
    PersonalData,
    ApplicationData,
    HardwareInventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollectorErrorKind {
    Unsupported,
    PermissionDenied,
    CommandFailed,
    ParseFailed,
    Cancelled,
    TimedOut,
    OutputLimitExceeded,
    Internal,
}

/// Typed collector failure for future cancellable and timeout-aware adapters.
/// Existing 0.2.1 collectors retain their current in-result status behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectorError {
    pub collector: CollectorName,
    pub kind: CollectorErrorKind,
    pub safe_message: String,
}

pub type CollectorResult<T> = Result<T, CollectorError>;

pub fn run_scan() -> ScanResult {
    run_scan_with(&NativePlatformAdapter)
}

pub fn run_scan_with<A>(adapter: &A) -> ScanResult
where
    A: PlatformAdapter + ?Sized,
{
    run_scan_selected(adapter, None, &mut |_, _, _, _| {})
}

fn run_scan_selected<A, F>(
    adapter: &A,
    drive_letters: Option<&[String]>,
    progress: &mut F,
) -> ScanResult
where
    A: PlatformAdapter + ?Sized,
    F: FnMut(u8, u8, &str, &str),
{
    progress(
        6,
        0,
        "Preparing verification",
        "Getting this PC ready for a local assessment.",
    );

    progress(
        14,
        1,
        "Confirming device identity",
        "Reading the Windows computer name and model.",
    );
    let device = adapter.collect_device();
    let operating_system = adapter.collect_os();

    progress(
        28,
        2,
        "Collecting hardware information",
        "Recording processor, memory and firmware details.",
    );
    let cpu = adapter.collect_cpu();
    let storage = adapter.collect_storage();
    let volumes_all = adapter.collect_volumes();
    let volumes = filter_volumes(&volumes_all, drive_letters);
    let encryption = adapter.collect_encryption();
    let user_profiles = adapter.collect_user_profiles();
    let hardware_inventory = adapter.collect_hardware_inventory();

    let mut profile_paths = profile_paths(&user_profiles);
    if let Some(letters) = drive_letters {
        profile_paths.retain(|path| path_matches_letters(path, letters));
    }

    let volume_roots = volume_roots(&volumes);
    let scanned = scanned_drive_label(&volumes);

    progress(
        48,
        3,
        "Assessing personal-data locations",
        &format!("Looking for documents and known file types on {scanned}."),
    );
    let personal_data = adapter.collect_personal_data(&profile_paths, &volume_roots);

    progress(
        62,
        3,
        "Assessing personal-data locations",
        "Checking application data folders without opening messages.",
    );
    let application_data = adapter.collect_application_data(&profile_paths);

    progress(
        76,
        4,
        "Building the Privacy Exposure Map",
        "Summarising where personal files appear to live.",
    );
    let pdem = pdem::build(&personal_data, &application_data);

    progress(
        86,
        5,
        "Preparing evidence",
        "Recording what was assessed on this PC.",
    );
    let evidence = evidence::collect();

    progress(
        93,
        6,
        "Checking consistency",
        "Confirming the assessment stayed read-only.",
    );
    let assessment = assessment::assess();

    progress(
        100,
        7,
        "Preparing results",
        "The local assessment is ready to review.",
    );

    ScanResult {
        device,
        operating_system,
        cpu,
        storage,
        volumes,
        encryption,
        user_profiles,
        personal_data,
        application_data,
        pdem,
        evidence,
        assessment,
        hardware_inventory,
    }
}

fn profile_paths(inventory: &user_profiles::UserProfileInventory) -> Vec<String> {
    let mut paths = inventory
        .profiles
        .iter()
        .filter(|profile| !profile.special && profile.path != "unknown")
        .map(|profile| profile.path.clone())
        .collect::<Vec<_>>();

    if inventory.current_profile != "unknown" {
        paths.push(inventory.current_profile.clone());
    }

    paths.sort();
    paths.dedup();
    paths
}

fn volume_roots(volumes: &[volume::VolumeProfile]) -> Vec<String> {
    volumes
        .iter()
        .filter(|volume| volume.drive_letter != "unknown")
        .map(|volume| format!("{}:\\", volume.drive_letter))
        .collect()
}

fn letter_key(value: &str) -> String {
    value
        .chars()
        .find(|character| character.is_ascii_alphabetic())
        .map(|character| character.to_ascii_uppercase().to_string())
        .unwrap_or_default()
}

fn path_matches_letters(path: &str, letters: &[String]) -> bool {
    let key = letter_key(path);
    !key.is_empty() && letters.iter().any(|letter| letter_key(letter) == key)
}

fn filter_volumes(
    volumes: &[volume::VolumeProfile],
    drive_letters: Option<&[String]>,
) -> Vec<volume::VolumeProfile> {
    let Some(letters) = drive_letters else {
        return volumes.to_vec();
    };

    let selected: Vec<String> = letters.iter().map(|letter| letter_key(letter)).collect();
    volumes
        .iter()
        .filter(|volume| selected.contains(&letter_key(&volume.drive_letter)))
        .cloned()
        .collect()
}

fn scanned_drive_label(volumes: &[volume::VolumeProfile]) -> String {
    let letters: Vec<String> = volumes
        .iter()
        .filter(|volume| volume.drive_letter != "unknown")
        .map(|volume| format!("{}:", volume.drive_letter))
        .collect();
    if letters.is_empty() {
        "the selected drive".to_string()
    } else {
        letters.join(", ")
    }
}

fn system_drive_letter() -> String {
    std::env::var("SystemDrive")
        .ok()
        .or_else(|| std::env::var("SYSTEMDRIVE").ok())
        .map(|value| letter_key(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "C".to_string())
}

#[derive(Debug, Clone)]
pub struct ScanTarget {
    pub letter: String,
    pub label: String,
    pub kind: String,
    pub size_label: String,
    pub default_selected: bool,
    pub hint: String,
}

pub fn list_scan_targets() -> Vec<ScanTarget> {
    let system = system_drive_letter();
    volume::collect()
        .into_iter()
        .filter(|volume| {
            let letter = letter_key(&volume.drive_letter);
            !letter.is_empty() && volume.drive_letter != "unknown"
        })
        .map(|volume| {
            let letter = letter_key(&volume.drive_letter);
            let is_system = letter == system;
            let kind = match volume.drive_kind.as_str() {
                "removable" => "Removable or USB",
                "network" => "Network location",
                "optical" => "Optical drive",
                "internal" => "Internal disk",
                _ => "Other",
            };
            let default_selected = is_system && volume.drive_kind != "removable";
            let hint = if volume.drive_kind == "removable" || volume.drive_kind == "optical" {
                "Left off by default. Select this only if you want it included."
                    .to_string()
            } else if is_system {
                "This is the Windows system drive. Recommended for every assessment.".to_string()
            } else {
                "May be an extra internal disk or an attached USB drive. Leave it off unless you need it."
                    .to_string()
            };
            let volume_name = if volume.label == "unknown" || volume.label.is_empty() {
                if is_system {
                    "Windows".to_string()
                } else {
                    "Local disk".to_string()
                }
            } else {
                volume.label.clone()
            };

            ScanTarget {
                letter,
                label: volume_name,
                kind: kind.to_string(),
                size_label: format_drive_size(volume.size_bytes),
                default_selected,
                hint,
            }
        })
        .collect()
}

fn format_drive_size(bytes: u64) -> String {
    const GB: f64 = 1_073_741_824.0;
    if bytes == 0 {
        "Size not reported".to_string()
    } else if bytes as f64 >= GB {
        format!("{:.0} GB", bytes as f64 / GB)
    } else {
        format!("{:.0} MB", bytes as f64 / 1_048_576.0)
    }
}

impl ScanResult {
    /// Preserve the 0.2.1 engineering JSON output while callers migrate to typed data.
    #[must_use]
    pub fn render_json(&self) -> String {
        let a6 = report::A6Evidence {
            volumes: &self.volumes,
            encryption: &self.encryption,
        };

        let a7 = report::A7Evidence {
            user_profiles: &self.user_profiles,
            personal_data: &self.personal_data,
            application_data: &self.application_data,
            pdem: &self.pdem,
        };

        let context = report::ReportContext {
            a6: &a6,
            a7: &a7,
            evidence: &self.evidence,
            assessment: &self.assessment,
        };

        report::render(
            &self.device,
            &self.operating_system,
            &self.cpu,
            &self.storage,
            &context,
        )
    }
}

#[derive(Debug, Clone)]
pub struct NamedValue {
    pub label: String,
    pub value: String,
}

#[derive(Debug)]
pub struct CustomerVerification {
    pub hardware_passed: bool,
    pub hardware_result: String,
    pub hardware_validation: String,
    pub report_json: String,
    pub manufacturer: String,
    pub model: String,
    pub hostname: String,
    pub os_caption: String,
    pub personal_location_count: u64,
    pub pdem_object_count: u64,
    pub content_inspected: bool,
    pub destructive_operations_enabled: bool,
    pub assessment_status: String,
    pub assessment_summary: String,
    pub scanned_drives: String,
    pub hardware_fields: Vec<NamedValue>,
    pub location_groups: Vec<NamedValue>,
}

pub fn run_customer_verification() -> CustomerVerification {
    run_customer_verification_on_drives(&[], &mut |_, _, _, _| {})
}

pub fn run_customer_verification_on_drives<F>(
    drive_letters: &[String],
    progress: &mut F,
) -> CustomerVerification
where
    F: FnMut(u8, u8, &str, &str),
{
    let selected: Vec<String> = if drive_letters.is_empty() {
        vec![system_drive_letter()]
    } else {
        drive_letters
            .iter()
            .map(|letter| letter_key(letter))
            .filter(|letter| !letter.is_empty())
            .collect()
    };

    let scan = run_scan_selected(&NativePlatformAdapter, Some(&selected), progress);
    let (hardware_validation, hardware_passed, hardware_result, mut hardware_fields) =
        match &scan.hardware_inventory {
            Some(inventory) => {
                let report = hardware_validation::build_report(inventory);
                let result = if report.passed { "pass" } else { "fail" };
                (
                    report.lines.join("\n"),
                    report.passed,
                    result.to_string(),
                    hardware_validation::customer_hardware_fields(inventory)
                        .into_iter()
                        .map(|(label, value)| NamedValue { label, value })
                        .collect::<Vec<_>>(),
                )
            }
            None => (
                hardware_validation::not_windows_text(),
                false,
                "not_windows".to_string(),
                Vec::new(),
            ),
        };

    prepend_field(
        &mut hardware_fields,
        "Computer name",
        scan.device.hostname.clone(),
    );
    prepend_field(
        &mut hardware_fields,
        "Operating system",
        scan.operating_system.caption.clone(),
    );
    overlay_bios_serial_from_device(&mut hardware_fields, &scan.device.serial_number);
    append_disk_serials(&mut hardware_fields, &scan.storage);

    CustomerVerification {
        hardware_passed,
        hardware_result,
        hardware_validation,
        report_json: scan.render_json(),
        manufacturer: scan.device.manufacturer.clone(),
        model: scan.device.model.clone(),
        hostname: scan.device.hostname.clone(),
        os_caption: scan.operating_system.caption.clone(),
        personal_location_count: scan.personal_data.locations.len() as u64,
        pdem_object_count: scan.pdem.objects.len() as u64,
        content_inspected: scan.personal_data.content_inspected
            || scan.application_data.content_inspected,
        destructive_operations_enabled: scan.storage.destructive_operations_enabled,
        assessment_status: scan.assessment.status.to_string(),
        assessment_summary: scan.assessment.summary.to_string(),
        scanned_drives: scanned_drive_label(&scan.volumes),
        hardware_fields,
        location_groups: location_groups(&scan),
    }
}

fn prepend_field(fields: &mut Vec<NamedValue>, label: &str, value: String) {
    if value.is_empty() || value == "unknown" {
        return;
    }
    if fields.iter().any(|field| field.label == label) {
        return;
    }
    fields.insert(
        0,
        NamedValue {
            label: label.to_string(),
            value,
        },
    );
}

fn looks_like_missing_serial(value: &str) -> bool {
    let compact: String = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect();
    compact.is_empty()
        || compact.chars().all(|character| character == '0')
        || compact == "unknown"
        || compact == "tobefilledbyoem"
        || compact == "defaultstring"
        || compact == "systemserialnumber"
        || compact == "none"
        || compact == "na"
}

fn overlay_bios_serial_from_device(fields: &mut Vec<NamedValue>, serial: &str) {
    let serial = serial.trim();
    if looks_like_missing_serial(serial) {
        return;
    }
    const LABEL: &str = "BIOS / OEM serial";
    const MISSING: &str = "Not reported by firmware";
    if let Some(row) = fields.iter_mut().find(|field| field.label == LABEL) {
        if row.value == MISSING {
            row.value = serial.to_string();
        }
        return;
    }
    fields.push(NamedValue {
        label: LABEL.to_string(),
        value: serial.to_string(),
    });
}

fn append_disk_serials(fields: &mut Vec<NamedValue>, storage: &crate::storage::StorageProfile) {
    for disk in &storage.disks {
        if looks_like_missing_serial(&disk.serial_number) {
            continue;
        }
        let label = format!("Disk {} serial", disk.index);
        if fields.iter().any(|field| field.label == label) {
            continue;
        }
        fields.push(NamedValue {
            label,
            value: disk.serial_number.trim().to_string(),
        });
    }
}

fn location_groups(scan: &ScanResult) -> Vec<NamedValue> {
    use std::collections::BTreeMap;

    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for location in &scan.personal_data.locations {
        *totals
            .entry(friendly_category(&location.category))
            .or_insert(0) += location.file_count;
    }

    let mut groups: Vec<NamedValue> = totals
        .into_iter()
        .map(|(label, count)| NamedValue {
            label,
            value: if count == 1 {
                "1 file".to_string()
            } else {
                format!("{count} files")
            },
        })
        .collect();

    let application_paths = scan.application_data.locations.len() as u64;
    if application_paths > 0 {
        groups.push(NamedValue {
            label: "Application data paths".to_string(),
            value: if application_paths == 1 {
                "1 location".to_string()
            } else {
                format!("{application_paths} locations")
            },
        });
    }

    groups
}

fn friendly_category(category: &str) -> String {
    match category {
        "document" => "Documents".to_string(),
        "pdf" => "PDF files".to_string(),
        "spreadsheet" => "Spreadsheets".to_string(),
        "presentation" => "Presentations".to_string(),
        "image" => "Pictures".to_string(),
        "video" => "Videos".to_string(),
        "audio" => "Audio files".to_string(),
        "archive" => "Archives".to_string(),
        "email" => "Email stores".to_string(),
        "database" => "Databases".to_string(),
        "backup" => "Backup files".to_string(),
        other => other.replace('_', " "),
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformAdapter, profile_paths, run_scan_with};
    use crate::{
        application_data::ApplicationDataInventory,
        cpu::CpuProfile,
        device::DeviceIdentity,
        encryption::EncryptionProfile,
        os::OsProfile,
        personal_data::PersonalDataInventory,
        storage::StorageProfile,
        user_profiles::{UserProfile, UserProfileInventory},
        volume::VolumeProfile,
    };

    struct FixtureAdapter;

    impl PlatformAdapter for FixtureAdapter {
        fn collect_device(&self) -> DeviceIdentity {
            DeviceIdentity {
                hostname: "fixture-device".to_string(),
                platform: "windows",
                architecture: "x86_64",
                manufacturer: "CYVORIQ Fixture".to_string(),
                model: "Model 1".to_string(),
                serial_number: "fixture-local-only".to_string(),
            }
        }

        fn collect_os(&self) -> OsProfile {
            OsProfile {
                operating_system: "windows",
                family: "windows",
                architecture: "x86_64",
                caption: "Windows fixture".to_string(),
                version: "10.0".to_string(),
                build_number: "fixture".to_string(),
            }
        }

        fn collect_cpu(&self) -> CpuProfile {
            CpuProfile {
                name: "Fixture CPU".to_string(),
                manufacturer: "Fixture".to_string(),
                cores: 4,
                logical_processors: 8,
                address_width: 64,
            }
        }

        fn collect_storage(&self) -> StorageProfile {
            StorageProfile {
                discovery_status: "completed".to_string(),
                destructive_operations_enabled: false,
                note: "Read-only fixture.".to_string(),
                disks: Vec::new(),
            }
        }

        fn collect_volumes(&self) -> Vec<VolumeProfile> {
            Vec::new()
        }

        fn collect_encryption(&self) -> EncryptionProfile {
            EncryptionProfile {
                collection_status: "completed".to_string(),
                note: "No recovery material collected.".to_string(),
                volumes: Vec::new(),
            }
        }

        fn collect_user_profiles(&self) -> UserProfileInventory {
            UserProfileInventory {
                discovery_status: "completed".to_string(),
                current_user: "fixture-user".to_string(),
                current_profile: r"C:\Users\Fixture".to_string(),
                profiles: Vec::new(),
            }
        }

        fn collect_personal_data(
            &self,
            _profile_paths: &[String],
            _volume_roots: &[String],
        ) -> PersonalDataInventory {
            PersonalDataInventory {
                discovery_status: "completed".to_string(),
                content_inspected: false,
                locations: Vec::new(),
                inaccessible_entries: 0,
            }
        }

        fn collect_application_data(&self, _profile_paths: &[String]) -> ApplicationDataInventory {
            ApplicationDataInventory {
                discovery_status: "completed".to_string(),
                content_inspected: false,
                locations: Vec::new(),
                inaccessible_entries: 0,
            }
        }
    }

    #[test]
    fn profile_paths_exclude_special_unknown_and_duplicates() {
        let inventory = UserProfileInventory {
            discovery_status: "fixture".to_string(),
            current_user: "fixture-user".to_string(),
            current_profile: r"C:\Users\Alex".to_string(),
            profiles: vec![
                UserProfile {
                    sid: "S-1".to_string(),
                    path: r"C:\Users\Alex".to_string(),
                    loaded: true,
                    special: false,
                },
                UserProfile {
                    sid: "S-2".to_string(),
                    path: r"C:\Windows\System32\config\systemprofile".to_string(),
                    loaded: true,
                    special: true,
                },
                UserProfile {
                    sid: "S-3".to_string(),
                    path: "unknown".to_string(),
                    loaded: false,
                    special: false,
                },
            ],
        };

        assert_eq!(
            profile_paths(&inventory),
            vec![r"C:\Users\Alex".to_string()]
        );
    }

    #[test]
    fn typed_core_keeps_hardware_uncollected_and_destructive_work_disabled() {
        let scan = run_scan_with(&FixtureAdapter);

        assert!(scan.hardware_inventory.is_none());
        assert!(!scan.storage.destructive_operations_enabled);
        assert!(!scan.personal_data.content_inspected);
        assert!(!scan.application_data.content_inspected);

        let report = scan.render_json();
        assert!(report.contains(r#""product": "CYVRA Erase Verification""#));
        assert!(report.contains(r#""agentVersion": "0.2.1""#));
        assert!(report.contains(r#""scanMode": "non_destructive""#));
        assert!(report.contains(r#""destructiveOperationsEnabled": false"#));
    }
}
