use crate::{
    encryption::{EncryptionProfile, EncryptionVolume},
    volume::VolumeProfile,
};

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn render_volume(volume: &VolumeProfile) -> String {
    format!(
        r#"{{
      "driveLetter": "{}",
      "label": "{}",
      "fileSystem": "{}",
      "sizeBytes": {},
      "freeBytes": {},
      "healthStatus": "{}"
    }}"#,
        escape_json(&volume.drive_letter),
        escape_json(&volume.label),
        escape_json(&volume.file_system),
        volume.size_bytes,
        volume.free_bytes,
        escape_json(&volume.health_status),
    )
}

pub fn render_volumes(volumes: &[VolumeProfile]) -> String {
    if volumes.is_empty() {
        return "[]".to_string();
    }

    let rendered = volumes
        .iter()
        .map(render_volume)
        .collect::<Vec<_>>()
        .join(",\n    ");

    format!("[\n    {rendered}\n  ]")
}

fn render_protector_types(types: &[String]) -> String {
    let values = types
        .iter()
        .map(|value| format!(r#""{}""#, escape_json(value)))
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{values}]")
}

fn render_encryption_volume(volume: &EncryptionVolume) -> String {
    let protectors = render_protector_types(&volume.key_protector_types);

    format!(
        r#"{{
      "mountPoint": "{}",
      "volumeStatus": "{}",
      "protectionStatus": "{}",
      "encryptionPercentage": {},
      "encryptionMethod": "{}",
      "keyProtectorTypes": {}
    }}"#,
        escape_json(&volume.mount_point),
        escape_json(&volume.volume_status),
        escape_json(&volume.protection_status),
        volume.encryption_percentage,
        escape_json(&volume.encryption_method),
        protectors,
    )
}

pub fn render_encryption(profile: &EncryptionProfile) -> String {
    let volumes = if profile.volumes.is_empty() {
        "[]".to_string()
    } else {
        let rendered = profile
            .volumes
            .iter()
            .map(render_encryption_volume)
            .collect::<Vec<_>>()
            .join(",\n      ");

        format!("[\n      {rendered}\n    ]")
    };

    format!(
        r#"{{
    "collectionStatus": "{}",
    "note": "{}",
    "volumes": {}
  }}"#,
        escape_json(&profile.collection_status),
        escape_json(&profile.note),
        volumes,
    )
}
