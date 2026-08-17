use std::process::Command;

#[derive(Debug)]
pub struct PhysicalDisk {
    pub index: u32,
    pub model: String,
    pub serial_number: String,
    pub size_bytes: u64,
    pub interface_type: String,
    pub media_type: String,
    pub firmware_revision: String,
}

#[derive(Debug)]
pub struct StorageProfile {
    pub discovery_status: String,
    pub destructive_operations_enabled: bool,
    pub note: String,
    pub disks: Vec<PhysicalDisk>,
}

pub fn collect() -> StorageProfile {
    if !cfg!(target_os = "windows") {
        return StorageProfile {
            discovery_status: "not_windows".to_string(),
            destructive_operations_enabled: false,
            note: "Windows physical-disk scanning is disabled because this executable is not running on Windows."
                .to_string(),
            disks: Vec::new(),
        };
    }

    match collect_windows_disks() {
        Some(disks) => StorageProfile {
            discovery_status: "completed".to_string(),
            destructive_operations_enabled: false,
            note: "Read-only Windows physical-disk inventory completed.".to_string(),
            disks,
        },
        None => StorageProfile {
            discovery_status: "collection_failed".to_string(),
            destructive_operations_enabled: false,
            note: "Windows physical-disk inventory could not be collected.".to_string(),
            disks: Vec::new(),
        },
    }
}

fn collect_windows_disks() -> Option<Vec<PhysicalDisk>> {
    let script = r#"
Get-CimInstance -ClassName Win32_DiskDrive | ForEach-Object {
    $fields = @(
        [string]$_.Index,
        ([string]$_.Model).Replace("|"," "),
        ([string]$_.SerialNumber).Replace("|"," "),
        [string]$_.Size,
        ([string]$_.InterfaceType).Replace("|"," "),
        ([string]$_.MediaType).Replace("|"," "),
        ([string]$_.FirmwareRevision).Replace("|"," ")
    )

    Write-Output ($fields -join "|")
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

    let disks = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(parse_disk)
        .collect();

    Some(disks)
}

fn parse_disk(line: &str) -> Option<PhysicalDisk> {
    let mut fields = line.splitn(7, '|');

    Some(PhysicalDisk {
        index: fields.next()?.trim().parse().unwrap_or(0),
        model: clean(fields.next()?),
        serial_number: clean(fields.next()?),
        size_bytes: fields.next()?.trim().parse().unwrap_or(0),
        interface_type: clean(fields.next()?),
        media_type: clean(fields.next()?),
        firmware_revision: clean(fields.next()?),
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
