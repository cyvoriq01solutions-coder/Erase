use std::{
    fs,
    path::{Path, PathBuf},
};

const MAX_FILES_PER_LOCATION: u64 = 250_000;
const MAX_DIRECTORIES_PER_LOCATION: u64 = 50_000;

#[derive(Debug)]
pub struct ApplicationDataLocation {
    pub application: String,
    pub category: String,
    pub classification: String,
    pub path: String,
    pub file_count: u64,
    pub total_bytes: u64,
    pub risk: String,
    pub confidence: String,
    pub scan_status: String,
    pub content_inspected: bool,
}

#[derive(Debug)]
pub struct ApplicationDataInventory {
    pub discovery_status: String,
    pub content_inspected: bool,
    pub locations: Vec<ApplicationDataLocation>,
    pub inaccessible_entries: u64,
}

impl ApplicationDataInventory {
    fn not_windows() -> Self {
        Self {
            discovery_status: "not_windows".to_string(),
            content_inspected: false,
            locations: Vec::new(),
            inaccessible_entries: 0,
        }
    }
}

struct Candidate {
    application: &'static str,
    category: &'static str,
    classification: &'static str,
    path: PathBuf,
    risk: &'static str,
    confidence: &'static str,
}

pub fn collect(profile_paths: &[String]) -> ApplicationDataInventory {
    if !cfg!(target_os = "windows") {
        return ApplicationDataInventory::not_windows();
    }

    let mut candidates = Vec::<Candidate>::new();

    for profile in profile_paths {
        let base = PathBuf::from(profile);

        add_candidate(
            &mut candidates,
            "Google Chrome",
            "browser_profile",
            "application_data",
            base.join(r"AppData\Local\Google\Chrome\User Data"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Microsoft Edge",
            "browser_profile",
            "application_data",
            base.join(r"AppData\Local\Microsoft\Edge\User Data"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Mozilla Firefox",
            "browser_profile",
            "application_data",
            base.join(r"AppData\Roaming\Mozilla\Firefox\Profiles"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Microsoft Outlook",
            "email_store",
            "application_data",
            base.join(r"AppData\Local\Microsoft\Outlook"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Microsoft Outlook",
            "email_store",
            "confirmed_user_location",
            base.join(r"Documents\Outlook Files"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Mozilla Thunderbird",
            "email_profile",
            "application_data",
            base.join(r"AppData\Roaming\Thunderbird\Profiles"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Telegram Desktop",
            "messaging_data",
            "application_data",
            base.join(r"AppData\Roaming\Telegram Desktop\tdata"),
            "high",
            "high",
        );
        add_candidate(
            &mut candidates,
            "Microsoft Teams",
            "collaboration_data",
            "application_data",
            base.join(r"AppData\Roaming\Microsoft\Teams"),
            "high",
            "medium",
        );

        add_store_app_candidates(&base, &mut candidates);
    }

    for variable in ["OneDrive", "OneDriveCommercial", "OneDriveConsumer"] {
        if let Ok(value) = std::env::var(variable) {
            let value = value.trim();
            if !value.is_empty() {
                add_candidate(
                    &mut candidates,
                    "Microsoft OneDrive",
                    "cloud_sync",
                    "confirmed_user_location",
                    PathBuf::from(value),
                    "high",
                    "high",
                );
            }
        }
    }

    candidates.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.application.cmp(right.application))
    });
    candidates
        .dedup_by(|left, right| left.path == right.path && left.application == right.application);

    let mut locations = Vec::new();
    let mut inaccessible_entries = 0_u64;

    for candidate in candidates {
        if !candidate.path.exists() {
            continue;
        }

        let summary = summarize_location(&candidate.path, &mut inaccessible_entries);

        locations.push(ApplicationDataLocation {
            application: candidate.application.to_string(),
            category: candidate.category.to_string(),
            classification: candidate.classification.to_string(),
            path: candidate.path.display().to_string(),
            file_count: summary.file_count,
            total_bytes: summary.total_bytes,
            risk: candidate.risk.to_string(),
            confidence: candidate.confidence.to_string(),
            scan_status: summary.scan_status,
            content_inspected: false,
        });
    }

    ApplicationDataInventory {
        discovery_status: "completed".to_string(),
        content_inspected: false,
        locations,
        inaccessible_entries,
    }
}

fn add_candidate(
    candidates: &mut Vec<Candidate>,
    application: &'static str,
    category: &'static str,
    classification: &'static str,
    path: PathBuf,
    risk: &'static str,
    confidence: &'static str,
) {
    candidates.push(Candidate {
        application,
        category,
        classification,
        path,
        risk,
        confidence,
    });
}

fn add_store_app_candidates(base: &Path, candidates: &mut Vec<Candidate>) {
    let packages = base.join(r"AppData\Local\Packages");
    let Ok(entries) = fs::read_dir(packages) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();

        if name.contains("whatsapp") {
            add_candidate(
                candidates,
                "WhatsApp",
                "messaging_data",
                "application_data",
                path,
                "high",
                "high",
            );
        } else if name.contains("msteams") {
            add_candidate(
                candidates,
                "Microsoft Teams",
                "collaboration_data",
                "application_data",
                path,
                "high",
                "high",
            );
        } else if name.contains("windowscommunicationsapps") {
            add_candidate(
                candidates,
                "Windows Mail",
                "email_profile",
                "application_data",
                path,
                "high",
                "high",
            );
        } else if name.contains("outlookforwindows") {
            add_candidate(
                candidates,
                "Microsoft Outlook",
                "email_profile",
                "application_data",
                path,
                "high",
                "high",
            );
        }
    }
}

struct LocationSummary {
    file_count: u64,
    total_bytes: u64,
    scan_status: String,
}

fn summarize_location(path: &Path, inaccessible_entries: &mut u64) -> LocationSummary {
    let mut file_count = 0_u64;
    let mut directory_count = 0_u64;
    let mut total_bytes = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    let mut partial = false;
    let mut limit_reached = false;

    while let Some(directory) = stack.pop() {
        directory_count = directory_count.saturating_add(1);
        if directory_count > MAX_DIRECTORIES_PER_LOCATION {
            limit_reached = true;
            break;
        }

        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                *inaccessible_entries = inaccessible_entries.saturating_add(1);
                partial = true;
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    *inaccessible_entries = inaccessible_entries.saturating_add(1);
                    partial = true;
                    continue;
                }
            };

            let child = entry.path();
            let metadata = match fs::symlink_metadata(&child) {
                Ok(metadata) => metadata,
                Err(_) => {
                    *inaccessible_entries = inaccessible_entries.saturating_add(1);
                    partial = true;
                    continue;
                }
            };

            if metadata.file_type().is_symlink() {
                continue;
            }

            if metadata.is_dir() {
                stack.push(child);
                continue;
            }

            if !metadata.is_file() {
                continue;
            }

            file_count = file_count.saturating_add(1);
            total_bytes = total_bytes.saturating_add(metadata.len());

            if file_count >= MAX_FILES_PER_LOCATION {
                limit_reached = true;
                break;
            }
        }

        if limit_reached {
            break;
        }
    }

    let scan_status = if limit_reached {
        "partial_limit_reached"
    } else if partial {
        "partial"
    } else {
        "completed"
    };

    LocationSummary {
        file_count,
        total_bytes,
        scan_status: scan_status.to_string(),
    }
}
