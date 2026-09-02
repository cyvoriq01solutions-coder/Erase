//! Storage identity and SMART collection for Advance scan.
//!
//! Report A already lists disks. A4 adds the health telemetry an ITAD buyer
//! actually needs: power-on hours, wear, spare, and the firmware predict-failure
//! bit. Collection is read-only. The script never names a write, erase, TRIM,
//! format, sanitize or firmware-update command.
//!
//! Two transports, both through the existing bounded runtime:
//!   1. `Get-PhysicalDisk` plus `Get-StorageReliabilityCounter` (NVMe/SSD wear,
//!      temperature, power-on hours when Windows exposes them).
//!   2. `MSStorageDriver_FailurePredictStatus` / `FailurePredictData` (ATA
//!      predict-failure and SMART attributes 5 / 9 / 12 / 197).
//!
//! Identity comes from `Win32_DiskDrive` so a disk that withholds SMART still
//! prints a model and serial. Nothing here estimates remaining life. A missing
//! spare is not assumed to be 100%. Elevation is not prompted in this slice
//! (unsigned EXE). If Windows refuses a class, Report D prints permission
//! denied and storage stays not assessable.

use crate::collector_runtime::{
    CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
};
use crate::hardware_diagnostics_v1::{CriticalFault, ata_storage_points, nvme_storage_points};
use crate::{CollectorErrorKind, CollectorName};
use std::collections::BTreeMap;
use std::time::Duration;

const PROBE_TIMEOUT: Duration = Duration::from_secs(45);
const MAX_STDOUT_BYTES: usize = 96 * 1024;
const MAX_STDERR_BYTES: usize = 16 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROTOCOL_LINES: usize = 768;
const MAX_VALUE_BYTES: usize = 2 * 1024;
const MAX_INDEX: usize = 16;

const STORAGE_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
    r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Emit-Value([string]$section, [int]$index, [string]$name, $value) {
    if ($null -eq $value) { return }
    $text = [Convert]::ToString($value, [Globalization.CultureInfo]::InvariantCulture)
    if ($null -eq $text) { return }
    $text = $text.Trim()
    if ($text.Length -eq 0) { return }
    $bytes = [Text.Encoding]::UTF8.GetBytes($text)
    $hex = -join ($bytes | ForEach-Object { $_.ToString('x2') })
    [Console]::Out.WriteLine("$section`t$index`t$name`t$hex")
}

function Emit-Status([string]$section, $errorRecord) {
    $status = 'collection_error'
    if ($null -ne $errorRecord) {
        $hresult = $errorRecord.Exception.HResult
        $native = $errorRecord.Exception.NativeErrorCode
        if (($hresult -eq -2147024891) -or ($native -eq 5) -or ("$($errorRecord.Exception.Message)" -match 'Access is denied|UnauthorizedAccess')) { $status = 'permission_denied' }
        elseif ($errorRecord.Exception -is [Management.Automation.CommandNotFoundException]) { $status = 'unsupported' }
        elseif ("$($errorRecord.Exception.Message)" -match 'not supported|Invalid class|Not found') { $status = 'unsupported' }
    }
    Emit-Value $section 0 'source_status' $status
}

function Bus-Label([int]$code) {
    switch ($code) {
        3  { return 'ATA' }
        7  { return 'USB' }
        8  { return 'RAID' }
        10 { return 'SAS' }
        11 { return 'SATA' }
        12 { return 'SD' }
        17 { return 'NVMe' }
        default { return $null }
    }
}

function Media-Label([int]$code) {
    switch ($code) {
        3 { return 'HDD' }
        4 { return 'SSD' }
        5 { return 'SCM' }
        default { return $null }
    }
}

try {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    $admin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    Emit-Value 'session' 0 'source_status' 'reported'
    Emit-Value 'session' 0 'elevated' $admin
} catch {
    Emit-Value 'session' 0 'source_status' 'reported'
    Emit-Value 'session' 0 'elevated' $false
}

try {
    $items = @(Get-CimInstance -ClassName Win32_DiskDrive -Property Model,SerialNumber,FirmwareRevision,Size,InterfaceType,MediaType,PNPDeviceID,Index,Status -ErrorAction Stop)
    Emit-Value 'disk' 0 'source_status' 'reported'
    Emit-Value 'disk' 0 'record_count' $items.Count
    for ($i = 0; $i -lt $items.Count; $i++) {
        if ($i -ge 16) { break }
        $d = $items[$i]
        Emit-Value 'disk' $i 'present' $true
        Emit-Value 'disk' $i 'model' $d.Model
        Emit-Value 'disk' $i 'serial_number' $d.SerialNumber
        Emit-Value 'disk' $i 'firmware_revision' $d.FirmwareRevision
        Emit-Value 'disk' $i 'capacity_bytes' $d.Size
        Emit-Value 'disk' $i 'interface_type' $d.InterfaceType
        Emit-Value 'disk' $i 'media_type' $d.MediaType
        Emit-Value 'disk' $i 'status' $d.Status
        if ("$($d.InterfaceType) $($d.MediaType) $($d.PNPDeviceID)" -match 'USB|Removable') {
            Emit-Value 'disk' $i 'removable' $true
        }
    }
} catch { Emit-Status 'disk' $_ }

try {
    $disks = @(Get-PhysicalDisk -ErrorAction Stop)
    Emit-Value 'physical' 0 'source_status' 'reported'
    Emit-Value 'physical' 0 'record_count' $disks.Count
    $reliabilityReported = $false
    for ($i = 0; $i -lt $disks.Count; $i++) {
        if ($i -ge 16) { break }
        $p = $disks[$i]
        Emit-Value 'physical' $i 'present' $true
        Emit-Value 'physical' $i 'model' $p.FriendlyName
        Emit-Value 'physical' $i 'serial_number' $p.SerialNumber
        Emit-Value 'physical' $i 'firmware_revision' $p.FirmwareVersion
        Emit-Value 'physical' $i 'capacity_bytes' $p.Size
        $bus = Bus-Label ([int]$p.BusType)
        if ($bus) { Emit-Value 'physical' $i 'bus_type' $bus }
        $media = Media-Label ([int]$p.MediaType)
        if ($media) { Emit-Value 'physical' $i 'media_kind' $media }
        if ([int]$p.BusType -eq 7 -or [int]$p.BusType -eq 12 -or [int]$p.BusType -eq 15) {
            Emit-Value 'physical' $i 'removable' $true
        }
        if ($null -ne $p.SpindleSpeed -and [int64]$p.SpindleSpeed -gt 1) {
            Emit-Value 'physical' $i 'rotational' $true
        } elseif ($media -eq 'HDD') {
            Emit-Value 'physical' $i 'rotational' $true
        } elseif ($media -eq 'SSD' -or $bus -eq 'NVMe') {
            Emit-Value 'physical' $i 'rotational' $false
        }
        Emit-Value 'physical' $i 'health_status' $p.HealthStatus
        try {
            $r = $p | Get-StorageReliabilityCounter -ErrorAction Stop
            if (-not $reliabilityReported) {
                Emit-Value 'reliability' 0 'source_status' 'reported'
                $reliabilityReported = $true
            }
            Emit-Value 'reliability' $i 'present' $true
            if ($null -ne $r.Wear) { Emit-Value 'reliability' $i 'percentage_used' $r.Wear }
            if ($null -ne $r.Temperature) { Emit-Value 'reliability' $i 'temperature_c' $r.Temperature }
            if ($null -ne $r.PowerOnHours) { Emit-Value 'reliability' $i 'power_on_hours' $r.PowerOnHours }
            if ($null -ne $r.StartStopCycleCount) { Emit-Value 'reliability' $i 'power_cycles' $r.StartStopCycleCount }
            if ($null -ne $r.ReadErrorsUncorrected) { Emit-Value 'reliability' $i 'read_errors_uncorrected' $r.ReadErrorsUncorrected }
            if ($null -ne $r.WriteErrorsUncorrected) { Emit-Value 'reliability' $i 'write_errors_uncorrected' $r.WriteErrorsUncorrected }
        } catch {
            if (-not $reliabilityReported) { Emit-Status 'reliability' $_; $reliabilityReported = $true }
        }
    }
    if (-not $reliabilityReported) {
        Emit-Value 'reliability' 0 'source_status' 'unsupported'
    }
} catch { Emit-Status 'physical' $_; Emit-Status 'reliability' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/wmi' -ClassName MSStorageDriver_FailurePredictStatus -ErrorAction Stop)
    Emit-Value 'predict' 0 'source_status' 'reported'
    Emit-Value 'predict' 0 'record_count' $items.Count
    for ($i = 0; $i -lt $items.Count; $i++) {
        if ($i -ge 16) { break }
        $s = $items[$i]
        Emit-Value 'predict' $i 'present' $true
        Emit-Value 'predict' $i 'predicts_failure' ([bool]$s.PredictFailure)
        Emit-Value 'predict' $i 'reason' $s.Reason
        Emit-Value 'predict' $i 'instance' $s.InstanceName
    }
} catch { Emit-Status 'predict' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/wmi' -ClassName MSStorageDriver_FailurePredictData -ErrorAction Stop)
    Emit-Value 'smartdata' 0 'source_status' 'reported'
    for ($i = 0; $i -lt $items.Count; $i++) {
        if ($i -ge 16) { break }
        $s = $items[$i]
        $vs = $s.VendorSpecific
        if ($null -eq $vs) { continue }
        Emit-Value 'smartdata' $i 'present' $true
        for ($a = 0; $a -lt 30; $a++) {
            $off = 2 + ($a * 12)
            if (($off + 11) -ge $vs.Length) { break }
            $id = [int]$vs[$off]
            if ($id -eq 0) { continue }
            $raw = [uint64]$vs[$off + 5] + ([uint64]$vs[$off + 6] * 256) + ([uint64]$vs[$off + 7] * 65536) + ([uint64]$vs[$off + 8] * 16777216)
            if ($id -eq 5) { Emit-Value 'smartdata' $i 'reallocated_sectors' $raw }
            if ($id -eq 9) { Emit-Value 'smartdata' $i 'power_on_hours' $raw }
            if ($id -eq 12) { Emit-Value 'smartdata' $i 'power_cycles' $raw }
            if ($id -eq 197) { Emit-Value 'smartdata' $i 'pending_sectors' $raw }
            if ($id -eq 194 -or $id -eq 190) {
                $temp = [int]($raw -band 0xFF)
                if ($temp -gt 0 -and $temp -lt 127) { Emit-Value 'smartdata' $i 'temperature_c' $temp }
            }
        }
    }
} catch { Emit-Status 'smartdata' $_ }
"#,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StorageSource {
    DiskIdentity,
    PhysicalDisk,
    ReliabilityCounter,
    PredictFailure,
    SmartAttributes,
}

impl StorageSource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::DiskIdentity => "Disk identity",
            Self::PhysicalDisk => "Physical disk class",
            Self::ReliabilityCounter => "Storage reliability counters",
            Self::PredictFailure => "SMART predict-failure",
            Self::SmartAttributes => "ATA SMART attributes",
        }
    }

    const fn section(self) -> &'static str {
        match self {
            Self::DiskIdentity => "disk",
            Self::PhysicalDisk => "physical",
            Self::ReliabilityCounter => "reliability",
            Self::PredictFailure => "predict",
            Self::SmartAttributes => "smartdata",
        }
    }

    const ALL: [Self; 5] = [
        Self::DiskIdentity,
        Self::PhysicalDisk,
        Self::ReliabilityCounter,
        Self::PredictFailure,
        Self::SmartAttributes,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceOutcome {
    Reported,
    NotQueried,
    PermissionDenied,
    Unsupported,
    CollectionError,
}

impl SourceOutcome {
    #[must_use]
    pub const fn customer_label(self) -> &'static str {
        match self {
            Self::Reported => "Answered",
            Self::NotQueried => "Not queried on this PC",
            Self::PermissionDenied => "Refused without administrator rights",
            Self::Unsupported => "Not available on this PC",
            Self::CollectionError => "Windows returned an error",
        }
    }

    const fn from_wire(value: &str) -> Self {
        match value.as_bytes() {
            b"reported" => Self::Reported,
            b"permission_denied" => Self::PermissionDenied,
            b"unsupported" => Self::Unsupported,
            _ => Self::CollectionError,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceStatus {
    pub source: StorageSource,
    pub outcome: SourceOutcome,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DriveReading {
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_revision: Option<String>,
    pub bus_type: Option<String>,
    pub media_kind: Option<String>,
    pub capacity_bytes: Option<u64>,
    pub rotational: Option<bool>,
    pub removable: bool,
    pub power_on_hours: Option<u64>,
    pub power_cycles: Option<u64>,
    pub percentage_used: Option<u32>,
    pub available_spare_percent: Option<u32>,
    pub temperature_c: Option<f64>,
    pub media_errors: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub critical_warning: Option<bool>,
    pub predicts_failure: Option<bool>,
    pub remaining_life_percent: Option<u32>,
}

impl DriveReading {
    #[must_use]
    pub fn display_name(&self) -> String {
        self.model
            .clone()
            .or_else(|| self.serial_number.clone())
            .unwrap_or_else(|| "Storage device".to_string())
    }

    #[must_use]
    pub fn is_scorable(&self) -> bool {
        scoring_kind(self).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScoringKind {
    Nvme,
    Ata,
    PredictedFailure,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageProbe {
    pub drives: Vec<DriveReading>,
    pub sources: Vec<SourceStatus>,
    pub elevated: Option<bool>,
    pub probe_error: Option<&'static str>,
}

impl StorageProbe {
    #[must_use]
    pub fn unavailable(reason: &'static str) -> Self {
        Self {
            drives: Vec::new(),
            sources: StorageSource::ALL
                .iter()
                .map(|source| SourceStatus {
                    source: *source,
                    outcome: SourceOutcome::NotQueried,
                })
                .collect(),
            elevated: None,
            probe_error: Some(reason),
        }
    }

    #[must_use]
    pub fn outcome_for(&self, source: StorageSource) -> SourceOutcome {
        self.sources
            .iter()
            .find(|status| status.source == source)
            .map_or(SourceOutcome::NotQueried, |status| status.outcome)
    }

    #[must_use]
    pub fn reliability_refused(&self) -> bool {
        matches!(
            self.outcome_for(StorageSource::ReliabilityCounter),
            SourceOutcome::PermissionDenied
        ) || matches!(
            self.outcome_for(StorageSource::PredictFailure),
            SourceOutcome::PermissionDenied
        )
    }

    /// Drives used for CG-1.0. Internal media wins over USB sticks.
    #[must_use]
    pub fn scoring_drives(&self) -> Vec<&DriveReading> {
        let internal: Vec<&DriveReading> = self
            .drives
            .iter()
            .filter(|drive| !drive.removable && drive.is_scorable())
            .collect();
        if !internal.is_empty() {
            return internal;
        }
        self.drives
            .iter()
            .filter(|drive| drive.is_scorable())
            .collect()
    }

    #[must_use]
    pub fn awarded_points(&self) -> Option<u32> {
        let drives = self.scoring_drives();
        if drives.is_empty() {
            return None;
        }
        Some(
            drives
                .iter()
                .map(|drive| drive_points(drive))
                .min()
                .unwrap_or(0),
        )
    }

    #[must_use]
    pub fn critical_faults(&self) -> Vec<CriticalFault> {
        let mut faults = Vec::new();
        for drive in self.scoring_drives() {
            if drive.predicts_failure == Some(true) {
                push_unique(&mut faults, CriticalFault::StoragePredictsFailure);
            }
            if drive.critical_warning == Some(true) {
                push_unique(&mut faults, CriticalFault::NvmeCriticalWarning);
            }
            if drive.pending_sectors.is_some_and(|pending| pending > 0) {
                push_unique(&mut faults, CriticalFault::PendingSectorsPresent);
            }
        }
        faults
    }
}

fn scoring_kind(drive: &DriveReading) -> Option<ScoringKind> {
    if drive.predicts_failure == Some(true)
        || drive.critical_warning == Some(true)
        || drive.pending_sectors.is_some_and(|pending| pending > 0)
    {
        return Some(ScoringKind::PredictedFailure);
    }
    if drive.percentage_used.is_some() {
        return Some(ScoringKind::Nvme);
    }
    if drive.reallocated_sectors.is_some() && drive.pending_sectors.is_some() {
        return Some(ScoringKind::Ata);
    }
    None
}

fn drive_points(drive: &DriveReading) -> u32 {
    if drive.predicts_failure == Some(true) || drive.critical_warning == Some(true) {
        return 0;
    }
    if let Some(pending) = drive.pending_sectors
        && pending > 0
    {
        return 0;
    }
    if let Some(used) = drive.percentage_used {
        return nvme_storage_points(
            used,
            drive.available_spare_percent,
            drive.critical_warning.unwrap_or(false),
            drive.media_errors.unwrap_or(0),
        );
    }
    if let (Some(reallocated), Some(pending)) = (drive.reallocated_sectors, drive.pending_sectors) {
        return ata_storage_points(reallocated, pending);
    }
    0
}

fn push_unique(faults: &mut Vec<CriticalFault>, fault: CriticalFault) {
    if !faults.contains(&fault) {
        faults.push(fault);
    }
}

#[must_use]
pub fn collect(cancellation: &CancellationToken) -> StorageProbe {
    let limits = CollectorLimits::new(
        PROBE_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );

    match run_fixed_powershell(
        CollectorName::HardwareInventory,
        STORAGE_SCRIPT,
        limits,
        cancellation,
    ) {
        Ok(output) => match std::str::from_utf8(output.stdout()) {
            Ok(text) => parse_probe(text),
            Err(_) => StorageProbe::unavailable("The storage probe returned unreadable output."),
        },
        Err(error) => StorageProbe::unavailable(match error.kind {
            CollectorErrorKind::Unsupported => {
                "Storage SMART collection is only available on Windows."
            }
            CollectorErrorKind::PermissionDenied => {
                "Windows refused the storage health query on this account."
            }
            CollectorErrorKind::TimedOut => "The storage health probe exceeded its time limit.",
            CollectorErrorKind::Cancelled => "The storage health probe was cancelled.",
            CollectorErrorKind::OutputLimitExceeded => {
                "The storage health probe returned more data than allowed."
            }
            _ => "The storage health probe could not be completed on this PC.",
        }),
    }
}

#[must_use]
pub fn parse_probe(text: &str) -> StorageProbe {
    let mut values: BTreeMap<(String, usize, String), String> = BTreeMap::new();

    for (line_number, line) in text.lines().enumerate() {
        if line_number >= MAX_PROTOCOL_LINES || line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let (Some(section), Some(index), Some(name), Some(encoded)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if parts.next().is_some() || !is_allowed(section, name) {
            continue;
        }
        let Ok(index) = index.parse::<usize>() else {
            continue;
        };
        if index > MAX_INDEX {
            continue;
        }
        let Some(value) = decode_hex(encoded) else {
            continue;
        };
        values.insert((section.to_string(), index, name.to_string()), value);
    }

    let sources = StorageSource::ALL
        .iter()
        .map(|source| SourceStatus {
            source: *source,
            outcome: values
                .get(&(source.section().to_string(), 0, "source_status".to_string()))
                .map_or(SourceOutcome::NotQueried, |value| {
                    SourceOutcome::from_wire(value)
                }),
        })
        .collect();

    let elevated = bool_field(&values, "session", 0, "elevated");

    StorageProbe {
        drives: merge_drives(&values),
        sources,
        elevated,
        probe_error: None,
    }
}

fn merge_drives(values: &BTreeMap<(String, usize, String), String>) -> Vec<DriveReading> {
    let mut drives = Vec::new();
    for index in 0..=MAX_INDEX {
        let disk_present = flag(values, "disk", index, "present");
        let physical_present = flag(values, "physical", index, "present");
        if !disk_present && !physical_present {
            continue;
        }

        let percentage_used = number_u32(values, "reliability", index, "percentage_used");
        let remaining_life_percent =
            percentage_used.map(|used| 100u32.saturating_sub(used.min(100)));
        let media_errors =
            number_u64_allow_zero(values, "reliability", index, "read_errors_uncorrected").or_else(
                || number_u64_allow_zero(values, "reliability", index, "write_errors_uncorrected"),
            );

        drives.push(DriveReading {
            model: text(values, "physical", index, "model")
                .or_else(|| text(values, "disk", index, "model")),
            serial_number: text(values, "physical", index, "serial_number")
                .or_else(|| text(values, "disk", index, "serial_number")),
            firmware_revision: text(values, "physical", index, "firmware_revision")
                .or_else(|| text(values, "disk", index, "firmware_revision")),
            bus_type: text(values, "physical", index, "bus_type")
                .or_else(|| text(values, "disk", index, "interface_type")),
            media_kind: text(values, "physical", index, "media_kind")
                .or_else(|| text(values, "disk", index, "media_type")),
            capacity_bytes: number_u64(values, "physical", index, "capacity_bytes")
                .or_else(|| number_u64(values, "disk", index, "capacity_bytes")),
            rotational: bool_field(values, "physical", index, "rotational"),
            removable: flag(values, "physical", index, "removable")
                || flag(values, "disk", index, "removable"),
            power_on_hours: number_u64_allow_zero(values, "reliability", index, "power_on_hours")
                .or_else(|| number_u64_allow_zero(values, "smartdata", index, "power_on_hours")),
            power_cycles: number_u64_allow_zero(values, "reliability", index, "power_cycles")
                .or_else(|| number_u64_allow_zero(values, "smartdata", index, "power_cycles")),
            percentage_used,
            available_spare_percent: number_u32(values, "reliability", index, "available_spare"),
            temperature_c: number_u64_allow_zero(values, "reliability", index, "temperature_c")
                .or_else(|| number_u64_allow_zero(values, "smartdata", index, "temperature_c"))
                .map(|value| value as f64),
            media_errors,
            reallocated_sectors: number_u64_allow_zero(
                values,
                "smartdata",
                index,
                "reallocated_sectors",
            ),
            pending_sectors: number_u64_allow_zero(values, "smartdata", index, "pending_sectors"),
            critical_warning: None,
            predicts_failure: bool_field(values, "predict", index, "predicts_failure"),
            remaining_life_percent,
        });
    }

    // Predict-failure / SMART data can arrive on a different index than
    // PhysicalDisk. If we have exactly one drive, copy unmatched SMART onto it.
    if drives.len() == 1 {
        if drives[0].predicts_failure.is_none() {
            drives[0].predicts_failure = bool_field(values, "predict", 0, "predicts_failure");
        }
        if drives[0].reallocated_sectors.is_none() {
            drives[0].reallocated_sectors =
                number_u64_allow_zero(values, "smartdata", 0, "reallocated_sectors");
        }
        if drives[0].pending_sectors.is_none() {
            drives[0].pending_sectors =
                number_u64_allow_zero(values, "smartdata", 0, "pending_sectors");
        }
        if drives[0].power_on_hours.is_none() {
            drives[0].power_on_hours =
                number_u64_allow_zero(values, "smartdata", 0, "power_on_hours");
        }
    } else if drives.is_empty() {
        // SMART-only answer with no identity still has to be representable.
        if flag(values, "predict", 0, "present") || flag(values, "smartdata", 0, "present") {
            drives.push(DriveReading {
                model: None,
                serial_number: None,
                firmware_revision: None,
                bus_type: None,
                media_kind: None,
                capacity_bytes: None,
                rotational: None,
                removable: false,
                power_on_hours: number_u64_allow_zero(values, "smartdata", 0, "power_on_hours"),
                power_cycles: number_u64_allow_zero(values, "smartdata", 0, "power_cycles"),
                percentage_used: None,
                available_spare_percent: None,
                temperature_c: number_u64_allow_zero(values, "smartdata", 0, "temperature_c")
                    .map(|value| value as f64),
                media_errors: None,
                reallocated_sectors: number_u64_allow_zero(
                    values,
                    "smartdata",
                    0,
                    "reallocated_sectors",
                ),
                pending_sectors: number_u64_allow_zero(values, "smartdata", 0, "pending_sectors"),
                critical_warning: None,
                predicts_failure: bool_field(values, "predict", 0, "predicts_failure"),
                remaining_life_percent: None,
            });
        }
    }

    drives
}

fn text(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<String> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && !is_placeholder(value))
}

fn number_u64(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<u64> {
    number_u64_allow_zero(values, section, index, name).filter(|value| *value > 0)
}

fn number_u64_allow_zero(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<u64> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .and_then(|value| value.parse::<u64>().ok())
}

fn number_u32(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<u32> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .and_then(|value| value.parse::<u32>().ok())
}

fn flag(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> bool {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn bool_field(
    values: &BTreeMap<(String, usize, String), String>,
    section: &str,
    index: usize,
    name: &str,
) -> Option<bool> {
    values
        .get(&(section.to_string(), index, name.to_string()))
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .or_else(|| {
            values
                .get(&(section.to_string(), index, name.to_string()))
                .filter(|value| value.eq_ignore_ascii_case("false") || *value == "0")
                .map(|_| false)
        })
}

fn is_placeholder(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    compact.is_empty()
        || matches!(
            compact.as_str(),
            "unknown" | "none" | "na" | "notavailable" | "tobefilledbyoem" | "defaultstring"
        )
}

fn is_allowed(section: &str, name: &str) -> bool {
    matches!(
        section,
        "session" | "disk" | "physical" | "reliability" | "predict" | "smartdata"
    ) && matches!(
        name,
        "source_status"
            | "record_count"
            | "present"
            | "elevated"
            | "model"
            | "serial_number"
            | "firmware_revision"
            | "capacity_bytes"
            | "interface_type"
            | "media_type"
            | "media_kind"
            | "bus_type"
            | "status"
            | "health_status"
            | "rotational"
            | "removable"
            | "percentage_used"
            | "available_spare"
            | "temperature_c"
            | "power_on_hours"
            | "power_cycles"
            | "read_errors_uncorrected"
            | "write_errors_uncorrected"
            | "predicts_failure"
            | "reason"
            | "instance"
            | "reallocated_sectors"
            | "pending_sectors"
    )
}

fn decode_hex(encoded: &str) -> Option<String> {
    if !encoded.len().is_multiple_of(2) || encoded.len() / 2 > MAX_VALUE_BYTES {
        return None;
    }
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = hex_nibble(pair[0])?;
        let low = hex_nibble(pair[1])?;
        decoded.push((high << 4) | low);
    }
    let value = String::from_utf8(decoded).ok()?;
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

const fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    struct Fixture(String);

    impl Fixture {
        fn new() -> Self {
            Self(String::new())
        }

        fn line(mut self, section: &str, index: usize, name: &str, value: &str) -> Self {
            self.0
                .push_str(&format!("{section}\t{index}\t{name}\t{}\n", hex(value)));
            self
        }

        fn probe(&self) -> StorageProbe {
            parse_probe(&self.0)
        }
    }

    #[test]
    fn a_healthy_nvme_with_wear_is_scorable() {
        let probe = Fixture::new()
            .line("physical", 0, "source_status", "reported")
            .line("physical", 0, "present", "True")
            .line("physical", 0, "model", "Samsung SSD 990 PRO 1TB")
            .line("physical", 0, "serial_number", "S6Z1NS0W123456")
            .line("physical", 0, "bus_type", "NVMe")
            .line("physical", 0, "media_kind", "SSD")
            .line("physical", 0, "rotational", "False")
            .line("reliability", 0, "source_status", "reported")
            .line("reliability", 0, "present", "True")
            .line("reliability", 0, "percentage_used", "2")
            .line("reliability", 0, "available_spare", "100")
            .line("reliability", 0, "power_on_hours", "1840")
            .line("reliability", 0, "temperature_c", "38")
            .line("predict", 0, "source_status", "reported")
            .line("predict", 0, "present", "True")
            .line("predict", 0, "predicts_failure", "False")
            .probe();

        assert_eq!(probe.drives.len(), 1);
        assert_eq!(probe.drives[0].percentage_used, Some(2));
        assert_eq!(probe.drives[0].remaining_life_percent, Some(98));
        assert_eq!(probe.drives[0].predicts_failure, Some(false));
        assert_eq!(probe.awarded_points(), Some(20));
        assert!(probe.critical_faults().is_empty());
    }

    #[test]
    fn wear_zero_is_a_real_new_drive_not_missing_data() {
        let probe = Fixture::new()
            .line("physical", 0, "source_status", "reported")
            .line("physical", 0, "present", "True")
            .line("physical", 0, "model", "Kioxia NVMe")
            .line("physical", 0, "bus_type", "NVMe")
            .line("reliability", 0, "source_status", "reported")
            .line("reliability", 0, "present", "True")
            .line("reliability", 0, "percentage_used", "0")
            .line("reliability", 0, "available_spare", "100")
            .probe();

        assert_eq!(probe.drives[0].percentage_used, Some(0));
        assert_eq!(probe.awarded_points(), Some(20));
    }

    #[test]
    fn ata_pending_sectors_are_a_critical_fault() {
        let probe = Fixture::new()
            .line("disk", 0, "source_status", "reported")
            .line("disk", 0, "present", "True")
            .line("disk", 0, "model", "ST1000LM035")
            .line("smartdata", 0, "source_status", "reported")
            .line("smartdata", 0, "present", "True")
            .line("smartdata", 0, "reallocated_sectors", "0")
            .line("smartdata", 0, "pending_sectors", "4")
            .line("predict", 0, "source_status", "reported")
            .line("predict", 0, "present", "True")
            .line("predict", 0, "predicts_failure", "False")
            .probe();

        assert_eq!(probe.awarded_points(), Some(0));
        assert_eq!(
            probe.critical_faults(),
            vec![CriticalFault::PendingSectorsPresent]
        );
    }

    #[test]
    fn predict_failure_forces_zero_points() {
        let probe = Fixture::new()
            .line("physical", 0, "source_status", "reported")
            .line("physical", 0, "present", "True")
            .line("physical", 0, "model", "WDC HDD")
            .line("predict", 0, "source_status", "reported")
            .line("predict", 0, "present", "True")
            .line("predict", 0, "predicts_failure", "True")
            .probe();

        assert_eq!(probe.awarded_points(), Some(0));
        assert_eq!(
            probe.critical_faults(),
            vec![CriticalFault::StoragePredictsFailure]
        );
    }

    #[test]
    fn identity_without_smart_is_not_scorable() {
        let probe = Fixture::new()
            .line("disk", 0, "source_status", "reported")
            .line("disk", 0, "present", "True")
            .line("disk", 0, "model", "USB Disk")
            .line("reliability", 0, "source_status", "unsupported")
            .line("predict", 0, "source_status", "unsupported")
            .probe();

        assert_eq!(probe.drives[0].model.as_deref(), Some("USB Disk"));
        assert!(!probe.drives[0].is_scorable());
        assert_eq!(probe.awarded_points(), None);
    }

    #[test]
    fn a_refused_reliability_query_is_visible() {
        let probe = Fixture::new()
            .line("disk", 0, "source_status", "reported")
            .line("reliability", 0, "source_status", "permission_denied")
            .line("predict", 0, "source_status", "permission_denied")
            .probe();

        assert!(probe.reliability_refused());
        assert_eq!(
            probe.outcome_for(StorageSource::ReliabilityCounter),
            SourceOutcome::PermissionDenied
        );
        assert_eq!(probe.awarded_points(), None);
    }

    #[test]
    fn usb_stick_is_ignored_when_an_internal_nvme_is_scorable() {
        let probe = Fixture::new()
            .line("physical", 0, "source_status", "reported")
            .line("physical", 0, "present", "True")
            .line("physical", 0, "model", "Internal NVMe")
            .line("physical", 0, "bus_type", "NVMe")
            .line("reliability", 0, "source_status", "reported")
            .line("reliability", 0, "present", "True")
            .line("reliability", 0, "percentage_used", "3")
            .line("reliability", 0, "available_spare", "99")
            .line("physical", 1, "present", "True")
            .line("physical", 1, "model", "SanDisk USB")
            .line("physical", 1, "bus_type", "USB")
            .line("physical", 1, "removable", "True")
            .line("reliability", 1, "present", "True")
            .line("reliability", 1, "percentage_used", "90")
            .probe();

        assert_eq!(probe.scoring_drives().len(), 1);
        assert_eq!(
            probe.scoring_drives()[0].model.as_deref(),
            Some("Internal NVMe")
        );
        assert_eq!(probe.awarded_points(), Some(20));
    }

    #[test]
    fn unavailable_off_windows_claims_nothing() {
        let probe =
            StorageProbe::unavailable("Storage SMART collection is only available on Windows.");

        assert!(probe.drives.is_empty());
        assert_eq!(probe.awarded_points(), None);
        assert!(probe.critical_faults().is_empty());
    }

    #[test]
    fn the_script_never_names_a_destructive_command() {
        let script = STORAGE_SCRIPT.as_str();
        for forbidden in [
            "Secure Erase",
            "SECURE_ERASE",
            "Sanitize",
            "IOCTL_STORAGE_PROTOCOL_COMMAND",
            "Format-Volume",
            "Clear-Disk",
            "Initialize-Disk",
            "Optimize-Volume",
            "TRIM",
            "cipher.exe",
        ] {
            assert!(
                !script.contains(forbidden),
                "storage script named destructive token {forbidden}"
            );
        }
        assert!(script.contains("Get-StorageReliabilityCounter"));
        assert!(script.contains("MSStorageDriver_FailurePredictStatus"));
    }
}
