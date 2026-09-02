//! Advance scan orchestrator.
//!
//! Advance scan is the deep, opt-in counterpart to the basic local assessment.
//! This slice lands the boundary, the report shape and the grading arithmetic
//! with no collection at all, so the honest empty state is provable before a
//! single low-level probe exists. Later slices fill the sections in; nothing
//! here may ever infer a value it did not read.

use crate::NamedValue;
use crate::hardware_diagnostics_v1::{
    CoverageSummary, DeviceForm, DiagnosticDomain, DomainEvidence, HardwareDiagnosticsV1, evaluate,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// The frozen customer phrase for a subsystem this build cannot read yet.
const NOT_COLLECTED: &str = "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.";

/// Used where the gap is architectural rather than merely unwritten.
const NEEDS_KERNEL_SENSOR: &str = "Not collected in this scan. This value requires a kernel-mode sensor driver, which CYVRA deliberately does not ship.";

const DECLINED: &str = "Declined by the operator. No benchmark was run and nothing was written.";

const NOT_ATTEMPTED: &str =
    "Not attempted in this scan. A technician records this at physical verification.";

/// What the operator agreed to before Advance scan started. Both default off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AdvanceScanRequest {
    pub benchmarks_consented: bool,
    pub write_benchmark_consented: bool,
}

/// A titled block of Report D rows.
#[derive(Debug, Clone)]
pub struct TelemetryGroup {
    pub title: String,
    pub rows: Vec<NamedValue>,
}

/// Everything Report D prints, already reduced to customer-safe strings.
#[derive(Debug, Clone)]
pub struct CustomerAdvanceScan {
    pub ok: bool,
    pub message: String,
    pub schema_version: &'static str,
    pub elevation_state: &'static str,
    pub elevation_label: &'static str,
    pub benchmarks_consented: bool,
    pub write_benchmark_consented: bool,
    pub bytes_written: u64,
    pub destructive_operations_enabled: bool,
    pub content_inspected: bool,
    pub boundary_note: String,
    pub telemetry_groups: Vec<TelemetryGroup>,
    pub coverage_rows: Vec<NamedValue>,
    pub not_assessable: Vec<String>,
    pub grading_engine: &'static str,
    pub grading_rubric: &'static str,
    pub grade_label: &'static str,
    pub grade_condition: &'static str,
    pub grade_withheld: bool,
    pub grade_withheld_reason: Option<String>,
    pub coverage_percent: u32,
    pub index_percent: Option<u32>,
    pub provisional: bool,
}

/// Run Advance scan and build Report D.
///
/// In this slice no probe runs, so every section reports honestly that it was
/// not collected and the grading engine withholds the grade. The write
/// boundary is asserted rather than assumed: if any byte were recorded as
/// written without consent, the scan fails closed.
#[must_use]
pub fn run_advance_scan(request: &AdvanceScanRequest) -> CustomerAdvanceScan {
    let collected_at_unix = current_unix_timestamp();
    let mut diagnostics = HardwareDiagnosticsV1::not_collected(collected_at_unix);
    diagnostics.benchmarks_consented = request.benchmarks_consented;
    diagnostics.write_benchmark_consented = request.write_benchmark_consented;

    build_report(&diagnostics)
}

fn build_report(diagnostics: &HardwareDiagnosticsV1) -> CustomerAdvanceScan {
    let wrote_without_consent =
        diagnostics.bytes_written > 0 && !diagnostics.write_benchmark_consented;
    let summary = evaluate(&domain_evidence(diagnostics), device_form(diagnostics), &[]);

    CustomerAdvanceScan {
        ok: !wrote_without_consent,
        message: outcome_message(&summary, wrote_without_consent),
        schema_version: diagnostics.schema_version,
        elevation_state: diagnostics.elevation_state.as_str(),
        elevation_label: diagnostics.elevation_state.customer_label(),
        benchmarks_consented: diagnostics.benchmarks_consented,
        write_benchmark_consented: diagnostics.write_benchmark_consented,
        bytes_written: diagnostics.bytes_written,
        destructive_operations_enabled: false,
        content_inspected: false,
        boundary_note: boundary_note(diagnostics),
        telemetry_groups: telemetry_groups(diagnostics),
        coverage_rows: coverage_rows(&summary),
        not_assessable: not_assessable(diagnostics),
        grading_engine: summary.engine,
        grading_rubric: summary.rubric,
        grade_label: summary.band.label(),
        grade_condition: summary.band.condition(),
        grade_withheld: summary.is_withheld(),
        grade_withheld_reason: summary.withheld_reason.clone(),
        coverage_percent: summary.coverage_percent,
        index_percent: summary.index_percent,
        provisional: summary.provisional,
    }
}

fn outcome_message(summary: &CoverageSummary, wrote_without_consent: bool) -> String {
    if wrote_without_consent {
        return "CYVRA stopped Advance scan because a write was recorded without consent."
            .to_string();
    }
    if summary.is_withheld() {
        return "Advance scan finished. No grade was issued because too little of this device could be assessed.".to_string();
    }
    format!(
        "Advance scan finished. Provisional grade {} from {}% coverage.",
        summary.band.label(),
        summary.coverage_percent
    )
}

fn boundary_note(diagnostics: &HardwareDiagnosticsV1) -> String {
    let benchmarks = if diagnostics.benchmarks_consented {
        "Benchmarks were permitted by the operator."
    } else {
        "Benchmarks were not permitted, so none were run."
    };
    let writes = if diagnostics.bytes_written == 0 {
        "Nothing was written to any drive."
    } else {
        "A temporary benchmark file was written and removed."
    };
    format!(
        "Advance scan is read-only in this version. {benchmarks} {writes} File contents were not opened. No data was erased. Purge stays off."
    )
}

/// One row of evidence per domain. Nothing is collected yet, so every domain
/// is explicitly not assessable and none of them can earn a point.
fn domain_evidence(diagnostics: &HardwareDiagnosticsV1) -> Vec<DomainEvidence> {
    DiagnosticDomain::ALL
        .iter()
        .map(|domain| DomainEvidence::not_assessable(*domain, domain_gap(*domain, diagnostics)))
        .collect()
}

fn domain_gap(domain: DiagnosticDomain, diagnostics: &HardwareDiagnosticsV1) -> &'static str {
    match domain {
        DiagnosticDomain::BatteryAndPower => "Battery telemetry is not collected in this scan",
        DiagnosticDomain::ProcessorAndThermal => {
            if diagnostics.benchmarks_consented {
                "Processor benchmark is not collected in this scan"
            } else {
                "Processor benchmark was declined by the operator"
            }
        }
        DiagnosticDomain::MemoryIntegrity => {
            if diagnostics.benchmarks_consented {
                "Memory pattern check is not collected in this scan"
            } else {
                "Memory pattern check was declined by the operator"
            }
        }
        DiagnosticDomain::StorageHealth => "Storage SMART telemetry is not collected in this scan",
        DiagnosticDomain::PortsAndConnectivity => {
            "Port topology and radio telemetry are not collected in this scan"
        }
        DiagnosticDomain::ScreenAndPeripherals => {
            "Interactive technician checks were not attempted in this scan"
        }
    }
}

/// Report D is deliberately gradeless until we know the chassis, so the
/// mandatory-domain rule cannot be dodged by guessing the form factor.
const fn device_form(_diagnostics: &HardwareDiagnosticsV1) -> DeviceForm {
    DeviceForm::Unknown
}

fn coverage_rows(summary: &CoverageSummary) -> Vec<NamedValue> {
    let mut rows = vec![
        row("Points in scope", summary.in_scope_points.to_string()),
        row("Points assessed", summary.assessed_points.to_string()),
        row("Points awarded", summary.awarded_points.to_string()),
        row(
            "Points not assessable",
            summary.not_assessable_points.to_string(),
        ),
        row("Coverage", format!("{}%", summary.coverage_percent)),
    ];
    rows.push(row(
        "Assessed Health Index",
        summary.index_percent.map_or_else(
            || "Not assessable in this scan".to_string(),
            |index| format!("{index} / 100"),
        ),
    ));
    rows.push(row("Grading engine", summary.engine.to_string()));
    rows
}

fn not_assessable(diagnostics: &HardwareDiagnosticsV1) -> Vec<String> {
    DiagnosticDomain::ALL
        .iter()
        .map(|domain| format!("{} — {}", domain.label(), domain_gap(*domain, diagnostics)))
        .collect()
}

fn telemetry_groups(diagnostics: &HardwareDiagnosticsV1) -> Vec<TelemetryGroup> {
    let benchmark_value = if diagnostics.benchmarks_consented {
        NOT_COLLECTED
    } else {
        DECLINED
    };

    vec![
        group(
            "Battery and power",
            &[
                ("Battery present", NOT_COLLECTED),
                ("Design capacity", NOT_COLLECTED),
                ("Full charge capacity", NOT_COLLECTED),
                ("Battery wear", NOT_COLLECTED),
                ("Cycle count", NOT_COLLECTED),
                ("Chemistry", NOT_COLLECTED),
            ],
        ),
        group(
            "Processor and thermal",
            &[
                ("Base clock", NOT_COLLECTED),
                ("Maximum clock", NOT_COLLECTED),
                ("Cache hierarchy", NOT_COLLECTED),
                ("Instruction sets", NOT_COLLECTED),
                ("Package temperature", NEEDS_KERNEL_SENSOR),
                ("Fan speed", NEEDS_KERNEL_SENSOR),
            ],
        ),
        group(
            "Memory",
            &[
                ("Installed total", NOT_COLLECTED),
                ("Available", NOT_COLLECTED),
                ("Channel mode", NOT_COLLECTED),
            ],
        ),
        group(
            "Storage health and SMART",
            &[
                ("Bus type", NOT_COLLECTED),
                ("Power-on hours", NOT_COLLECTED),
                ("Power cycles", NOT_COLLECTED),
                ("Total bytes written", NOT_COLLECTED),
                ("Percentage used", NOT_COLLECTED),
                ("Available spare", NOT_COLLECTED),
                ("Media errors", NOT_COLLECTED),
                ("Sectors pending reallocation", NOT_COLLECTED),
                ("Predicted failure", NOT_COLLECTED),
            ],
        ),
        group(
            "Ports and connectivity",
            &[
                ("USB controller ports", NOT_COLLECTED),
                ("Negotiated port speeds", NOT_COLLECTED),
                ("Physically verified ports", NOT_ATTEMPTED),
                ("Wi-Fi signal quality", NOT_COLLECTED),
                ("Wi-Fi link speed", NOT_COLLECTED),
                ("Bluetooth radio", NOT_COLLECTED),
                ("Ethernet link", NOT_COLLECTED),
            ],
        ),
        group(
            "Display panel",
            &[
                ("Panel manufacturer", NOT_COLLECTED),
                ("Panel model", NOT_COLLECTED),
                ("Native resolution", NOT_COLLECTED),
                ("Refresh rate", NOT_COLLECTED),
                ("HDR capability", NOT_COLLECTED),
                ("Panel manufacture year", NOT_COLLECTED),
            ],
        ),
        group(
            "Cameras and microphones",
            &[("Cameras", NOT_COLLECTED), ("Microphones", NOT_COLLECTED)],
        ),
        group(
            "Benchmarks",
            &[
                ("Processor sustained clock", benchmark_value),
                ("Memory pattern check", benchmark_value),
                ("Sequential read", benchmark_value),
                ("Random read", benchmark_value),
                (
                    "Write benchmark",
                    if diagnostics.write_benchmark_consented {
                        NOT_COLLECTED
                    } else {
                        DECLINED
                    },
                ),
            ],
        ),
        group(
            "Technician checks",
            &[
                ("Keyboard", NOT_ATTEMPTED),
                ("Display inspection", NOT_ATTEMPTED),
                ("Trackpad", NOT_ATTEMPTED),
                ("Speakers", NOT_ATTEMPTED),
                ("Camera image", NOT_ATTEMPTED),
            ],
        ),
    ]
}

fn group(title: &str, rows: &[(&str, &str)]) -> TelemetryGroup {
    TelemetryGroup {
        title: title.to_string(),
        rows: rows
            .iter()
            .map(|(label, value)| row(label, (*value).to_string()))
            .collect(),
    }
}

fn row(label: &str, value: String) -> NamedValue {
    NamedValue {
        label: label.to_string(),
        value,
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_scan_is_non_destructive_and_writes_nothing() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());

        assert!(outcome.ok);
        assert!(!outcome.destructive_operations_enabled);
        assert!(!outcome.content_inspected);
        assert_eq!(outcome.bytes_written, 0);
        assert!(outcome.boundary_note.contains("No data was erased"));
        assert!(
            outcome
                .boundary_note
                .contains("Nothing was written to any drive")
        );
    }

    #[test]
    fn empty_advance_scan_withholds_the_grade() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());

        assert!(outcome.grade_withheld);
        assert_eq!(outcome.grade_label, "Grade withheld");
        assert_eq!(outcome.coverage_percent, 0);
        assert_eq!(outcome.index_percent, None);
        assert!(outcome.provisional);
        assert!(
            outcome
                .grade_withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("Storage health")
        );
    }

    #[test]
    fn grading_block_names_the_engine_without_an_ai_claim() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());

        assert_eq!(outcome.grading_engine, "CYVRA Grading Engine");
        assert_eq!(outcome.grading_rubric, "CG-1.0");
        assert!(!outcome.grading_engine.contains("AI"));
    }

    #[test]
    fn every_row_is_honest_about_what_was_not_collected() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());

        assert!(!outcome.telemetry_groups.is_empty());
        for group in &outcome.telemetry_groups {
            assert!(!group.rows.is_empty(), "{} had no rows", group.title);
            for row in &group.rows {
                assert!(
                    row.value.contains("Not collected in this scan")
                        || row.value.contains("Not attempted in this scan")
                        || row.value.contains("Declined by the operator"),
                    "{} / {} claimed a value it never read: {}",
                    group.title,
                    row.label,
                    row.value
                );
            }
        }
    }

    #[test]
    fn declining_benchmarks_is_reported_as_a_decline_not_a_failure() {
        let declined = run_advance_scan(&AdvanceScanRequest::default());
        let permitted = run_advance_scan(&AdvanceScanRequest {
            benchmarks_consented: true,
            write_benchmark_consented: false,
        });

        let declined_rows = benchmark_rows(&declined);
        let permitted_rows = benchmark_rows(&permitted);

        assert!(declined_rows.iter().any(|value| value.contains("Declined")));
        assert!(
            permitted_rows
                .iter()
                .any(|value| value.contains("Not collected in this scan"))
        );
        assert!(!permitted.benchmarks_consented || permitted.bytes_written == 0);
    }

    #[test]
    fn thermal_gaps_state_the_architectural_reason() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());
        let processor = outcome
            .telemetry_groups
            .iter()
            .find(|group| group.title == "Processor and thermal")
            .expect("processor group must exist");

        let temperature = processor
            .rows
            .iter()
            .find(|row| row.label == "Package temperature")
            .expect("package temperature row must exist");

        assert!(temperature.value.contains("kernel-mode sensor driver"));
    }

    #[test]
    fn coverage_rows_never_render_a_zero_index() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());
        let index = outcome
            .coverage_rows
            .iter()
            .find(|row| row.label == "Assessed Health Index")
            .expect("index row must exist");

        assert_eq!(index.value, "Not assessable in this scan");
        assert!(!index.value.contains('0'));
    }

    #[test]
    fn a_recorded_write_without_consent_fails_closed() {
        let mut diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        diagnostics.bytes_written = 4_096;

        let outcome = build_report(&diagnostics);

        assert!(!outcome.ok);
        assert!(outcome.message.contains("without consent"));
    }

    fn benchmark_rows(outcome: &CustomerAdvanceScan) -> Vec<String> {
        outcome
            .telemetry_groups
            .iter()
            .filter(|group| group.title == "Benchmarks")
            .flat_map(|group| group.rows.iter().map(|row| row.value.clone()))
            .collect()
    }
}
