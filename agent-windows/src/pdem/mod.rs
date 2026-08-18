#[derive(Debug)]
pub struct PdemObject {
    pub object_id: String,
    pub object_type: String,
    pub category: String,
    pub classification: String,
    pub source: String,
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
// Reserved by PDEM v1.2; relationships are populated after location discovery.
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

pub fn build(
    personal_data: &crate::personal_data::PersonalDataInventory,
    application_data: &crate::application_data::ApplicationDataInventory,
) -> PdemProfile {
    let mut objects = Vec::new();

    for location in &personal_data.locations {
        objects.push(PdemObject {
            object_id: format!("pdem-{:04}", objects.len() + 1),
            object_type: "data_location".to_string(),
            category: location.category.clone(),
            classification: location.classification.clone(),
            source: location.source.clone(),
            location: location.path.clone(),
            storage_scope: storage_scope(&location.path),
            file_count: location.file_count,
            total_bytes: location.total_bytes,
            risk: risk_for_location(&location.classification, &location.category).to_string(),
            confidence: location.confidence.clone(),
            coverage: "filesystem_metadata".to_string(),
            status: "detected".to_string(),
            content_inspected: false,
            discovery_method: "extension_and_location_context_metadata".to_string(),
        });
    }

    for location in &application_data.locations {
        objects.push(PdemObject {
            object_id: format!("pdem-{:04}", objects.len() + 1),
            object_type: "application_data_location".to_string(),
            category: location.category.clone(),
            classification: location.classification.clone(),
            source: location.application.clone(),
            location: location.path.clone(),
            storage_scope: storage_scope(&location.path),
            file_count: location.file_count,
            total_bytes: location.total_bytes,
            risk: location.risk.clone(),
            confidence: location.confidence.clone(),
            coverage: "known_application_path_metadata".to_string(),
            status: "detected".to_string(),
            content_inspected: location.content_inspected,
            discovery_method: "known_application_path_and_filesystem_metadata".to_string(),
        });
    }

    PdemProfile {
        schema_version: "pdem-1.2",
        collection_status: combined_status(personal_data, application_data),
        objects,
        relationships: Vec::new(),
    }
}

fn combined_status(
    personal_data: &crate::personal_data::PersonalDataInventory,
    application_data: &crate::application_data::ApplicationDataInventory,
) -> String {
    if personal_data.discovery_status == "not_windows"
        && application_data.discovery_status == "not_windows"
    {
        "not_windows".to_string()
    } else if personal_data.discovery_status == "completed"
        && application_data.discovery_status == "completed"
    {
        "completed".to_string()
    } else {
        "partial".to_string()
    }
}

fn storage_scope(path: &str) -> String {
    if path.len() >= 3 && path.as_bytes()[1] == b':' {
        path[..3].to_string()
    } else {
        "unknown".to_string()
    }
}

fn risk_for_location(classification: &str, category: &str) -> &'static str {
    match classification {
        "software_resource" => "low",
        "unknown" | "system_data" => "unknown",
        "confirmed_user_location" | "likely_personal_data" => risk_for_category(category),
        _ => "unknown",
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
