mod a6;

use crate::{
    assessment::AssessmentResult,
    cpu::CpuProfile,
    device::DeviceIdentity,
    evidence::EvidenceRecord,
    os::OsProfile,
    storage::{PhysicalDisk, StorageProfile},
};
use a6::{render_encryption, render_volumes};

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

pub struct A6Evidence<'a> {
    pub volumes: &'a [crate::volume::VolumeProfile],
    pub encryption: &'a crate::encryption::EncryptionProfile,
}

pub fn render(
    device: &DeviceIdentity,
    os: &OsProfile,
    cpu: &CpuProfile,
    storage: &StorageProfile,
    a6: &A6Evidence,
    evidence: &EvidenceRecord,
    assessment: &AssessmentResult,
) -> String {
    let physical_disks = render_disks(&storage.disks);
    let volume_data = render_volumes(a6.volumes);
    let encryption_data = render_encryption(a6.encryption);

    format!(
        r#"{{
  "product": "CYVORIQ Verification Agent",
  "agentVersion": "0.1.2",
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
        evidence.collected_at_unix,
        escape_json(evidence.source),
        escape_json(assessment.status),
        escape_json(assessment.summary),
    )
}
