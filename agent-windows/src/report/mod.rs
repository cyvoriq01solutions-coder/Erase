use crate::{
    assessment::AssessmentResult, device::DeviceIdentity, evidence::EvidenceRecord, os::OsProfile,
    storage::StorageProfile,
};

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

pub fn render(
    device: &DeviceIdentity,
    os: &OsProfile,
    storage: &StorageProfile,
    evidence: &EvidenceRecord,
    assessment: &AssessmentResult,
) -> String {
    format!(
        r#"{{
  "product": "CYVORIQ Verification Agent",
  "agentVersion": "0.1.0",
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
  "storage": {{
    "discoveryStatus": "{}",
    "destructiveOperationsEnabled": {},
    "note": "{}"
  }},
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
        escape_json(storage.discovery_status),
        storage.destructive_operations_enabled,
        escape_json(storage.note),
        evidence.collected_at_unix,
        escape_json(evidence.source),
        escape_json(assessment.status),
        escape_json(assessment.summary),
    )
}
