use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct EvidenceRecord {
    pub collected_at_unix: u64,
    pub source: &'static str,
}

pub fn collect() -> EvidenceRecord {
    let collected_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    EvidenceRecord {
        collected_at_unix,
        source: "cyvoriq-local-agent",
    }
}
