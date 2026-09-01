//! Shared hardware-validation engine used by the internal CLI and the customer GUI.
//! Does not spawn a process. Assessment only.

use crate::hardware_inventory_v1::{
    CollectionStatus, DeviceClassification, DeviceIdentifier, FirmwareMode, FormFactor,
    HardwareInventoryV1, InventoryField, PortCategory, SCHEMA_VERSION, SensorCategory,
};
#[cfg(target_os = "windows")]
use crate::{CollectorErrorKind, collector_runtime::CancellationToken};
use std::fmt::Display;

pub const VALIDATOR_NAME: &str = "cyvra_w1_3c_hardware_validation";

#[cfg(target_os = "windows")]
pub fn run_cli() -> i32 {
    let cancellation = CancellationToken::new();

    match crate::windows_hardware::try_collect(&cancellation) {
        Ok(inventory) => {
            let report = build_report(&inventory);
            for line in report.lines {
                println!("{line}");
            }
            if report.passed { 0 } else { 1 }
        }
        Err(error) => {
            println!("validator={VALIDATOR_NAME}");
            println!("validator_version={}", env!("CARGO_PKG_VERSION"));
            println!("mode=passive_read_only");
            println!("destructive_operations=false");
            println!("identifiers=redacted");
            println!("collector_error_kind={}", error_kind_name(error.kind));
            println!(
                "collector_error_message={}",
                flatten_text(&error.safe_message)
            );
            println!("result=fail");
            1
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn run_cli() -> i32 {
    eprintln!("This internal validation executable runs only on Windows.");
    2
}

pub struct ValidationReport {
    pub lines: Vec<String>,
    pub passed: bool,
}

pub fn build_report(inventory: &HardwareInventoryV1) -> ValidationReport {
    let field_consistency = requested_fields_are_consistent(inventory);
    let section_consistency = requested_sections_are_consistent(inventory);
    let requested_coverage = requested_sections_have_records(inventory);
    let deferred_untouched = deferred_sections_are_untouched(inventory);
    let schema_valid = inventory.schema_version == SCHEMA_VERSION;
    let timestamp_valid = inventory.collected_at_unix > 0;
    let passed = schema_valid
        && timestamp_valid
        && field_consistency
        && section_consistency
        && requested_coverage
        && deferred_untouched;

    let mut lines = vec![
        format!("validator={VALIDATOR_NAME}"),
        format!("validator_version={}", env!("CARGO_PKG_VERSION")),
        "mode=passive_read_only".to_string(),
        "destructive_operations=false".to_string(),
        "identifiers=redacted".to_string(),
        format!("schema_version={}", inventory.schema_version),
        format!("collected_at_unix={}", inventory.collected_at_unix),
        format!(
            "device_section_status={}",
            inventory.device_and_chassis.status.as_str()
        ),
        format!(
            "device_record_count={}",
            inventory.device_and_chassis.records.len()
        ),
    ];

    if let Some(device) = inventory.device_and_chassis.records.first() {
        lines.push(format!(
            "system_manufacturer={}",
            display_field(&device.system_manufacturer)
        ));
        lines.push(format!(
            "system_model={}",
            display_field(&device.system_model)
        ));
        lines.push(format!(
            "device_classification={}",
            display_mapped_field(&device.classification, classification_name)
        ));
        lines.push(format!(
            "form_factor={}",
            display_mapped_field(&device.form_factor, form_factor_name)
        ));
    }

    lines.push(format!(
        "firmware_section_status={}",
        inventory.firmware.status.as_str()
    ));
    lines.push(format!(
        "firmware_record_count={}",
        inventory.firmware.records.len()
    ));

    if let Some(firmware) = inventory.firmware.records.first() {
        lines.push(format!(
            "firmware_vendor={}",
            display_field(&firmware.vendor)
        ));
        lines.push(format!(
            "firmware_mode={}",
            display_mapped_field(&firmware.mode, firmware_mode_name)
        ));
        lines.push(format!(
            "secure_boot_present={}",
            display_field(&firmware.secure_boot_present)
        ));
        lines.push(format!(
            "secure_boot_enabled={}",
            display_field(&firmware.secure_boot_enabled)
        ));
        lines.push(format!(
            "tpm_present={}",
            display_field(&firmware.tpm_present)
        ));
        lines.push(format!(
            "tpm_specification={}",
            display_field(&firmware.tpm_specification_version)
        ));
    }

    lines.push(format!(
        "processor_section_status={}",
        inventory.processors.status.as_str()
    ));
    lines.push(format!(
        "processor_record_count={}",
        inventory.processors.records.len()
    ));

    for (index, processor) in inventory.processors.records.iter().enumerate() {
        lines.push(format!(
            "processor_{index}_manufacturer={}",
            display_field(&processor.manufacturer)
        ));
        lines.push(format!(
            "processor_{index}_model={}",
            display_field(&processor.model)
        ));
        lines.push(format!(
            "processor_{index}_architecture={}",
            display_field(&processor.architecture)
        ));
        lines.push(format!(
            "processor_{index}_physical_cores={}",
            display_field(&processor.physical_core_count)
        ));
        lines.push(format!(
            "processor_{index}_logical_processors={}",
            display_field(&processor.logical_processor_count)
        ));
    }

    lines.push(format!(
        "memory_summary_status={}",
        inventory.memory.summary.status.as_str()
    ));
    lines.push(format!(
        "memory_summary_record_count={}",
        inventory.memory.summary.records.len()
    ));

    if let Some(summary) = inventory.memory.summary.records.first() {
        lines.push(format!(
            "installed_physical_bytes={}",
            display_field(&summary.installed_physical_bytes)
        ));
        lines.push(format!(
            "visible_physical_bytes={}",
            display_field(&summary.visible_physical_bytes)
        ));
        lines.push(format!(
            "physical_slot_count={}",
            display_field(&summary.physical_slot_count)
        ));
        lines.push(format!(
            "populated_slot_count={}",
            display_field(&summary.populated_slot_count)
        ));
        lines.push(format!(
            "memory_error_correction={}",
            display_field(&summary.error_correction_capability)
        ));
    }

    lines.push(format!(
        "memory_modules_status={}",
        inventory.memory.modules.status.as_str()
    ));
    lines.push(format!(
        "memory_module_count={}",
        inventory.memory.modules.records.len()
    ));

    for (index, module) in inventory.memory.modules.records.iter().enumerate() {
        lines.push(format!(
            "memory_module_{index}_capacity_bytes={}",
            display_field(&module.capacity_bytes)
        ));
        lines.push(format!(
            "memory_module_{index}_speed_mhz={}",
            display_field(&module.speed_mhz)
        ));
        lines.push(format!(
            "memory_module_{index}_configured_speed_mhz={}",
            display_field(&module.configured_speed_mhz)
        ));
        lines.push(format!(
            "memory_module_{index}_type={}",
            display_field(&module.memory_type)
        ));
        lines.push(format!(
            "memory_module_{index}_form_factor={}",
            display_field(&module.form_factor)
        ));
        lines.push(format!(
            "memory_module_{index}_manufacturer={}",
            display_field(&module.manufacturer)
        ));
    }

    lines.extend([
        format!("requested_sections_consistent={section_consistency}"),
        format!("requested_fields_consistent={field_consistency}"),
        format!("requested_coverage_complete={requested_coverage}"),
        format!("deferred_sections_untouched={deferred_untouched}"),
        format!("result={}", if passed { "pass" } else { "fail" }),
    ]);

    ValidationReport { lines, passed }
}

pub fn render_text(inventory: &HardwareInventoryV1) -> String {
    build_report(inventory).lines.join("\n")
}

pub fn not_windows_text() -> String {
    [
        format!("validator={VALIDATOR_NAME}"),
        format!("validator_version={}", env!("CARGO_PKG_VERSION")),
        "mode=passive_read_only".to_string(),
        "destructive_operations=false".to_string(),
        "identifiers=redacted".to_string(),
        "result=not_windows".to_string(),
    ]
    .join("\n")
}

pub fn customer_hardware_fields(inventory: &HardwareInventoryV1) -> Vec<(String, String)> {
    let mut rows = Vec::new();

    if let Some(device) = inventory.device_and_chassis.records.first() {
        push_row(
            &mut rows,
            "Manufacturer",
            plain_field(&device.system_manufacturer),
        );
        push_row(&mut rows, "Model", plain_field(&device.system_model));
        push_row(
            &mut rows,
            "Device type",
            mapped_field(&device.classification, classification_name),
        );
        push_row(
            &mut rows,
            "Form factor",
            mapped_field(&device.form_factor, form_factor_name),
        );
        push_audit_identifier(
            &mut rows,
            "BIOS / OEM serial",
            identifier_field(&device.serial_number),
            true,
        );
        push_audit_identifier(
            &mut rows,
            "Chassis serial",
            identifier_field(&device.chassis_serial_number),
            true,
        );
        push_audit_identifier(
            &mut rows,
            "Motherboard serial",
            identifier_field(&device.baseboard_serial_number),
            true,
        );
        push_audit_identifier(
            &mut rows,
            "SMBIOS UUID",
            identifier_field(&device.system_uuid),
            false,
        );
        push_audit_identifier(
            &mut rows,
            "Asset tag",
            identifier_field(&device.asset_tag),
            false,
        );
    }

    if let Some(firmware) = inventory.firmware.records.first() {
        push_row(&mut rows, "Firmware vendor", plain_field(&firmware.vendor));
        push_row(
            &mut rows,
            "Firmware mode",
            mapped_field(&firmware.mode, firmware_mode_name),
        );
        push_row(
            &mut rows,
            "Secure Boot present",
            yes_no_field(&firmware.secure_boot_present),
        );
        push_row(
            &mut rows,
            "Secure Boot enabled",
            yes_no_field(&firmware.secure_boot_enabled),
        );
        push_row(
            &mut rows,
            "TPM present",
            yes_no_field(&firmware.tpm_present),
        );
        push_row(
            &mut rows,
            "TPM specification",
            plain_field(&firmware.tpm_specification_version),
        );
    }

    if let Some(processor) = inventory.processors.records.first() {
        push_row(&mut rows, "Processor", plain_field(&processor.model));
        push_row(
            &mut rows,
            "Processor manufacturer",
            plain_field(&processor.manufacturer),
        );
        push_row(
            &mut rows,
            "Physical cores",
            plain_field(&processor.physical_core_count),
        );
        push_row(
            &mut rows,
            "Logical processors",
            plain_field(&processor.logical_processor_count),
        );
    }

    if let Some(summary) = inventory.memory.summary.records.first() {
        push_row(
            &mut rows,
            "Installed memory",
            bytes_field(&summary.installed_physical_bytes),
        );
        push_row(
            &mut rows,
            "Visible memory",
            bytes_field(&summary.visible_physical_bytes),
        );
        push_row(
            &mut rows,
            "Memory slots used",
            slot_field(&summary.populated_slot_count, &summary.physical_slot_count),
        );
    }

    for module in &inventory.memory.modules.records {
        let Some(serial) = identifier_field(&module.serial_number) else {
            continue;
        };
        let locator = plain_field(&module.locator).unwrap_or_else(|| "module".to_string());
        push_row(
            &mut rows,
            &format!("Memory serial ({locator})"),
            Some(serial),
        );
    }

    append_battery_rows(&mut rows, inventory);
    append_sensor_rows(&mut rows, inventory);
    append_port_rows(&mut rows, inventory);

    rows
}

fn append_battery_rows(rows: &mut Vec<(String, String)>, inventory: &HardwareInventoryV1) {
    let Some(battery) = inventory.batteries.records.first() else {
        return;
    };
    push_row(rows, "Battery present", yes_no_field(&battery.present));
    if let Some(ratio) = battery
        .health_ratio
        .value
        .filter(|ratio| (0.01..=1.5).contains(ratio))
    {
        push_row(
            rows,
            "Battery health %",
            Some(format!(
                "{:.0}% full-charge vs design",
                (ratio * 100.0).round()
            )),
        );
    }
}

fn append_sensor_rows(rows: &mut Vec<(String, String)>, inventory: &HardwareInventoryV1) {
    push_enumerated_sensor(rows, inventory, SensorCategory::Camera, "Cameras");
    push_enumerated_sensor(rows, inventory, SensorCategory::Microphone, "Microphones");
}

fn push_enumerated_sensor(
    rows: &mut Vec<(String, String)>,
    inventory: &HardwareInventoryV1,
    category: SensorCategory,
    label: &str,
) {
    let matching = inventory
        .sensors
        .records
        .iter()
        .filter(|record| record.category == category)
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return;
    }
    let present = matching
        .iter()
        .filter(|record| record.present.value == Some(true))
        .collect::<Vec<_>>();
    if present.is_empty() {
        push_row(rows, label, Some("None enumerated by Windows".to_string()));
        return;
    }
    let names = present
        .iter()
        .filter_map(|record| plain_field(&record.model))
        .collect::<Vec<_>>();
    let value = if names.is_empty() {
        present.len().to_string()
    } else {
        format!("{} — {}", present.len(), names.join(", "))
    };
    push_row(rows, label, Some(value));
}

fn append_port_rows(rows: &mut Vec<(String, String)>, inventory: &HardwareInventoryV1) {
    let usb = port_count(inventory, PortCategory::UsbA)
        + port_count(inventory, PortCategory::UsbC)
        + port_count(inventory, PortCategory::Usb4OrThunderbolt);
    if usb > 0 {
        push_row(
            rows,
            "USB ports",
            Some(format!("{usb} firmware connectors")),
        );
    }
    push_port_row(rows, inventory, PortCategory::Hdmi, "HDMI ports");
    push_port_row(rows, inventory, PortCategory::DisplayPort, "DisplayPort");
    let jacks = inventory
        .ports
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.category,
                PortCategory::Ethernet | PortCategory::Audio
            ) && record.count.value.unwrap_or(0) > 0
        })
        .map(|record| record.count.value.unwrap_or(0))
        .sum::<u32>();
    if jacks > 0 {
        push_row(
            rows,
            "Ethernet / audio jacks",
            Some(format!("{jacks} firmware connectors")),
        );
    }
}

fn port_count(inventory: &HardwareInventoryV1, category: PortCategory) -> u32 {
    inventory
        .ports
        .records
        .iter()
        .filter(|record| record.category == category)
        .filter_map(|record| record.count.value)
        .sum()
}

fn push_port_row(
    rows: &mut Vec<(String, String)>,
    inventory: &HardwareInventoryV1,
    category: PortCategory,
    label: &str,
) {
    let count = inventory
        .ports
        .records
        .iter()
        .filter(|record| record.category == category)
        .filter_map(|record| record.count.value)
        .sum::<u32>();
    if count == 0 {
        return;
    }
    push_row(rows, label, Some(format!("{count} firmware connectors")));
}

const FIRMWARE_NOT_REPORTED: &str = "Not reported by firmware";

fn identifier_field(field: &InventoryField<DeviceIdentifier>) -> Option<String> {
    field
        .value
        .as_ref()
        .map(|value| flatten_text(value.expose_for_authorized_use()))
        .filter(|value| !value.is_empty())
}

fn push_audit_identifier(
    rows: &mut Vec<(String, String)>,
    label: &str,
    value: Option<String>,
    required: bool,
) {
    match value {
        Some(value) => rows.push((label.to_string(), value)),
        None if required => rows.push((label.to_string(), FIRMWARE_NOT_REPORTED.to_string())),
        None => {}
    }
}

fn push_row(rows: &mut Vec<(String, String)>, label: &str, value: Option<String>) {
    let Some(value) = value.filter(|value| !value.is_empty() && value.as_str() != "unknown") else {
        return;
    };
    rows.push((label.to_string(), value));
}

fn plain_field<T>(field: &InventoryField<T>) -> Option<String>
where
    T: Display,
{
    field
        .value
        .as_ref()
        .map(|value| flatten_text(&value.to_string()))
        .filter(|value| !value.is_empty())
}

fn mapped_field<T>(field: &InventoryField<T>, mapper: fn(T) -> &'static str) -> Option<String>
where
    T: Copy,
{
    field.value.map(mapper).map(|value| value.replace('_', " "))
}

fn yes_no_field(field: &InventoryField<bool>) -> Option<String> {
    field
        .value
        .map(|value| if value { "Yes" } else { "No" }.to_string())
}

fn bytes_field(field: &InventoryField<u64>) -> Option<String> {
    field.value.map(format_bytes)
}

fn slot_field(populated: &InventoryField<u32>, physical: &InventoryField<u32>) -> Option<String> {
    match (populated.value, physical.value) {
        (Some(used), Some(total)) => Some(format!("{used} of {total}")),
        (Some(used), None) => Some(used.to_string()),
        _ => None,
    }
}

fn format_bytes(bytes: u64) -> String {
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;
    if bytes as f64 >= GB {
        format!("{:.1} GB", bytes as f64 / GB)
    } else if bytes as f64 >= MB {
        format!("{:.0} MB", bytes as f64 / MB)
    } else {
        format!("{bytes} bytes")
    }
}

fn requested_sections_are_consistent(inventory: &HardwareInventoryV1) -> bool {
    inventory.device_and_chassis.is_consistent()
        && inventory.firmware.is_consistent()
        && inventory.processors.is_consistent()
        && inventory.memory.summary.is_consistent()
        && inventory.memory.modules.is_consistent()
}

fn requested_sections_have_records(inventory: &HardwareInventoryV1) -> bool {
    inventory.device_and_chassis.status == CollectionStatus::Reported
        && !inventory.device_and_chassis.records.is_empty()
        && inventory.firmware.status == CollectionStatus::Reported
        && !inventory.firmware.records.is_empty()
        && inventory.processors.status == CollectionStatus::Reported
        && !inventory.processors.records.is_empty()
        && inventory.memory.summary.status == CollectionStatus::Reported
        && !inventory.memory.summary.records.is_empty()
        && inventory.memory.modules.status == CollectionStatus::Reported
        && !inventory.memory.modules.records.is_empty()
}

fn requested_fields_are_consistent(inventory: &HardwareInventoryV1) -> bool {
    inventory
        .device_and_chassis
        .records
        .iter()
        .all(device_fields_are_consistent)
        && inventory
            .firmware
            .records
            .iter()
            .all(firmware_fields_are_consistent)
        && inventory
            .processors
            .records
            .iter()
            .all(processor_fields_are_consistent)
        && inventory
            .memory
            .summary
            .records
            .iter()
            .all(memory_summary_fields_are_consistent)
        && inventory
            .memory
            .modules
            .records
            .iter()
            .all(memory_module_fields_are_consistent)
}

fn device_fields_are_consistent(
    record: &crate::hardware_inventory_v1::DeviceAndChassisIdentity,
) -> bool {
    record.system_manufacturer.is_consistent()
        && record.system_model.is_consistent()
        && record.system_family.is_consistent()
        && record.serial_number.is_consistent()
        && record.system_uuid.is_consistent()
        && record.chassis_manufacturer.is_consistent()
        && record.chassis_type.is_consistent()
        && record.chassis_serial_number.is_consistent()
        && record.baseboard_manufacturer.is_consistent()
        && record.baseboard_product.is_consistent()
        && record.baseboard_version.is_consistent()
        && record.baseboard_serial_number.is_consistent()
        && record.asset_tag.is_consistent()
        && record.form_factor.is_consistent()
        && record.classification.is_consistent()
}

fn firmware_fields_are_consistent(record: &crate::hardware_inventory_v1::FirmwareProfile) -> bool {
    record.vendor.is_consistent()
        && record.version.is_consistent()
        && record.release_date.is_consistent()
        && record.smbios_version.is_consistent()
        && record.mode.is_consistent()
        && record.secure_boot_present.is_consistent()
        && record.secure_boot_enabled.is_consistent()
        && record.tpm_present.is_consistent()
        && record.tpm_specification_version.is_consistent()
        && record.virtualization_indicator.is_consistent()
}

fn processor_fields_are_consistent(
    record: &crate::hardware_inventory_v1::ProcessorProfile,
) -> bool {
    record.manufacturer.is_consistent()
        && record.model.is_consistent()
        && record.architecture.is_consistent()
        && record.physical_package_count.is_consistent()
        && record.physical_core_count.is_consistent()
        && record.logical_processor_count.is_consistent()
        && record.maximum_clock_mhz.is_consistent()
        && record.current_clock_mhz.is_consistent()
        && record.address_width_bits.is_consistent()
        && record.virtualization_capable.is_consistent()
}

fn memory_summary_fields_are_consistent(
    record: &crate::hardware_inventory_v1::MemorySummary,
) -> bool {
    record.installed_physical_bytes.is_consistent()
        && record.visible_physical_bytes.is_consistent()
        && record.physical_slot_count.is_consistent()
        && record.populated_slot_count.is_consistent()
        && record.error_correction_capability.is_consistent()
}

fn memory_module_fields_are_consistent(
    record: &crate::hardware_inventory_v1::MemoryModule,
) -> bool {
    record.locator.is_consistent()
        && record.capacity_bytes.is_consistent()
        && record.speed_mhz.is_consistent()
        && record.configured_speed_mhz.is_consistent()
        && record.memory_type.is_consistent()
        && record.form_factor.is_consistent()
        && record.manufacturer.is_consistent()
        && record.part_number.is_consistent()
        && record.serial_number.is_consistent()
}

fn deferred_sections_are_untouched(inventory: &HardwareInventoryV1) -> bool {
    section_is_not_reported(
        inventory.storage.devices.status,
        inventory.storage.devices.records.len(),
    ) && section_is_not_reported(
        inventory.storage.volumes.status,
        inventory.storage.volumes.records.len(),
    ) && section_is_not_reported(
        inventory.graphics.adapters.status,
        inventory.graphics.adapters.records.len(),
    ) && section_is_not_reported(
        inventory.graphics.displays.status,
        inventory.graphics.displays.records.len(),
    ) && section_is_not_reported(inventory.network.status, inventory.network.records.len())
        && section_is_not_reported(
            inventory.peripherals.status,
            inventory.peripherals.records.len(),
        )
}

fn section_is_not_reported(status: CollectionStatus, records: usize) -> bool {
    status == CollectionStatus::NotReported && records == 0
}

fn display_field<T>(field: &InventoryField<T>) -> String
where
    T: Display,
{
    match &field.value {
        Some(value) => format!(
            "{}|{}",
            field.status.as_str(),
            flatten_text(&value.to_string())
        ),
        None => field.status.as_str().to_string(),
    }
}

fn display_mapped_field<T>(field: &InventoryField<T>, mapper: fn(T) -> &'static str) -> String
where
    T: Copy,
{
    match field.value {
        Some(value) => format!("{}|{}", field.status.as_str(), mapper(value)),
        None => field.status.as_str().to_string(),
    }
}

fn flatten_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() || matches!(character, '=' | '|') {
                ' '
            } else {
                character
            }
        })
        .take(160)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn classification_name(value: DeviceClassification) -> &'static str {
    match value {
        DeviceClassification::OemReported => "oem_reported",
        DeviceClassification::CustomOrUnidentified => "custom_or_unidentified",
        DeviceClassification::Virtual => "virtual",
        DeviceClassification::ConflictingFirmwareData => "conflicting_firmware_data",
        DeviceClassification::Unknown => "unknown",
    }
}

fn form_factor_name(value: FormFactor) -> &'static str {
    match value {
        FormFactor::Desktop => "desktop",
        FormFactor::Laptop => "laptop",
        FormFactor::Tablet => "tablet",
        FormFactor::VirtualMachine => "virtual_machine",
        FormFactor::Unknown => "unknown",
    }
}

fn firmware_mode_name(value: FirmwareMode) -> &'static str {
    match value {
        FirmwareMode::Uefi => "uefi",
        FirmwareMode::LegacyBios => "legacy_bios",
        FirmwareMode::Unknown => "unknown",
    }
}

#[cfg(target_os = "windows")]
fn error_kind_name(value: CollectorErrorKind) -> &'static str {
    match value {
        CollectorErrorKind::Unsupported => "unsupported",
        CollectorErrorKind::PermissionDenied => "permission_denied",
        CollectorErrorKind::CommandFailed => "command_failed",
        CollectorErrorKind::ParseFailed => "parse_failed",
        CollectorErrorKind::Cancelled => "cancelled",
        CollectorErrorKind::TimedOut => "timed_out",
        CollectorErrorKind::OutputLimitExceeded => "output_limit_exceeded",
        CollectorErrorKind::Internal => "internal",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware_inventory_v1::{
        Confidence, DeviceAndChassisIdentity, DeviceIdentifier, InventorySection, PrivacyClass,
        Provenance,
    };

    #[test]
    fn not_collected_inventory_fails_validation_explicitly() {
        let report = build_report(&HardwareInventoryV1::not_collected(1_000));
        let output = report.lines.join("\n");

        assert!(!report.passed);
        assert!(output.contains("requested_coverage_complete=false"));
        assert!(output.contains("result=fail"));
        assert!(output.contains("destructive_operations=false"));
    }

    #[test]
    fn report_never_emits_device_identifiers() {
        let mut inventory = HardwareInventoryV1::not_collected(1_000);
        let provenance = Provenance::not_collected(1_000);
        let unknown_text =
            || InventoryField::<String>::unknown(PrivacyClass::NonSensitive, provenance.clone());
        let unknown_identifier = || {
            InventoryField::<DeviceIdentifier>::unknown(
                PrivacyClass::DeviceIdentifier,
                provenance.clone(),
            )
        };

        inventory.device_and_chassis = InventorySection {
            status: CollectionStatus::Reported,
            provenance: provenance.clone(),
            records: vec![DeviceAndChassisIdentity {
                system_manufacturer: unknown_text(),
                system_model: unknown_text(),
                system_family: unknown_text(),
                serial_number: InventoryField::reported(
                    DeviceIdentifier::from_reported("DO-NOT-PRINT-SERIAL")
                        .expect("test identifier"),
                    Confidence::High,
                    PrivacyClass::DeviceIdentifier,
                    provenance.clone(),
                ),
                system_uuid: unknown_identifier(),
                chassis_manufacturer: unknown_text(),
                chassis_type: unknown_text(),
                chassis_serial_number: unknown_identifier(),
                baseboard_manufacturer: unknown_text(),
                baseboard_product: unknown_text(),
                baseboard_version: unknown_text(),
                baseboard_serial_number: unknown_identifier(),
                asset_tag: unknown_identifier(),
                form_factor: InventoryField::unknown(
                    PrivacyClass::NonSensitive,
                    provenance.clone(),
                ),
                classification: InventoryField::unknown(
                    PrivacyClass::NonSensitive,
                    provenance.clone(),
                ),
            }],
        };

        let output = build_report(&inventory).lines.join("\n");
        assert!(output.contains("identifiers=redacted"));
        assert!(!output.contains("DO-NOT-PRINT-SERIAL"));

        let customer = customer_hardware_fields(&inventory);
        assert!(
            customer.iter().any(|(label, value)| {
                label == "BIOS / OEM serial" && value == "DO-NOT-PRINT-SERIAL"
            }),
            "{customer:?}"
        );
    }

    #[test]
    fn customer_report_exposes_authorized_serials() {
        let mut inventory = HardwareInventoryV1::not_collected(1_000);
        let provenance = Provenance::not_collected(1_000);
        let unknown_text =
            || InventoryField::<String>::unknown(PrivacyClass::NonSensitive, provenance.clone());
        let unknown_identifier = || {
            InventoryField::<DeviceIdentifier>::unknown(
                PrivacyClass::DeviceIdentifier,
                provenance.clone(),
            )
        };
        inventory.device_and_chassis = InventorySection {
            status: CollectionStatus::Reported,
            provenance: provenance.clone(),
            records: vec![DeviceAndChassisIdentity {
                system_manufacturer: unknown_text(),
                system_model: unknown_text(),
                system_family: unknown_text(),
                serial_number: InventoryField::reported(
                    DeviceIdentifier::from_reported("SAMPLE-BIOS-01").expect("test identifier"),
                    Confidence::High,
                    PrivacyClass::DeviceIdentifier,
                    provenance.clone(),
                ),
                system_uuid: unknown_identifier(),
                chassis_manufacturer: unknown_text(),
                chassis_type: unknown_text(),
                chassis_serial_number: unknown_identifier(),
                baseboard_manufacturer: unknown_text(),
                baseboard_product: unknown_text(),
                baseboard_version: unknown_text(),
                baseboard_serial_number: InventoryField::reported(
                    DeviceIdentifier::from_reported("BOARD-88").expect("test identifier"),
                    Confidence::High,
                    PrivacyClass::DeviceIdentifier,
                    provenance.clone(),
                ),
                asset_tag: unknown_identifier(),
                form_factor: InventoryField::unknown(
                    PrivacyClass::NonSensitive,
                    provenance.clone(),
                ),
                classification: InventoryField::unknown(
                    PrivacyClass::NonSensitive,
                    provenance.clone(),
                ),
            }],
        };

        let customer = customer_hardware_fields(&inventory);
        assert!(
            customer.iter().any(|(label, value)| {
                label == "BIOS / OEM serial" && value == "SAMPLE-BIOS-01"
            })
        );
        assert!(
            customer
                .iter()
                .any(|(label, value)| { label == "Motherboard serial" && value == "BOARD-88" })
        );
        assert!(
            customer.iter().any(|(label, value)| {
                label == "Chassis serial" && value == FIRMWARE_NOT_REPORTED
            })
        );
        assert!(!customer.iter().any(|(label, _)| label == "SMBIOS UUID"));
    }

    #[test]
    fn diagnostic_text_is_single_line_and_bounded() {
        let value = format!("alpha=beta|gamma\n{}", "x".repeat(200));
        let flattened = flatten_text(&value);

        assert!(!flattened.contains('\n'));
        assert!(!flattened.contains('='));
        assert!(!flattened.contains('|'));
        assert!(flattened.chars().count() <= 160);
    }
}
