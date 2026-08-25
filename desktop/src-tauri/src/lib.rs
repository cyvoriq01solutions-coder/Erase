use serde::Serialize;

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
        live_activation_enabled: false,
        live_collection_enabled: false,
        grading_issuance_enabled: false,
        report_authentication_enabled: false,
    }
}

#[tauri::command]
fn get_shell_bootstrap() -> ShellBootstrap {
    safe_bootstrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_shell_bootstrap])
        .run(tauri::generate_context!())
        .expect("CYVRA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::safe_bootstrap;

    #[test]
    fn foundation_bootstrap_fails_closed() {
        let bootstrap = safe_bootstrap();

        assert_eq!(bootstrap.runtime_mode, "w2_1b_shell_foundation");
        assert_eq!(bootstrap.core_boundary, "direct_typed_cyvra_core");
        assert!(!bootstrap.destructive_operations_enabled);
        assert!(!bootstrap.live_activation_enabled);
        assert!(!bootstrap.live_collection_enabled);
        assert!(!bootstrap.grading_issuance_enabled);
        assert!(!bootstrap.report_authentication_enabled);
    }
}
