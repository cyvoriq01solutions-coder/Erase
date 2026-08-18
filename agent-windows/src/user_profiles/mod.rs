use std::process::Command;

#[derive(Debug)]
pub struct UserProfile {
    pub sid: String,
    pub path: String,
    pub loaded: bool,
    pub special: bool,
}

#[derive(Debug)]
pub struct UserProfileInventory {
    pub discovery_status: String,
    pub current_user: String,
    pub current_profile: String,
    pub profiles: Vec<UserProfile>,
}

pub fn collect() -> UserProfileInventory {
    if !cfg!(target_os = "windows") {
        return UserProfileInventory {
            discovery_status: "not_windows".to_string(),
            current_user: "unknown".to_string(),
            current_profile: "unknown".to_string(),
            profiles: Vec::new(),
        };
    }

    let current_user = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());

    let current_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "unknown".to_string());

    UserProfileInventory {
        discovery_status: "completed".to_string(),
        current_user,
        current_profile,
        profiles: collect_windows_profiles().unwrap_or_default(),
    }
}

fn collect_windows_profiles() -> Option<Vec<UserProfile>> {
    let script = r#"
Get-CimInstance Win32_UserProfile |
Where-Object { $_.LocalPath } |
ForEach-Object {
    $fields = @(
        ([string]$_.SID).Replace("|"," "),
        ([string]$_.LocalPath).Replace("|"," "),
        [string]$_.Loaded,
        [string]$_.Special
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

    Some(
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(parse_profile)
            .collect(),
    )
}

fn parse_profile(line: &str) -> Option<UserProfile> {
    let mut fields = line.splitn(4, '|');

    Some(UserProfile {
        sid: clean(fields.next()?),
        path: clean(fields.next()?),
        loaded: parse_bool(fields.next()?),
        special: parse_bool(fields.next()?),
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

fn parse_bool(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case("true")
}
