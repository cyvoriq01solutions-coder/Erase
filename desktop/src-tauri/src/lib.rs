use serde::{Deserialize, Serialize};

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
fn run_device_verification() -> Result<VerificationOutcome, String> {
    if !safe_bootstrap().live_collection_enabled {
        return Err("Device verification is not enabled in this build.".to_string());
    }
    if safe_bootstrap().destructive_operations_enabled {
        return Err("Destructive operations are not permitted.".to_string());
    }

    let verification = cyvra_core::run_customer_verification();
    if verification.destructive_operations_enabled || verification.content_inspected {
        return Err("CYVRA stopped because the scan crossed the assessment boundary.".to_string());
    }

    Ok(VerificationOutcome {
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
    })
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
            run_device_verification
        ])
        .run(tauri::generate_context!())
        .expect("CYVRA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::{run_device_verification, safe_bootstrap};

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
        let outcome = run_device_verification().expect("local assessment should run");
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
    }
}
