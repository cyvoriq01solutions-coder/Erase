use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellBootstrap {
    app_version: &'static str,
    runtime_mode: &'static str,
    core_boundary: &'static str,
    destructive_operations_enabled: bool,
    live_activation_enabled: bool,
    live_collection_enabled: bool,
    grading_issuance_enabled: bool,
    report_authentication_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivationOutcome {
    ok: bool,
    message: String,
    key_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NamedValueDto {
    label: String,
    value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanTargetDto {
    letter: String,
    label: String,
    kind: String,
    size_label: String,
    default_selected: bool,
    hint: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerificationProgress {
    percent: u8,
    stage_index: u8,
    stage: String,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VerificationOutcome {
    ok: bool,
    message: String,
    hardware_result: String,
    hardware_passed: bool,
    hardware_validation: String,
    report_json: String,
    manufacturer: String,
    model: String,
    hostname: String,
    os_caption: String,
    personal_location_count: u64,
    pdem_object_count: u64,
    content_inspected: bool,
    destructive_operations_enabled: bool,
    assessment_status: String,
    assessment_summary: String,
    scanned_drives: String,
    hardware_fields: Vec<NamedValueDto>,
    location_groups: Vec<NamedValueDto>,
}

fn typed_core_boundary() -> &'static str {
    let _ = std::any::TypeId::of::<cyvra_core::CollectorError>();
    "direct_typed_cyvra_core"
}

fn safe_bootstrap() -> ShellBootstrap {
    ShellBootstrap {
        app_version: env!("CARGO_PKG_VERSION"),
        runtime_mode: "w_collect_local_assessment",
        core_boundary: typed_core_boundary(),
        destructive_operations_enabled: false,
        live_activation_enabled: true,
        live_collection_enabled: true,
        grading_issuance_enabled: false,
        report_authentication_enabled: false,
    }
}

fn named_values(items: Vec<cyvra_core::NamedValue>) -> Vec<NamedValueDto> {
    items
        .into_iter()
        .map(|item| NamedValueDto {
            label: item.label,
            value: item.value,
        })
        .collect()
}

fn verification_outcome(verification: cyvra_core::CustomerVerification) -> VerificationOutcome {
    VerificationOutcome {
        ok: true,
        message: verification.assessment_summary.clone(),
        hardware_result: verification.hardware_result,
        hardware_passed: verification.hardware_passed,
        hardware_validation: verification.hardware_validation,
        report_json: verification.report_json,
        manufacturer: verification.manufacturer,
        model: verification.model,
        hostname: verification.hostname,
        os_caption: verification.os_caption,
        personal_location_count: verification.personal_location_count,
        pdem_object_count: verification.pdem_object_count,
        content_inspected: verification.content_inspected,
        destructive_operations_enabled: verification.destructive_operations_enabled,
        assessment_status: verification.assessment_status,
        assessment_summary: verification.assessment_summary,
        scanned_drives: verification.scanned_drives,
        hardware_fields: named_values(verification.hardware_fields),
        location_groups: named_values(verification.location_groups),
    }
}

fn run_device_verification_inner(
    drive_letters: Vec<String>,
    mut progress: impl FnMut(u8, u8, &str, &str),
) -> Result<VerificationOutcome, String> {
    if !safe_bootstrap().live_collection_enabled {
        return Err("Device verification is not enabled in this build.".to_string());
    }
    if safe_bootstrap().destructive_operations_enabled {
        return Err("Destructive operations are not permitted.".to_string());
    }

    let verification =
        cyvra_core::run_customer_verification_on_drives(&drive_letters, &mut progress);
    if verification.destructive_operations_enabled || verification.content_inspected {
        return Err("CYVRA stopped because the scan crossed the assessment boundary.".to_string());
    }

    Ok(verification_outcome(verification))
}

#[tauri::command]
fn get_shell_bootstrap() -> ShellBootstrap {
    safe_bootstrap()
}

#[tauri::command]
fn activate_license(activation_key: String) -> Result<ActivationOutcome, String> {
    #[cfg(not(windows))]
    {
        let _ = activation_key;
        Err("Activation is only available on Windows.".to_string())
    }
    #[cfg(windows)]
    {
        activate_license_windows(activation_key)
    }
}

#[tauri::command]
fn list_scan_targets() -> Vec<ScanTargetDto> {
    cyvra_core::list_scan_targets()
        .into_iter()
        .map(|target| ScanTargetDto {
            letter: target.letter,
            label: target.label,
            kind: target.kind,
            size_label: target.size_label,
            default_selected: target.default_selected,
            hint: target.hint,
        })
        .collect()
}

#[tauri::command]
async fn run_device_verification(
    app: tauri::AppHandle,
    drive_letters: Option<Vec<String>>,
) -> Result<VerificationOutcome, String> {
    let letters = drive_letters.unwrap_or_default();
    let progress_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_device_verification_inner(letters, |percent, stage_index, stage, detail| {
            let _ = progress_app.emit(
                "verification-progress",
                VerificationProgress {
                    percent,
                    stage_index,
                    stage: stage.to_string(),
                    detail: detail.to_string(),
                },
            );
        })
    })
    .await
    .map_err(|_| "CYVRA could not finish device verification.".to_string())?
}

#[tauri::command]
fn close_window(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(windows)]
fn activate_license_windows(activation_key: String) -> Result<ActivationOutcome, String> {
    let machine_guid = windows_machine_guid()?;
    let hostname = std::env::var("COMPUTERNAME").ok();
    let body = serde_json::json!({
        "activationKey": activation_key.trim(),
        "machineGuid": machine_guid,
        "hostname": hostname,
    });
    let response = ureq::post("https://api.cyvra.co.in/api/v1/auth/activate")
        .set("Content-Type", "application/json")
        .send_json(body)
        .map_err(|_| "CYVRA could not reach the activation service.".to_string())?;
    let value: serde_json::Value = response
        .into_json()
        .map_err(|_| "CYVRA returned an unreadable activation response.".to_string())?;
    let status = value
        .get("status")
        .and_then(|item| item.as_str())
        .unwrap_or("");
    let message = value
        .get("message")
        .and_then(|item| item.as_str())
        .unwrap_or("Activation could not be completed.")
        .to_string();
    let key_prefix = value
        .get("keyPrefix")
        .and_then(|item| item.as_str())
        .map(str::to_string);
    if status == "bound" || status == "already_bound" {
        return Ok(ActivationOutcome {
            ok: true,
            message,
            key_prefix,
        });
    }
    Err(message)
}

#[cfg(windows)]
fn windows_machine_guid() -> Result<String, String> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey_with_flags(
            "SOFTWARE\\Microsoft\\Cryptography",
            KEY_READ | KEY_WOW64_64KEY,
        )
        .map_err(|_| "CYVRA could not read the Windows device identity.".to_string())?;
    key.get_value::<String, _>("MachineGuid")
        .map_err(|_| "CYVRA could not read the Windows device identity.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_shell_bootstrap,
            activate_license,
            list_scan_targets,
            run_device_verification,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("CYVRA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::{run_device_verification_inner, safe_bootstrap};

    #[test]
    fn w_collect_enables_assessment_and_keeps_purge_off() {
        let bootstrap = safe_bootstrap();

        assert_eq!(bootstrap.runtime_mode, "w_collect_local_assessment");
        assert_eq!(bootstrap.core_boundary, "direct_typed_cyvra_core");
        assert!(!bootstrap.destructive_operations_enabled);
        assert!(bootstrap.live_activation_enabled);
        assert!(bootstrap.live_collection_enabled);
        assert!(!bootstrap.grading_issuance_enabled);
        assert!(!bootstrap.report_authentication_enabled);
    }

    #[test]
    fn local_verification_stays_non_destructive() {
        let outcome = run_device_verification_inner(Vec::new(), |_, _, _, _| {})
            .expect("local assessment should run");
        assert!(outcome.ok);
        assert!(!outcome.destructive_operations_enabled);
        assert!(!outcome.content_inspected);
        assert!(outcome.report_json.contains("CYVRA Erase Verification"));
        assert!(
            outcome
                .hardware_validation
                .contains("destructive_operations=false")
        );
        assert!(
            outcome.hardware_result == "pass"
                || outcome.hardware_result == "fail"
                || outcome.hardware_result == "not_windows"
        );
        assert!(!outcome.hardware_fields.is_empty());
    }
}
