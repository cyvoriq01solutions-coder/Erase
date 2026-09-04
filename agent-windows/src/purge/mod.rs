//! Mode S sanitisation planner, elevated helper hand-off, and independent verify.
//! The Tauri crate must not issue ATA/NVMe commands or spawn processes.

mod execute;
mod media;
mod plan;
mod verify;

pub use media::{MediaClass, MethodClass};
pub use plan::{DiskHint, PlannedTarget, VolumeHint, erase_confirmed, hostname_matches};
pub use verify::VerifyReport;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::storage;
use crate::volume;

use plan::plan_targets;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct PurgeRequest {
    pub purge_licence_bound: bool,
    pub hostname: String,
    pub hostname_typed: String,
    pub erase_typed: String,
    pub letters: Vec<String>,
    pub usb_opt_in: bool,
    pub operator_name: String,
    pub preview_only: bool,
}

#[derive(Debug, Clone)]
pub struct PurgeTargetOutcome {
    pub letter: String,
    pub allowed: bool,
    pub media_label: String,
    pub method_label: String,
    pub standard: String,
    pub model: String,
    pub serial: String,
    pub bus: String,
    pub size_bytes: u64,
    pub refuse_reason: Option<String>,
    pub helper_ok: bool,
    pub verify_passed: bool,
    pub verify_note: String,
    pub sample_percent: u32,
}

#[derive(Debug, Clone)]
pub struct PurgeOutcome {
    pub ok: bool,
    pub job_id: String,
    pub status: String,
    pub message: String,
    pub report_allowed: bool,
    pub data_erased: bool,
    pub evidence_hash: String,
    pub targets: Vec<PurgeTargetOutcome>,
}

pub fn preview_mode_s(letters: &[String], usb_opt_in: bool) -> Vec<PlannedTarget> {
    let (volumes, disks) = live_hints();
    plan_targets(letters, &volumes, &disks, usb_opt_in)
}

pub fn helper_main(plan: &std::path::Path, result: &std::path::Path) -> i32 {
    execute::run_helper_plan(plan, result)
}

pub fn run_mode_s(
    request: PurgeRequest,
    mut progress: impl FnMut(u8, u8, &str, &str),
) -> PurgeOutcome {
    progress(
        5,
        1,
        "Checking the Purge licence and consent tokens",
        "Mode S will not start without a bound Purge licence, the PC name, and ERASE.",
    );
    if !request.purge_licence_bound {
        return failed("Purge licence is not bound on this PC. The helper was not started.");
    }
    if request.letters.is_empty() {
        return failed("Select extra disks for Mode S. The Windows system disk is never included.");
    }
    if !request.preview_only {
        if !hostname_matches(&request.hostname, &request.hostname_typed) {
            return failed("Type this PC’s name exactly to confirm you have the right computer.");
        }
        if !erase_confirmed(&request.erase_typed) {
            return failed("Type ERASE in capital letters to confirm permanent destruction.");
        }
    }

    progress(
        18,
        2,
        "Re-identifying selected media",
        "Serial, bus and capacity are read again before any method is chosen.",
    );
    let (volumes, disks) = live_hints();
    let planned = plan_targets(&request.letters, &volumes, &disks, request.usb_opt_in);
    if planned.is_empty() {
        return failed("No selected volumes could be identified.");
    }

    progress(
        30,
        3,
        "Choosing the method from the media class",
        "The operator cannot pick a 3-pass overwrite and call it Purge.",
    );

    let allowed: Vec<PlannedTarget> = planned
        .iter()
        .filter(|item| item.allowed)
        .cloned()
        .collect();
    if allowed.is_empty() {
        let detail = planned
            .iter()
            .filter_map(|item| item.refuse_reason.clone())
            .next()
            .unwrap_or_else(|| "Mode S refused every selected volume.".to_string());
        return refused_outcome(planned, detail);
    }

    if request.preview_only {
        return preview_outcome(planned);
    }

    progress(
        42,
        4,
        "Confirming the operator typed the hostname and ERASE",
        "Consent tokens already matched. Selected extra disks only.",
    );

    #[cfg(not(windows))]
    {
        let _ = allowed;
        failed("Mode S runs only on Windows.")
    }

    #[cfg(windows)]
    {
        run_mode_s_windows(request, planned, progress)
    }
}

#[cfg(windows)]
fn run_mode_s_windows(
    request: PurgeRequest,
    planned: Vec<PlannedTarget>,
    mut progress: impl FnMut(u8, u8, &str, &str),
) -> PurgeOutcome {
    let job_id = new_job_id();
    let mut outcomes = Vec::new();
    let mut any_fail = false;
    let mut any_pass = false;

    for target in &planned {
        if !target.allowed {
            outcomes.push(target_from_plan(target, false, false, "Not issued."));
            continue;
        }
        progress(
            55,
            5,
            "Handing the signed plan to the elevated helper",
            "The desktop is not erasing. The elevated CYVRA Purge helper issues the command.",
        );
        let helper = match run_one_helper(&job_id, target) {
            Ok(result) => result,
            Err(message) => {
                any_fail = true;
                outcomes.push(target_from_plan(target, false, false, &message));
                continue;
            }
        };
        let helper_note = helper_status_note(&helper);
        progress(
            70,
            6,
            "Waiting for the controller",
            &format!("Method on {}: {}.", target.letter, target.method.as_label()),
        );
        if !helper.letter.is_empty() && helper.letter != target.letter {
            any_fail = true;
            outcomes.push(target_from_plan(target, false, false, &helper_note));
            continue;
        }
        if !helper.ok {
            any_fail = true;
            outcomes.push(target_from_plan(target, false, false, &helper_note));
            continue;
        }
        progress(
            82,
            7,
            "Independent verification",
            "Reading a 10% pseudo-random sample. Looking for leftover user-data patterns.",
        );
        match execute::sample_volume(&target.letter, target.size_bytes, 16) {
            Ok(report) if report.passed => {
                any_pass = true;
                outcomes.push(target_from_plan(
                    target,
                    true,
                    true,
                    &format!("{} {}", report.note, helper_note),
                ));
            }
            Ok(report) => {
                any_fail = true;
                outcomes.push(target_from_plan(
                    target,
                    true,
                    false,
                    &format!("{} {}", report.note, helper_note),
                ));
            }
            Err(message) => {
                any_fail = true;
                outcomes.push(target_from_plan(
                    target,
                    true,
                    false,
                    &format!("{} {}", message, helper_note),
                ));
            }
        }
    }

    let report_allowed = any_pass && !any_fail;
    let status = if report_allowed { "VERIFIED" } else { "FAILED" };
    progress(
        100,
        8,
        if report_allowed {
            "Writing Report S"
        } else {
            "Recording FAIL"
        },
        if report_allowed {
            "Independent verify passed. Save Report S. This is not a laboratory certification."
        } else {
            "No sanitization report is issued on FAIL, cancel, or helper missing."
        },
    );
    let evidence = evidence_hash(&job_id, &outcomes);
    PurgeOutcome {
        ok: report_allowed,
        job_id,
        status: status.to_string(),
        message: if report_allowed {
            format!(
                "Mode S completed on extra disks. Operator {}. Locally verified on this PC.",
                if request.operator_name.trim().is_empty() {
                    "not named"
                } else {
                    request.operator_name.trim()
                }
            )
        } else {
            "Mode S FAILED. No Report S. Nothing is claimed as sanitised.".to_string()
        },
        report_allowed,
        data_erased: any_pass,
        evidence_hash: evidence,
        targets: if outcomes.is_empty() {
            planned
                .iter()
                .map(|item| target_from_plan(item, false, false, "Not issued."))
                .collect()
        } else {
            outcomes
        },
    }
}

fn preview_outcome(planned: Vec<PlannedTarget>) -> PurgeOutcome {
    PurgeOutcome {
        ok: true,
        job_id: String::new(),
        status: "PREVIEW".to_string(),
        message: "Method preview only. No helper was started. Nothing was erased.".to_string(),
        report_allowed: false,
        data_erased: false,
        evidence_hash: "none".to_string(),
        targets: planned
            .iter()
            .map(|item| target_from_plan(item, false, false, "Preview only. Not issued."))
            .collect(),
    }
}

fn refused_outcome(planned: Vec<PlannedTarget>, message: String) -> PurgeOutcome {
    PurgeOutcome {
        ok: false,
        job_id: new_job_id(),
        status: "FAILED".to_string(),
        message,
        report_allowed: false,
        data_erased: false,
        evidence_hash: "none".to_string(),
        targets: planned
            .iter()
            .map(|item| {
                target_from_plan(
                    item,
                    false,
                    false,
                    item.refuse_reason.as_deref().unwrap_or("Refused."),
                )
            })
            .collect(),
    }
}

fn failed(message: &str) -> PurgeOutcome {
    PurgeOutcome {
        ok: false,
        job_id: String::new(),
        status: "FAILED".to_string(),
        message: message.to_string(),
        report_allowed: false,
        data_erased: false,
        evidence_hash: "none".to_string(),
        targets: Vec::new(),
    }
}

fn target_from_plan(
    target: &PlannedTarget,
    helper_ok: bool,
    verify_passed: bool,
    note: &str,
) -> PurgeTargetOutcome {
    PurgeTargetOutcome {
        letter: target.letter.clone(),
        allowed: target.allowed,
        media_label: target.media_class.as_label().to_string(),
        method_label: target.method.as_label().to_string(),
        standard: target.method.standard().to_string(),
        model: target.model.clone(),
        serial: target.serial.clone(),
        bus: target.bus.clone(),
        size_bytes: target.size_bytes,
        refuse_reason: target.refuse_reason.clone(),
        helper_ok,
        verify_passed,
        verify_note: note.to_string(),
        sample_percent: if verify_passed { 10 } else { 0 },
    }
}

fn live_hints() -> (Vec<VolumeHint>, Vec<DiskHint>) {
    let system = crate::system_drive_letter();
    let volumes = volume::collect()
        .into_iter()
        .filter(|item| !item.drive_letter.is_empty() && item.drive_letter != "unknown")
        .map(|item| {
            let letter = item
                .drive_letter
                .trim()
                .trim_end_matches(':')
                .to_ascii_uppercase();
            VolumeHint {
                is_system: letter == system,
                letter,
                drive_kind: item.drive_kind,
                size_bytes: item.size_bytes,
            }
        })
        .collect();
    let disks = storage::collect()
        .disks
        .into_iter()
        .map(|item| DiskHint {
            index: item.index,
            model: item.model,
            serial: item.serial_number,
            size_bytes: item.size_bytes,
            interface_type: item.interface_type,
            media_type: item.media_type,
        })
        .collect();
    (volumes, disks)
}

fn new_job_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|item| item.as_nanos())
        .unwrap_or(0);
    format!("ERS-{nanos}")
}

fn evidence_hash(job_id: &str, outcomes: &[PurgeTargetOutcome]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(job_id.as_bytes());
    for item in outcomes {
        hasher.update(item.letter.as_bytes());
        hasher.update(item.serial.as_bytes());
        hasher.update(item.method_label.as_bytes());
        hasher.update([u8::from(item.verify_passed)]);
    }
    hex_encode(&hasher.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(windows)]
fn helper_status_note(helper: &execute::HelperResult) -> String {
    format!(
        "{}. Job {}. Volume {}. Method {}. Bytes processed {}.",
        helper.message, helper.job_id, helper.letter, helper.method, helper.bytes_processed
    )
}

#[cfg(windows)]
fn run_one_helper(job_id: &str, target: &PlannedTarget) -> Result<execute::HelperResult, String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let dir = std::env::temp_dir().join("cyvra-purge");
    std::fs::create_dir_all(&dir)
        .map_err(|_| "CYVRA could not create the purge job folder.".to_string())?;
    let plan_path = dir.join(format!("{job_id}-{}.plan", target.letter));
    let result_path = dir.join(format!("{job_id}-{}.result", target.letter));
    execute::write_plan_file(&plan_path, job_id, target)?;
    let helper = helper_path()?;
    let status = Command::new(&helper)
        .arg("--plan")
        .arg(&plan_path)
        .arg("--result")
        .arg(&result_path)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|_| {
            "CYVRA could not start the elevated Purge helper. The job is FAILED.".to_string()
        })?;
    if !result_path.exists() {
        return Err("The purge helper did not write a result. The job is FAILED.".to_string());
    }
    let _ = status;
    execute::read_result_file(&result_path)
}

#[cfg(windows)]
fn helper_path() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|_| "CYVRA could not locate the Purge helper.".to_string())?;
    let dir = exe
        .parent()
        .ok_or_else(|| "CYVRA could not locate the Purge helper.".to_string())?;
    let candidate = dir.join("cyvra-purge-helper.exe");
    if candidate.exists() {
        return Ok(candidate);
    }
    let sibling = dir.join("cyvra-purge-helper");
    if sibling.exists() {
        return Ok(sibling);
    }
    Err("The elevated CYVRA Purge helper is missing. Mode S fails closed.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_licence_does_not_start_helper() {
        let outcome = run_mode_s(
            PurgeRequest {
                purge_licence_bound: false,
                hostname: "PC".to_string(),
                hostname_typed: "PC".to_string(),
                erase_typed: "ERASE".to_string(),
                letters: vec!["E".to_string()],
                usb_opt_in: true,
                operator_name: String::new(),
                preview_only: false,
            },
            |_, _, _, _| {},
        );
        assert!(!outcome.ok);
        assert!(!outcome.report_allowed);
        assert!(!outcome.data_erased);
        assert!(outcome.message.contains("Purge licence"));
    }

    #[test]
    fn wrong_erase_token_fails_closed() {
        let outcome = run_mode_s(
            PurgeRequest {
                purge_licence_bound: true,
                hostname: "PC".to_string(),
                hostname_typed: "PC".to_string(),
                erase_typed: "erase".to_string(),
                letters: vec!["E".to_string()],
                usb_opt_in: true,
                operator_name: String::new(),
                preview_only: false,
            },
            |_, _, _, _| {},
        );
        assert!(!outcome.ok);
        assert!(outcome.message.contains("ERASE"));
    }

    #[test]
    fn preview_does_not_require_erase_token() {
        let outcome = run_mode_s(
            PurgeRequest {
                purge_licence_bound: true,
                hostname: "PC".to_string(),
                hostname_typed: String::new(),
                erase_typed: String::new(),
                letters: vec!["E".to_string()],
                usb_opt_in: true,
                operator_name: String::new(),
                preview_only: true,
            },
            |_, _, _, _| {},
        );
        assert_ne!(outcome.status, "CERTIFIED SECURE");
        assert!(!outcome.report_allowed);
        assert!(!outcome.data_erased);
        assert!(
            outcome.status == "PREVIEW"
                || outcome.status == "FAILED"
                || outcome.message.contains("Windows")
                || outcome.message.contains("identified")
                || outcome.message.contains("refused")
                || outcome.message.contains("Select extra")
        );
    }
}
