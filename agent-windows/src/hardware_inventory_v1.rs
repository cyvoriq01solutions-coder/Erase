//! Platform-neutral contract for passive hardware inventory.
//!
//! This module deliberately contains no collector implementation. Windows adapters are
//! added only with fixture/unit tests and must never invent values, sample sensor data,
//! capture private content, or execute a destructive operation.

use std::fmt;

pub const SCHEMA_VERSION: &str = "hardware_inventory_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollectionStatus {
    Reported,
    Observed,
    Derived,
    Unknown,
    NotReported,
    NotApplicable,
    PermissionDenied,
    Unsupported,
    CollectionError,
}

impl CollectionStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reported => "reported",
            Self::Observed => "observed",
            Self::Derived => "derived",
            Self::Unknown => "unknown",
            Self::NotReported => "not_reported",
            Self::NotApplicable => "not_applicable",
            Self::PermissionDenied => "permission_denied",
            Self::Unsupported => "unsupported",
            Self::CollectionError => "collection_error",
        }
    }

    #[must_use]
    pub const fn requires_value(self) -> bool {
        matches!(self, Self::Reported | Self::Observed | Self::Derived)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Confidence {
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PermissionState {
    NotRequired,
    Granted,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollectionSource {
    NativeWindowsApi,
    SetupApiOrPnp,
    CimOrWmi,
    SmbiosOrAcpi,
    PowerShellCimFallback,
    Derived,
    TestFixture,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PrivacyClass {
    NonSensitive,
    OperationalMetadata,
    DeviceIdentifier,
    PotentialPersonalData,
    ProhibitedSecret,
}

impl PrivacyClass {
    #[must_use]
    pub const fn allowed_in_hardware_inventory(self) -> bool {
        !matches!(self, Self::ProhibitedSecret)
    }
}

/// Exact serials, UUIDs, MAC addresses, asset tags, and hardware identifiers remain
/// local unless a later approved contract authorizes protected handling. Debug output
/// is always redacted so routine diagnostics cannot leak the raw value.
#[derive(Clone, PartialEq, Eq)]
pub struct DeviceIdentifier(String);

impl DeviceIdentifier {
    #[must_use]
    pub fn from_reported(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(Self(value.to_string()))
        }
    }

    /// Expose only to an explicitly authorized local report or pseudonymization path.
    #[must_use]
    pub fn expose_for_authorized_use(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for DeviceIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-device-identifier>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    pub source: CollectionSource,
    pub source_detail: Option<String>,
    pub parser_version: String,
    pub collected_at_unix: u64,
    pub permission: PermissionState,
}

impl Provenance {
    #[must_use]
    pub fn not_collected(collected_at_unix: u64) -> Self {
        Self {
            source: CollectionSource::Unknown,
            source_detail: None,
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            collected_at_unix,
            permission: PermissionState::Unknown,
        }
    }

    #[cfg(test)]
    fn fixture(collected_at_unix: u64) -> Self {
        Self {
            source: CollectionSource::TestFixture,
            source_detail: Some("unit-test".to_string()),
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            collected_at_unix,
            permission: PermissionState::NotRequired,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InventoryField<T> {
    pub value: Option<T>,
    pub status: CollectionStatus,
    pub confidence: Confidence,
    pub privacy_class: PrivacyClass,
    pub provenance: Provenance,
}

impl<T> InventoryField<T> {
    #[must_use]
    pub fn reported(
        value: T,
        confidence: Confidence,
        privacy_class: PrivacyClass,
        provenance: Provenance,
    ) -> Self {
        Self {
            value: Some(value),
            status: CollectionStatus::Reported,
            confidence,
            privacy_class,
            provenance,
        }
    }

    #[must_use]
    pub fn observed(
        value: T,
        confidence: Confidence,
        privacy_class: PrivacyClass,
        provenance: Provenance,
    ) -> Self {
        Self {
            value: Some(value),
            status: CollectionStatus::Observed,
            confidence,
            privacy_class,
            provenance,
        }
    }

    #[must_use]
    pub fn derived(
        value: T,
        confidence: Confidence,
        privacy_class: PrivacyClass,
        provenance: Provenance,
    ) -> Self {
        Self {
            value: Some(value),
            status: CollectionStatus::Derived,
            confidence,
            privacy_class,
            provenance,
        }
    }

    #[must_use]
    pub fn unknown(privacy_class: PrivacyClass, provenance: Provenance) -> Self {
        Self {
            value: None,
            status: CollectionStatus::Unknown,
            confidence: Confidence::Unknown,
            privacy_class,
            provenance,
        }
    }

    #[must_use]
    pub fn not_applicable(privacy_class: PrivacyClass, provenance: Provenance) -> Self {
        Self {
            value: None,
            status: CollectionStatus::NotApplicable,
            confidence: Confidence::High,
            privacy_class,
            provenance,
        }
    }

    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.privacy_class.allowed_in_hardware_inventory()
            && (self.status.requires_value() == self.value.is_some())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InventorySection<T> {
    pub status: CollectionStatus,
    pub provenance: Provenance,
    pub records: Vec<T>,
}

impl<T> InventorySection<T> {
    #[must_use]
    pub fn not_reported(provenance: Provenance) -> Self {
        Self {
            status: CollectionStatus::NotReported,
            provenance,
            records: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_consistent(&self) -> bool {
        self.status.requires_value() || self.records.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceClassification {
    OemReported,
    CustomOrUnidentified,
    Virtual,
    ConflictingFirmwareData,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FormFactor {
    Desktop,
    Laptop,
    Tablet,
    VirtualMachine,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceAndChassisIdentity {
    pub system_manufacturer: InventoryField<String>,
    pub system_model: InventoryField<String>,
    pub system_family: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
    pub system_uuid: InventoryField<DeviceIdentifier>,
    pub chassis_manufacturer: InventoryField<String>,
    pub chassis_type: InventoryField<String>,
    pub chassis_serial_number: InventoryField<DeviceIdentifier>,
    pub baseboard_manufacturer: InventoryField<String>,
    pub baseboard_product: InventoryField<String>,
    pub baseboard_version: InventoryField<String>,
    pub baseboard_serial_number: InventoryField<DeviceIdentifier>,
    pub asset_tag: InventoryField<DeviceIdentifier>,
    pub form_factor: InventoryField<FormFactor>,
    pub classification: InventoryField<DeviceClassification>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FirmwareMode {
    Uefi,
    LegacyBios,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FirmwareProfile {
    pub vendor: InventoryField<String>,
    pub version: InventoryField<String>,
    pub release_date: InventoryField<String>,
    pub smbios_version: InventoryField<String>,
    pub mode: InventoryField<FirmwareMode>,
    pub secure_boot_present: InventoryField<bool>,
    pub secure_boot_enabled: InventoryField<bool>,
    pub tpm_present: InventoryField<bool>,
    pub tpm_specification_version: InventoryField<String>,
    pub virtualization_indicator: InventoryField<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorProfile {
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub architecture: InventoryField<String>,
    pub physical_package_count: InventoryField<u32>,
    pub physical_core_count: InventoryField<u32>,
    pub logical_processor_count: InventoryField<u32>,
    pub maximum_clock_mhz: InventoryField<u32>,
    pub current_clock_mhz: InventoryField<u32>,
    pub address_width_bits: InventoryField<u32>,
    pub virtualization_capable: InventoryField<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemorySummary {
    pub installed_physical_bytes: InventoryField<u64>,
    pub visible_physical_bytes: InventoryField<u64>,
    pub physical_slot_count: InventoryField<u32>,
    pub populated_slot_count: InventoryField<u32>,
    pub error_correction_capability: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryModule {
    pub locator: InventoryField<String>,
    pub capacity_bytes: InventoryField<u64>,
    pub speed_mhz: InventoryField<u32>,
    pub configured_speed_mhz: InventoryField<u32>,
    pub memory_type: InventoryField<String>,
    pub form_factor: InventoryField<String>,
    pub manufacturer: InventoryField<String>,
    pub part_number: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryInventory {
    pub summary: InventorySection<MemorySummary>,
    pub modules: InventorySection<MemoryModule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StorageMediaKind {
    Hdd,
    Ssd,
    Nvme,
    Removable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageDevice {
    pub index: InventoryField<u32>,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
    pub firmware_revision: InventoryField<String>,
    pub size_bytes: InventoryField<u64>,
    pub interface_type: InventoryField<String>,
    pub media_kind: InventoryField<StorageMediaKind>,
    pub operational_status: InventoryField<String>,
    pub sanitization_capability_hint: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalVolume {
    pub mount_point: InventoryField<String>,
    pub file_system: InventoryField<String>,
    pub capacity_bytes: InventoryField<u64>,
    pub free_bytes: InventoryField<u64>,
    pub windows_health_status: InventoryField<String>,
    pub bitlocker_protection_status: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageInventory {
    pub devices: InventorySection<StorageDevice>,
    pub volumes: InventorySection<LogicalVolume>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsAdapter {
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub adapter_memory_bytes: InventoryField<u64>,
    pub driver_version: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayDevice {
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
    pub native_resolution: InventoryField<String>,
    pub current_resolution: InventoryField<String>,
    pub internal_display: InventoryField<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsInventory {
    pub adapters: InventorySection<GraphicsAdapter>,
    pub displays: InventorySection<DisplayDevice>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatteryProfile {
    pub present: InventoryField<bool>,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
    pub chemistry: InventoryField<String>,
    pub designed_capacity_mwh: InventoryField<u64>,
    pub full_charge_capacity_mwh: InventoryField<u64>,
    pub remaining_capacity_mwh: InventoryField<u64>,
    pub cycle_count: InventoryField<u32>,
    pub charge_status: InventoryField<String>,
    pub health_ratio: InventoryField<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PortCategory {
    UsbA,
    UsbC,
    Usb4OrThunderbolt,
    Hdmi,
    DisplayPort,
    Ethernet,
    Audio,
    Serial,
    Parallel,
    Docking,
    CardReader,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PortCountKind {
    PhysicalConnector,
    LogicalController,
    Hub,
    AttachedDevice,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortRecord {
    pub category: PortCategory,
    pub count_kind: PortCountKind,
    pub count: InventoryField<u32>,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub hardware_identifier: InventoryField<DeviceIdentifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SensorCategory {
    Accelerometer,
    Gyroscope,
    Orientation,
    AmbientLight,
    Proximity,
    MagnetometerOrCompass,
    LocationCapability,
    Lid,
    TabletMode,
    BiometricReader,
    Camera,
    Microphone,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorPresence {
    pub category: SensorCategory,
    pub present: InventoryField<bool>,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub hardware_identifier: InventoryField<DeviceIdentifier>,
    pub driver_provider: InventoryField<String>,
    pub driver_version: InventoryField<String>,
    pub availability: InventoryField<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NetworkCategory {
    Ethernet,
    Wifi,
    Bluetooth,
    CellularOrWwan,
    Nfc,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkAdapter {
    pub category: NetworkCategory,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub hardware_identifier: InventoryField<DeviceIdentifier>,
    pub mac_address: InventoryField<DeviceIdentifier>,
    pub driver_version: InventoryField<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PeripheralCategory {
    AudioController,
    ImagingDevice,
    Keyboard,
    Mouse,
    Touch,
    Pen,
    OpticalDrive,
    CardReader,
    DockingStation,
    Tpm,
    SmartCardReader,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeripheralDevice {
    pub category: PeripheralCategory,
    pub present: InventoryField<bool>,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub hardware_identifier: InventoryField<DeviceIdentifier>,
    pub driver_version: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HardwareInventoryV1 {
    pub schema_version: &'static str,
    pub collected_at_unix: u64,
    pub device_and_chassis: InventorySection<DeviceAndChassisIdentity>,
    pub firmware: InventorySection<FirmwareProfile>,
    pub processors: InventorySection<ProcessorProfile>,
    pub memory: MemoryInventory,
    pub storage: StorageInventory,
    pub graphics: GraphicsInventory,
    pub batteries: InventorySection<BatteryProfile>,
    pub ports: InventorySection<PortRecord>,
    pub sensors: InventorySection<SensorPresence>,
    pub network: InventorySection<NetworkAdapter>,
    pub peripherals: InventorySection<PeripheralDevice>,
}

impl HardwareInventoryV1 {
    /// Honest initial state used before passive collectors are implemented.
    #[must_use]
    pub fn not_collected(collected_at_unix: u64) -> Self {
        let provenance = Provenance::not_collected(collected_at_unix);

        Self {
            schema_version: SCHEMA_VERSION,
            collected_at_unix,
            device_and_chassis: InventorySection::not_reported(provenance.clone()),
            firmware: InventorySection::not_reported(provenance.clone()),
            processors: InventorySection::not_reported(provenance.clone()),
            memory: MemoryInventory {
                summary: InventorySection::not_reported(provenance.clone()),
                modules: InventorySection::not_reported(provenance.clone()),
            },
            storage: StorageInventory {
                devices: InventorySection::not_reported(provenance.clone()),
                volumes: InventorySection::not_reported(provenance.clone()),
            },
            graphics: GraphicsInventory {
                adapters: InventorySection::not_reported(provenance.clone()),
                displays: InventorySection::not_reported(provenance.clone()),
            },
            batteries: InventorySection::not_reported(provenance.clone()),
            ports: InventorySection::not_reported(provenance.clone()),
            sensors: InventorySection::not_reported(provenance.clone()),
            network: InventorySection::not_reported(provenance.clone()),
            peripherals: InventorySection::not_reported(provenance),
        }
    }
}

/// Derive the firmware-reported battery health ratio without testing the battery.
/// A missing or zero design capacity is `unknown`, never zero or failed.
#[must_use]
pub fn derive_battery_health_ratio(
    designed_capacity_mwh: &InventoryField<u64>,
    full_charge_capacity_mwh: &InventoryField<u64>,
    provenance: Provenance,
) -> InventoryField<f64> {
    match (designed_capacity_mwh.value, full_charge_capacity_mwh.value) {
        (Some(designed), Some(full_charge)) if designed > 0 && full_charge > 0 => {
            InventoryField::derived(
                full_charge as f64 / designed as f64,
                Confidence::Medium,
                PrivacyClass::OperationalMetadata,
                provenance,
            )
        }
        _ => InventoryField::unknown(PrivacyClass::OperationalMetadata, provenance),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_status_wire_values_are_stable() {
        let statuses = [
            (CollectionStatus::Reported, "reported"),
            (CollectionStatus::Observed, "observed"),
            (CollectionStatus::Derived, "derived"),
            (CollectionStatus::Unknown, "unknown"),
            (CollectionStatus::NotReported, "not_reported"),
            (CollectionStatus::NotApplicable, "not_applicable"),
            (CollectionStatus::PermissionDenied, "permission_denied"),
            (CollectionStatus::Unsupported, "unsupported"),
            (CollectionStatus::CollectionError, "collection_error"),
        ];

        for (status, expected) in statuses {
            assert_eq!(status.as_str(), expected);
        }
    }

    #[test]
    fn unknown_numeric_field_never_invents_zero() {
        let field = InventoryField::<u64>::unknown(
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );

        assert_eq!(field.value, None);
        assert_eq!(field.status, CollectionStatus::Unknown);
        assert!(field.is_consistent());
    }

    #[test]
    fn prohibited_secrets_fail_field_consistency() {
        let field = InventoryField::reported(
            "must-not-be-collected".to_string(),
            Confidence::High,
            PrivacyClass::ProhibitedSecret,
            Provenance::fixture(1_000),
        );

        assert!(!field.is_consistent());
    }

    #[test]
    fn device_identifier_debug_output_is_redacted() {
        let identifier = DeviceIdentifier::from_reported("SERIAL-123")
            .expect("fixture identifier must be accepted");

        assert_eq!(format!("{identifier:?}"), "<redacted-device-identifier>");
        assert!(!format!("{identifier:?}").contains("SERIAL-123"));
        assert_eq!(identifier.expose_for_authorized_use(), "SERIAL-123");
    }

    #[test]
    fn not_collected_inventory_is_explicit_and_empty() {
        let inventory = HardwareInventoryV1::not_collected(1_000);

        assert_eq!(inventory.schema_version, SCHEMA_VERSION);
        assert_eq!(
            inventory.device_and_chassis.status,
            CollectionStatus::NotReported
        );
        assert!(inventory.device_and_chassis.records.is_empty());
        assert_eq!(inventory.sensors.status, CollectionStatus::NotReported);
        assert!(inventory.sensors.records.is_empty());
    }

    #[test]
    fn battery_health_is_derived_only_from_reported_capacities() {
        let designed = InventoryField::reported(
            50_000_u64,
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );
        let full_charge = InventoryField::reported(
            40_000_u64,
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );

        let ratio =
            derive_battery_health_ratio(&designed, &full_charge, Provenance::fixture(1_000));

        assert_eq!(ratio.status, CollectionStatus::Derived);
        assert!((ratio.value.expect("derived ratio") - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn battery_health_is_unknown_without_design_capacity() {
        let designed = InventoryField::<u64>::unknown(
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );
        let full_charge = InventoryField::reported(
            40_000_u64,
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );

        let ratio =
            derive_battery_health_ratio(&designed, &full_charge, Provenance::fixture(1_000));

        assert_eq!(ratio.value, None);
        assert_eq!(ratio.status, CollectionStatus::Unknown);
    }

    #[test]
    fn battery_health_is_unknown_when_full_charge_is_zero() {
        let designed = InventoryField::reported(
            50_000_u64,
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );
        let full_charge = InventoryField::reported(
            0_u64,
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            Provenance::fixture(1_000),
        );

        let ratio =
            derive_battery_health_ratio(&designed, &full_charge, Provenance::fixture(1_000));

        assert_eq!(ratio.value, None);
        assert_eq!(ratio.status, CollectionStatus::Unknown);
    }
}
