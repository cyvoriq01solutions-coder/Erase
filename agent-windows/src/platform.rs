use crate::{
    application_data::ApplicationDataInventory, cpu::CpuProfile, device::DeviceIdentity,
    encryption::EncryptionProfile, hardware_inventory_v1::HardwareInventoryV1, os::OsProfile,
    personal_data::PersonalDataInventory, storage::StorageProfile,
    user_profiles::UserProfileInventory, volume::VolumeProfile,
};

/// Read-only operating-system boundary used by GUI, CLI, and future headless callers.
///
/// Implementations must use fixed, trusted commands or native APIs. No caller-supplied
/// command, script, or shell fragment may cross this boundary.
pub trait PlatformAdapter {
    fn collect_device(&self) -> DeviceIdentity;
    fn collect_os(&self) -> OsProfile;
    fn collect_cpu(&self) -> CpuProfile;
    fn collect_storage(&self) -> StorageProfile;
    fn collect_volumes(&self) -> Vec<VolumeProfile>;
    fn collect_encryption(&self) -> EncryptionProfile;
    fn collect_user_profiles(&self) -> UserProfileInventory;

    /// Unsupported and fixture adapters remain explicit instead of fabricating an
    /// empty hardware result. Windows overrides this with the bounded V1 collector.
    fn collect_hardware_inventory(&self) -> Option<HardwareInventoryV1> {
        None
    }

    fn collect_personal_data(
        &self,
        profile_paths: &[String],
        volume_roots: &[String],
    ) -> PersonalDataInventory;

    fn collect_application_data(&self, profile_paths: &[String]) -> ApplicationDataInventory;
}

/// Adapter preserving agent 0.2.1 collection behavior while the collectors are
/// progressively moved behind typed Windows-native implementations.
#[derive(Debug, Default, Clone, Copy)]
pub struct NativePlatformAdapter;

impl PlatformAdapter for NativePlatformAdapter {
    fn collect_device(&self) -> DeviceIdentity {
        crate::device::collect()
    }

    fn collect_os(&self) -> OsProfile {
        crate::os::collect()
    }

    fn collect_cpu(&self) -> CpuProfile {
        crate::cpu::collect()
    }

    fn collect_storage(&self) -> StorageProfile {
        crate::storage::collect()
    }

    fn collect_volumes(&self) -> Vec<VolumeProfile> {
        crate::volume::collect()
    }

    fn collect_encryption(&self) -> EncryptionProfile {
        crate::encryption::collect()
    }

    fn collect_user_profiles(&self) -> UserProfileInventory {
        crate::user_profiles::collect()
    }

    fn collect_hardware_inventory(&self) -> Option<HardwareInventoryV1> {
        if cfg!(target_os = "windows") {
            Some(crate::windows_hardware::collect(
                &crate::collector_runtime::CancellationToken::new(),
            ))
        } else {
            None
        }
    }

    fn collect_personal_data(
        &self,
        profile_paths: &[String],
        volume_roots: &[String],
    ) -> PersonalDataInventory {
        crate::personal_data::collect(profile_paths, volume_roots)
    }

    fn collect_application_data(&self, profile_paths: &[String]) -> ApplicationDataInventory {
        crate::application_data::collect(profile_paths)
    }
}
