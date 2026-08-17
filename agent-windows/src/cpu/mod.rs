use std::process::Command;

#[derive(Debug)]
pub struct CpuProfile {
    pub name: String,
    pub manufacturer: String,
    pub cores: u32,
    pub logical_processors: u32,
    pub address_width: u32,
}

pub fn collect() -> CpuProfile {
    let mut profile = CpuProfile {
        name: "unknown".to_string(),
        manufacturer: "unknown".to_string(),
        cores: 0,
        logical_processors: 0,
        address_width: 0,
    };

    if cfg!(target_os = "windows")
        && let Some((name, manufacturer, cores, logical_processors, address_width)) =
            collect_windows_cpu()
    {
        profile.name = name;
        profile.manufacturer = manufacturer;
        profile.cores = cores;
        profile.logical_processors = logical_processors;
        profile.address_width = address_width;
    }

    profile
}

fn collect_windows_cpu() -> Option<(String, String, u32, u32, u32)> {
    let script = r#"
$cpu = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
Write-Output ("{0}|{1}|{2}|{3}|{4}" -f $cpu.Name,$cpu.Manufacturer,$cpu.NumberOfCores,$cpu.NumberOfLogicalProcessors,$cpu.AddressWidth)
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
    let mut values = stdout.trim().splitn(5, '|');

    let name = clean(values.next()?);
    let manufacturer = clean(values.next()?);
    let cores = values.next()?.trim().parse().unwrap_or(0);
    let logical_processors = values.next()?.trim().parse().unwrap_or(0);
    let address_width = values.next()?.trim().parse().unwrap_or(0);

    Some((name, manufacturer, cores, logical_processors, address_width))
}

fn clean(value: &str) -> String {
    let value = value.trim();

    if value.is_empty() {
        "unknown".to_string()
    } else {
        value.to_string()
    }
}
