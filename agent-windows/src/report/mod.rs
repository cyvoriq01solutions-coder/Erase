mod a6;
mod a7;

use crate::{
    assessment::AssessmentResult,
    cpu::CpuProfile,
    device::DeviceIdentity,
    evidence::EvidenceRecord,
    os::OsProfile,
    storage::{PhysicalDisk, StorageProfile},
};
use a6::{render_encryption, render_volumes};
use a7::{render_application_data, render_pdem, render_personal_data, render_user_profiles};

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn render_disk(disk: &PhysicalDisk) -> String {
    format!(
        r#"{{
        "index": {},
        "model": "{}",
        "serialNumber": "{}",
        "sizeBytes": {},
        "interfaceType": "{}",
        "mediaType": "{}",
        "firmwareRevision": "{}"
      }}"#,
        disk.index,
        escape_json(&disk.model),
        escape_json(&disk.serial_number),
        disk.size_bytes,
        escape_json(&disk.interface_type),
        escape_json(&disk.media_type),
        escape_json(&disk.firmware_revision),
    )
}

fn render_disks(disks: &[PhysicalDisk]) -> String {
    if disks.is_empty() {
        return "[]".to_string();
    }

    let rendered = disks
        .iter()
        .map(render_disk)
        .collect::<Vec<_>>()
        .join(",\n      ");

    format!("[\n      {rendered}\n    ]")
}

pub struct A7Evidence<'a> {
    pub user_profiles: &'a crate::user_profiles::UserProfileInventory,
    pub personal_data: &'a crate::personal_data::PersonalDataInventory,
    pub application_data: &'a crate::application_data::ApplicationDataInventory,
    pub pdem: &'a crate::pdem::PdemProfile,
}

pub struct A6Evidence<'a> {
    pub volumes: &'a [crate::volume::VolumeProfile],
    pub encryption: &'a crate::encryption::EncryptionProfile,
}

pub struct ReportContext<'a> {
    pub a6: &'a A6Evidence<'a>,
    pub a7: &'a A7Evidence<'a>,
    pub evidence: &'a EvidenceRecord,
    pub assessment: &'a AssessmentResult,
}

pub fn render(
    device: &DeviceIdentity,
    os: &OsProfile,
    cpu: &CpuProfile,
    storage: &StorageProfile,
    context: &ReportContext,
) -> String {
    let a6 = context.a6;
    let a7 = context.a7;
    let evidence = context.evidence;
    let assessment = context.assessment;
    let physical_disks = render_disks(&storage.disks);
    let volume_data = render_volumes(a6.volumes);
    let encryption_data = render_encryption(a6.encryption);
    let user_profile_data = render_user_profiles(a7.user_profiles);
    let personal_data = render_personal_data(a7.personal_data);
    let application_data = render_application_data(a7.application_data);
    let pdem_data = render_pdem(a7.pdem);

    format!(
        r#"{{
  "product": "CYVRA Erase Verification",
  "agentVersion": "0.2.1",
  "scanMode": "{}",
  "device": {{
    "hostname": "{}",
    "platform": "{}",
    "architecture": "{}",
    "manufacturer": "{}",
    "model": "{}",
    "serialNumber": "{}"
  }},
  "operatingSystem": {{
    "name": "{}",
    "family": "{}",
    "architecture": "{}",
    "caption": "{}",
    "version": "{}",
    "buildNumber": "{}"
  }},
  "cpu": {{
    "name": "{}",
    "manufacturer": "{}",
    "cores": {},
    "logicalProcessors": {},
    "addressWidth": {}
  }},
  "storage": {{
    "discoveryStatus": "{}",
    "destructiveOperationsEnabled": {},
    "note": "{}",
    "physicalDisks": {}
  }},
  "volumes": {},
  "encryption": {},
  "userProfiles": {},
  "personalData": {},
  "applicationData": {},
  "pdem": {},
  "evidence": {{
    "collectedAtUnix": {},
    "source": "{}"
  }},
  "assessment": {{
    "status": "{}",
    "summary": "{}"
  }}
}}"#,
        escape_json(assessment.scan_mode),
        escape_json(&device.hostname),
        escape_json(device.platform),
        escape_json(device.architecture),
        escape_json(&device.manufacturer),
        escape_json(&device.model),
        escape_json(&device.serial_number),
        escape_json(os.operating_system),
        escape_json(os.family),
        escape_json(os.architecture),
        escape_json(&os.caption),
        escape_json(&os.version),
        escape_json(&os.build_number),
        escape_json(&cpu.name),
        escape_json(&cpu.manufacturer),
        cpu.cores,
        cpu.logical_processors,
        cpu.address_width,
        escape_json(&storage.discovery_status),
        storage.destructive_operations_enabled,
        escape_json(&storage.note),
        physical_disks,
        volume_data,
        encryption_data,
        user_profile_data,
        personal_data,
        application_data,
        pdem_data,
        evidence.collected_at_unix,
        escape_json(evidence.source),
        escape_json(assessment.status),
        escape_json(assessment.summary),
    )
}
