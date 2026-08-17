use std::process::Command;

#[derive(Debug)]
pub struct DeviceIdentity {
    pub hostname: String,
    pub platform: &'static str,
    pub architecture: &'static str,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
}

pub fn collect() -> DeviceIdentity {
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let mut device = DeviceIdentity {
        hostname,
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        manufacturer: "unknown".to_string(),
        model: "unknown".to_string(),
        serial_number: "unknown".to_string(),
    };

    if !cfg!(target_os = "windows") {
        return device;
    }

    if let Some((manufacturer, model, serial_number)) = collect_windows_identity() {
        device.manufacturer = manufacturer;
        device.model = model;
        device.serial_number = serial_number;
    }

    device
}

fn collect_windows_identity() -> Option<(String, String, String)> {
    let script = r#"
$computer = Get-CimInstance -ClassName Win32_ComputerSystem
$bios = Get-CimInstance -ClassName Win32_BIOS

Write-Output ("manufacturer=" + [string]$computer.Manufacturer)
Write-Output ("model=" + [string]$computer.Model)
Write-Output ("serial_number=" + [string]$bios.SerialNumber)
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

    Some((
        get_value(&stdout, "manufacturer"),
        get_value(&stdout, "model"),
        get_value(&stdout, "serial_number"),
    ))
}

fn get_value(output: &str, key: &str) -> String {
    output
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once('=')?;

            if name.trim() == key {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}
