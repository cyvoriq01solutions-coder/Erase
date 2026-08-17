use std::process::Command;

#[derive(Debug)]
pub struct VolumeProfile {
    pub drive_letter: String,
    pub label: String,
    pub file_system: String,
    pub size_bytes: u64,
    pub free_bytes: u64,
    pub health_status: String,
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
    Get-Volume -ErrorAction Stop | ForEach-Object {
        $drive = if ($null -eq $_.DriveLetter) { "" } else { [string]$_.DriveLetter }

        $fields = @(
            $drive,
            ([string]$_.FileSystemLabel).Replace("|"," "),
            ([string]$_.FileSystem).Replace("|"," "),
            [string]$_.Size,
            [string]$_.SizeRemaining,
            [string]$_.HealthStatus
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
    let mut fields = line.splitn(6, '|');

    Some(VolumeProfile {
        drive_letter: clean(fields.next()?),
        label: clean(fields.next()?),
        file_system: clean(fields.next()?),
        size_bytes: fields.next()?.trim().parse().unwrap_or(0),
        free_bytes: fields.next()?.trim().parse().unwrap_or(0),
        health_status: clean(fields.next()?),
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
