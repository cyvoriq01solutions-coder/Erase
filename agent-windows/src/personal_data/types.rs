#[derive(Debug)]
pub struct DataLocation {
    pub path: String,
    pub category: String,
    pub file_count: u64,
    pub total_bytes: u64,
    pub scan_status: String,
}

#[derive(Debug)]
pub struct PersonalDataInventory {
    pub discovery_status: String,
    pub content_inspected: bool,
    pub locations: Vec<DataLocation>,
    pub inaccessible_entries: u64,
}

impl PersonalDataInventory {
    pub fn not_windows() -> Self {
        Self {
            discovery_status: "not_windows".to_string(),
            content_inspected: false,
            locations: Vec::new(),
            inaccessible_entries: 0,
        }
    }
}
