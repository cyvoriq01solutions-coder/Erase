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

fn typed_core_boundary() -> &'static str {
    let _ = std::any::TypeId::of::<cyvra_core::CollectorError>();
    "direct_typed_cyvra_core"
}

fn safe_bootstrap() -> ShellBootstrap {
    ShellBootstrap {
        app_version: env!("CARGO_PKG_VERSION"),
        runtime_mode: "w2_1b_shell_foundation",
        core_boundary: typed_core_boundary(),
        destructive_operations_enabled: false,
        live_activation_enabled: true,
        live_collection_enabled: false,
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
    let status = value.get("status").and_then(|item| item.as_str()).unwrap_or("");
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
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
    use winreg::RegKey;

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
            activate_license
        ])
        .run(tauri::generate_context!())
        .expect("CYVRA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::safe_bootstrap;

    #[test]
    fn foundation_bootstrap_fails_closed_except_live_activation() {
        let bootstrap = safe_bootstrap();

        assert_eq!(bootstrap.runtime_mode, "w2_1b_shell_foundation");
        assert_eq!(bootstrap.core_boundary, "direct_typed_cyvra_core");
        assert!(!bootstrap.destructive_operations_enabled);
        assert!(bootstrap.live_activation_enabled);
        assert!(!bootstrap.live_collection_enabled);
        assert!(!bootstrap.grading_issuance_enabled);
        assert!(!bootstrap.report_authentication_enabled);
    }
}
