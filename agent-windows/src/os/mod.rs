#[derive(Debug)]
pub struct OsProfile {
    pub operating_system: &'static str,
    pub family: &'static str,
    pub architecture: &'static str,
}

pub fn collect() -> OsProfile {
    OsProfile {
        operating_system: std::env::consts::OS,
        family: std::env::consts::FAMILY,
        architecture: std::env::consts::ARCH,
    }
}
