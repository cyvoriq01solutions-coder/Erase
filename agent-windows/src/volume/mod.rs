use std::process::Command;

#[derive(Debug, Clone)]
pub struct VolumeProfile {
    pub drive_letter: String,
    pub label: String,
    pub file_system: String,
    pub size_bytes: u64,
    pub free_bytes: u64,
    pub health_status: String,
    pub drive_kind: String,
}

pub fn collect() -> Vec<VolumeProfile> {
    if !cfg!(target_os = "windows") {
        return Vec::new();
    }

    collect_windows_volumes().unwrap_or_default()
}

fn collect_windows_volumes() -> Option<Vec<VolumeProfile>> {
    let script = r#"
try {
    $kinds = @{}
    Get-CimInstance Win32_LogicalDisk -ErrorAction SilentlyContinue | ForEach-Object {
        if ($_.DeviceID) {
            $kinds[$_.DeviceID.Substring(0,1).ToUpper()] = [int]$_.DriveType
        }
    }

    Get-Volume -ErrorAction Stop | ForEach-Object {
        $drive = if ($null -eq $_.DriveLetter) { "" } else { [string]$_.DriveLetter }
        $kindCode = 0
        if ($drive -ne "" -and $kinds.ContainsKey($drive.ToUpper())) {
            $kindCode = [int]$kinds[$drive.ToUpper()]
        }

        $fields = @(
            $drive,
            ([string]$_.FileSystemLabel).Replace("|"," "),
            ([string]$_.FileSystem).Replace("|"," "),
            [string]$_.Size,
            [string]$_.SizeRemaining,
            [string]$_.HealthStatus,
            [string]$kindCode
        )

        Write-Output ($fields -join "|")
    }
}
catch {
    exit 3
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

fn parse_volume(line: &str) -> Option<VolumeProfile> {
    let mut fields = line.splitn(7, '|');

    Some(VolumeProfile {
        drive_letter: clean(fields.next()?),
        label: clean(fields.next()?),
        file_system: clean(fields.next()?),
        size_bytes: fields.next()?.trim().parse().unwrap_or(0),
        free_bytes: fields.next()?.trim().parse().unwrap_or(0),
        health_status: clean(fields.next()?),
        drive_kind: drive_kind_from_code(fields.next().unwrap_or("0")),
    })
}

fn drive_kind_from_code(value: &str) -> String {
    match value.trim() {
        "2" => "removable".to_string(),
        "3" => "internal".to_string(),
        "4" => "network".to_string(),
        "5" => "optical".to_string(),
        _ => "other".to_string(),
    }
}

fn clean(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        "unknown".to_string()
    } else {
        value.to_string()
    }
}
