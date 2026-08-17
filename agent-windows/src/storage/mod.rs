#[derive(Debug)]
pub struct StorageProfile {
    pub discovery_status: &'static str,
    pub destructive_operations_enabled: bool,
    pub note: &'static str,
}

pub fn collect() -> StorageProfile {
    #[cfg(target_os = "windows")]
    let note = "Windows storage discovery will be implemented in the next agent gate.";

    #[cfg(not(target_os = "windows"))]
    let note =
        "Windows hardware scanning is disabled because this executable is not running on Windows.";

    StorageProfile {
        discovery_status: "foundation_only",
        destructive_operations_enabled: false,
        note,
    }
}
