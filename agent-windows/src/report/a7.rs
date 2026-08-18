use crate::{
    pdem::{PdemObject, PdemProfile},
    personal_data::{DataLocation, PersonalDataInventory},
    user_profiles::{UserProfile, UserProfileInventory},
};

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn render_profile(profile: &UserProfile) -> String {
    format!(
        r#"{{"sid":"{}","path":"{}","loaded":{},"special":{}}}"#,
        escape_json(&profile.sid),
        escape_json(&profile.path),
        profile.loaded,
        profile.special,
    )
}

pub fn render_user_profiles(inventory: &UserProfileInventory) -> String {
    let profiles = inventory
        .profiles
        .iter()
        .map(render_profile)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"discoveryStatus":"{}","currentUser":"{}","currentProfile":"{}","profiles":[{}]}}"#,
        escape_json(&inventory.discovery_status),
        escape_json(&inventory.current_user),
        escape_json(&inventory.current_profile),
        profiles,
    )
}

fn render_location(location: &DataLocation) -> String {
    format!(
        r#"{{"path":"{}","category":"{}","fileCount":{},"totalBytes":{},"scanStatus":"{}"}}"#,
        escape_json(&location.path),
        escape_json(&location.category),
        location.file_count,
        location.total_bytes,
        escape_json(&location.scan_status),
    )
}

pub fn render_personal_data(inventory: &PersonalDataInventory) -> String {
    let locations = inventory
        .locations
        .iter()
        .map(render_location)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"discoveryStatus":"{}","contentInspected":{},"inaccessibleEntries":{},"locations":[{}]}}"#,
        escape_json(&inventory.discovery_status),
        inventory.content_inspected,
        inventory.inaccessible_entries,
        locations,
    )
}

fn render_pdem_object(object: &PdemObject) -> String {
    format!(
        r#"{{"objectId":"{}","objectType":"{}","category":"{}","location":"{}","storageScope":"{}","fileCount":{},"totalBytes":{},"risk":"{}","confidence":"{}","coverage":"{}","status":"{}","contentInspected":{},"discoveryMethod":"{}"}}"#,
        escape_json(&object.object_id),
        escape_json(&object.object_type),
        escape_json(&object.category),
        escape_json(&object.location),
        escape_json(&object.storage_scope),
        object.file_count,
        object.total_bytes,
        escape_json(&object.risk),
        escape_json(&object.confidence),
        escape_json(&object.coverage),
        escape_json(&object.status),
        object.content_inspected,
        escape_json(&object.discovery_method),
    )
}

pub fn render_pdem(profile: &PdemProfile) -> String {
    let objects = profile
        .objects
        .iter()
        .map(render_pdem_object)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"schemaVersion":"{}","collectionStatus":"{}","objects":[{}],"relationshipCount":{}}}"#,
        escape_json(profile.schema_version),
        escape_json(&profile.collection_status),
        objects,
        profile.relationships.len(),
    )
}
