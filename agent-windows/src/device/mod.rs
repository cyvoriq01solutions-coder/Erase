#[derive(Debug)]
pub struct DeviceIdentity {
    pub hostname: String,
    pub platform: &'static str,
    pub architecture: &'static str,
}

pub fn collect() -> DeviceIdentity {
    let hostname = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    DeviceIdentity {
        hostname,
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
    }
}
