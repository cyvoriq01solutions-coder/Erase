//! Advance scan orchestrator.
//!
//! Advance scan is the deep, opt-in counterpart to the basic local assessment.
//! It walks each subsystem in turn, reports progress as it goes, records what
//! it could not read and why, and only then asks the grading engine for a
//! verdict. Nothing here infers a value it did not read.

use crate::NamedValue;
use crate::battery_probe::{self, BatteryProbe, BatterySource, SourceOutcome};
use crate::collector_runtime::CancellationToken;
use crate::hardware_diagnostics_v1::{
    CoverageSummary, DeviceForm, DiagnosticDomain, DomainApplicability, DomainEvidence,
    HardwareDiagnosticsV1, battery_points, evaluate,
};
use crate::hardware_inventory_v1::Confidence;
use std::time::{SystemTime, UNIX_EPOCH};

/// The frozen customer phrase for a subsystem this build cannot read yet.
const NOT_COLLECTED: &str = "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.";

/// Used where the gap is architectural rather than merely unwritten.
const NEEDS_KERNEL_SENSOR: &str = "Not collected in this scan. This value requires a kernel-mode sensor driver, which CYVRA deliberately does not ship.";

const DECLINED: &str = "Declined by the operator. No benchmark was run and nothing was written.";

const NOT_ATTEMPTED: &str =
    "Not attempted in this scan. A technician records this at physical verification.";

/// Ordered stages, reported to the operator while the scan runs.
pub const STAGES: [&str; 11] = [
    "Preparing advance scan",
    "Reading battery and power",
    "Reading processor and cache",
    "Reading memory",
    "Reading storage identity and health",
    "Reading ports and connectivity",
    "Reading display panel",
    "Reading cameras and microphones",
    "Running permitted benchmarks",
    "Scoring coverage",
    "Preparing Report D",
];

/// What the operator agreed to before Advance scan started. Both permissions
/// default off. `device_form` comes from the basic assessment when one has run,
/// so the grading engine knows whether a battery is even expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvanceScanRequest {
    pub benchmarks_consented: bool,
    pub write_benchmark_consented: bool,
    pub device_form: DeviceForm,
}

impl Default for AdvanceScanRequest {
    fn default() -> Self {
        Self {
            benchmarks_consented: false,
            write_benchmark_consented: false,
            device_form: DeviceForm::Unknown,
        }
    }
}

/// A titled block of Report D rows.
#[derive(Debug, Clone)]
pub struct TelemetryGroup {
    pub title: String,
    pub note: Option<String>,
    pub rows: Vec<NamedValue>,
}

/// One row of the per-domain coverage table on Report D.
#[derive(Debug, Clone)]
pub struct DomainCoverageRow {
    pub domain: String,
    pub awarded: u32,
    pub assessed: u32,
    pub not_assessable: u32,
    pub weight: u32,
    pub state: String,
    pub note: String,
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
    pub temporary_files_note: String,
    pub destructive_operations_enabled: bool,
    pub content_inspected: bool,
    pub boundary_note: String,
    pub telemetry_groups: Vec<TelemetryGroup>,
    pub coverage_rows: Vec<NamedValue>,
    pub coverage_domains: Vec<DomainCoverageRow>,
    pub method_rows: Vec<NamedValue>,
    pub rubric_rows: Vec<NamedValue>,
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

/// Run Advance scan without progress reporting.
#[must_use]
pub fn run_advance_scan(request: &AdvanceScanRequest) -> CustomerAdvanceScan {
    run_advance_scan_with(request, &CancellationToken::new(), |_, _, _, _| {})
}

/// Run Advance scan, reporting `(percent, stage_index, stage, detail)` as each
/// subsystem is visited so the shell can show what is happening right now.
#[must_use]
pub fn run_advance_scan_with(
    request: &AdvanceScanRequest,
    cancellation: &CancellationToken,
    mut progress: impl FnMut(u8, u8, &str, &str),
) -> CustomerAdvanceScan {
    let mut report = |index: usize, detail: &str| {
        let percent = ((index as f64 / (STAGES.len() - 1) as f64) * 100.0).round() as u8;
        progress(percent, index as u8, STAGES[index], detail);
    };

    report(0, "Checking the Advance scan boundary and permissions.");

    let collected_at_unix = current_unix_timestamp();
    let mut diagnostics = HardwareDiagnosticsV1::not_collected(collected_at_unix);
    diagnostics.benchmarks_consented = request.benchmarks_consented;
    diagnostics.write_benchmark_consented = request.write_benchmark_consented;

    report(1, "Asking Windows and the battery firmware for capacity.");
    let battery = battery_probe::collect(cancellation);
    report(1, &battery_progress_detail(&battery));

    report(2, NOT_COLLECTED);
    report(3, NOT_COLLECTED);
    report(4, NOT_COLLECTED);
    report(5, NOT_COLLECTED);
    report(6, NOT_COLLECTED);
    report(7, NOT_COLLECTED);
    report(
        8,
        if request.benchmarks_consented {
            "Benchmarks were permitted, but none are implemented in this collector version."
        } else {
            "Benchmarks were not permitted, so none were run."
        },
    );

    report(9, "Scoring only the areas that were actually assessed.");
    let outcome = build_report(&diagnostics, &battery, request.device_form);

    report(
        10,
        if outcome.grade_withheld {
            "Report D is ready. No grade was issued."
        } else {
            "Report D is ready."
        },
    );

    outcome
}

fn battery_progress_detail(battery: &BatteryProbe) -> String {
    if let Some(error) = battery.probe_error {
        return error.to_string();
    }
    if let Some(reading) = battery.primary().filter(|_| battery.has_capacity()) {
        return reading.health_percent().map_or_else(
            || "Battery answered but withheld its capacity.".to_string(),
            |health| format!("Battery reported {health:.0}% of its design capacity."),
        );
    }
    if battery.reports_no_battery() {
        return "Windows reports no battery on this chassis.".to_string();
    }
    "The battery did not report a design capacity.".to_string()
}

fn build_report(
    diagnostics: &HardwareDiagnosticsV1,
    battery: &BatteryProbe,
    device_form: DeviceForm,
) -> CustomerAdvanceScan {
    let wrote_without_consent =
        diagnostics.bytes_written > 0 && !diagnostics.write_benchmark_consented;
    let evidence = domain_evidence(diagnostics, battery, device_form);
    let summary = evaluate(&evidence, device_form, &[]);

    CustomerAdvanceScan {
        ok: !wrote_without_consent,
        message: outcome_message(&summary, wrote_without_consent),
        schema_version: diagnostics.schema_version,
        elevation_state: diagnostics.elevation_state.as_str(),
        elevation_label: diagnostics.elevation_state.customer_label(),
        benchmarks_consented: diagnostics.benchmarks_consented,
        write_benchmark_consented: diagnostics.write_benchmark_consented,
        bytes_written: diagnostics.bytes_written,
        temporary_files_note: temporary_files_note(battery),
        destructive_operations_enabled: false,
        content_inspected: false,
        boundary_note: boundary_note(diagnostics),
        telemetry_groups: telemetry_groups(diagnostics, battery),
        coverage_rows: coverage_rows(&summary),
        coverage_domains: coverage_domains(&evidence),
        method_rows: method_rows(),
        rubric_rows: rubric_rows(),
        not_assessable: not_assessable(&evidence),
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
        return format!(
            "Advance scan finished. Coverage {}%. No grade was issued because too little of this device could be assessed.",
            summary.coverage_percent
        );
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
        "Nothing was written to any assessed drive."
    } else {
        "A temporary benchmark file was written and removed."
    };
    format!(
        "Advance scan collection is read-only. {benchmarks} {writes} File contents were not opened. No data was erased. Purge stays off."
    )
}

fn temporary_files_note(battery: &BatteryProbe) -> String {
    if !battery.temporary_file_written {
        return "No temporary file was created by this scan.".to_string();
    }
    if battery.temporary_file_removed {
        return "One temporary battery report was written to the Windows temporary folder by powercfg and then deleted. It was not written to an assessed drive."
            .to_string();
    }
    "One temporary battery report was written to the Windows temporary folder by powercfg and could not be confirmed as deleted."
        .to_string()
}

fn domain_evidence(
    diagnostics: &HardwareDiagnosticsV1,
    battery: &BatteryProbe,
    device_form: DeviceForm,
) -> Vec<DomainEvidence> {
    DiagnosticDomain::ALL
        .iter()
        .map(|domain| match domain {
            DiagnosticDomain::BatteryAndPower => battery_evidence(battery, device_form),
            other => DomainEvidence::not_assessable(*other, domain_gap(*other, diagnostics)),
        })
        .collect()
}

/// The only domain A2 can score. Health bands come from rubric CG-1.0.
fn battery_evidence(battery: &BatteryProbe, device_form: DeviceForm) -> DomainEvidence {
    let domain = DiagnosticDomain::BatteryAndPower;

    if let Some(health) = battery
        .primary()
        .and_then(crate::battery_probe::BatteryReading::health_percent)
    {
        return DomainEvidence::measured(
            domain,
            battery_points(health),
            domain.weight(),
            Confidence::High,
        );
    }

    if battery.reports_no_battery() {
        return match device_form {
            DeviceForm::Fixed => DomainEvidence::not_applicable(
                domain,
                "Windows reports no battery and this chassis is not portable",
            ),
            _ => DomainEvidence::not_assessable(
                domain,
                "Windows reported no battery on a chassis that should have one",
            ),
        };
    }

    DomainEvidence::not_assessable(domain, battery_gap(battery))
}

fn battery_gap(battery: &BatteryProbe) -> &'static str {
    if battery.probe_error.is_some() {
        return "The battery probe could not run on this PC";
    }
    match battery.outcome_for(BatterySource::ManagementClass) {
        SourceOutcome::PermissionDenied => "Windows refused the battery query on this account",
        SourceOutcome::Unsupported => "Windows does not expose a battery class on this PC",
        SourceOutcome::CollectionError => "Windows returned an error for the battery query",
        SourceOutcome::NotQueried => "The battery query did not run",
        SourceOutcome::Reported => "The battery firmware did not report a design capacity",
    }
}

fn domain_gap(domain: DiagnosticDomain, diagnostics: &HardwareDiagnosticsV1) -> &'static str {
    match domain {
        DiagnosticDomain::BatteryAndPower => "Battery telemetry is not collected in this scan",
        DiagnosticDomain::ProcessorAndThermal => {
            if diagnostics.benchmarks_consented {
                "Processor benchmark is not implemented in this collector version"
            } else {
                "Processor benchmark was declined by the operator"
            }
        }
        DiagnosticDomain::MemoryIntegrity => {
            if diagnostics.benchmarks_consented {
                "Memory pattern check is not implemented in this collector version"
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

fn coverage_rows(summary: &CoverageSummary) -> Vec<NamedValue> {
    vec![
        row("Points in scope", summary.in_scope_points.to_string()),
        row("Points assessed", summary.assessed_points.to_string()),
        row("Points awarded", summary.awarded_points.to_string()),
        row(
            "Points not assessable",
            summary.not_assessable_points.to_string(),
        ),
        row("Coverage", format!("{}%", summary.coverage_percent)),
        row(
            "Assessed Health Index",
            summary.index_percent.map_or_else(
                || "Not assessable in this scan".to_string(),
                |index| format!("{index} / 100"),
            ),
        ),
        row("Grading engine", summary.engine.to_string()),
        row("Rubric", summary.rubric.to_string()),
    ]
}

fn coverage_domains(evidence: &[DomainEvidence]) -> Vec<DomainCoverageRow> {
    evidence
        .iter()
        .map(|domain| DomainCoverageRow {
            domain: domain.domain.label().to_string(),
            awarded: domain.awarded,
            assessed: domain.assessed,
            not_assessable: domain.not_assessable_points(),
            weight: domain.domain.weight(),
            state: match domain.applicability {
                DomainApplicability::NotApplicable => "Not applicable".to_string(),
                DomainApplicability::Assessable if domain.assessed == 0 => {
                    "Not assessable".to_string()
                }
                DomainApplicability::Assessable if domain.assessed == domain.domain.weight() => {
                    "Fully assessed".to_string()
                }
                DomainApplicability::Assessable => "Partly assessed".to_string(),
            },
            note: domain
                .note
                .clone()
                .unwrap_or_else(|| "Measured".to_string()),
        })
        .collect()
}

fn method_rows() -> Vec<NamedValue> {
    vec![
        row(
            "Collection mode",
            "Read-only. Windows management classes, firmware tables and Windows' own battery report.".to_string(),
        ),
        row(
            "Battery capacity",
            "Design capacity and full-charge capacity as reported by firmware. Wear is the difference between them, never inferred from a charge level.".to_string(),
        ),
        row(
            "Temperatures and fan speed",
            "Not collected. Reading CPU package temperature or fan RPM requires a kernel-mode sensor driver. CYVRA does not ship one, because the drivers commonly used for this are on Microsoft's vulnerable-driver blocklist.".to_string(),
        ),
        row(
            "Memory testing",
            "A user-mode pattern check can never cover memory the kernel occupies, so full-coverage memory testing belongs to a pre-boot environment.".to_string(),
        ),
        row(
            "Physical ports",
            "Windows exposes controller topology, not the plastic connectors. A port count is only confirmed when a technician inserts a device.".to_string(),
        ),
        row(
            "Unknown values",
            "A value that was not read is printed as not collected. It is never replaced with zero and never estimated.".to_string(),
        ),
    ]
}

fn rubric_rows() -> Vec<NamedValue> {
    DiagnosticDomain::ALL
        .iter()
        .map(|domain| {
            row(
                domain.label(),
                format!("{} points of 100", domain.weight()),
            )
        })
        .chain([
            row(
                "Grade bands",
                "A+ 90-100, A 80-89, B 65-79, C 50-64, F below 50, on the assessed index".to_string(),
            ),
            row(
                "Coverage floor",
                "Below 70% coverage, or with a required area unassessed, the grade is withheld rather than banded".to_string(),
            ),
            row(
                "Confirmed fault",
                "A measured critical fault forces F, because that is evidence held rather than evidence missing".to_string(),
            ),
        ])
        .collect()
}

fn not_assessable(evidence: &[DomainEvidence]) -> Vec<String> {
    evidence
        .iter()
        .filter(|domain| {
            domain.applicability == DomainApplicability::Assessable
                && domain.not_assessable_points() > 0
        })
        .map(|domain| {
            format!(
                "{} — {} ({} of {} points)",
                domain.domain.label(),
                domain
                    .note
                    .clone()
                    .unwrap_or_else(|| "not assessed".to_string()),
                domain.not_assessable_points(),
                domain.domain.weight()
            )
        })
        .collect()
}

fn telemetry_groups(
    diagnostics: &HardwareDiagnosticsV1,
    battery: &BatteryProbe,
) -> Vec<TelemetryGroup> {
    let benchmark_value = if diagnostics.benchmarks_consented {
        NOT_COLLECTED
    } else {
        DECLINED
    };

    let mut groups = vec![battery_group(battery), battery_source_group(battery)];

    groups.extend([
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
    ]);

    groups
}

fn battery_group(battery: &BatteryProbe) -> TelemetryGroup {
    let mut rows = Vec::new();

    if let Some(error) = battery.probe_error {
        rows.push(row("Battery probe", error.to_string()));
        return TelemetryGroup {
            title: "Battery and power".to_string(),
            note: None,
            rows,
        };
    }

    let Some(reading) = battery.primary() else {
        let value = if battery.reports_no_battery() {
            "Windows reports no battery on this chassis. This is expected on a desktop.".to_string()
        } else {
            format!("Not collected in this scan. {}.", battery_gap(battery))
        };
        rows.push(row("Battery present", value));
        return TelemetryGroup {
            title: "Battery and power".to_string(),
            note: None,
            rows,
        };
    };

    let unit = reading.capacity_unit();
    rows.push(row("Battery present", "Yes".to_string()));
    rows.push(row("Packs reported", battery.readings.len().to_string()));
    rows.push(optional_row("Battery name", reading.device_name.clone()));
    rows.push(optional_row("Manufacturer", reading.manufacturer.clone()));
    rows.push(optional_row("Chemistry", reading.chemistry.clone()));
    rows.push(optional_row("Serial number", reading.serial_number.clone()));
    rows.push(optional_row(
        "Manufacture date",
        reading.manufacture_date.clone(),
    ));
    rows.push(optional_row(
        "Design capacity",
        reading
            .designed_capacity
            .map(|value| format!("{value}{unit}")),
    ));
    rows.push(optional_row(
        "Full charge capacity",
        reading
            .full_charge_capacity
            .map(|value| format!("{value}{unit}")),
    ));
    rows.push(optional_row(
        "Battery wear",
        reading.wear_percent().map(|wear| format!("{wear:.0}%")),
    ));
    rows.push(optional_row(
        "Battery health",
        reading
            .health_percent()
            .map(|health| format!("{health:.0}% of design capacity")),
    ));
    rows.push(optional_row(
        "Health classification",
        reading.health_band().map(str::to_string),
    ));
    rows.push(optional_row(
        "Cycle count",
        reading.cycle_count.map(|value| value.to_string()),
    ));
    rows.push(optional_row(
        "Charge level now",
        reading
            .charge_percent
            .map(|percent| format!("{percent}% (charge level, not health)")),
    ));
    rows.push(optional_row(
        "Design voltage",
        reading.design_voltage_mv.map(|value| format!("{value} mV")),
    ));
    rows.push(optional_row("Power state", reading.status_text.clone()));
    rows.push(optional_row(
        "Capacity reported by",
        reading
            .capacity_source
            .map(|source| source.label().to_string()),
    ));

    TelemetryGroup {
        title: "Battery and power".to_string(),
        note: battery.relative_capacity_note().map(str::to_string),
        rows,
    }
}

/// Printing which source answered is what turns "not collected" into a fault
/// an engineer can act on.
fn battery_source_group(battery: &BatteryProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "Battery sources consulted".to_string(),
        note: Some(
            "Advance scan asks every source Windows offers and records the answer, so a missing value can be explained rather than guessed."
                .to_string(),
        ),
        rows: battery
            .sources
            .iter()
            .map(|status| {
                row(
                    status.source.label(),
                    status.outcome.customer_label().to_string(),
                )
            })
            .collect(),
    }
}

fn optional_row(label: &str, value: Option<String>) -> NamedValue {
    row(
        label,
        value.unwrap_or_else(|| "Not reported by firmware".to_string()),
    )
}

fn group(title: &str, rows: &[(&str, &str)]) -> TelemetryGroup {
    TelemetryGroup {
        title: title.to_string(),
        note: None,
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
    use crate::battery_probe::parse_probe;

    fn hex(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn probe_from(lines: &[(&str, usize, &str, &str)]) -> BatteryProbe {
        let text = lines
            .iter()
            .map(|(section, index, name, value)| {
                format!("{section}\t{index}\t{name}\t{}", hex(value))
            })
            .collect::<Vec<_>>()
            .join("\n");
        parse_probe(&text)
    }

    fn healthy_probe() -> BatteryProbe {
        probe_from(&[
            ("battery", 0, "source_status", "reported"),
            ("battery", 0, "present", "True"),
            ("battery", 0, "name", "Primary"),
            ("battery", 0, "chemistry_code", "6"),
            ("battery", 0, "designed_capacity", "45000"),
            ("battery", 0, "full_charge_capacity", "39600"),
        ])
    }

    fn report_for(probe: &BatteryProbe, form: DeviceForm) -> CustomerAdvanceScan {
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        build_report(&diagnostics, probe, form)
    }

    fn group_named<'a>(outcome: &'a CustomerAdvanceScan, title: &str) -> &'a TelemetryGroup {
        outcome
            .telemetry_groups
            .iter()
            .find(|group| group.title == title)
            .expect("group must exist")
    }

    fn value_of(outcome: &CustomerAdvanceScan, title: &str, label: &str) -> String {
        group_named(outcome, title)
            .rows
            .iter()
            .find(|row| row.label == label)
            .map(|row| row.value.clone())
            .unwrap_or_default()
    }

    #[test]
    fn advance_scan_is_non_destructive_and_writes_nothing() {
        let outcome = run_advance_scan(&AdvanceScanRequest::default());

        assert!(outcome.ok);
        assert!(!outcome.destructive_operations_enabled);
        assert!(!outcome.content_inspected);
        assert_eq!(outcome.bytes_written, 0);
        assert!(outcome.boundary_note.contains("No data was erased"));
    }

    #[test]
    fn progress_visits_every_stage_in_order_and_ends_at_one_hundred() {
        let mut seen: Vec<(u8, u8, String)> = Vec::new();
        let outcome = run_advance_scan_with(
            &AdvanceScanRequest::default(),
            &CancellationToken::new(),
            |percent, index, stage, detail| {
                assert!(!detail.is_empty(), "stage {stage} reported no detail");
                seen.push((percent, index, stage.to_string()));
            },
        );

        let indices: Vec<u8> = seen.iter().map(|(_, index, _)| *index).collect();
        for step in 0..STAGES.len() {
            assert!(
                indices.contains(&(step as u8)),
                "stage {step} was never reported"
            );
        }
        assert!(indices.windows(2).all(|pair| pair[0] <= pair[1]));
        assert_eq!(seen.first().expect("first").0, 0);
        assert_eq!(seen.last().expect("last").0, 100);
        assert_eq!(
            seen.last().expect("last").2,
            "Preparing Report D".to_string()
        );
        assert!(outcome.ok);
    }

    #[test]
    fn a_healthy_battery_earns_points_and_is_printed_with_its_source() {
        let outcome = report_for(&healthy_probe(), DeviceForm::Portable);

        assert_eq!(
            value_of(&outcome, "Battery and power", "Battery wear"),
            "12%"
        );
        assert_eq!(
            value_of(&outcome, "Battery and power", "Battery health"),
            "88% of design capacity"
        );
        assert_eq!(
            value_of(&outcome, "Battery and power", "Health classification"),
            "Good"
        );
        assert_eq!(
            value_of(&outcome, "Battery and power", "Capacity reported by"),
            "Windows battery class"
        );
        assert_eq!(
            value_of(&outcome, "Battery and power", "Design capacity"),
            "45000 mWh"
        );

        let battery_domain = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Battery and power")
            .expect("battery domain");
        assert_eq!(battery_domain.assessed, 20);
        assert_eq!(battery_domain.awarded, 20);
        assert_eq!(battery_domain.state, "Fully assessed");
        assert_eq!(outcome.coverage_percent, 20);
    }

    #[test]
    fn one_readable_domain_still_withholds_the_grade_when_storage_is_unknown() {
        let outcome = report_for(&healthy_probe(), DeviceForm::Portable);

        assert!(outcome.grade_withheld);
        assert_eq!(outcome.grade_label, "Grade withheld");
        assert_eq!(outcome.index_percent, Some(100));
        assert!(
            outcome
                .grade_withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("Storage health")
        );
    }

    #[test]
    fn a_refused_battery_query_explains_itself_on_the_report() {
        let probe = probe_from(&[("battery", 0, "source_status", "permission_denied")]);
        let outcome = report_for(&probe, DeviceForm::Portable);

        assert!(
            value_of(&outcome, "Battery and power", "Battery present")
                .contains("refused the battery query")
        );
        assert_eq!(
            value_of(
                &outcome,
                "Battery sources consulted",
                "Windows battery class"
            ),
            "Refused without administrator rights"
        );
        assert!(
            outcome
                .not_assessable
                .iter()
                .any(|entry| entry.contains("refused the battery query"))
        );
    }

    #[test]
    fn a_desktop_without_a_battery_loses_the_weight_instead_of_the_points() {
        let probe = probe_from(&[
            ("battery", 0, "source_status", "reported"),
            ("battery", 0, "record_count", "0"),
        ]);
        let outcome = report_for(&probe, DeviceForm::Fixed);

        let battery_domain = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Battery and power")
            .expect("battery domain");
        assert_eq!(battery_domain.state, "Not applicable");
        assert_eq!(battery_domain.not_assessable, 0);
        assert!(
            !outcome
                .not_assessable
                .iter()
                .any(|entry| entry.contains("Battery"))
        );
        assert!(
            value_of(&outcome, "Battery and power", "Battery present")
                .contains("expected on a desktop")
        );
    }

    #[test]
    fn a_missing_battery_on_a_laptop_is_a_gap_not_an_exemption() {
        let probe = probe_from(&[
            ("battery", 0, "source_status", "reported"),
            ("battery", 0, "record_count", "0"),
        ]);
        let outcome = report_for(&probe, DeviceForm::Portable);

        let battery_domain = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Battery and power")
            .expect("battery domain");
        assert_eq!(battery_domain.state, "Not assessable");
        assert_eq!(battery_domain.not_assessable, 20);
    }

    #[test]
    fn a_charge_level_is_never_presented_as_health() {
        let probe = probe_from(&[
            ("battery", 0, "source_status", "reported"),
            ("battery", 0, "present", "True"),
            ("battery", 0, "charge_percent", "64"),
        ]);
        let outcome = report_for(&probe, DeviceForm::Portable);

        assert_eq!(
            value_of(&outcome, "Battery and power", "Battery health"),
            "Not reported by firmware"
        );
        assert!(value_of(&outcome, "Battery and power", "Charge level now").contains("not health"));
    }

    #[test]
    fn a_temporary_firmware_report_is_disclosed_and_kept_off_assessed_drives() {
        let probe = probe_from(&[
            ("battery", 0, "source_status", "reported"),
            ("battery", 0, "present", "True"),
            ("firmware", 0, "source_status", "reported"),
            ("firmware", 0, "temporary_file_written", "True"),
            ("firmware", 0, "temporary_file_removed", "True"),
            ("firmware", 0, "designed_capacity", "45000"),
            ("firmware", 0, "full_charge_capacity", "22000"),
        ]);
        let outcome = report_for(&probe, DeviceForm::Portable);

        assert!(outcome.temporary_files_note.contains("deleted"));
        assert!(
            outcome
                .temporary_files_note
                .contains("not written to an assessed drive")
        );
        assert_eq!(outcome.bytes_written, 0);
        assert_eq!(
            value_of(&outcome, "Battery and power", "Health classification"),
            "Critical"
        );
    }

    #[test]
    fn every_unread_row_says_so_and_none_invents_a_number() {
        let outcome = report_for(&healthy_probe(), DeviceForm::Portable);

        for group in &outcome.telemetry_groups {
            if group.title.starts_with("Battery") {
                continue;
            }
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
    fn report_d_explains_its_method_rubric_and_gaps() {
        let outcome = report_for(&healthy_probe(), DeviceForm::Portable);

        assert_eq!(outcome.coverage_domains.len(), 6);
        assert!(outcome.method_rows.len() >= 6);
        assert!(outcome.rubric_rows.len() >= 9);
        assert!(
            outcome
                .method_rows
                .iter()
                .any(|row| row.value.contains("vulnerable-driver blocklist"))
        );
        assert!(
            outcome
                .rubric_rows
                .iter()
                .any(|row| row.label == "Coverage floor")
        );
        assert_eq!(outcome.grading_engine, "CYVRA Grading Engine");
        assert!(!outcome.grading_engine.contains("AI"));
    }

    #[test]
    fn a_recorded_write_without_consent_fails_closed() {
        let mut diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        diagnostics.bytes_written = 4_096;

        let outcome = build_report(&diagnostics, &healthy_probe(), DeviceForm::Portable);

        assert!(!outcome.ok);
        assert!(outcome.message.contains("without consent"));
    }
}
