use std::process::Command;

#[derive(Debug)]
pub struct EncryptionVolume {
    pub mount_point: String,
    pub volume_status: String,
    pub protection_status: String,
    pub encryption_percentage: u32,
    pub encryption_method: String,
    pub key_protector_types: Vec<String>,
}

#[derive(Debug)]
pub struct EncryptionProfile {
    pub collection_status: String,
    pub note: String,
    pub volumes: Vec<EncryptionVolume>,
}

pub fn collect() -> EncryptionProfile {
    if !cfg!(target_os = "windows") {
        return EncryptionProfile {
            collection_status: "not_windows".to_string(),
            note: "BitLocker discovery is available only on Windows.".to_string(),
            volumes: Vec::new(),
        };
    }

    if matches!(is_windows_elevated(), Some(false)) {
        return EncryptionProfile {
            collection_status: "requires_elevation".to_string(),
            note: "Administrator privileges are required to collect BitLocker status on this Windows device.".to_string(),
            volumes: Vec::new(),
        };
    }

    match collect_windows_bitlocker() {
        Some(volumes) => EncryptionProfile {
            collection_status: "completed".to_string(),
            note: "Only BitLocker status and protector types are collected. No recovery passwords or key material are collected."
                .to_string(),
            volumes,
        },

        None => EncryptionProfile {
            collection_status: "unavailable".to_string(),
            note: "BitLocker information could not be collected.".to_string(),
            volumes: Vec::new(),
        },
    }
}

fn collect_windows_bitlocker() -> Option<Vec<EncryptionVolume>> {
    let script = r#"
if ($null -eq (Get-Command Get-BitLockerVolume -ErrorAction SilentlyContinue)) {
    exit 4
}

try {
    Get-BitLockerVolume -ErrorAction Stop | ForEach-Object {
        $protectorTypes = @(
            $_.KeyProtector | ForEach-Object {
                [string]$_.KeyProtectorType
            }
        ) -join ","

        $fields = @(
            ([string]$_.MountPoint).Replace("|"," "),
            [string]$_.VolumeStatus,
            [string]$_.ProtectionStatus,
            [string]$_.EncryptionPercentage,
            [string]$_.EncryptionMethod,
            $protectorTypes
        )

        Write-Output ($fields -join "|")
    }
}
catch {
    exit 5
}
"#;

    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    Some(
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(parse_volume)
            .collect(),
    )
}

fn parse_volume(line: &str) -> Option<EncryptionVolume> {
    let mut fields = line.splitn(6, '|');

    let mount_point = clean(fields.next()?);
    let volume_status = clean(fields.next()?);
    let protection_status = clean(fields.next()?);

    let encryption_percentage = fields.next()?.trim().parse().unwrap_or(0);

    let encryption_method = clean(fields.next()?);

    let key_protector_types = fields
        .next()?
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    Some(EncryptionVolume {
        mount_point,
        volume_status,
        protection_status,
        encryption_percentage,
        encryption_method,
        key_protector_types,
    })
}

fn clean(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        "unknown".to_string()
    } else {
        value.to_string()
    }
}

fn is_windows_elevated() -> Option<bool> {
    let script = r#"
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)

if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Output "true"
} else {
    Write-Output "false"
}
"#;

    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    match String::from_utf8_lossy(&output.stdout).trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}
