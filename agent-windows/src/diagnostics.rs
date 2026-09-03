//! Advance scan orchestrator.
//!
//! Advance scan is the deep, opt-in counterpart to the basic local assessment.
//! It walks each subsystem in turn, reports progress as it goes, records what
//! it could not read and why, and only then asks the grading engine for a
//! verdict. Nothing here infers a value it did not read.

use crate::NamedValue;
use crate::advance_bench::{self, BenchResult};
use crate::battery_probe::{self, BatteryProbe, BatterySource, SourceOutcome};
use crate::capture_probe::{self, CaptureProbe};
use crate::collector_runtime::CancellationToken;
use crate::cpu_memory::{self, CpuMemoryProbe};
use crate::display_radio::{self, DisplayRadioProbe, DisplayRadioSource};
use crate::hardware_diagnostics_v1::{
    CoverageSummary, DeviceForm, DiagnosticDomain, DomainApplicability, DomainEvidence,
    ElevationState, HardwareDiagnosticsV1, InteractiveAttestations, PhysicalPortAttestation,
    battery_points, bluetooth_radio_points, ethernet_radio_points, evaluate,
    memory_bandwidth_points, memory_inventory_points, memory_pattern_points,
    processor_clock_points, processor_identity_points, usb_topology_points, wifi_radio_points,
};
use crate::hardware_inventory_v1::Confidence;
use crate::storage_health::{self, StorageProbe, StorageSource};
use crate::usb_topology::{self, UsbProbe, UsbSource};
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// The frozen customer phrase for a subsystem this build cannot read yet.
const NOT_COLLECTED: &str = "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.";

/// Used where the gap is architectural rather than merely unwritten.
const NEEDS_KERNEL_SENSOR: &str = "Not collected in this scan. This value requires a kernel-mode sensor driver, which CYVRA deliberately does not ship.";

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
    pub interactive: InteractiveAttestations,
}

impl Default for AdvanceScanRequest {
    fn default() -> Self {
        Self {
            benchmarks_consented: false,
            write_benchmark_consented: false,
            device_form: DeviceForm::Unknown,
            interactive: InteractiveAttestations::default(),
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

    report(2, "Reading processor identity and cache.");
    let identity = cpu_memory::collect(cancellation);
    report(2, &processor_progress_detail(&identity));

    report(3, "Reading memory modules and installed capacity.");
    report(3, &memory_progress_detail(&identity));

    report(
        4,
        "Asking Windows for disk identity and SMART. Nothing is written, erased or trimmed.",
    );
    let storage = storage_health::collect(cancellation);
    report(4, &storage_progress_detail(&storage));
    diagnostics.elevation_state = if storage.elevated == Some(true) {
        ElevationState::Granted
    } else {
        ElevationState::NotRequested
    };

    report(5, "Walking USB controllers, hubs and attached devices.");
    let usb = usb_topology::collect(cancellation);
    report(5, &usb_progress_detail(&usb));

    report(
        5,
        "Reading Wi-Fi, Bluetooth and Ethernet. MAC addresses are not collected.",
    );
    let radios = display_radio::collect(cancellation);
    report(5, &radio_progress_detail(&radios));

    report(
        6,
        "Reading panel identity from EDID. Native resolution is the preferred timing, not the current desktop mode.",
    );
    report(6, &display_progress_detail(&radios));

    report(
        7,
        "Enumerating cameras and microphones. No image or audio is captured.",
    );
    let capture = capture_probe::collect(cancellation);
    report(7, &capture_progress_detail(&capture));
    report(
        8,
        if request.benchmarks_consented {
            "Running consented CPU, memory and storage workloads. Package temperature is not collected."
        } else {
            "Benchmarks were not permitted, so none were run."
        },
    );
    let benches = advance_bench::run(
        request.benchmarks_consented,
        request.write_benchmark_consented,
        &battery,
        &storage,
        &identity,
        cancellation,
    );
    diagnostics.bytes_written = benches.bytes_written;
    report(
        8,
        &bench_progress_detail(&benches, request.benchmarks_consented),
    );

    report(
        9,
        "Scoring only the areas that were actually assessed, including technician attestations.",
    );
    let outcome = build_report(
        &diagnostics,
        &battery,
        &storage,
        &usb,
        &radios,
        &capture,
        &identity,
        &benches,
        request.interactive,
        request.device_form,
    );

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

fn storage_progress_detail(storage: &StorageProbe) -> String {
    if let Some(error) = storage.probe_error {
        return error.to_string();
    }
    if storage.reliability_refused() {
        return "Windows refused storage SMART on this account. Report D will still be written."
            .to_string();
    }
    let count = storage.drives.len();
    if count == 0 {
        return "Windows did not name a storage device.".to_string();
    }
    if storage.awarded_points().is_some() {
        return format!(
            "Read identity and SMART on {count} storage device(s). No data was erased."
        );
    }
    format!("Identified {count} storage device(s). SMART health was not returned.")
}

fn usb_progress_detail(usb: &UsbProbe) -> String {
    if let Some(error) = usb.probe_error {
        return error.to_string();
    }
    if !usb.topology_enumerated() {
        return "Windows did not enumerate USB controllers on this PC.".to_string();
    }
    format!(
        "USB topology: {} controller(s), {} hub(s), {} attached device(s). Empty plastic connectors are not visible.",
        usb.controllers.len(),
        usb.hubs.len(),
        usb.devices.len()
    )
}

fn radio_progress_detail(radios: &DisplayRadioProbe) -> String {
    if let Some(error) = radios.probe_error {
        return error.to_string();
    }
    let mut parts = Vec::new();
    if radios.wifi_reporting() {
        parts.push("Wi-Fi adapter reporting");
    }
    if radios.bluetooth_present() {
        parts.push("Bluetooth present");
    }
    if radios.ethernet_link_readable() {
        parts.push("Ethernet link state readable");
    }
    if parts.is_empty() {
        return "No Wi-Fi, Bluetooth or Ethernet adapter reported on this PC. MAC addresses were not collected.".to_string();
    }
    format!("{}. MAC addresses were not collected.", parts.join("; "))
}

fn display_progress_detail(radios: &DisplayRadioProbe) -> String {
    if let Some(error) = radios.probe_error {
        return error.to_string();
    }
    let Some(panel) = radios.panels.first() else {
        return "Windows did not name a display panel in this scan.".to_string();
    };
    match (panel.native_width, panel.native_height) {
        (Some(width), Some(height)) => format!(
            "Panel native resolution {width}×{height} from EDID preferred timing, not the current desktop mode."
        ),
        _ => {
            if panel.identified() {
                format!(
                    "Panel {} identified. Native resolution was not in the EDID block.",
                    panel.display_name()
                )
            } else {
                "Display identity was not returned. Native resolution stays not collected."
                    .to_string()
            }
        }
    }
}

fn processor_progress_detail(identity: &CpuMemoryProbe) -> String {
    if let Some(error) = identity.probe_error {
        return error.to_string();
    }
    match &identity.processor {
        Some(cpu) if cpu.identity_complete() => format!(
            "Processor {} · {} core(s). Cache {}.",
            cpu.name.as_deref().unwrap_or("named"),
            cpu.cores.unwrap_or(0),
            cpu.cache_summary().as_deref().unwrap_or("not reported")
        ),
        Some(cpu) => format!(
            "Processor {} answered without a complete identity (model, cores and cache).",
            cpu.name.as_deref().unwrap_or("unnamed")
        ),
        None => "Windows did not name a processor in this scan.".to_string(),
    }
}

fn memory_progress_detail(identity: &CpuMemoryProbe) -> String {
    if let Some(error) = identity.probe_error {
        return error.to_string();
    }
    if identity.inventory_complete() {
        let modules = identity.modules.len();
        return format!("Memory inventory: {modules} module(s). Channel mode is not inferred.");
    }
    "Memory modules were not fully reported in this scan.".to_string()
}

fn bench_progress_detail(benches: &BenchResult, consented: bool) -> String {
    if !consented {
        return "Benchmarks were not permitted, so none were run.".to_string();
    }
    if benches.bytes_written > 0 {
        return format!(
            "Consented workloads finished. {} bytes were written to the Windows temporary folder.",
            benches.bytes_written
        );
    }
    "Consented CPU, memory and storage read workloads finished. Nothing was written to an assessed drive.".to_string()
}

fn capture_progress_detail(capture: &CaptureProbe) -> String {
    if let Some(error) = capture.probe_error {
        return error.to_string();
    }
    let cameras = capture.camera_names().len();
    let mics = capture.microphone_names().len();
    format!("Found {cameras} camera(s) and {mics} microphone(s). No image or audio was captured.")
}

#[allow(clippy::too_many_arguments)]
fn build_report(
    diagnostics: &HardwareDiagnosticsV1,
    battery: &BatteryProbe,
    storage: &StorageProbe,
    usb: &UsbProbe,
    radios: &DisplayRadioProbe,
    capture: &CaptureProbe,
    identity: &CpuMemoryProbe,
    benches: &BenchResult,
    interactive: InteractiveAttestations,
    device_form: DeviceForm,
) -> CustomerAdvanceScan {
    let wrote_without_consent =
        diagnostics.bytes_written > 0 && !diagnostics.write_benchmark_consented;
    let evidence = domain_evidence(
        battery,
        storage,
        usb,
        radios,
        identity,
        benches,
        interactive,
        device_form,
    );
    let mut criticals = storage.critical_faults();
    if let Some(fault) = benches.memory_critical() {
        criticals.push(fault);
    }
    let summary = evaluate(&evidence, device_form, &criticals);

    CustomerAdvanceScan {
        ok: !wrote_without_consent,
        message: outcome_message(&summary, wrote_without_consent),
        schema_version: diagnostics.schema_version,
        elevation_state: diagnostics.elevation_state.as_str(),
        elevation_label: diagnostics.elevation_state.customer_label(),
        benchmarks_consented: diagnostics.benchmarks_consented,
        write_benchmark_consented: diagnostics.write_benchmark_consented,
        bytes_written: diagnostics.bytes_written,
        temporary_files_note: temporary_files_note(battery, benches),
        destructive_operations_enabled: false,
        content_inspected: false,
        boundary_note: boundary_note(diagnostics),
        telemetry_groups: telemetry_groups(
            diagnostics,
            battery,
            storage,
            usb,
            radios,
            capture,
            identity,
            benches,
            interactive,
        ),
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

fn temporary_files_note(battery: &BatteryProbe, benches: &BenchResult) -> String {
    let mut parts = Vec::new();
    if battery.temporary_file_written {
        if battery.temporary_file_removed {
            parts.push("One temporary battery report was written to the Windows temporary folder by powercfg and then deleted. It was not written to an assessed drive.");
        } else {
            parts.push("One temporary battery report was written to the Windows temporary folder by powercfg and could not be confirmed as deleted.");
        }
    }
    if benches.bytes_written > 0 {
        if benches.temporary_file_removed {
            parts.push("One temporary benchmark file was written to the Windows temporary folder and then deleted. It was not written as a wipe.");
        } else {
            parts.push("One temporary benchmark file was written to the Windows temporary folder and could not be confirmed as deleted.");
        }
    }
    if parts.is_empty() {
        return "No temporary file was created by this scan.".to_string();
    }
    parts.join(" ")
}

#[allow(clippy::too_many_arguments)]
fn domain_evidence(
    battery: &BatteryProbe,
    storage: &StorageProbe,
    usb: &UsbProbe,
    radios: &DisplayRadioProbe,
    identity: &CpuMemoryProbe,
    benches: &BenchResult,
    interactive: InteractiveAttestations,
    device_form: DeviceForm,
) -> Vec<DomainEvidence> {
    DiagnosticDomain::ALL
        .iter()
        .map(|domain| match domain {
            DiagnosticDomain::BatteryAndPower => battery_evidence(battery, device_form),
            DiagnosticDomain::ProcessorAndThermal => processor_evidence(identity, benches),
            DiagnosticDomain::MemoryIntegrity => memory_evidence(identity, benches),
            DiagnosticDomain::StorageHealth => storage_evidence(storage),
            DiagnosticDomain::PortsAndConnectivity => ports_evidence(usb, radios, interactive),
            DiagnosticDomain::ScreenAndPeripherals => screen_evidence(interactive),
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

fn storage_evidence(storage: &StorageProbe) -> DomainEvidence {
    let domain = DiagnosticDomain::StorageHealth;
    if let Some(awarded) = storage.awarded_points() {
        let mut evidence =
            DomainEvidence::measured(domain, awarded, domain.weight(), Confidence::High);
        evidence.note = Some(
            "Storage SMART was read. Power-on hours and wear are printed; they are not scored in CG-1.0."
                .to_string(),
        );
        return evidence;
    }
    DomainEvidence::not_assessable(domain, storage_gap(storage))
}

fn processor_evidence(identity: &CpuMemoryProbe, benches: &BenchResult) -> DomainEvidence {
    let domain = DiagnosticDomain::ProcessorAndThermal;
    let identity_pts = processor_identity_points(
        identity
            .processor
            .as_ref()
            .is_some_and(crate::cpu_memory::ProcessorIdentity::identity_complete),
    );
    let clock_measured = benches.cpu.ratio.is_some();
    let clock_pts = benches
        .cpu
        .ratio
        .map(|percent| processor_clock_points(f64::from(percent) / 100.0))
        .unwrap_or(0);
    let assessed = identity_pts + if clock_measured { 16 } else { 0 };
    if assessed == 0 {
        return DomainEvidence::not_assessable(
            domain,
            if identity.probe_error.is_some() {
                "The processor identity probe could not run on this PC"
            } else {
                "Processor identity was not collected in this scan"
            },
        );
    }
    let mut evidence =
        DomainEvidence::measured(domain, identity_pts + clock_pts, assessed, Confidence::High);
    evidence.note = Some(
        "Package temperature is not collected. Clock points are awarded only after a consented workload.".to_string(),
    );
    evidence
}

fn memory_evidence(identity: &CpuMemoryProbe, benches: &BenchResult) -> DomainEvidence {
    let domain = DiagnosticDomain::MemoryIntegrity;
    if benches.memory.pattern_passed == Some(false) {
        let mut evidence = DomainEvidence::measured(domain, 0, domain.weight(), Confidence::High);
        evidence.note = Some(
            "Memory pattern spot check failed. The domain is scored 0 because that is evidence held."
                .to_string(),
        );
        return evidence;
    }
    let inventory_pts = memory_inventory_points(identity.inventory_complete());
    let pattern_run = benches.memory.pattern_passed.is_some();
    let pattern_pts = memory_pattern_points(benches.memory.pattern_passed);
    let bandwidth_run = benches.memory.bandwidth_mib_s.is_some();
    let bandwidth_pts = memory_bandwidth_points(benches.memory.bandwidth_mib_s);
    let assessed =
        inventory_pts + if pattern_run { 7 } else { 0 } + if bandwidth_run { 3 } else { 0 };
    if assessed == 0 {
        return DomainEvidence::not_assessable(
            domain,
            if identity.probe_error.is_some() {
                "The memory inventory probe could not run on this PC"
            } else {
                "Memory inventory was not collected in this scan"
            },
        );
    }
    let mut evidence = DomainEvidence::measured(
        domain,
        inventory_pts + pattern_pts + bandwidth_pts,
        assessed,
        Confidence::High,
    );
    evidence.note = Some(
        "This is a user-mode pattern spot check, not full-coverage memory testing.".to_string(),
    );
    evidence
}

fn storage_gap(storage: &StorageProbe) -> &'static str {
    if storage.probe_error.is_some() {
        return "The storage health probe could not run on this PC";
    }
    if storage.reliability_refused() {
        return "Windows refused the storage SMART query on this account";
    }
    match storage.outcome_for(StorageSource::ReliabilityCounter) {
        storage_health::SourceOutcome::PermissionDenied => {
            "Windows refused the storage SMART query on this account"
        }
        storage_health::SourceOutcome::Unsupported => {
            "Windows does not expose storage reliability counters on this PC"
        }
        storage_health::SourceOutcome::CollectionError => {
            "Windows returned an error for the storage SMART query"
        }
        storage_health::SourceOutcome::NotQueried | storage_health::SourceOutcome::Reported => {
            "Storage SMART telemetry was not returned for the disks on this PC"
        }
    }
}

fn ports_evidence(
    usb: &UsbProbe,
    radios: &DisplayRadioProbe,
    interactive: InteractiveAttestations,
) -> DomainEvidence {
    let domain = DiagnosticDomain::PortsAndConnectivity;
    let topology = usb_topology_points(usb.topology_enumerated())
        + wifi_radio_points(radios.wifi_reporting())
        + bluetooth_radio_points(radios.bluetooth_present())
        + ethernet_radio_points(radios.ethernet_link_readable());
    let (port_awarded, port_assessed) = interactive.physical_ports.points();
    let awarded = topology + port_awarded;
    let assessed = topology + port_assessed;
    if assessed == 0 {
        return DomainEvidence::not_assessable(domain, ports_gap(usb, radios));
    }
    let mut evidence = DomainEvidence::measured(domain, awarded, assessed, Confidence::High);
    evidence.note = Some(ports_note(usb, radios, interactive, awarded, assessed));
    evidence
}

fn screen_evidence(interactive: InteractiveAttestations) -> DomainEvidence {
    let domain = DiagnosticDomain::ScreenAndPeripherals;
    let (awarded, assessed) = interactive.screen_points();
    if assessed == 0 {
        return DomainEvidence::not_assessable(
            domain,
            "Interactive technician checks were not attempted in this scan",
        );
    }
    let mut evidence = DomainEvidence::measured(domain, awarded, assessed, Confidence::High);
    evidence.note = Some(
        "Screen, keyboard and peripheral points are operator-attested. Live camera and microphone capture is not part of this scan.".to_string(),
    );
    evidence
}

fn ports_note(
    usb: &UsbProbe,
    radios: &DisplayRadioProbe,
    interactive: InteractiveAttestations,
    awarded: u32,
    assessed: u32,
) -> String {
    let mut parts = Vec::new();
    if usb.topology_enumerated() {
        parts.push("USB controller topology enumerated");
    }
    if radios.wifi_reporting() {
        parts.push("Wi-Fi adapter reporting");
    }
    if radios.bluetooth_present() {
        parts.push("Bluetooth radio present");
    }
    if radios.ethernet_link_readable() {
        parts.push("Ethernet link state readable");
    }
    match interactive.physical_ports {
        PhysicalPortAttestation::NotAttempted => parts.push("physical insertion not attempted"),
        PhysicalPortAttestation::AllPassed => parts.push("all attempted ports passed"),
        PhysicalPortAttestation::Partial => parts.push("some attempted ports passed"),
        PhysicalPortAttestation::AnyFailed => parts.push("an attempted port failed"),
    }
    format!(
        "{}. {} of {assessed} assessed ports points awarded ({} of 10). MAC addresses were not collected.",
        parts.join("; "),
        awarded,
        awarded
    )
}

fn ports_gap(usb: &UsbProbe, radios: &DisplayRadioProbe) -> &'static str {
    if usb.probe_error.is_some() && radios.probe_error.is_some() {
        return "Port topology and radio telemetry could not run on this PC";
    }
    if !usb.topology_enumerated() {
        return usb_gap(usb);
    }
    "Wi-Fi, Bluetooth and Ethernet adapters were not returned in this scan"
}

fn usb_gap(usb: &UsbProbe) -> &'static str {
    if usb.probe_error.is_some() {
        return "The USB topology probe could not run on this PC";
    }
    match usb.outcome_for(UsbSource::Controller) {
        crate::usb_topology::SourceOutcome::PermissionDenied => {
            "Windows refused the USB controller query on this account"
        }
        crate::usb_topology::SourceOutcome::Unsupported => {
            "Windows does not expose USB controllers on this PC"
        }
        crate::usb_topology::SourceOutcome::CollectionError => {
            "Windows returned an error for the USB controller query"
        }
        crate::usb_topology::SourceOutcome::NotQueried => "The USB controller query did not run",
        crate::usb_topology::SourceOutcome::Reported => {
            "USB controller topology was not collected in this scan"
        }
    }
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
            "Read-only. Windows management classes, firmware tables, Windows' own battery report, storage reliability counters, EDID, and network adapters. MAC addresses are never collected.".to_string(),
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
            "A user-mode pattern check can never cover memory the kernel occupies, so full-coverage memory testing belongs to a pre-boot environment. Advance scan never prints 'memory verified'.".to_string(),
        ),
        row(
            "Processor clock",
            "Identity is collected without a workload. The 16 sustained-clock points are awarded only after a consented CPU loop, from Windows current/max megahertz. Package temperature is not collected.".to_string(),
        ),
        row(
            "Benchmarks",
            "CPU, memory and storage-read workloads run only when the operator allows benchmarks. The write test needs a second permission, writes one temporary file, then deletes it. Predicted-failure disks are not exercised.".to_string(),
        ),
        row(
            "Storage SMART",
            "Advance scan reads disk identity, Windows storage reliability counters and the SMART predict-failure bit. It never issues a write, erase, TRIM, format, sanitize or firmware-update command. Available spare is printed only when Windows returns it.".to_string(),
        ),
        row(
            "Physical ports",
            "Windows exposes USB controller topology and attached devices, not the plastic connectors. A port is only confirmed when a technician inserts a test device. Insertion is operator-attested; this scan does not write to the stick or to an assessed drive.".to_string(),
        ),
        row(
            "Display panel",
            "Native width and height come from the EDID preferred timing, never from the current desktop mode. HDR is not guessed. Colour-wash points are awarded only after a technician attests the inspection.".to_string(),
        ),
        row(
            "Radios",
            "Wi-Fi, Bluetooth and Ethernet adapters are enumerated without printing a MAC address. Signal quality is printed only when Windows returns it.".to_string(),
        ),
        row(
            "Cameras and microphones",
            "Advance scan enumerates capture devices across several PnP classes, including the USB video service that the Camera ClassGuid misses. No frame is captured and no audio is recorded. Presence confirmation is an operator attestation, not a live preview.".to_string(),
        ),
        row(
            "Keyboard",
            "A webview cannot see Fn combinations and some OEM hotkeys. Keyboard points are awarded only when the operator attests the keys they could try. Keystrokes are not stored.".to_string(),
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

#[allow(clippy::too_many_arguments)]
fn telemetry_groups(
    _diagnostics: &HardwareDiagnosticsV1,
    battery: &BatteryProbe,
    storage: &StorageProbe,
    usb: &UsbProbe,
    radios: &DisplayRadioProbe,
    capture: &CaptureProbe,
    identity: &CpuMemoryProbe,
    benches: &BenchResult,
    interactive: InteractiveAttestations,
) -> Vec<TelemetryGroup> {
    let mut groups = vec![battery_group(battery), battery_source_group(battery)];

    groups.extend([
        processor_group(identity),
        memory_group(identity),
        identity_source_group(identity),
        storage_group(storage),
        storage_source_group(storage),
        usb_group(usb, radios, interactive),
        usb_source_group(usb),
        radio_source_group(radios),
        display_group(radios),
        display_source_group(radios),
        capture_group(capture),
        capture_source_group(capture),
        bench_group(benches),
        interactive_group(interactive),
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

fn processor_group(identity: &CpuMemoryProbe) -> TelemetryGroup {
    let mut rows = Vec::new();
    if let Some(error) = identity.probe_error {
        rows.push(row("Processor probe", error.to_string()));
        rows.push(row("Package temperature", NEEDS_KERNEL_SENSOR.to_string()));
        rows.push(row("Fan speed", NEEDS_KERNEL_SENSOR.to_string()));
        return TelemetryGroup {
            title: "Processor and thermal".to_string(),
            note: None,
            rows,
        };
    }
    let Some(cpu) = &identity.processor else {
        rows.push(row(
            "Processor",
            "Not collected in this scan. Windows did not name a processor.".to_string(),
        ));
        rows.push(row("Package temperature", NEEDS_KERNEL_SENSOR.to_string()));
        rows.push(row("Fan speed", NEEDS_KERNEL_SENSOR.to_string()));
        return TelemetryGroup {
            title: "Processor and thermal".to_string(),
            note: None,
            rows,
        };
    };
    rows.push(optional_row("Processor", cpu.name.clone()));
    rows.push(optional_row("Manufacturer", cpu.manufacturer.clone()));
    rows.push(optional_row(
        "Cores",
        cpu.cores.map(|cores| cores.to_string()),
    ));
    rows.push(optional_row(
        "Logical processors",
        cpu.logical_processors.map(|count| count.to_string()),
    ));
    rows.push(optional_row(
        "Maximum clock",
        cpu.max_mhz.map(|mhz| format!("{mhz} MHz")),
    ));
    rows.push(optional_row(
        "Current clock (idle sample)",
        cpu.current_mhz.map(|mhz| format!("{mhz} MHz")),
    ));
    rows.push(optional_row("Cache hierarchy", cpu.cache_summary()));
    rows.push(row("Instruction sets", NOT_COLLECTED.to_string()));
    rows.push(row("Package temperature", NEEDS_KERNEL_SENSOR.to_string()));
    rows.push(row("Fan speed", NEEDS_KERNEL_SENSOR.to_string()));
    TelemetryGroup {
        title: "Processor and thermal".to_string(),
        note: Some(
            "Idle clock is not a load measurement. Sustained-clock points wait for a consented workload."
                .to_string(),
        ),
        rows,
    }
}

fn memory_group(identity: &CpuMemoryProbe) -> TelemetryGroup {
    let mut rows = Vec::new();
    if let Some(error) = identity.probe_error {
        rows.push(row("Memory probe", error.to_string()));
        return TelemetryGroup {
            title: "Memory".to_string(),
            note: None,
            rows,
        };
    }
    rows.push(optional_row(
        "Installed total",
        identity
            .installed_bytes
            .map(|bytes| format!("{bytes} bytes")),
    ));
    rows.push(optional_row(
        "Available",
        identity
            .available_bytes
            .map(|bytes| format!("{bytes} bytes")),
    ));
    rows.push(row(
        "Channel mode",
        "Not inferred. Windows module list is not a proof of dual-channel interleave.".to_string(),
    ));
    if identity.modules.is_empty() {
        rows.push(row("Modules", "None enumerated by Windows.".to_string()));
    } else {
        rows.push(row(
            "Modules",
            identity
                .modules
                .iter()
                .map(|module| {
                    let locator = module.locator.as_deref().unwrap_or("slot");
                    let cap = module
                        .capacity_bytes
                        .map(|bytes| format!("{bytes} bytes"))
                        .unwrap_or_else(|| "capacity not reported".to_string());
                    let speed = module
                        .speed_mhz
                        .map(|mhz| format!("{mhz} MHz"))
                        .unwrap_or_else(|| "speed not reported".to_string());
                    format!("{locator} · {cap} · {speed}")
                })
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    TelemetryGroup {
        title: "Memory".to_string(),
        note: Some(
            "Inventory only until a consented pattern spot check runs. Never printed as memory verified."
                .to_string(),
        ),
        rows,
    }
}

fn identity_source_group(identity: &CpuMemoryProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "Processor and memory sources consulted".to_string(),
        note: Some(
            "Advance scan asks Windows for processor identity and physical memory modules before any workload."
                .to_string(),
        ),
        rows: identity
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

fn bench_group(benches: &BenchResult) -> TelemetryGroup {
    TelemetryGroup {
        title: "Benchmarks".to_string(),
        note: Some(
            "Workloads run only with consent. Package temperature is never inferred from a missing sensor."
                .to_string(),
        ),
        rows: vec![
            row("Processor sustained clock", benches.cpu.status.clone()),
            row("Memory pattern check", benches.memory.status.clone()),
            row("Sequential read", benches.storage.sequential_status.clone()),
            row("Random read", benches.storage.random_status.clone()),
            row("Write benchmark", benches.storage.write_status.clone()),
        ],
    }
}

fn storage_group(storage: &StorageProbe) -> TelemetryGroup {
    let mut rows = Vec::new();

    if let Some(error) = storage.probe_error {
        rows.push(row("Storage probe", error.to_string()));
        rows.extend(unread_storage_rows());
        return TelemetryGroup {
            title: "Storage health and SMART".to_string(),
            note: None,
            rows,
        };
    }

    if storage.drives.is_empty() {
        rows.push(row(
            "Storage devices",
            format!("Not collected in this scan. {}.", storage_gap(storage)),
        ));
        rows.extend(unread_storage_rows());
        return TelemetryGroup {
            title: "Storage health and SMART".to_string(),
            note: None,
            rows,
        };
    }

    rows.push(row(
        "Storage devices",
        storage
            .drives
            .iter()
            .map(storage_health::DriveReading::display_name)
            .collect::<Vec<_>>()
            .join("; "),
    ));

    let scoring = storage.scoring_drives();
    let primary = scoring
        .first()
        .copied()
        .or_else(|| storage.drives.first())
        .expect("drives is not empty");

    rows.push(optional_row("Model", primary.model.clone()));
    rows.push(optional_row("Serial number", primary.serial_number.clone()));
    rows.push(optional_row(
        "Firmware revision",
        primary.firmware_revision.clone(),
    ));
    rows.push(optional_row("Bus type", primary.bus_type.clone()));
    rows.push(optional_row("Media kind", primary.media_kind.clone()));
    rows.push(optional_row(
        "Capacity",
        primary.capacity_bytes.map(|bytes| format!("{bytes} bytes")),
    ));
    rows.push(optional_row(
        "Rotational",
        primary.rotational.map(|rotational| {
            if rotational {
                "Yes (magnetic)".to_string()
            } else {
                "No (solid state)".to_string()
            }
        }),
    ));
    rows.push(optional_row(
        "Power-on hours",
        primary.power_on_hours.map(|hours| hours.to_string()),
    ));
    rows.push(optional_row(
        "Power cycles",
        primary.power_cycles.map(|cycles| cycles.to_string()),
    ));
    rows.push(optional_row(
        "Percentage used",
        primary.percentage_used.map(|used| {
            format!("{used}% (firmware wear, not a remaining-life estimate we invented)")
        }),
    ));
    rows.push(optional_row(
        "Remaining life (derived)",
        primary
            .remaining_life_percent
            .map(|life| format!("{life}% (100 minus percentage used)")),
    ));
    rows.push(optional_row(
        "Available spare",
        primary
            .available_spare_percent
            .map(|spare| format!("{spare}%")),
    ));
    rows.push(optional_row(
        "Composite temperature",
        primary.temperature_c.map(|temp| format!("{temp:.0} °C")),
    ));
    rows.push(optional_row(
        "Uncorrected media errors",
        primary.media_errors.map(|errors| errors.to_string()),
    ));
    rows.push(optional_row(
        "Reallocated sectors",
        primary
            .reallocated_sectors
            .map(|sectors| sectors.to_string()),
    ));
    rows.push(optional_row(
        "Sectors pending reallocation",
        primary.pending_sectors.map(|sectors| sectors.to_string()),
    ));
    rows.push(optional_row(
        "Predicted failure",
        primary.predicts_failure.map(|predicts| {
            if predicts {
                "Yes — firmware reports a predicted failure".to_string()
            } else {
                "No".to_string()
            }
        }),
    ));
    rows.push(row(
        "Total bytes written",
        "Not collected in this scan. Windows reliability counters on this build do not expose host TBW.".to_string(),
    ));

    TelemetryGroup {
        title: "Storage health and SMART".to_string(),
        note: Some(
            "Power-on hours and wear are printed for the buyer. They are not scored in rubric CG-1.0."
                .to_string(),
        ),
        rows,
    }
}

fn unread_storage_rows() -> Vec<NamedValue> {
    [
        ("Bus type", NOT_COLLECTED),
        ("Power-on hours", NOT_COLLECTED),
        ("Power cycles", NOT_COLLECTED),
        ("Total bytes written", NOT_COLLECTED),
        ("Percentage used", NOT_COLLECTED),
        ("Available spare", NOT_COLLECTED),
        ("Media errors", NOT_COLLECTED),
        ("Sectors pending reallocation", NOT_COLLECTED),
        ("Predicted failure", NOT_COLLECTED),
    ]
    .into_iter()
    .map(|(label, value)| row(label, value.to_string()))
    .collect()
}

fn storage_source_group(storage: &StorageProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "Storage sources consulted".to_string(),
        note: Some(
            "Advance scan asks Windows for identity, reliability counters and the SMART predict-failure bit, then records which source answered."
                .to_string(),
        ),
        rows: storage
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

fn usb_group(
    usb: &UsbProbe,
    radios: &DisplayRadioProbe,
    interactive: InteractiveAttestations,
) -> TelemetryGroup {
    let mut rows = Vec::new();
    if let Some(error) = usb.probe_error {
        rows.push(row("USB probe", error.to_string()));
        rows.push(row(
            "Physically verified ports",
            interactive.physical_ports.customer_label().to_string(),
        ));
        rows.extend(radio_rows(radios));
        return TelemetryGroup {
            title: "Ports and connectivity".to_string(),
            note: None,
            rows,
        };
    }

    let controllers = if usb.controllers.is_empty() {
        if usb.topology_enumerated() {
            "Windows reported USB controllers but named none on this PC.".to_string()
        } else {
            format!("Not collected in this scan. {}.", usb_gap(usb))
        }
    } else {
        format!(
            "{} · {}",
            usb.controllers.len(),
            usb.controllers
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    };

    let hubs = if usb.hubs.is_empty() {
        "None enumerated by Windows".to_string()
    } else {
        format!(
            "{} · {}",
            usb.hubs.len(),
            usb.hubs
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    };

    let attached = if usb.devices.is_empty() {
        "None attached at scan time. Empty plastic connectors are not visible to Windows."
            .to_string()
    } else {
        usb.devices
            .iter()
            .map(|device| {
                let speed = device.speed.as_deref().unwrap_or("speed not reported");
                match device.port_index {
                    Some(port) => format!("{} (port {port}, {speed})", device.name),
                    None => format!("{} ({speed})", device.name),
                }
            })
            .collect::<Vec<_>>()
            .join("; ")
    };

    rows.push(row("USB controllers", controllers));
    rows.push(row("USB hubs", hubs));
    rows.push(row("USB controller ports", attached));
    rows.push(optional_row("Negotiated port speeds", usb.speed_summary()));
    rows.push(row(
        "Physically verified ports",
        interactive.physical_ports.customer_label().to_string(),
    ));
    rows.extend(radio_rows(radios));

    TelemetryGroup {
        title: "Ports and connectivity".to_string(),
        note: Some(
            "Controller topology is not a count of plastic connectors. A port is confirmed only when a technician inserts a device. MAC addresses are never printed."
                .to_string(),
        ),
        rows,
    }
}

fn radio_rows(radios: &DisplayRadioProbe) -> Vec<NamedValue> {
    if let Some(error) = radios.probe_error {
        return vec![
            row("Wi-Fi signal quality", error.to_string()),
            row("Wi-Fi link speed", error.to_string()),
            row("Bluetooth radio", error.to_string()),
            row("Ethernet link", error.to_string()),
        ];
    }

    let none = "None enumerated by Windows".to_string();
    let wifi = radios.wifi.as_ref();
    let wifi_quality = wifi.and_then(|adapter| {
        adapter
            .signal_quality_percent
            .map(|quality| format!("{quality}%"))
    });
    let wifi_speed = wifi.and_then(|adapter| {
        adapter
            .receive_mbps
            .or(adapter.transmit_mbps)
            .or(adapter.link_mbps)
            .map(|mbps| format!("{mbps} Mbps"))
    });

    vec![
        row(
            "Wi-Fi adapter",
            wifi.map(|adapter| {
                adapter
                    .name
                    .clone()
                    .unwrap_or_else(|| "Present".to_string())
            })
            .unwrap_or_else(|| none.clone()),
        ),
        row(
            "Wi-Fi state",
            wifi.and_then(|adapter| adapter.state.clone())
                .unwrap_or_else(|| none.clone()),
        ),
        row(
            "Wi-Fi signal quality",
            wifi_quality.unwrap_or_else(|| none.clone()),
        ),
        row(
            "Wi-Fi link speed",
            wifi_speed.unwrap_or_else(|| none.clone()),
        ),
        row(
            "Wi-Fi radio standards",
            wifi.and_then(|adapter| adapter.radio_standards.clone())
                .unwrap_or_else(|| none.clone()),
        ),
        row(
            "Bluetooth radio",
            radios.bluetooth.as_ref().map_or_else(
                || none.clone(),
                |adapter| match (&adapter.name, &adapter.state) {
                    (Some(name), Some(state)) => format!("{name} ({state})"),
                    (Some(name), None) => name.clone(),
                    (None, Some(state)) => format!("Present ({state})"),
                    (None, None) => "Present".to_string(),
                },
            ),
        ),
        row(
            "Ethernet link",
            if radios.ethernet.is_empty() {
                none
            } else {
                radios
                    .ethernet
                    .iter()
                    .map(|adapter| {
                        let label = adapter
                            .name
                            .clone()
                            .or_else(|| adapter.description.clone())
                            .unwrap_or_else(|| "Ethernet".to_string());
                        match (&adapter.state, adapter.link_mbps) {
                            (Some(state), Some(mbps)) => format!("{label} ({state}, {mbps} Mbps)"),
                            (Some(state), None) => format!("{label} ({state})"),
                            (None, Some(mbps)) => format!("{label} ({mbps} Mbps)"),
                            (None, None) => label,
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            },
        ),
    ]
}

fn radio_source_group(radios: &DisplayRadioProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "Radio sources consulted".to_string(),
        note: Some(
            "Advance scan asks Windows for Wi-Fi, Bluetooth and Ethernet adapters. MAC addresses are dropped before they can be printed."
                .to_string(),
        ),
        rows: radios
            .sources
            .iter()
            .filter(|status| {
                matches!(
                    status.source,
                    DisplayRadioSource::Wifi
                        | DisplayRadioSource::Bluetooth
                        | DisplayRadioSource::Ethernet
                )
            })
            .map(|status| {
                row(
                    status.source.label(),
                    status.outcome.customer_label().to_string(),
                )
            })
            .collect(),
    }
}

fn display_source_group(radios: &DisplayRadioProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "Display sources consulted".to_string(),
        note: Some(
            "Advance scan reads monitor identity and the first 128 bytes of EDID. Native resolution is the preferred timing, not the current desktop mode."
                .to_string(),
        ),
        rows: radios
            .sources
            .iter()
            .filter(|status| {
                matches!(
                    status.source,
                    DisplayRadioSource::Panel
                        | DisplayRadioSource::Edid
                        | DisplayRadioSource::Video
                )
            })
            .map(|status| {
                row(
                    status.source.label(),
                    status.outcome.customer_label().to_string(),
                )
            })
            .collect(),
    }
}

fn display_group(radios: &DisplayRadioProbe) -> TelemetryGroup {
    let mut rows = Vec::new();
    if let Some(error) = radios.probe_error {
        rows.push(row("Display probe", error.to_string()));
        rows.extend(unread_display_rows());
        return TelemetryGroup {
            title: "Display panel".to_string(),
            note: None,
            rows,
        };
    }

    if radios.panels.is_empty() {
        rows.push(row(
            "Display panel",
            "Not collected in this scan. Windows did not name a panel in this scan.".to_string(),
        ));
        rows.extend(unread_display_rows());
        return TelemetryGroup {
            title: "Display panel".to_string(),
            note: None,
            rows,
        };
    }

    let panel = &radios.panels[0];
    rows.push(row("Display panel", panel.display_name()));
    rows.push(optional_row(
        "Panel manufacturer",
        panel.manufacturer.clone(),
    ));
    rows.push(optional_row("Panel model", panel.name.clone()));
    rows.push(optional_row(
        "Native resolution",
        match (panel.native_width, panel.native_height) {
            (Some(width), Some(height)) => {
                Some(format!("{width} × {height} (EDID preferred timing)"))
            }
            _ => None,
        },
    ));
    rows.push(optional_row(
        "Current desktop mode",
        match (panel.current_width, panel.current_height) {
            (Some(width), Some(height)) => {
                Some(format!("{width} × {height} (current mode, not native)"))
            }
            _ => None,
        },
    ));
    rows.push(optional_row(
        "Refresh rate",
        panel
            .refresh_hz
            .map(|hz| format!("{hz} Hz (current desktop mode)")),
    ));
    rows.push(row(
        "HDR capability",
        "Not collected in this scan. HDR is not inferred from the current desktop colour profile."
            .to_string(),
    ));
    rows.push(optional_row(
        "Panel manufacture year",
        panel.manufacture_year.map(|year| year.to_string()),
    ));
    rows.push(optional_row(
        "Panel serial number",
        panel.serial_number.clone(),
    ));

    TelemetryGroup {
        title: "Display panel".to_string(),
        note: Some(
            "Native resolution is the EDID preferred timing. Colour-wash points are operator-attested and are not inferred from EDID."
                .to_string(),
        ),
        rows,
    }
}

fn unread_display_rows() -> Vec<NamedValue> {
    [
        ("Panel manufacturer", NOT_COLLECTED),
        ("Panel model", NOT_COLLECTED),
        ("Native resolution", NOT_COLLECTED),
        ("Refresh rate", NOT_COLLECTED),
        ("HDR capability", NOT_COLLECTED),
        ("Panel manufacture year", NOT_COLLECTED),
    ]
    .into_iter()
    .map(|(label, value)| row(label, value.to_string()))
    .collect()
}

fn interactive_group(interactive: InteractiveAttestations) -> TelemetryGroup {
    TelemetryGroup {
        title: "Technician checks".to_string(),
        note: Some(
            "These points are operator-attested. Keystrokes, speaker tones, and colour washes are not stored. Live camera and microphone capture is not part of this scan.".to_string(),
        ),
        rows: vec![
            row(
                "Display inspection",
                interactive.colour_wash.customer_label().to_string(),
            ),
            row(
                "Keyboard",
                interactive.keyboard.customer_label().to_string(),
            ),
            row(
                "Trackpad",
                interactive.trackpad.customer_label().to_string(),
            ),
            row(
                "Speakers",
                interactive.speakers.customer_label().to_string(),
            ),
            row(
                "Camera and microphone",
                interactive.capture.customer_label().to_string(),
            ),
            row(
                "Physically verified ports",
                interactive.physical_ports.customer_label().to_string(),
            ),
        ],
    }
}

fn usb_source_group(usb: &UsbProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "USB sources consulted".to_string(),
        note: Some(
            "Advance scan walks USB controllers, hubs and attached devices. It does not guess empty sockets from SMBIOS labels."
                .to_string(),
        ),
        rows: usb
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

fn capture_group(capture: &CaptureProbe) -> TelemetryGroup {
    let mut rows = Vec::new();
    if let Some(error) = capture.probe_error {
        rows.push(row("Capture probe", error.to_string()));
        rows.push(row("Frames captured", "No".to_string()));
        rows.push(row("Audio recorded", "No".to_string()));
        return TelemetryGroup {
            title: "Cameras and microphones".to_string(),
            note: None,
            rows,
        };
    }

    let cameras = if capture.camera_names().is_empty() {
        if capture.reports_no_cameras() {
            "None enumerated by Windows after querying Camera, Image and USB video classes."
                .to_string()
        } else {
            "Not collected in this scan. The camera query did not complete.".to_string()
        }
    } else {
        capture.camera_names().join("; ")
    };

    let microphones = if capture.microphone_names().is_empty() {
        if capture.reports_no_microphones() {
            "None enumerated by Windows after querying audio endpoints and sound devices."
                .to_string()
        } else {
            "Not collected in this scan. The microphone query did not complete.".to_string()
        }
    } else {
        capture.microphone_names().join("; ")
    };

    let camera_via = capture
        .cameras
        .iter()
        .filter_map(|device| device.enumerated_by.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("; ");

    rows.push(row("Cameras", cameras));
    if !camera_via.is_empty() {
        rows.push(row("Camera enumerated by", camera_via));
    }
    rows.push(row("Microphones", microphones));
    rows.push(row(
        "Frames captured",
        if capture.frames_captured { "Yes" } else { "No" }.to_string(),
    ));
    rows.push(row(
        "Audio recorded",
        if capture.audio_recorded { "Yes" } else { "No" }.to_string(),
    ));
    rows.push(row("Camera image", NOT_ATTEMPTED.to_string()));

    TelemetryGroup {
        title: "Cameras and microphones".to_string(),
        note: Some(
            "Enumeration only. No webcam frame is captured and no microphone audio is recorded in this slice."
                .to_string(),
        ),
        rows,
    }
}

fn capture_source_group(capture: &CaptureProbe) -> TelemetryGroup {
    TelemetryGroup {
        title: "Capture sources consulted".to_string(),
        note: Some(
            "The Camera ClassGuid alone misses some UVC webcams, so Advance scan also asks the USB video service and Image class."
                .to_string(),
        ),
        rows: capture
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
    use crate::advance_bench::BenchResult;
    use crate::battery_probe::parse_probe as parse_battery;
    use crate::capture_probe::parse_probe as parse_capture;
    use crate::cpu_memory::parse_probe as parse_identity;
    use crate::display_radio::parse_probe as parse_radios;
    use crate::hardware_diagnostics_v1::OperatorAttestation;
    use crate::storage_health::parse_probe as parse_storage;
    use crate::usb_topology::parse_probe as parse_usb;

    fn hex(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn protocol(lines: &[(&str, usize, &str, &str)]) -> String {
        lines
            .iter()
            .map(|(section, index, name, value)| {
                format!("{section}\t{index}\t{name}\t{}", hex(value))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn probe_from(lines: &[(&str, usize, &str, &str)]) -> BatteryProbe {
        parse_battery(&protocol(lines))
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

    fn silent_identity() -> CpuMemoryProbe {
        CpuMemoryProbe::unavailable(
            "Processor and memory identity collection is only available on Windows.",
        )
    }

    fn silent_radios() -> DisplayRadioProbe {
        DisplayRadioProbe::unavailable("Display and radio collection is only available on Windows.")
    }

    fn silent_usb() -> UsbProbe {
        UsbProbe::unavailable("USB topology collection is only available on Windows.")
    }

    fn silent_storage() -> StorageProbe {
        StorageProbe::unavailable("Storage SMART collection is only available on Windows.")
    }

    fn silent_capture() -> CaptureProbe {
        CaptureProbe::unavailable("Camera and microphone collection is only available on Windows.")
    }

    fn enumerated_usb() -> UsbProbe {
        parse_usb(&protocol(&[
            ("controller", 0, "source_status", "reported"),
            ("controller", 0, "present", "True"),
            (
                "controller",
                0,
                "name",
                "Intel USB 3.0 eXtensible Host Controller",
            ),
            ("hub", 0, "source_status", "reported"),
            ("hub", 0, "present", "True"),
            ("hub", 0, "name", "USB Root Hub (USB 3.0)"),
            ("hub", 0, "root_hub", "True"),
            ("device", 0, "source_status", "reported"),
            ("device", 0, "present", "True"),
            ("device", 0, "name", "USB Composite Device"),
            ("device", 0, "speed", "USB 2.0"),
            ("device", 0, "port_index", "2"),
        ]))
    }

    fn uvc_capture() -> CaptureProbe {
        parse_capture(&protocol(&[
            ("camera_class", 0, "source_status", "reported"),
            ("usbvideo", 0, "source_status", "reported"),
            ("audio_endpoint", 0, "source_status", "reported"),
            ("camera", 0, "present", "True"),
            ("camera", 0, "name", "USB Video Device"),
            ("camera", 0, "enumerated_by", "USB video service"),
            ("microphone", 0, "present", "True"),
            ("microphone", 0, "name", "Microphone Array"),
            ("capture", 0, "frames_captured", "False"),
            ("capture", 0, "audio_recorded", "False"),
        ]))
    }

    fn healthy_nvme() -> StorageProbe {
        parse_storage(&protocol(&[
            ("physical", 0, "source_status", "reported"),
            ("physical", 0, "present", "True"),
            ("physical", 0, "model", "Samsung SSD 990 PRO 1TB"),
            ("physical", 0, "serial_number", "S6Z1NS0W123456"),
            ("physical", 0, "bus_type", "NVMe"),
            ("physical", 0, "media_kind", "SSD"),
            ("physical", 0, "rotational", "False"),
            ("reliability", 0, "source_status", "reported"),
            ("reliability", 0, "present", "True"),
            ("reliability", 0, "percentage_used", "2"),
            ("reliability", 0, "available_spare", "100"),
            ("reliability", 0, "power_on_hours", "1840"),
            ("predict", 0, "source_status", "reported"),
            ("predict", 0, "present", "True"),
            ("predict", 0, "predicts_failure", "False"),
        ]))
    }

    fn report_for(probe: &BatteryProbe, form: DeviceForm) -> CustomerAdvanceScan {
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        build_report(
            &diagnostics,
            probe,
            &silent_storage(),
            &silent_usb(),
            &silent_radios(),
            &silent_capture(),
            &silent_identity(),
            &BenchResult::declined(),
            InteractiveAttestations::default(),
            form,
        )
    }

    fn report_full(
        battery: &BatteryProbe,
        storage: &StorageProbe,
        usb: &UsbProbe,
        radios: &DisplayRadioProbe,
        capture: &CaptureProbe,
        form: DeviceForm,
    ) -> CustomerAdvanceScan {
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        build_report(
            &diagnostics,
            battery,
            storage,
            usb,
            radios,
            capture,
            &silent_identity(),
            &BenchResult::declined(),
            InteractiveAttestations::default(),
            form,
        )
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
            if group.title.starts_with("Battery")
                || group.title.starts_with("USB")
                || group.title.starts_with("Ports")
                || group.title.starts_with("Cameras")
                || group.title.starts_with("Capture")
                || group.title.starts_with("Storage")
                || group.title.starts_with("Display")
                || group.title.starts_with("Radio")
                || group.title.starts_with("Processor")
                || group.title.starts_with("Memory")
                || group.title.starts_with("Technician")
                || group.title.starts_with("Screen")
            {
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

        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &silent_storage(),
            &silent_usb(),
            &silent_radios(),
            &silent_capture(),
            &silent_identity(),
            &BenchResult::declined(),
            InteractiveAttestations::default(),
            DeviceForm::Portable,
        );

        assert!(!outcome.ok);
        assert!(outcome.message.contains("without consent"));
    }

    #[test]
    fn a_uvc_camera_is_printed_and_does_not_award_screen_points() {
        let outcome = report_full(
            &healthy_probe(),
            &silent_storage(),
            &silent_usb(),
            &silent_radios(),
            &uvc_capture(),
            DeviceForm::Portable,
        );

        assert_eq!(
            value_of(&outcome, "Cameras and microphones", "Cameras"),
            "USB Video Device"
        );
        assert_eq!(
            value_of(&outcome, "Cameras and microphones", "Camera enumerated by"),
            "USB video service"
        );
        assert_eq!(
            value_of(&outcome, "Cameras and microphones", "Microphones"),
            "Microphone Array"
        );
        assert_eq!(
            value_of(&outcome, "Cameras and microphones", "Frames captured"),
            "No"
        );
        assert_eq!(
            value_of(&outcome, "Cameras and microphones", "Audio recorded"),
            "No"
        );
        assert!(
            value_of(&outcome, "Cameras and microphones", "Camera image")
                .contains("Not attempted in this scan")
        );
        assert!(!outcome.content_inspected);

        let screen = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Screen, keyboard and peripherals")
            .expect("screen domain");
        assert_eq!(screen.awarded, 0);
        assert_eq!(screen.assessed, 0);
        assert_eq!(outcome.coverage_percent, 20);
    }

    #[test]
    fn enumerated_usb_awards_topology_points_and_leaves_physical_ports_unattempted() {
        let outcome = report_full(
            &healthy_probe(),
            &silent_storage(),
            &enumerated_usb(),
            &silent_radios(),
            &uvc_capture(),
            DeviceForm::Portable,
        );

        assert!(
            value_of(&outcome, "Ports and connectivity", "USB controllers")
                .contains("Intel USB 3.0 eXtensible Host Controller")
        );
        assert!(
            value_of(&outcome, "Ports and connectivity", "USB hubs")
                .contains("USB Root Hub (USB 3.0)")
        );
        assert!(
            value_of(&outcome, "Ports and connectivity", "USB controller ports")
                .contains("USB Composite Device")
        );
        assert_eq!(
            value_of(
                &outcome,
                "Ports and connectivity",
                "Physically verified ports"
            ),
            NOT_ATTEMPTED
        );
        assert!(
            value_of(&outcome, "Ports and connectivity", "Wi-Fi signal quality")
                .contains("only available on Windows")
        );

        let ports = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Ports and connectivity")
            .expect("ports domain");
        assert_eq!(ports.awarded, 2);
        assert_eq!(ports.assessed, 2);
        assert_eq!(ports.not_assessable, 8);
        assert_eq!(ports.state, "Partly assessed");
        assert_eq!(outcome.coverage_percent, 22);
        assert!(outcome.grade_withheld);
        assert_eq!(outcome.grade_label, "Grade withheld");
        assert!(
            outcome
                .grade_withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("Storage health")
        );
        assert!(
            outcome
                .method_rows
                .iter()
                .any(|row| row.value.contains("USB video service"))
        );
        assert!(
            outcome
                .method_rows
                .iter()
                .any(|row| row.value.contains("never issues a write"))
        );
    }

    #[test]
    fn a_healthy_nvme_awards_storage_points_and_still_withholds_for_coverage() {
        let outcome = report_full(
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &silent_radios(),
            &uvc_capture(),
            DeviceForm::Portable,
        );

        assert_eq!(
            value_of(&outcome, "Storage health and SMART", "Model"),
            "Samsung SSD 990 PRO 1TB"
        );
        assert_eq!(
            value_of(&outcome, "Storage health and SMART", "Serial number"),
            "S6Z1NS0W123456"
        );
        assert!(
            value_of(&outcome, "Storage health and SMART", "Percentage used").starts_with("2%")
        );
        assert_eq!(
            value_of(&outcome, "Storage health and SMART", "Predicted failure"),
            "No"
        );
        assert!(
            value_of(&outcome, "Storage health and SMART", "Total bytes written")
                .contains("Not collected in this scan")
        );

        let storage = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Storage health and SMART")
            .expect("storage domain");
        assert_eq!(storage.assessed, 20);
        assert_eq!(storage.awarded, 20);
        assert_eq!(storage.state, "Fully assessed");
        assert_eq!(outcome.coverage_percent, 42);
        assert!(outcome.grade_withheld);
        assert!(
            outcome
                .grade_withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("could be assessed")
        );
        assert!(!outcome.destructive_operations_enabled);
        assert_eq!(outcome.bytes_written, 0);
    }

    #[test]
    fn a_refused_smart_query_explains_itself_and_keeps_storage_mandatory() {
        let storage = parse_storage(&protocol(&[
            ("disk", 0, "source_status", "reported"),
            ("reliability", 0, "source_status", "permission_denied"),
            ("predict", 0, "source_status", "permission_denied"),
        ]));
        let outcome = report_full(
            &healthy_probe(),
            &storage,
            &silent_usb(),
            &silent_radios(),
            &silent_capture(),
            DeviceForm::Portable,
        );

        assert_eq!(
            value_of(
                &outcome,
                "Storage sources consulted",
                "Storage reliability counters"
            ),
            "Refused without administrator rights"
        );
        assert!(
            outcome
                .not_assessable
                .iter()
                .any(|entry| entry.contains("refused the storage SMART query"))
        );
        assert!(outcome.grade_withheld);
        assert!(
            outcome
                .grade_withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("Storage health")
        );
    }

    #[test]
    fn a_predicted_failure_forces_f_instead_of_withholding() {
        let storage = parse_storage(&protocol(&[
            ("physical", 0, "source_status", "reported"),
            ("physical", 0, "present", "True"),
            ("physical", 0, "model", "Failing disk"),
            ("predict", 0, "source_status", "reported"),
            ("predict", 0, "present", "True"),
            ("predict", 0, "predicts_failure", "True"),
        ]));
        let outcome = report_full(
            &healthy_probe(),
            &storage,
            &silent_usb(),
            &silent_radios(),
            &silent_capture(),
            DeviceForm::Portable,
        );

        assert!(!outcome.grade_withheld);
        assert_eq!(outcome.grade_label, "F");
        assert!(
            outcome
                .grade_withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("predicted failure")
        );
        let storage_domain = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Storage health and SMART")
            .expect("storage domain");
        assert_eq!(storage_domain.awarded, 0);
        assert_eq!(storage_domain.assessed, 20);
    }

    fn sample_edid_hex() -> String {
        let mut bytes = [0_u8; 128];
        bytes[0] = 0x00;
        bytes[7] = 0x00;
        let packed: u16 = (12 << 10) | (5 << 5) | 14;
        bytes[8] = (packed >> 8) as u8;
        bytes[9] = packed as u8;
        bytes[16] = 12;
        bytes[17] = 34;
        bytes[54] = 0x02;
        bytes[55] = 0x3A;
        bytes[56] = 0x80;
        bytes[58] = 0x70;
        bytes[59] = 0x38;
        bytes[61] = 0x40;
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn panel_and_radios() -> DisplayRadioProbe {
        parse_radios(&protocol(&[
            ("panel", 0, "source_status", "reported"),
            ("panel", 0, "present", "True"),
            ("panel", 0, "manufacturer", "LEN"),
            ("panel", 0, "name", "LENovo LCD"),
            ("panel", 0, "serial_number", "PF11ARPM"),
            ("edid", 0, "source_status", "reported"),
            ("edid", 0, "present", "True"),
            ("edid", 0, "block_hex", &sample_edid_hex()),
            ("video", 0, "source_status", "reported"),
            ("video", 0, "present", "True"),
            ("video", 0, "current_width", "1280"),
            ("video", 0, "current_height", "720"),
            ("video", 0, "refresh_hz", "60"),
            ("wifi", 0, "source_status", "reported"),
            ("wifi", 0, "present", "True"),
            ("wifi", 0, "name", "Wi-Fi"),
            ("wifi", 0, "state", "Up"),
            ("wifi", 0, "signal_quality", "72"),
            ("wifi", 0, "receive_mbps", "400"),
            ("bluetooth", 0, "source_status", "reported"),
            ("bluetooth", 0, "present", "True"),
            ("bluetooth", 0, "name", "Intel Wireless Bluetooth"),
            ("ethernet", 0, "source_status", "reported"),
            ("ethernet", 0, "present", "True"),
            ("ethernet", 0, "name", "Ethernet"),
            ("ethernet", 0, "state", "Disconnected"),
        ]))
    }

    fn radios_with_mac_in_the_wire() -> DisplayRadioProbe {
        parse_radios(&protocol(&[
            ("wifi", 0, "source_status", "reported"),
            ("wifi", 0, "present", "True"),
            ("wifi", 0, "name", "AA:BB:CC:DD:EE:FF"),
            ("wifi", 0, "state", "Up"),
            ("bluetooth", 0, "source_status", "reported"),
            ("bluetooth", 0, "present", "True"),
            ("ethernet", 0, "source_status", "reported"),
            ("ethernet", 0, "present", "True"),
            ("ethernet", 0, "name", "Ethernet"),
            ("ethernet", 0, "state", "Up"),
        ]))
    }

    #[test]
    fn edid_native_resolution_is_printed_separately_from_current_mode() {
        let outcome = report_full(
            &healthy_probe(),
            &silent_storage(),
            &silent_usb(),
            &panel_and_radios(),
            &silent_capture(),
            DeviceForm::Portable,
        );

        assert_eq!(
            value_of(&outcome, "Display panel", "Native resolution"),
            "1920 × 1080 (EDID preferred timing)"
        );
        assert_eq!(
            value_of(&outcome, "Display panel", "Current desktop mode"),
            "1280 × 720 (current mode, not native)"
        );
        assert!(value_of(&outcome, "Display panel", "HDR capability").contains("not inferred"));
        let screen = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Screen, keyboard and peripherals")
            .expect("screen domain");
        assert_eq!(screen.awarded, 0);
        assert_eq!(screen.assessed, 0);
    }

    #[test]
    fn usb_plus_radios_award_six_port_points_and_still_withhold() {
        let outcome = report_full(
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &panel_and_radios(),
            &uvc_capture(),
            DeviceForm::Portable,
        );

        assert_eq!(
            value_of(&outcome, "Ports and connectivity", "Wi-Fi signal quality"),
            "72%"
        );
        assert!(
            value_of(&outcome, "Ports and connectivity", "Bluetooth radio")
                .contains("Intel Wireless Bluetooth")
        );
        assert!(
            value_of(&outcome, "Ports and connectivity", "Ethernet link").contains("Disconnected")
        );
        assert_eq!(
            value_of(
                &outcome,
                "Ports and connectivity",
                "Physically verified ports"
            ),
            NOT_ATTEMPTED
        );

        let ports = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Ports and connectivity")
            .expect("ports domain");
        assert_eq!(ports.awarded, 6);
        assert_eq!(ports.assessed, 6);
        assert_eq!(ports.not_assessable, 4);
        assert_eq!(ports.state, "Partly assessed");
        assert_eq!(outcome.coverage_percent, 46);
        assert!(outcome.grade_withheld);
        assert_eq!(outcome.grade_label, "Grade withheld");
        assert!(
            outcome
                .method_rows
                .iter()
                .any(|row| row.value.contains("EDID preferred timing"))
        );
        assert!(
            outcome
                .method_rows
                .iter()
                .any(|row| row.value.contains("MAC address"))
        );
    }

    #[test]
    fn a_mac_address_never_appears_on_report_d() {
        let outcome = report_full(
            &healthy_probe(),
            &silent_storage(),
            &silent_usb(),
            &radios_with_mac_in_the_wire(),
            &silent_capture(),
            DeviceForm::Portable,
        );

        for group in &outcome.telemetry_groups {
            for row in &group.rows {
                assert!(
                    !row.value.contains("AA:BB:CC:DD:EE:FF"),
                    "{} / {} leaked a MAC address: {}",
                    group.title,
                    row.label,
                    row.value
                );
                assert!(
                    !row.label.eq_ignore_ascii_case("mac")
                        && !row.label.to_ascii_lowercase().contains("mac address")
                        && !row.label.to_ascii_lowercase().contains("macaddress"),
                    "Report D must not have a MAC label: {}",
                    row.label
                );
            }
        }
        assert_eq!(
            value_of(&outcome, "Ports and connectivity", "Wi-Fi adapter"),
            "Present"
        );
        assert_eq!(
            value_of(&outcome, "Ports and connectivity", "Wi-Fi state"),
            "Up"
        );
    }

    fn named_processor_and_ram() -> CpuMemoryProbe {
        parse_identity(&protocol(&[
            ("cpu", 0, "source_status", "reported"),
            ("cpu", 0, "present", "True"),
            ("cpu", 0, "name", "Intel Core i7-6500U"),
            ("cpu", 0, "cores", "2"),
            ("cpu", 0, "l3_kb", "4096"),
            ("cpu", 0, "max_mhz", "2500"),
            ("memory", 0, "source_status", "reported"),
            ("memory", 0, "total_kb", "8388608"),
            ("module", 0, "source_status", "reported"),
            ("module", 0, "present", "True"),
            ("module", 0, "locator", "ChannelA-DIMM0"),
            ("module", 0, "capacity_bytes", "8589934592"),
            ("module", 0, "speed_mhz", "2133"),
        ]))
    }

    #[test]
    fn processor_identity_awards_four_points_without_a_workload() {
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &panel_and_radios(),
            &uvc_capture(),
            &named_processor_and_ram(),
            &BenchResult::declined(),
            InteractiveAttestations::default(),
            DeviceForm::Portable,
        );
        let cpu = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Processor and thermal stability")
            .expect("cpu domain");
        assert_eq!(cpu.awarded, 4);
        assert_eq!(cpu.assessed, 4);
        let memory = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Memory integrity and speed")
            .expect("memory domain");
        assert_eq!(memory.awarded, 5);
        assert_eq!(memory.assessed, 5);
        assert_eq!(outcome.coverage_percent, 55);
        assert!(outcome.grade_withheld);
        assert!(value_of(&outcome, "Processor and thermal", "Processor").contains("i7-6500U"));
        assert!(value_of(&outcome, "Benchmarks", "Processor sustained clock").contains("Declined"));
        assert!(
            outcome
                .method_rows
                .iter()
                .any(|row| row.value.contains("never prints 'memory verified'")
                    || row.value.contains("never prints"))
        );
    }

    #[test]
    fn consented_workloads_can_reach_the_coverage_floor_provisionally() {
        let mut benches = BenchResult::declined();
        benches.cpu.ratio = Some(92);
        benches.cpu.status =
            "CPU workload finished. Windows reported 2300 MHz after the loop against a maximum of 2500 MHz (92% of maximum). Package temperature is not collected."
                .to_string();
        benches.memory.pattern_passed = Some(true);
        benches.memory.bandwidth_mib_s = Some(420.0);
        benches.memory.status =
            "Memory pattern spot check passed on 32.0 MiB. This is not full-coverage memory testing; kernel-resident memory was not included."
                .to_string();
        benches.storage.sequential_status =
            "Sequential read 800 MiB/s of an existing Windows system file. No file was created."
                .to_string();
        benches.storage.random_status =
            "Random 4 KiB read 12000 IOPS of an existing Windows system file. No file was created."
                .to_string();

        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &panel_and_radios(),
            &uvc_capture(),
            &named_processor_and_ram(),
            &benches,
            InteractiveAttestations::default(),
            DeviceForm::Portable,
        );

        let cpu = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Processor and thermal stability")
            .expect("cpu domain");
        assert_eq!(cpu.awarded, 20);
        assert_eq!(cpu.assessed, 20);
        let memory = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Memory integrity and speed")
            .expect("memory domain");
        assert_eq!(memory.awarded, 15);
        assert_eq!(memory.assessed, 15);
        assert_eq!(outcome.coverage_percent, 81);
        assert!(!outcome.grade_withheld);
        assert!(outcome.provisional);
        assert_eq!(outcome.grade_label, "A+");
        assert!(
            value_of(&outcome, "Benchmarks", "Memory pattern check")
                .contains("not full-coverage memory testing")
        );
        assert!(
            !value_of(&outcome, "Benchmarks", "Memory pattern check")
                .to_ascii_lowercase()
                .contains("memory verified")
        );
    }

    #[test]
    fn a_failed_pattern_spot_check_forces_f() {
        let mut benches = BenchResult::declined();
        benches.memory.pattern_passed = Some(false);
        benches.memory.status =
            "Memory pattern spot check failed: the tested region did not match the pattern that was written."
                .to_string();
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &panel_and_radios(),
            &uvc_capture(),
            &named_processor_and_ram(),
            &benches,
            InteractiveAttestations::default(),
            DeviceForm::Portable,
        );
        assert_eq!(outcome.grade_label, "F");
        assert!(!outcome.grade_withheld);
        let memory = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Memory integrity and speed")
            .expect("memory domain");
        assert_eq!(memory.awarded, 0);
        assert_eq!(memory.assessed, 15);
    }

    #[test]
    fn skipped_interactive_checks_do_not_score_the_screen_domain() {
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &panel_and_radios(),
            &uvc_capture(),
            &named_processor_and_ram(),
            &BenchResult::declined(),
            InteractiveAttestations::default(),
            DeviceForm::Portable,
        );
        let screen = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Screen, keyboard and peripherals")
            .expect("screen domain");
        assert_eq!(screen.awarded, 0);
        assert_eq!(screen.assessed, 0);
        assert_eq!(outcome.coverage_percent, 55);
        assert!(value_of(&outcome, "Technician checks", "Keyboard").contains("Not attempted"));
    }

    #[test]
    fn attested_interactive_checks_award_screen_and_physical_port_points() {
        let interactive = InteractiveAttestations {
            colour_wash: OperatorAttestation::Passed,
            keyboard: OperatorAttestation::Passed,
            trackpad: OperatorAttestation::Passed,
            speakers: OperatorAttestation::Passed,
            capture: OperatorAttestation::Passed,
            physical_ports: PhysicalPortAttestation::AllPassed,
        };
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &healthy_nvme(),
            &enumerated_usb(),
            &panel_and_radios(),
            &uvc_capture(),
            &named_processor_and_ram(),
            &BenchResult::declined(),
            interactive,
            DeviceForm::Portable,
        );
        let screen = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Screen, keyboard and peripherals")
            .expect("screen domain");
        assert_eq!(screen.awarded, 15);
        assert_eq!(screen.assessed, 15);
        let ports = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Ports and connectivity")
            .expect("ports domain");
        assert_eq!(ports.awarded, 10);
        assert_eq!(ports.assessed, 10);
        assert_eq!(outcome.coverage_percent, 74);
        assert!(!outcome.grade_withheld);
        assert!(outcome.provisional);
        assert!(value_of(&outcome, "Technician checks", "Display inspection").contains("Passed"));
        assert!(!value_of(&outcome, "Technician checks", "Keyboard").contains("Ctrl"));
    }

    #[test]
    fn a_failed_colour_wash_assesses_the_subject_without_awarding_it() {
        let interactive = InteractiveAttestations {
            colour_wash: OperatorAttestation::Failed,
            ..InteractiveAttestations::default()
        };
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);
        let outcome = build_report(
            &diagnostics,
            &healthy_probe(),
            &silent_storage(),
            &silent_usb(),
            &silent_radios(),
            &silent_capture(),
            &silent_identity(),
            &BenchResult::declined(),
            interactive,
            DeviceForm::Portable,
        );
        let screen = outcome
            .coverage_domains
            .iter()
            .find(|domain| domain.domain == "Screen, keyboard and peripherals")
            .expect("screen domain");
        assert_eq!(screen.awarded, 0);
        assert_eq!(screen.assessed, 4);
        assert!(value_of(&outcome, "Technician checks", "Display inspection").contains("Failed"));
    }
}
