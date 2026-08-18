#[derive(Debug)]
pub struct PdemObject {
    pub object_id: String,
    pub object_type: String,
    pub category: String,
    pub location: String,
    pub storage_scope: String,
    pub file_count: u64,
    pub total_bytes: u64,
    pub risk: String,
    pub confidence: String,
    pub coverage: String,
    pub status: String,
    pub content_inspected: bool,
    pub discovery_method: String,
}

#[derive(Debug)]
// Reserved by PDEM v1.0; relationships are populated after location discovery.
#[allow(dead_code)]
pub struct PdemRelationship {
    pub from_object_id: String,
    pub relationship: String,
    pub to_object_id: String,
}

#[derive(Debug)]
pub struct PdemProfile {
    pub schema_version: &'static str,
    pub collection_status: String,
    pub objects: Vec<PdemObject>,
    pub relationships: Vec<PdemRelationship>,
}

pub fn build(personal_data: &crate::personal_data::PersonalDataInventory) -> PdemProfile {
    let objects = personal_data
        .locations
        .iter()
        .enumerate()
        .map(|(index, location)| PdemObject {
            object_id: format!("pdem-{:04}", index + 1),
            object_type: "data_location".to_string(),
            category: location.category.clone(),
            location: location.path.clone(),
            storage_scope: storage_scope(&location.path),
            file_count: location.file_count,
            total_bytes: location.total_bytes,
            risk: risk_for_category(&location.category).to_string(),
            confidence: "high".to_string(),
            coverage: "filesystem_metadata".to_string(),
            status: "detected".to_string(),
            content_inspected: false,
            discovery_method: "extension_and_filesystem_metadata".to_string(),
        })
        .collect();

    PdemProfile {
        schema_version: "pdem-1.0",
        collection_status: personal_data.discovery_status.clone(),
        objects,
        relationships: Vec::new(),
    }
}

fn storage_scope(path: &str) -> String {
    if path.len() >= 3 && path.as_bytes()[1] == b':' {
        path[..3].to_string()
    } else {
        "unknown".to_string()
    }
}

fn risk_for_category(category: &str) -> &'static str {
    match category {
        "email" | "database" | "backup" => "high",
        "document" | "pdf" | "spreadsheet" | "presentation" | "image" | "video" | "audio"
        | "archive" => "medium",
        _ => "unknown",
    }
}
