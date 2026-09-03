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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TelemetryGroupDto {
    title: String,
    note: Option<String>,
    rows: Vec<NamedValueDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DomainCoverageDto {
    domain: String,
    awarded: u32,
    assessed: u32,
    not_assessable: u32,
    weight: u32,
    state: String,
    confidence: String,
    note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdvanceScanProgress {
    percent: u8,
    stage_index: u8,
    stage: String,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdvanceScanOutcome {
    ok: bool,
    message: String,
    schema_version: String,
    elevation_state: String,
    elevation_label: String,
    benchmarks_consented: bool,
    write_benchmark_consented: bool,
    bytes_written: u64,
    destructive_operations_enabled: bool,
    content_inspected: bool,
    boundary_note: String,
    temporary_files_note: String,
    telemetry_groups: Vec<TelemetryGroupDto>,
    coverage_rows: Vec<NamedValueDto>,
    coverage_domains: Vec<DomainCoverageDto>,
    method_rows: Vec<NamedValueDto>,
    rubric_rows: Vec<NamedValueDto>,
    not_assessable: Vec<String>,
    grading_engine: String,
    grading_rubric: String,
    grade_label: String,
    grade_condition: String,
    grade_observation: Option<String>,
    grade_withheld: bool,
    grade_withheld_reason: Option<String>,
    coverage_percent: u32,
    index_percent: Option<u32>,
    provisional: bool,
    issuance_notice: String,
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

fn advance_scan_outcome(scan: cyvra_core::diagnostics::CustomerAdvanceScan) -> AdvanceScanOutcome {
    AdvanceScanOutcome {
        ok: scan.ok,
        message: scan.message,
        schema_version: scan.schema_version.to_string(),
        elevation_state: scan.elevation_state.to_string(),
        elevation_label: scan.elevation_label.to_string(),
        benchmarks_consented: scan.benchmarks_consented,
        write_benchmark_consented: scan.write_benchmark_consented,
        bytes_written: scan.bytes_written,
        destructive_operations_enabled: scan.destructive_operations_enabled,
        content_inspected: scan.content_inspected,
        boundary_note: scan.boundary_note,
        temporary_files_note: scan.temporary_files_note,
        telemetry_groups: scan
            .telemetry_groups
            .into_iter()
            .map(|group| TelemetryGroupDto {
                title: group.title,
                note: group.note,
                rows: named_values(group.rows),
            })
            .collect(),
        coverage_rows: named_values(scan.coverage_rows),
        coverage_domains: scan
            .coverage_domains
            .into_iter()
            .map(|domain| DomainCoverageDto {
                domain: domain.domain,
                awarded: domain.awarded,
                assessed: domain.assessed,
                not_assessable: domain.not_assessable,
                weight: domain.weight,
                state: domain.state,
                confidence: domain.confidence,
                note: domain.note,
            })
            .collect(),
        method_rows: named_values(scan.method_rows),
        rubric_rows: named_values(scan.rubric_rows),
        not_assessable: scan.not_assessable,
        grading_engine: scan.grading_engine.to_string(),
        grading_rubric: scan.grading_rubric.to_string(),
        grade_label: scan.grade_label.to_string(),
        grade_condition: scan.grade_condition.to_string(),
        grade_observation: scan.grade_observation.map(str::to_string),
        grade_withheld: scan.grade_withheld,
        grade_withheld_reason: scan.grade_withheld_reason,
        coverage_percent: scan.coverage_percent,
        index_percent: scan.index_percent,
        provisional: scan.provisional,
        issuance_notice: scan.issuance_notice.to_string(),
    }
}

fn parse_device_form(value: Option<String>) -> cyvra_core::hardware_diagnostics_v1::DeviceForm {
    use cyvra_core::hardware_diagnostics_v1::DeviceForm;

    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "portable" | "laptop" | "tablet" | "notebook" => DeviceForm::Portable,
        "fixed" | "desktop" | "server" => DeviceForm::Fixed,
        _ => DeviceForm::Unknown,
    }
}

fn run_advance_scan_inner(
    request: cyvra_core::diagnostics::AdvanceScanRequest,
    mut progress: impl FnMut(u8, u8, &str, &str),
) -> Result<AdvanceScanOutcome, String> {
    let bootstrap = safe_bootstrap();
    if !bootstrap.live_collection_enabled {
        return Err("Advance scan is not enabled in this build.".to_string());
    }
    if bootstrap.destructive_operations_enabled {
        return Err("Destructive operations are not permitted.".to_string());
    }

    let cancellation = cyvra_core::collector_runtime::CancellationToken::new();
    let scan =
        cyvra_core::diagnostics::run_advance_scan_with(&request, &cancellation, &mut progress);
    if !scan.ok || scan.destructive_operations_enabled || scan.content_inspected {
        return Err(scan.message);
    }

    Ok(advance_scan_outcome(scan))
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
#[allow(clippy::too_many_arguments)]
async fn run_advance_scan(
    app: tauri::AppHandle,
    benchmarks_consented: Option<bool>,
    write_benchmark_consented: Option<bool>,
    device_form: Option<String>,
    colour_wash: Option<String>,
    keyboard: Option<String>,
    trackpad: Option<String>,
    speakers: Option<String>,
    capture: Option<String>,
    physical_ports: Option<String>,
) -> Result<AdvanceScanOutcome, String> {
    let request = cyvra_core::diagnostics::AdvanceScanRequest {
        benchmarks_consented: benchmarks_consented.unwrap_or(false),
        write_benchmark_consented: write_benchmark_consented.unwrap_or(false),
        device_form: parse_device_form(device_form),
        interactive: cyvra_core::hardware_diagnostics_v1::InteractiveAttestations {
            colour_wash: cyvra_core::hardware_diagnostics_v1::OperatorAttestation::from_wire(
                colour_wash.as_deref(),
            ),
            keyboard: cyvra_core::hardware_diagnostics_v1::OperatorAttestation::from_wire(
                keyboard.as_deref(),
            ),
            trackpad: cyvra_core::hardware_diagnostics_v1::OperatorAttestation::from_wire(
                trackpad.as_deref(),
            ),
            speakers: cyvra_core::hardware_diagnostics_v1::OperatorAttestation::from_wire(
                speakers.as_deref(),
            ),
            capture: cyvra_core::hardware_diagnostics_v1::OperatorAttestation::from_wire(
                capture.as_deref(),
            ),
            physical_ports: cyvra_core::hardware_diagnostics_v1::PhysicalPortAttestation::from_wire(
                physical_ports.as_deref(),
            ),
        },
    };
    let progress_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_advance_scan_inner(request, |percent, stage_index, stage, detail| {
            let _ = progress_app.emit(
                "advance-scan-progress",
                AdvanceScanProgress {
                    percent,
                    stage_index,
                    stage: stage.to_string(),
                    detail: detail.to_string(),
                },
            );
        })
    })
    .await
    .map_err(|_| "CYVRA could not finish Advance scan.".to_string())?
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
            run_advance_scan,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("CYVRA desktop shell failed to start");
}

#[cfg(test)]
mod tests {
    use super::{run_advance_scan_inner, run_device_verification_inner, safe_bootstrap};

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

    #[test]
    fn advance_scan_withholds_the_grade_and_writes_nothing() {
        let mut stages: Vec<u8> = Vec::new();
        let outcome = run_advance_scan_inner(
            cyvra_core::diagnostics::AdvanceScanRequest::default(),
            |_, stage_index, _, _| stages.push(stage_index),
        )
        .expect("advance scan should complete");

        assert!(outcome.ok);
        assert!(!outcome.destructive_operations_enabled);
        assert!(!outcome.content_inspected);
        assert_eq!(outcome.bytes_written, 0);
        assert!(outcome.provisional);
        // Index is None only when nothing was assessed. On Windows, A3 may
        // enumerate USB and A4 may read SMART. Coverage can still sit below
        // the CG-1.0 floor, so the grade stays withheld unless a measured
        // storage critical forces F.
        match outcome.index_percent {
            None => assert_eq!(outcome.coverage_percent, 0),
            Some(index) => {
                assert!(outcome.coverage_percent > 0);
                assert!(index <= 100);
            }
        }
        if outcome.grade_withheld {
            let reason = outcome.grade_withheld_reason.unwrap_or_default();
            assert!(
                reason.contains("Storage")
                    || reason.contains("could be assessed")
                    || reason.contains("required area"),
                "{reason}"
            );
        } else {
            assert_eq!(outcome.grade_label, "F");
        }
        assert_eq!(outcome.grading_engine, "CYVRA Grading Engine");
        assert_eq!(
            outcome.issuance_notice,
            "This is not an issued CYVORIQ grading certificate."
        );
        assert!(outcome.provisional);
        assert!(
            !outcome
                .coverage_domains
                .iter()
                .any(|domain| domain.confidence.is_empty())
        );
        assert!(!outcome.telemetry_groups.is_empty());
        assert_eq!(outcome.coverage_domains.len(), 6);
        assert!(!outcome.method_rows.is_empty());
        assert!(!outcome.rubric_rows.is_empty());
        assert!(!stages.is_empty(), "progress must be reported");
    }

    #[test]
    fn advance_scan_cannot_be_issued_while_grading_is_disabled() {
        let bootstrap = safe_bootstrap();
        let outcome = run_advance_scan_inner(
            cyvra_core::diagnostics::AdvanceScanRequest {
                benchmarks_consented: true,
                write_benchmark_consented: true,
                ..Default::default()
            },
            |_, _, _, _| {},
        )
        .expect("advance scan should complete");

        assert!(!bootstrap.grading_issuance_enabled);
        assert!(outcome.provisional);
        assert!(
            outcome.bytes_written == 0 || outcome.bytes_written == 8 * 1024 * 1024,
            "write test is either skipped (Linux / no write path) or exactly 8 MiB in TEMP: {}",
            outcome.bytes_written
        );
    }
}
