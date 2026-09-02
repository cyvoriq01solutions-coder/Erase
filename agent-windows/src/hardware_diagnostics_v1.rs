//! Typed contract for Advance scan (Report D) and the CYVRA Grading Engine.
//!
//! This module contains no collector. It defines the vocabulary Advance scan
//! needs so that a subsystem which was never measured can never be scored as a
//! pass and can never be scored as a fault. Deep collection arrives in later
//! slices; the honest empty state and the grading arithmetic land first.
//!
//! Two rules are enforced here rather than left to the caller:
//! points are never awarded for evidence we do not hold, and a grade is
//! withheld outright when too little of the device could be assessed.

use crate::hardware_inventory_v1::{
    Confidence, DeviceIdentifier, InventoryField, InventorySection, Provenance,
};

pub const SCHEMA_VERSION: &str = "hardware_diagnostics_v1";

/// Printed on Report D. Deterministic rubric, no machine-learning inference.
pub const GRADING_ENGINE_NAME: &str = "CYVRA Grading Engine";
pub const GRADING_RUBRIC: &str = "CG-1.0";

/// Below this share of in-scope points the grade is withheld instead of banded.
pub const COVERAGE_FLOOR_PERCENT: u32 = 70;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceForm {
    Portable,
    Fixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ElevationState {
    NotRequested,
    Granted,
    Denied,
}

impl ElevationState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::Granted => "granted",
            Self::Denied => "denied",
        }
    }

    #[must_use]
    pub const fn customer_label(self) -> &'static str {
        match self {
            Self::NotRequested => "Administrator approval was not requested",
            Self::Granted => "Administrator approval was granted",
            Self::Denied => "Administrator approval was declined",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DiagnosticDomain {
    BatteryAndPower,
    ProcessorAndThermal,
    MemoryIntegrity,
    StorageHealth,
    PortsAndConnectivity,
    ScreenAndPeripherals,
}

impl DiagnosticDomain {
    pub const ALL: [Self; 6] = [
        Self::BatteryAndPower,
        Self::ProcessorAndThermal,
        Self::MemoryIntegrity,
        Self::StorageHealth,
        Self::PortsAndConnectivity,
        Self::ScreenAndPeripherals,
    ];

    #[must_use]
    pub const fn weight(self) -> u32 {
        match self {
            Self::BatteryAndPower => 20,
            Self::ProcessorAndThermal => 20,
            Self::MemoryIntegrity => 15,
            Self::StorageHealth => 20,
            Self::PortsAndConnectivity => 10,
            Self::ScreenAndPeripherals => 15,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BatteryAndPower => "Battery and power",
            Self::ProcessorAndThermal => "Processor and thermal stability",
            Self::MemoryIntegrity => "Memory integrity and speed",
            Self::StorageHealth => "Storage health and SMART",
            Self::PortsAndConnectivity => "Ports and connectivity",
            Self::ScreenAndPeripherals => "Screen, keyboard and peripherals",
        }
    }

    /// Storage is always required. Battery is required only on a chassis that
    /// is supposed to have one, so a desktop is never condemned for its absence.
    #[must_use]
    pub const fn is_mandatory_for(self, form: DeviceForm) -> bool {
        match self {
            Self::StorageHealth => true,
            Self::BatteryAndPower => matches!(form, DeviceForm::Portable),
            _ => false,
        }
    }
}

/// A measured hard fault. Unlike missing data this is positive evidence, so it
/// overrides the band instead of being averaged away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CriticalFault {
    MemoryIntegrityMismatch,
    StoragePredictsFailure,
    NvmeCriticalWarning,
    PendingSectorsPresent,
}

impl CriticalFault {
    #[must_use]
    pub const fn reason(self) -> &'static str {
        match self {
            Self::MemoryIntegrityMismatch => {
                "Memory pattern check returned data that did not match what was written"
            }
            Self::StoragePredictsFailure => "Storage firmware reports a predicted failure",
            Self::NvmeCriticalWarning => "NVMe controller reports a critical warning",
            Self::PendingSectorsPresent => "Storage reports sectors pending reallocation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GradeBand {
    APlus,
    A,
    B,
    C,
    F,
    NotGradable,
}

impl GradeBand {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::APlus => "A+",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::F => "F",
            Self::NotGradable => "Grade withheld",
        }
    }

    #[must_use]
    pub const fn condition(self) -> &'static str {
        match self {
            Self::APlus => "Pristine",
            Self::A => "Excellent",
            Self::B => "Fair",
            Self::C => "Degraded, service recommended",
            Self::F => "Defective or parts only",
            Self::NotGradable => "Not enough of this device could be assessed",
        }
    }

    #[must_use]
    pub const fn from_index(index_percent: u32) -> Self {
        match index_percent {
            90..=u32::MAX => Self::APlus,
            80..=89 => Self::A,
            65..=79 => Self::B,
            50..=64 => Self::C,
            _ => Self::F,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DomainApplicability {
    /// The domain applies to this chassis and counts toward the denominator.
    Assessable,
    /// The domain cannot exist on this chassis, so its weight leaves the
    /// denominator entirely rather than being counted as a gap.
    NotApplicable,
}

/// What one domain contributed. `assessed` is how many of the domain's points
/// were actually measurable; `awarded` is how many were earned. Points that
/// were never measurable are derived, never guessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainEvidence {
    pub domain: DiagnosticDomain,
    pub applicability: DomainApplicability,
    pub awarded: u32,
    pub assessed: u32,
    pub confidence: Confidence,
    pub note: Option<String>,
}

impl DomainEvidence {
    /// The domain applies but nothing in it could be measured in this scan.
    #[must_use]
    pub fn not_assessable(domain: DiagnosticDomain, note: &str) -> Self {
        Self {
            domain,
            applicability: DomainApplicability::Assessable,
            awarded: 0,
            assessed: 0,
            confidence: Confidence::Unknown,
            note: Some(note.to_string()),
        }
    }

    /// The domain cannot apply to this chassis, for example a battery on a desktop.
    #[must_use]
    pub fn not_applicable(domain: DiagnosticDomain, note: &str) -> Self {
        Self {
            domain,
            applicability: DomainApplicability::NotApplicable,
            awarded: 0,
            assessed: 0,
            confidence: Confidence::High,
            note: Some(note.to_string()),
        }
    }

    /// Measured evidence. `awarded` is clamped to `assessed` and `assessed` to
    /// the domain weight so no caller can inflate a score.
    #[must_use]
    pub fn measured(
        domain: DiagnosticDomain,
        awarded: u32,
        assessed: u32,
        confidence: Confidence,
    ) -> Self {
        let assessed = assessed.min(domain.weight());
        Self {
            domain,
            applicability: DomainApplicability::Assessable,
            awarded: awarded.min(assessed),
            assessed,
            confidence,
            note: None,
        }
    }

    #[must_use]
    pub const fn in_scope_points(&self) -> u32 {
        match self.applicability {
            DomainApplicability::Assessable => self.domain.weight(),
            DomainApplicability::NotApplicable => 0,
        }
    }

    #[must_use]
    pub const fn not_assessable_points(&self) -> u32 {
        self.in_scope_points().saturating_sub(self.assessed)
    }

    /// Invariants from the rubric: never award more than was assessed, never
    /// assess more than the weight, and never claim points outside scope.
    #[must_use]
    pub const fn is_consistent(&self) -> bool {
        self.awarded <= self.assessed
            && self.assessed <= self.in_scope_points()
            && self.assessed + self.not_assessable_points() == self.in_scope_points()
    }
}

/// The grade block printed on Report D.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageSummary {
    pub engine: &'static str,
    pub rubric: &'static str,
    pub in_scope_points: u32,
    pub assessed_points: u32,
    pub awarded_points: u32,
    pub not_assessable_points: u32,
    pub coverage_percent: u32,
    /// `None` when nothing at all was assessable. Never rendered as zero.
    pub index_percent: Option<u32>,
    pub band: GradeBand,
    pub withheld_reason: Option<String>,
    pub criticals: Vec<CriticalFault>,
    /// True while physical verification and issuance are still outstanding.
    pub provisional: bool,
}

impl CoverageSummary {
    #[must_use]
    pub fn is_withheld(&self) -> bool {
        self.band == GradeBand::NotGradable
    }
}

/// Apply rubric CG-1.0.
///
/// Precedence, in this order:
/// 1. a measured critical fault forces `F`, because it is evidence we hold;
/// 2. a mandatory domain with nothing assessable withholds the grade;
/// 3. coverage below the floor withholds the grade;
/// 4. otherwise the assessed index selects the band.
#[must_use]
pub fn evaluate(
    evidence: &[DomainEvidence],
    form: DeviceForm,
    criticals: &[CriticalFault],
) -> CoverageSummary {
    let in_scope_points: u32 = evidence.iter().map(DomainEvidence::in_scope_points).sum();
    let assessed_points: u32 = evidence.iter().map(|domain| domain.assessed).sum();
    let awarded_points: u32 = evidence.iter().map(|domain| domain.awarded).sum();
    let not_assessable_points: u32 = evidence
        .iter()
        .map(DomainEvidence::not_assessable_points)
        .sum();

    let coverage_percent = percent_of(assessed_points, in_scope_points);
    let index_percent = if assessed_points == 0 {
        None
    } else {
        Some(percent_of(awarded_points, assessed_points))
    };

    let missing_mandatory = evidence
        .iter()
        .filter(|domain| {
            domain.applicability == DomainApplicability::Assessable
                && domain.assessed == 0
                && domain.domain.is_mandatory_for(form)
        })
        .map(|domain| domain.domain.label())
        .collect::<Vec<_>>();

    let (band, withheld_reason) = if let Some(fault) = criticals.first() {
        (GradeBand::F, Some(fault.reason().to_string()))
    } else if !missing_mandatory.is_empty() {
        (
            GradeBand::NotGradable,
            Some(format!(
                "Grade withheld. A required area could not be assessed in this scan: {}.",
                missing_mandatory.join(", ")
            )),
        )
    } else if assessed_points == 0 {
        (
            GradeBand::NotGradable,
            Some("Grade withheld. No diagnostic area could be assessed in this scan.".to_string()),
        )
    } else if coverage_percent < COVERAGE_FLOOR_PERCENT {
        (
            GradeBand::NotGradable,
            Some(format!(
                "Grade withheld. Only {coverage_percent}% of this device could be assessed, below the {COVERAGE_FLOOR_PERCENT}% required for a grade."
            )),
        )
    } else {
        (
            index_percent.map_or(GradeBand::NotGradable, GradeBand::from_index),
            None,
        )
    };

    CoverageSummary {
        engine: GRADING_ENGINE_NAME,
        rubric: GRADING_RUBRIC,
        in_scope_points,
        assessed_points,
        awarded_points,
        not_assessable_points,
        coverage_percent,
        index_percent,
        band,
        withheld_reason,
        criticals: criticals.to_vec(),
        provisional: true,
    }
}

/// Battery points from measured health, per rubric CG-1.0 section 5.1.
/// Only ever called with a health figure derived from two real capacities.
#[must_use]
pub fn battery_points(health_percent: f64) -> u32 {
    if health_percent >= 85.0 {
        20
    } else if health_percent >= 75.0 {
        16
    } else if health_percent >= 60.0 {
        11
    } else if health_percent >= 50.0 {
        6
    } else {
        0
    }
}

fn percent_of(part: u32, whole: u32) -> u32 {
    if whole == 0 {
        return 0;
    }
    ((u64::from(part) * 100 + u64::from(whole) / 2) / u64::from(whole))
        .try_into()
        .unwrap_or(u32::MAX)
}

/// Deep battery telemetry. Populated by the battery slice, never inferred.
#[derive(Debug, Clone, PartialEq)]
pub struct BatteryDetail {
    pub present: InventoryField<bool>,
    pub manufacturer: InventoryField<String>,
    pub model: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
    pub chemistry: InventoryField<String>,
    pub manufacture_date: InventoryField<String>,
    pub designed_capacity_mwh: InventoryField<u64>,
    pub full_charge_capacity_mwh: InventoryField<u64>,
    pub remaining_capacity_mwh: InventoryField<u64>,
    pub voltage_mv: InventoryField<u32>,
    pub cycle_count: InventoryField<u32>,
    pub charge_rate_mw: InventoryField<i64>,
    pub wear_percent: InventoryField<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorDetail {
    pub brand_string: InventoryField<String>,
    pub base_mhz: InventoryField<u32>,
    pub maximum_mhz: InventoryField<u32>,
    pub current_mhz: InventoryField<u32>,
    pub l1_cache_bytes: InventoryField<u64>,
    pub l2_cache_bytes: InventoryField<u64>,
    pub l3_cache_bytes: InventoryField<u64>,
    pub instruction_sets: InventoryField<String>,
    /// Stays unknown on Windows: package temperature needs a kernel driver we
    /// deliberately do not ship.
    pub package_temperature_c: InventoryField<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRuntime {
    pub total_bytes: InventoryField<u64>,
    pub available_bytes: InventoryField<u64>,
    pub commit_limit_bytes: InventoryField<u64>,
    pub channel_mode: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageHealth {
    pub model: InventoryField<String>,
    pub serial_number: InventoryField<DeviceIdentifier>,
    pub firmware_revision: InventoryField<String>,
    pub bus_type: InventoryField<String>,
    pub capacity_bytes: InventoryField<u64>,
    pub rotational: InventoryField<bool>,
    pub power_on_hours: InventoryField<u64>,
    pub power_cycles: InventoryField<u64>,
    pub data_units_read: InventoryField<u64>,
    pub data_units_written: InventoryField<u64>,
    pub percentage_used: InventoryField<u32>,
    pub available_spare_percent: InventoryField<u32>,
    pub composite_temperature_c: InventoryField<f64>,
    pub media_errors: InventoryField<u64>,
    pub unsafe_shutdowns: InventoryField<u64>,
    pub reallocated_sectors: InventoryField<u64>,
    pub pending_sectors: InventoryField<u64>,
    pub critical_warning: InventoryField<bool>,
    pub predicts_failure: InventoryField<bool>,
    pub remaining_life_percent: InventoryField<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsbTopologyPort {
    pub controller: InventoryField<String>,
    pub hub: InventoryField<String>,
    pub port_index: InventoryField<u32>,
    pub occupied: InventoryField<bool>,
    pub negotiated_speed: InventoryField<String>,
    pub supported_speed: InventoryField<String>,
    pub physically_verified: InventoryField<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayPanel {
    pub manufacturer: InventoryField<String>,
    pub product_code: InventoryField<String>,
    pub panel_serial: InventoryField<DeviceIdentifier>,
    pub native_width: InventoryField<u32>,
    pub native_height: InventoryField<u32>,
    pub refresh_hz: InventoryField<u32>,
    pub bit_depth: InventoryField<u32>,
    pub hdr_capable: InventoryField<bool>,
    pub manufacture_week: InventoryField<u32>,
    pub manufacture_year: InventoryField<u32>,
    pub internal_panel: InventoryField<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadioDetail {
    pub kind: InventoryField<String>,
    pub description: InventoryField<String>,
    pub state: InventoryField<String>,
    pub signal_quality_percent: InventoryField<u32>,
    pub rssi_dbm: InventoryField<i32>,
    pub receive_mbps: InventoryField<u32>,
    pub transmit_mbps: InventoryField<u32>,
    pub radio_standards: InventoryField<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaptureDevice {
    pub kind: InventoryField<String>,
    pub friendly_name: InventoryField<String>,
    pub enumerated_by: InventoryField<String>,
    pub present: InventoryField<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkResult {
    pub kind: InventoryField<String>,
    pub duration_ms: InventoryField<u64>,
    pub sustained_clock_ratio: InventoryField<f64>,
    pub bytes_tested: InventoryField<u64>,
    pub integrity_verified: InventoryField<bool>,
    pub sequential_read_mbps: InventoryField<f64>,
    pub sequential_write_mbps: InventoryField<f64>,
    pub random_read_iops: InventoryField<u64>,
    pub random_write_iops: InventoryField<u64>,
    /// Recorded on the report so the write boundary is always visible.
    pub wrote_to_disk: InventoryField<bool>,
    pub bytes_written: InventoryField<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InteractiveCheck {
    pub subject: InventoryField<String>,
    pub outcome: InventoryField<String>,
    pub detail: InventoryField<String>,
    pub attested_by_operator: InventoryField<bool>,
}

/// Advance scan result. Every section starts explicitly not reported, so a
/// build that has not learned to collect something cannot imply that it did.
#[derive(Debug, Clone, PartialEq)]
pub struct HardwareDiagnosticsV1 {
    pub schema_version: &'static str,
    pub collected_at_unix: u64,
    pub elevation_state: ElevationState,
    pub benchmarks_consented: bool,
    pub write_benchmark_consented: bool,
    pub bytes_written: u64,
    pub batteries: InventorySection<BatteryDetail>,
    pub processors: InventorySection<ProcessorDetail>,
    pub memory: InventorySection<MemoryRuntime>,
    pub storage: InventorySection<StorageHealth>,
    pub usb_ports: InventorySection<UsbTopologyPort>,
    pub displays: InventorySection<DisplayPanel>,
    pub radios: InventorySection<RadioDetail>,
    pub capture_devices: InventorySection<CaptureDevice>,
    pub benchmarks: InventorySection<BenchmarkResult>,
    pub interactive: InventorySection<InteractiveCheck>,
}

impl HardwareDiagnosticsV1 {
    /// Honest initial state: nothing collected, nothing claimed, nothing written.
    #[must_use]
    pub fn not_collected(collected_at_unix: u64) -> Self {
        let provenance = Provenance::not_collected(collected_at_unix);

        Self {
            schema_version: SCHEMA_VERSION,
            collected_at_unix,
            elevation_state: ElevationState::NotRequested,
            benchmarks_consented: false,
            write_benchmark_consented: false,
            bytes_written: 0,
            batteries: InventorySection::not_reported(provenance.clone()),
            processors: InventorySection::not_reported(provenance.clone()),
            memory: InventorySection::not_reported(provenance.clone()),
            storage: InventorySection::not_reported(provenance.clone()),
            usb_ports: InventorySection::not_reported(provenance.clone()),
            displays: InventorySection::not_reported(provenance.clone()),
            radios: InventorySection::not_reported(provenance.clone()),
            capture_devices: InventorySection::not_reported(provenance.clone()),
            benchmarks: InventorySection::not_reported(provenance.clone()),
            interactive: InventorySection::not_reported(provenance),
        }
    }

    /// True while no section has produced a record. A report built from this
    /// state must show every row as not collected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.batteries.records.is_empty()
            && self.processors.records.is_empty()
            && self.memory.records.is_empty()
            && self.storage.records.is_empty()
            && self.usb_ports.records.is_empty()
            && self.displays.records.is_empty()
            && self.radios.records.is_empty()
            && self.capture_devices.records.is_empty()
            && self.benchmarks.records.is_empty()
            && self.interactive.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_not_assessable() -> Vec<DomainEvidence> {
        DiagnosticDomain::ALL
            .iter()
            .map(|domain| DomainEvidence::not_assessable(*domain, "not collected in this scan"))
            .collect()
    }

    #[test]
    fn domain_weights_total_one_hundred() {
        let total: u32 = DiagnosticDomain::ALL
            .iter()
            .map(|domain| domain.weight())
            .sum();

        assert_eq!(total, 100);
    }

    #[test]
    fn empty_diagnostics_claim_nothing() {
        let diagnostics = HardwareDiagnosticsV1::not_collected(1_000);

        assert_eq!(diagnostics.schema_version, SCHEMA_VERSION);
        assert!(diagnostics.is_empty());
        assert_eq!(diagnostics.elevation_state, ElevationState::NotRequested);
        assert!(!diagnostics.benchmarks_consented);
        assert!(!diagnostics.write_benchmark_consented);
        assert_eq!(diagnostics.bytes_written, 0);
    }

    #[test]
    fn nothing_assessable_withholds_the_grade_and_never_scores_zero() {
        let summary = evaluate(&all_not_assessable(), DeviceForm::Portable, &[]);

        assert_eq!(summary.band, GradeBand::NotGradable);
        assert!(summary.is_withheld());
        assert_eq!(summary.index_percent, None);
        assert_eq!(summary.coverage_percent, 0);
        assert_eq!(summary.not_assessable_points, 100);
        assert!(summary.withheld_reason.is_some());
        assert_eq!(summary.engine, "CYVRA Grading Engine");
    }

    #[test]
    fn unreadable_storage_is_withheld_not_failed() {
        let mut evidence = vec![
            DomainEvidence::measured(DiagnosticDomain::BatteryAndPower, 20, 20, Confidence::High),
            DomainEvidence::measured(
                DiagnosticDomain::ProcessorAndThermal,
                20,
                20,
                Confidence::High,
            ),
            DomainEvidence::measured(DiagnosticDomain::MemoryIntegrity, 15, 15, Confidence::High),
            DomainEvidence::measured(
                DiagnosticDomain::PortsAndConnectivity,
                10,
                10,
                Confidence::High,
            ),
            DomainEvidence::measured(
                DiagnosticDomain::ScreenAndPeripherals,
                15,
                15,
                Confidence::High,
            ),
        ];
        evidence.push(DomainEvidence::not_assessable(
            DiagnosticDomain::StorageHealth,
            "administrator approval was declined",
        ));

        let summary = evaluate(&evidence, DeviceForm::Portable, &[]);

        assert_eq!(summary.band, GradeBand::NotGradable);
        assert_ne!(summary.band, GradeBand::F);
        assert_eq!(summary.index_percent, Some(100));
        assert_eq!(summary.coverage_percent, 80);
    }

    #[test]
    fn desktop_without_a_battery_is_still_gradable() {
        let evidence = vec![
            DomainEvidence::not_applicable(
                DiagnosticDomain::BatteryAndPower,
                "no battery on this chassis",
            ),
            DomainEvidence::measured(
                DiagnosticDomain::ProcessorAndThermal,
                18,
                20,
                Confidence::High,
            ),
            DomainEvidence::measured(DiagnosticDomain::MemoryIntegrity, 15, 15, Confidence::High),
            DomainEvidence::measured(DiagnosticDomain::StorageHealth, 20, 20, Confidence::High),
            DomainEvidence::measured(
                DiagnosticDomain::PortsAndConnectivity,
                8,
                10,
                Confidence::Medium,
            ),
            DomainEvidence::measured(
                DiagnosticDomain::ScreenAndPeripherals,
                13,
                15,
                Confidence::Medium,
            ),
        ];

        let summary = evaluate(&evidence, DeviceForm::Fixed, &[]);

        assert_eq!(summary.in_scope_points, 80);
        assert_eq!(summary.assessed_points, 80);
        assert_eq!(summary.coverage_percent, 100);
        assert_eq!(summary.index_percent, Some(93));
        assert_eq!(summary.band, GradeBand::APlus);
    }

    #[test]
    fn a_measured_critical_fault_forces_f_even_with_a_high_index() {
        let evidence = vec![
            DomainEvidence::measured(DiagnosticDomain::BatteryAndPower, 20, 20, Confidence::High),
            DomainEvidence::measured(
                DiagnosticDomain::ProcessorAndThermal,
                20,
                20,
                Confidence::High,
            ),
            DomainEvidence::measured(DiagnosticDomain::MemoryIntegrity, 15, 15, Confidence::High),
            DomainEvidence::measured(DiagnosticDomain::StorageHealth, 20, 20, Confidence::High),
            DomainEvidence::measured(
                DiagnosticDomain::PortsAndConnectivity,
                10,
                10,
                Confidence::High,
            ),
            DomainEvidence::measured(
                DiagnosticDomain::ScreenAndPeripherals,
                15,
                15,
                Confidence::High,
            ),
        ];

        let summary = evaluate(
            &evidence,
            DeviceForm::Portable,
            &[CriticalFault::StoragePredictsFailure],
        );

        assert_eq!(summary.index_percent, Some(100));
        assert_eq!(summary.band, GradeBand::F);
        assert_eq!(summary.criticals.len(), 1);
    }

    #[test]
    fn coverage_below_the_floor_withholds_the_grade() {
        let evidence = vec![
            DomainEvidence::measured(DiagnosticDomain::StorageHealth, 20, 20, Confidence::High),
            DomainEvidence::measured(DiagnosticDomain::BatteryAndPower, 20, 20, Confidence::High),
            DomainEvidence::not_assessable(
                DiagnosticDomain::ProcessorAndThermal,
                "benchmark declined",
            ),
            DomainEvidence::not_assessable(DiagnosticDomain::MemoryIntegrity, "benchmark declined"),
            DomainEvidence::not_assessable(DiagnosticDomain::PortsAndConnectivity, "not attempted"),
            DomainEvidence::not_assessable(DiagnosticDomain::ScreenAndPeripherals, "not attempted"),
        ];

        let summary = evaluate(&evidence, DeviceForm::Portable, &[]);

        assert_eq!(summary.coverage_percent, 40);
        assert_eq!(summary.band, GradeBand::NotGradable);
        assert!(
            summary
                .withheld_reason
                .as_deref()
                .unwrap_or_default()
                .contains("40%")
        );
    }

    #[test]
    fn evidence_cannot_award_more_than_it_assessed() {
        let inflated =
            DomainEvidence::measured(DiagnosticDomain::MemoryIntegrity, 99, 99, Confidence::High);

        assert_eq!(inflated.assessed, 15);
        assert_eq!(inflated.awarded, 15);
        assert!(inflated.is_consistent());
    }

    #[test]
    fn every_evidence_shape_keeps_the_points_invariant() {
        let shapes = vec![
            DomainEvidence::not_assessable(DiagnosticDomain::StorageHealth, "gap"),
            DomainEvidence::not_applicable(DiagnosticDomain::BatteryAndPower, "desktop"),
            DomainEvidence::measured(
                DiagnosticDomain::PortsAndConnectivity,
                4,
                6,
                Confidence::Low,
            ),
        ];

        for shape in shapes {
            assert!(
                shape.is_consistent(),
                "{shape:?} broke the points invariant"
            );
        }
    }

    #[test]
    fn battery_points_follow_the_health_bands() {
        assert_eq!(battery_points(100.0), 20);
        assert_eq!(battery_points(85.0), 20);
        assert_eq!(battery_points(84.9), 16);
        assert_eq!(battery_points(75.0), 16);
        assert_eq!(battery_points(74.9), 11);
        assert_eq!(battery_points(60.0), 11);
        assert_eq!(battery_points(59.9), 6);
        assert_eq!(battery_points(50.0), 6);
        assert_eq!(battery_points(49.9), 0);
    }

    #[test]
    fn band_edges_follow_the_rubric() {
        assert_eq!(GradeBand::from_index(100), GradeBand::APlus);
        assert_eq!(GradeBand::from_index(90), GradeBand::APlus);
        assert_eq!(GradeBand::from_index(89), GradeBand::A);
        assert_eq!(GradeBand::from_index(80), GradeBand::A);
        assert_eq!(GradeBand::from_index(79), GradeBand::B);
        assert_eq!(GradeBand::from_index(65), GradeBand::B);
        assert_eq!(GradeBand::from_index(64), GradeBand::C);
        assert_eq!(GradeBand::from_index(50), GradeBand::C);
        assert_eq!(GradeBand::from_index(49), GradeBand::F);
    }
}
