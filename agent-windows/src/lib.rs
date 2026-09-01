pub mod application_data;
pub mod assessment;
pub mod collector_runtime;
pub mod cpu;
pub mod device;
pub mod encryption;
pub mod evidence;
pub mod hardware_inventory_v1;
pub mod hardware_validation;
pub mod os;
pub mod pdem;
pub mod personal_data;
pub mod report;
pub mod storage;
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
    let device = adapter.collect_device();
    let operating_system = adapter.collect_os();
    let cpu = adapter.collect_cpu();
    let storage = adapter.collect_storage();
    let volumes = adapter.collect_volumes();
    let encryption = adapter.collect_encryption();
    let user_profiles = adapter.collect_user_profiles();
    let hardware_inventory = adapter.collect_hardware_inventory();

    let profile_paths = profile_paths(&user_profiles);
    let volume_roots = volume_roots(&volumes);
    let personal_data = adapter.collect_personal_data(&profile_paths, &volume_roots);
    let application_data = adapter.collect_application_data(&profile_paths);
    let pdem = pdem::build(&personal_data, &application_data);
    let evidence = evidence::collect();
    let assessment = assessment::assess();

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
}

pub fn run_customer_verification() -> CustomerVerification {
    let scan = run_scan();
    let (hardware_validation, hardware_passed, hardware_result) = match &scan.hardware_inventory {
        Some(inventory) => {
            let report = hardware_validation::build_report(inventory);
            let result = if report.passed { "pass" } else { "fail" };
            (report.lines.join("\n"), report.passed, result.to_string())
        }
        None => (
            hardware_validation::not_windows_text(),
            false,
            "not_windows".to_string(),
        ),
    };

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
