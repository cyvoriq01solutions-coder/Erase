use std::process::Command;

#[derive(Debug)]
pub struct OsProfile {
    pub operating_system: &'static str,
    pub family: &'static str,
    pub architecture: &'static str,
    pub caption: String,
    pub version: String,
    pub build_number: String,
}

pub fn collect() -> OsProfile {
    let mut profile = OsProfile {
        operating_system: std::env::consts::OS,
        family: std::env::consts::FAMILY,
        architecture: std::env::consts::ARCH,
        caption: "unknown".to_string(),
        version: "unknown".to_string(),
        build_number: "unknown".to_string(),
    };

    if cfg!(target_os = "windows")
        && let Some((caption, version, build)) = collect_windows_os()
    {
        profile.caption = caption;
        profile.version = version;
        profile.build_number = build;
    }

    profile
}

fn collect_windows_os() -> Option<(String, String, String)> {
    let script = r#"
$os = Get-CimInstance -ClassName Win32_OperatingSystem
Write-Output ("{0}|{1}|{2}" -f $os.Caption,$os.Version,$os.BuildNumber)
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
    let mut values = stdout.trim().splitn(3, '|');

    Some((
        clean(values.next()?),
        clean(values.next()?),
        clean(values.next()?),
    ))
}

fn clean(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        "unknown".to_string()
    } else {
        value.to_string()
    }
}
