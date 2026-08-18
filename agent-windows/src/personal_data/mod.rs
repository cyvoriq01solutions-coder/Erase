mod types;

pub use types::{DataLocation, PersonalDataInventory};

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn collect(profile_paths: &[String], volume_roots: &[String]) -> PersonalDataInventory {
    if !cfg!(target_os = "windows") {
        return PersonalDataInventory::not_windows();
    }

    let mut scan_roots = Vec::<PathBuf>::new();

    for profile in profile_paths {
        let base = PathBuf::from(profile);

        for folder in [
            "Desktop",
            "Documents",
            "Downloads",
            "Pictures",
            "Videos",
            "Music",
            "OneDrive",
        ] {
            let candidate = base.join(folder);

            if candidate.exists() {
                scan_roots.push(candidate);
            }
        }
    }

    let system_drive = profile_paths
        .first()
        .and_then(|path| path.get(0..3))
        .map(|value| value.to_ascii_lowercase());

    for root in volume_roots {
        let root_path = PathBuf::from(root);

        let root_drive = root.get(0..3).map(|value| value.to_ascii_lowercase());

        if root_drive == system_drive {
            continue;
        }

        if root_path.exists() {
            add_data_drive_roots(&root_path, &mut scan_roots);
        }
    }

    scan_roots.sort();
    scan_roots.dedup();

    let mut locations = Vec::new();
    let mut inaccessible_entries = 0;

    for root in scan_roots {
        scan_root(&root, &mut locations, &mut inaccessible_entries);
    }

    PersonalDataInventory {
        discovery_status: "completed".to_string(),
        content_inspected: false,
        locations,
        inaccessible_entries,
    }
}

fn add_data_drive_roots(root: &Path, scan_roots: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        if should_skip_directory(&path) {
            continue;
        }

        scan_roots.push(path);
    }
}

fn scan_root(root: &Path, locations: &mut Vec<DataLocation>, inaccessible_entries: &mut u64) {
    let mut totals = BTreeMap::<String, (u64, u64)>::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(directory) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => {
                *inaccessible_entries += 1;
                continue;
            }
        };

        for entry in entries {
            let Ok(entry) = entry else {
                *inaccessible_entries += 1;
                continue;
            };

            let path = entry.path();

            let Ok(metadata) = fs::symlink_metadata(&path) else {
                *inaccessible_entries += 1;
                continue;
            };

            if metadata.file_type().is_symlink() {
                continue;
            }

            if metadata.is_dir() {
                if !should_skip_directory(&path) {
                    stack.push(path);
                }

                continue;
            }

            if !metadata.is_file() {
                continue;
            }

            let Some(category) = classify_file(&path) else {
                continue;
            };

            let total = totals.entry(category.to_string()).or_insert((0, 0));

            total.0 += 1;
            total.1 = total.1.saturating_add(metadata.len());
        }
    }

    for (category, (file_count, total_bytes)) in totals {
        locations.push(DataLocation {
            path: root.display().to_string(),
            category,
            file_count,
            total_bytes,
            scan_status: "completed".to_string(),
        });
    }
}

fn classify_file(path: &Path) -> Option<&'static str> {
    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();

    match extension.as_str() {
        "doc" | "docx" | "odt" | "rtf" | "txt" | "md" => Some("document"),

        "pdf" => Some("pdf"),

        "xls" | "xlsx" | "xlsm" | "csv" | "ods" => Some("spreadsheet"),

        "ppt" | "pptx" | "odp" => Some("presentation"),

        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tif" | "tiff" | "webp" | "heic" | "dng" => {
            Some("image")
        }

        "mp4" | "mov" | "avi" | "mkv" | "wmv" | "m4v" | "3gp" => Some("video"),

        "mp3" | "wav" | "m4a" | "aac" | "flac" | "ogg" | "wma" => Some("audio"),

        "zip" | "7z" | "rar" | "tar" | "gz" | "bz2" | "xz" => Some("archive"),

        "pst" | "ost" | "eml" | "msg" | "mbox" => Some("email"),

        "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" => Some("database"),

        "bak" | "backup" | "vhd" | "vhdx" => Some("backup"),

        _ => None,
    }
}

fn should_skip_directory(path: &Path) -> bool {
    let Some(name) = path.file_name() else {
        return false;
    };

    let name = name.to_string_lossy().to_ascii_lowercase();

    matches!(
        name.as_str(),
        "windows"
            | "program files"
            | "program files (x86)"
            | "programdata"
            | "$recycle.bin"
            | "system volume information"
            | "recovery"
            | "msocache"
    )
}
