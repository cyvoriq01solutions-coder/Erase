//! Passive Windows hardware collection for the first `hardware_inventory_v1` slice.
//!
//! The collector executes one fixed, application-owned PowerShell/CIM snapshot through
//! the bounded runtime. Its output is a strict, ASCII-only protocol whose values are
//! UTF-8 encoded as hexadecimal. No caller-controlled command or script is accepted.

use crate::{
    CollectorError, CollectorErrorKind, CollectorName, CollectorResult,
    collector_runtime::{
        CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
    },
    hardware_inventory_v1::{
        CollectionSource, CollectionStatus, Confidence, DeviceAndChassisIdentity,
        DeviceClassification, DeviceIdentifier, FirmwareMode, FirmwareProfile, FormFactor,
        HardwareInventoryV1, InventoryField, InventorySection, MemoryInventory, MemoryModule,
        MemorySummary, PermissionState, PrivacyClass, ProcessorProfile, Provenance,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    str,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const COLLECTOR_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_STDOUT_BYTES: usize = 256 * 1024;
const MAX_STDERR_BYTES: usize = 64 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROTOCOL_LINES: usize = 2_048;
const MAX_VALUE_BYTES: usize = 4 * 1024;

const SYSTEM: &str = "system";
const PRODUCT: &str = "product";
const CHASSIS: &str = "chassis";
const BASEBOARD: &str = "baseboard";
const BIOS: &str = "bios";
const FIRMWARE: &str = "firmware";
const SECURE_BOOT: &str = "secure_boot";
const TPM: &str = "tpm";
const PROCESSOR: &str = "processor";
const OPERATING_SYSTEM: &str = "operating_system";
const MEMORY_ARRAY: &str = "memory_array";
const MEMORY_MODULE: &str = "memory_module";

const WINDOWS_HARDWARE_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
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

function Emit-QueryFailure([string]$section, $errorRecord) {
    $status = 'collection_error'
    $hresult = $errorRecord.Exception.HResult
    $nativeError = $errorRecord.Exception.NativeErrorCode
    if (($hresult -eq -2147024891) -or ($nativeError -eq 5)) {
        $status = 'permission_denied'
    }
    Emit-Value $section 0 'query_status' $status
}

try {
    $item = Get-CimInstance -ClassName Win32_ComputerSystem -Property Manufacturer,Model,SystemFamily,TotalPhysicalMemory,NumberOfProcessors,PCSystemType,HypervisorPresent -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'system' 0 'query_ok' $true
    if ($null -ne $item) {
        Emit-Value 'system' 0 'record_present' $true
        Emit-Value 'system' 0 'manufacturer' $item.Manufacturer
        Emit-Value 'system' 0 'model' $item.Model
        Emit-Value 'system' 0 'family' $item.SystemFamily
        Emit-Value 'system' 0 'total_physical_memory' $item.TotalPhysicalMemory
        Emit-Value 'system' 0 'number_of_processors' $item.NumberOfProcessors
        Emit-Value 'system' 0 'pc_system_type' $item.PCSystemType
        Emit-Value 'system' 0 'hypervisor_present' $item.HypervisorPresent
    }
} catch { Emit-QueryFailure 'system' $_ }

try {
    $item = Get-CimInstance -ClassName Win32_ComputerSystemProduct -Property Vendor,IdentifyingNumber,UUID -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'product' 0 'query_ok' $true
    if ($null -ne $item) {
        Emit-Value 'product' 0 'record_present' $true
        Emit-Value 'product' 0 'vendor' $item.Vendor
        Emit-Value 'product' 0 'identifying_number' $item.IdentifyingNumber
        Emit-Value 'product' 0 'uuid' $item.UUID
    }
} catch { Emit-QueryFailure 'product' $_ }

try {
    $item = Get-CimInstance -ClassName Win32_SystemEnclosure -Property Manufacturer,ChassisTypes,SerialNumber,SMBIOSAssetTag -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'chassis' 0 'query_ok' $true
    if ($null -ne $item) {
        Emit-Value 'chassis' 0 'record_present' $true
        Emit-Value 'chassis' 0 'manufacturer' $item.Manufacturer
        if ($null -ne $item.ChassisTypes) {
            Emit-Value 'chassis' 0 'chassis_types' (($item.ChassisTypes | ForEach-Object { [string]$_ }) -join ',')
        }
        Emit-Value 'chassis' 0 'serial_number' $item.SerialNumber
        Emit-Value 'chassis' 0 'asset_tag' $item.SMBIOSAssetTag
    }
} catch { Emit-QueryFailure 'chassis' $_ }

try {
    $item = Get-CimInstance -ClassName Win32_BaseBoard -Property Manufacturer,Product,Version,SerialNumber -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'baseboard' 0 'query_ok' $true
    if ($null -ne $item) {
        Emit-Value 'baseboard' 0 'record_present' $true
        Emit-Value 'baseboard' 0 'manufacturer' $item.Manufacturer
        Emit-Value 'baseboard' 0 'product' $item.Product
        Emit-Value 'baseboard' 0 'version' $item.Version
        Emit-Value 'baseboard' 0 'serial_number' $item.SerialNumber
    }
} catch { Emit-QueryFailure 'baseboard' $_ }

try {
    $item = Get-CimInstance -ClassName Win32_BIOS -Property Manufacturer,SMBIOSBIOSVersion,ReleaseDate,SMBIOSMajorVersion,SMBIOSMinorVersion,SerialNumber -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'bios' 0 'query_ok' $true
    if ($null -ne $item) {
        Emit-Value 'bios' 0 'record_present' $true
        Emit-Value 'bios' 0 'manufacturer' $item.Manufacturer
        Emit-Value 'bios' 0 'version' $item.SMBIOSBIOSVersion
        if ($item.ReleaseDate -is [DateTime]) {
            Emit-Value 'bios' 0 'release_date' $item.ReleaseDate.ToUniversalTime().ToString('yyyy-MM-dd', [Globalization.CultureInfo]::InvariantCulture)
        } else {
            Emit-Value 'bios' 0 'release_date' $item.ReleaseDate
        }
        Emit-Value 'bios' 0 'smbios_major' $item.SMBIOSMajorVersion
        Emit-Value 'bios' 0 'smbios_minor' $item.SMBIOSMinorVersion
        Emit-Value 'bios' 0 'serial_number' $item.SerialNumber
    }
} catch { Emit-QueryFailure 'bios' $_ }

try {
    $item = Get-ComputerInfo -Property BiosFirmwareType -ErrorAction Stop
    Emit-Value 'firmware' 0 'query_ok' $true
    if (($null -ne $item) -and ($null -ne $item.BiosFirmwareType)) {
        Emit-Value 'firmware' 0 'record_present' $true
        Emit-Value 'firmware' 0 'mode' $item.BiosFirmwareType
    }
} catch { Emit-QueryFailure 'firmware' $_ }

try {
    $enabled = Confirm-SecureBootUEFI -ErrorAction Stop
    Emit-Value 'secure_boot' 0 'query_ok' $true
    Emit-Value 'secure_boot' 0 'record_present' $true
    Emit-Value 'secure_boot' 0 'enabled' $enabled
} catch { Emit-QueryFailure 'secure_boot' $_ }

try {
    $items = @(Get-CimInstance -Namespace 'root/CIMV2/Security/MicrosoftTpm' -ClassName Win32_Tpm -Property SpecVersion -ErrorAction Stop)
    Emit-Value 'tpm' 0 'query_ok' $true
    Emit-Value 'tpm' 0 'record_present' $true
    Emit-Value 'tpm' 0 'present' ($items.Count -gt 0)
    if ($items.Count -gt 0) {
        Emit-Value 'tpm' 0 'spec_version' $items[0].SpecVersion
    }
} catch { Emit-QueryFailure 'tpm' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_Processor -Property Manufacturer,Name,Architecture,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed,CurrentClockSpeed,AddressWidth,VMMonitorModeExtensions,VirtualizationFirmwareEnabled -ErrorAction Stop)
    Emit-Value 'processor' 0 'query_ok' $true
    for ($index = 0; $index -lt $items.Count; $index++) {
        $item = $items[$index]
        Emit-Value 'processor' $index 'record_present' $true
        Emit-Value 'processor' $index 'manufacturer' $item.Manufacturer
        Emit-Value 'processor' $index 'model' $item.Name
        Emit-Value 'processor' $index 'architecture' $item.Architecture
        Emit-Value 'processor' $index 'cores' $item.NumberOfCores
        Emit-Value 'processor' $index 'logical_processors' $item.NumberOfLogicalProcessors
        Emit-Value 'processor' $index 'maximum_clock_mhz' $item.MaxClockSpeed
        Emit-Value 'processor' $index 'current_clock_mhz' $item.CurrentClockSpeed
        Emit-Value 'processor' $index 'address_width_bits' $item.AddressWidth
        Emit-Value 'processor' $index 'vm_monitor_extensions' $item.VMMonitorModeExtensions
        Emit-Value 'processor' $index 'virtualization_firmware_enabled' $item.VirtualizationFirmwareEnabled
    }
} catch { Emit-QueryFailure 'processor' $_ }

try {
    $item = Get-CimInstance -ClassName Win32_OperatingSystem -Property TotalVisibleMemorySize -ErrorAction Stop | Select-Object -First 1
    Emit-Value 'operating_system' 0 'query_ok' $true
    if ($null -ne $item) {
        Emit-Value 'operating_system' 0 'record_present' $true
        Emit-Value 'operating_system' 0 'visible_memory_kib' $item.TotalVisibleMemorySize
    }
} catch { Emit-QueryFailure 'operating_system' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_PhysicalMemoryArray -Property MemoryDevices,MemoryErrorCorrection -ErrorAction Stop)
    Emit-Value 'memory_array' 0 'query_ok' $true
    for ($index = 0; $index -lt $items.Count; $index++) {
        $item = $items[$index]
        Emit-Value 'memory_array' $index 'record_present' $true
        Emit-Value 'memory_array' $index 'slot_count' $item.MemoryDevices
        Emit-Value 'memory_array' $index 'error_correction' $item.MemoryErrorCorrection
    }
} catch { Emit-QueryFailure 'memory_array' $_ }

try {
    $items = @(Get-CimInstance -ClassName Win32_PhysicalMemory -Property DeviceLocator,BankLabel,Capacity,Speed,ConfiguredClockSpeed,SMBIOSMemoryType,FormFactor,Manufacturer,PartNumber,SerialNumber -ErrorAction Stop)
    Emit-Value 'memory_module' 0 'query_ok' $true
    for ($index = 0; $index -lt $items.Count; $index++) {
        $item = $items[$index]
        Emit-Value 'memory_module' $index 'record_present' $true
        if (-not [string]::IsNullOrWhiteSpace([string]$item.DeviceLocator)) {
            Emit-Value 'memory_module' $index 'locator' $item.DeviceLocator
        } else {
            Emit-Value 'memory_module' $index 'locator' $item.BankLabel
        }
        Emit-Value 'memory_module' $index 'capacity_bytes' $item.Capacity
        Emit-Value 'memory_module' $index 'speed_mhz' $item.Speed
        Emit-Value 'memory_module' $index 'configured_speed_mhz' $item.ConfiguredClockSpeed
        Emit-Value 'memory_module' $index 'memory_type' $item.SMBIOSMemoryType
        Emit-Value 'memory_module' $index 'form_factor' $item.FormFactor
        Emit-Value 'memory_module' $index 'manufacturer' $item.Manufacturer
        Emit-Value 'memory_module' $index 'part_number' $item.PartNumber
        Emit-Value 'memory_module' $index 'serial_number' $item.SerialNumber
    }
} catch { Emit-QueryFailure 'memory_module' $_ }
"#,
);

/// Collect the first passive Windows hardware slice and preserve a complete inventory
/// object even when the bounded command fails. The requested sections carry an honest
/// failure status; later/deferred sections remain `not_reported`.
#[must_use]
pub fn collect(cancellation: &CancellationToken) -> HardwareInventoryV1 {
    let collected_at_unix = current_unix_timestamp().unwrap_or(0);

    match try_collect_at(collected_at_unix, cancellation) {
        Ok(inventory) => inventory,
        Err(error) => failed_inventory(collected_at_unix, error.kind),
    }
}

/// Typed variant for GUI/headless callers that need to distinguish cancellation,
/// timeout, output-limit, command, and parse failures.
pub fn try_collect(cancellation: &CancellationToken) -> CollectorResult<HardwareInventoryV1> {
    let collected_at_unix = current_unix_timestamp()?;
    try_collect_at(collected_at_unix, cancellation)
}

fn try_collect_at(
    collected_at_unix: u64,
    cancellation: &CancellationToken,
) -> CollectorResult<HardwareInventoryV1> {
    let limits = CollectorLimits::new(
        COLLECTOR_TIMEOUT,
        MAX_STDOUT_BYTES,
        MAX_STDERR_BYTES,
        POLL_INTERVAL,
    );
    let output = run_fixed_powershell(
        CollectorName::HardwareInventory,
        WINDOWS_HARDWARE_SCRIPT,
        limits,
        cancellation,
    )?;
    let snapshot = parse_snapshot(output.stdout())?;
    Ok(build_inventory(&snapshot, collected_at_unix))
}

fn current_unix_timestamp() -> CollectorResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| {
            typed_collector_error(
                CollectorErrorKind::Internal,
                "The system clock could not produce a collection timestamp.",
            )
        })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProtocolKey {
    section: String,
    index: usize,
    name: String,
}

/// Deliberately has no `Debug` implementation: raw values may include local-only
/// device identifiers and must never be emitted by routine diagnostics.
#[derive(Default)]
struct RawSnapshot {
    values: BTreeMap<ProtocolKey, String>,
}

impl RawSnapshot {
    fn value(&self, section: &str, index: usize, name: &str) -> Option<&str> {
        self.values
            .get(&ProtocolKey {
                section: section.to_string(),
                index,
                name: name.to_string(),
            })
            .map(String::as_str)
    }

    fn query_state(&self, section: &str) -> QueryState {
        match self.value(section, 0, "query_status") {
            Some("permission_denied") => QueryState::PermissionDenied,
            Some("unsupported") => QueryState::Unsupported,
            Some("collection_error") => QueryState::CollectionError,
            Some(_) => QueryState::CollectionError,
            None if self.value(section, 0, "query_ok") == Some("True") => QueryState::Available,
            None => QueryState::NotReported,
        }
    }

    fn record_indices(&self, section: &str) -> BTreeSet<usize> {
        self.values
            .iter()
            .filter(|(key, value)| {
                key.section == section && key.name == "record_present" && value.as_str() == "True"
            })
            .map(|(key, _)| key.index)
            .collect()
    }

    fn has_record(&self, section: &str) -> bool {
        !self.record_indices(section).is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryState {
    Available,
    NotReported,
    PermissionDenied,
    Unsupported,
    CollectionError,
}

impl QueryState {
    const fn status(self) -> CollectionStatus {
        match self {
            Self::Available | Self::NotReported => CollectionStatus::NotReported,
            Self::PermissionDenied => CollectionStatus::PermissionDenied,
            Self::Unsupported => CollectionStatus::Unsupported,
            Self::CollectionError => CollectionStatus::CollectionError,
        }
    }

    const fn permission(self) -> PermissionState {
        match self {
            Self::PermissionDenied => PermissionState::Denied,
            Self::Available => PermissionState::NotRequired,
            Self::NotReported | Self::Unsupported | Self::CollectionError => {
                PermissionState::Unknown
            }
        }
    }
}

fn parse_snapshot(output: &[u8]) -> CollectorResult<RawSnapshot> {
    let text = str::from_utf8(output)
        .map_err(|_| collector_error("The hardware collector output was not valid UTF-8."))?;
    let mut snapshot = RawSnapshot::default();

    for (line_index, line) in text.lines().enumerate() {
        if line_index >= MAX_PROTOCOL_LINES {
            return Err(collector_error(
                "The hardware collector returned too many protocol records.",
            ));
        }
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let section = parts.next().unwrap_or_default();
        let index_text = parts.next().unwrap_or_default();
        let name = parts.next().unwrap_or_default();
        let encoded_value = parts.next().unwrap_or_default();

        if parts.next().is_some() || !is_allowed_field(section, name) || encoded_value.is_empty() {
            return Err(collector_error(
                "The hardware collector returned an invalid protocol record.",
            ));
        }

        let index = index_text.parse::<usize>().map_err(|_| {
            collector_error("The hardware collector returned an invalid record index.")
        })?;
        if !is_allowed_index(section, index)
            || (matches!(name, "query_ok" | "query_status") && index != 0)
        {
            return Err(collector_error(
                "The hardware collector record index exceeded its limit.",
            ));
        }

        let value = decode_hex_value(encoded_value)?;
        if !is_valid_protocol_marker(name, &value) {
            return Err(collector_error(
                "The hardware collector returned an invalid protocol marker.",
            ));
        }
        let key = ProtocolKey {
            section: section.to_string(),
            index,
            name: name.to_string(),
        };
        if snapshot.values.insert(key, value).is_some() {
            return Err(collector_error(
                "The hardware collector returned a duplicate protocol field.",
            ));
        }
    }

    for section in [
        SYSTEM,
        PRODUCT,
        CHASSIS,
        BASEBOARD,
        BIOS,
        FIRMWARE,
        SECURE_BOOT,
        TPM,
        PROCESSOR,
        OPERATING_SYSTEM,
        MEMORY_ARRAY,
        MEMORY_MODULE,
    ] {
        if snapshot.value(section, 0, "query_ok").is_none()
            && snapshot.value(section, 0, "query_status").is_none()
        {
            return Err(collector_error(
                "The hardware collector returned an incomplete protocol snapshot.",
            ));
        }
    }

    Ok(snapshot)
}

fn is_valid_protocol_marker(name: &str, value: &str) -> bool {
    match name {
        "query_ok" | "record_present" => value == "True",
        "query_status" => matches!(
            value,
            "permission_denied" | "unsupported" | "collection_error"
        ),
        _ => true,
    }
}

fn decode_hex_value(encoded: &str) -> CollectorResult<String> {
    if !encoded.len().is_multiple_of(2) || encoded.len() / 2 > MAX_VALUE_BYTES {
        return Err(collector_error(
            "The hardware collector returned an invalid encoded value.",
        ));
    }

    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        let high = decode_hex_nibble(pair[0]).ok_or_else(|| {
            collector_error("The hardware collector returned non-hexadecimal data.")
        })?;
        let low = decode_hex_nibble(pair[1]).ok_or_else(|| {
            collector_error("The hardware collector returned non-hexadecimal data.")
        })?;
        decoded.push((high << 4) | low);
    }

    let value = String::from_utf8(decoded)
        .map_err(|_| collector_error("The hardware collector returned invalid encoded UTF-8."))?;
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(collector_error(
            "The hardware collector returned an empty encoded value.",
        ));
    }
    Ok(normalized)
}

const fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_allowed_field(section: &str, name: &str) -> bool {
    if matches!(name, "query_ok" | "query_status" | "record_present") {
        return is_known_section(section);
    }

    matches!(
        (section, name),
        (
            SYSTEM,
            "manufacturer"
                | "model"
                | "family"
                | "total_physical_memory"
                | "number_of_processors"
                | "pc_system_type"
                | "hypervisor_present"
        ) | (PRODUCT, "vendor" | "identifying_number" | "uuid")
            | (
                CHASSIS,
                "manufacturer" | "chassis_types" | "serial_number" | "asset_tag"
            )
            | (
                BASEBOARD,
                "manufacturer" | "product" | "version" | "serial_number"
            )
            | (
                BIOS,
                "manufacturer"
                    | "version"
                    | "release_date"
                    | "smbios_major"
                    | "smbios_minor"
                    | "serial_number"
            )
            | (FIRMWARE, "mode")
            | (SECURE_BOOT, "enabled")
            | (TPM, "present" | "spec_version")
            | (
                PROCESSOR,
                "manufacturer"
                    | "model"
                    | "architecture"
                    | "cores"
                    | "logical_processors"
                    | "maximum_clock_mhz"
                    | "current_clock_mhz"
                    | "address_width_bits"
                    | "vm_monitor_extensions"
                    | "virtualization_firmware_enabled"
            )
            | (OPERATING_SYSTEM, "visible_memory_kib")
            | (MEMORY_ARRAY, "slot_count" | "error_correction")
            | (
                MEMORY_MODULE,
                "locator"
                    | "capacity_bytes"
                    | "speed_mhz"
                    | "configured_speed_mhz"
                    | "memory_type"
                    | "form_factor"
                    | "manufacturer"
                    | "part_number"
                    | "serial_number"
            )
    )
}

fn is_known_section(section: &str) -> bool {
    matches!(
        section,
        SYSTEM
            | PRODUCT
            | CHASSIS
            | BASEBOARD
            | BIOS
            | FIRMWARE
            | SECURE_BOOT
            | TPM
            | PROCESSOR
            | OPERATING_SYSTEM
            | MEMORY_ARRAY
            | MEMORY_MODULE
    )
}

fn is_allowed_index(section: &str, index: usize) -> bool {
    match section {
        PROCESSOR => index < 64,
        MEMORY_ARRAY => index < 64,
        MEMORY_MODULE => index < 256,
        _ => index == 0,
    }
}

struct SourceRecord<'a> {
    snapshot: &'a RawSnapshot,
    section: &'static str,
    index: usize,
    source_detail: &'static str,
    collected_at_unix: u64,
}

impl<'a> SourceRecord<'a> {
    const fn new(
        snapshot: &'a RawSnapshot,
        section: &'static str,
        index: usize,
        source_detail: &'static str,
        collected_at_unix: u64,
    ) -> Self {
        Self {
            snapshot,
            section,
            index,
            source_detail,
            collected_at_unix,
        }
    }

    fn text(&self, name: &str, privacy: PrivacyClass) -> InventoryField<String> {
        match self.snapshot.value(self.section, self.index, name) {
            Some(value) if is_meaningful_text(value) => InventoryField::reported(
                value.to_string(),
                Confidence::High,
                privacy,
                self.provenance(),
            ),
            Some(_) => InventoryField::unknown(privacy, self.provenance()),
            None => self.unavailable(privacy),
        }
    }

    fn identifier(&self, name: &str) -> InventoryField<DeviceIdentifier> {
        match self.snapshot.value(self.section, self.index, name) {
            Some(value) if is_meaningful_identifier(value) => {
                match DeviceIdentifier::from_reported(value) {
                    Some(identifier) => InventoryField::reported(
                        identifier,
                        Confidence::High,
                        PrivacyClass::DeviceIdentifier,
                        self.provenance(),
                    ),
                    None => {
                        InventoryField::unknown(PrivacyClass::DeviceIdentifier, self.provenance())
                    }
                }
            }
            Some(_) => InventoryField::unknown(PrivacyClass::DeviceIdentifier, self.provenance()),
            None => self.unavailable(PrivacyClass::DeviceIdentifier),
        }
    }

    fn positive_u32(&self, name: &str) -> InventoryField<u32> {
        self.positive_number(name)
    }

    fn positive_u64(&self, name: &str) -> InventoryField<u64> {
        self.positive_number(name)
    }

    fn positive_number<T>(&self, name: &str) -> InventoryField<T>
    where
        T: str::FromStr + PartialEq + From<u8>,
    {
        match self.snapshot.value(self.section, self.index, name) {
            Some(value) => match value.parse::<T>() {
                Ok(number) if number != T::from(0) => InventoryField::reported(
                    number,
                    Confidence::High,
                    PrivacyClass::OperationalMetadata,
                    self.provenance(),
                ),
                Ok(_) | Err(_) => {
                    InventoryField::unknown(PrivacyClass::OperationalMetadata, self.provenance())
                }
            },
            None => self.unavailable(PrivacyClass::OperationalMetadata),
        }
    }

    fn boolean(&self, name: &str) -> InventoryField<bool> {
        match self.snapshot.value(self.section, self.index, name) {
            Some(value) => match parse_bool(value) {
                Some(boolean) => InventoryField::reported(
                    boolean,
                    Confidence::High,
                    PrivacyClass::OperationalMetadata,
                    self.provenance(),
                ),
                None => {
                    InventoryField::unknown(PrivacyClass::OperationalMetadata, self.provenance())
                }
            },
            None => self.unavailable(PrivacyClass::OperationalMetadata),
        }
    }

    fn unavailable<T>(&self, privacy: PrivacyClass) -> InventoryField<T> {
        let state = self.snapshot.query_state(self.section);
        InventoryField {
            value: None,
            status: state.status(),
            confidence: Confidence::Unknown,
            privacy_class: privacy,
            provenance: self.provenance_for_state(state),
        }
    }

    fn provenance(&self) -> Provenance {
        source_provenance(
            self.source_detail,
            self.collected_at_unix,
            PermissionState::NotRequired,
        )
    }

    fn provenance_for_state(&self, state: QueryState) -> Provenance {
        source_provenance(
            self.source_detail,
            self.collected_at_unix,
            state.permission(),
        )
    }
}

fn build_inventory(snapshot: &RawSnapshot, collected_at_unix: u64) -> HardwareInventoryV1 {
    let mut inventory = HardwareInventoryV1::not_collected(collected_at_unix);
    inventory.device_and_chassis = build_device_section(snapshot, collected_at_unix);
    inventory.firmware = build_firmware_section(snapshot, collected_at_unix);
    inventory.processors = build_processor_section(snapshot, collected_at_unix);
    inventory.memory = build_memory_inventory(snapshot, collected_at_unix);
    inventory
}

fn build_device_section(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventorySection<DeviceAndChassisIdentity> {
    let sources = [SYSTEM, PRODUCT, CHASSIS, BASEBOARD, BIOS];
    if !sources.iter().any(|section| snapshot.has_record(section)) {
        return empty_section(
            combined_query_state(snapshot, &sources),
            "device and chassis CIM sources",
            collected_at_unix,
        );
    }

    let system = SourceRecord::new(
        snapshot,
        SYSTEM,
        0,
        "Win32_ComputerSystem",
        collected_at_unix,
    );
    let product = SourceRecord::new(
        snapshot,
        PRODUCT,
        0,
        "Win32_ComputerSystemProduct",
        collected_at_unix,
    );
    let chassis = SourceRecord::new(
        snapshot,
        CHASSIS,
        0,
        "Win32_SystemEnclosure",
        collected_at_unix,
    );
    let baseboard = SourceRecord::new(snapshot, BASEBOARD, 0, "Win32_BaseBoard", collected_at_unix);
    let bios = SourceRecord::new(snapshot, BIOS, 0, "Win32_BIOS", collected_at_unix);

    let product_serial = product.identifier("identifying_number");
    let serial_number = if product_serial.value.is_some() {
        product_serial
    } else {
        bios.identifier("serial_number")
    };
    let classification = derive_classification(snapshot, collected_at_unix);
    let form_factor = derive_form_factor(snapshot, classification.value, collected_at_unix);

    InventorySection {
        status: CollectionStatus::Reported,
        provenance: source_provenance(
            "Win32 system, product, enclosure, baseboard, and BIOS",
            collected_at_unix,
            PermissionState::NotRequired,
        ),
        records: vec![DeviceAndChassisIdentity {
            system_manufacturer: system.text("manufacturer", PrivacyClass::NonSensitive),
            system_model: system.text("model", PrivacyClass::NonSensitive),
            system_family: system.text("family", PrivacyClass::NonSensitive),
            serial_number,
            system_uuid: product.identifier("uuid"),
            chassis_manufacturer: chassis.text("manufacturer", PrivacyClass::NonSensitive),
            chassis_type: chassis.text("chassis_types", PrivacyClass::OperationalMetadata),
            chassis_serial_number: chassis.identifier("serial_number"),
            baseboard_manufacturer: baseboard.text("manufacturer", PrivacyClass::NonSensitive),
            baseboard_product: baseboard.text("product", PrivacyClass::NonSensitive),
            baseboard_version: baseboard.text("version", PrivacyClass::OperationalMetadata),
            baseboard_serial_number: baseboard.identifier("serial_number"),
            asset_tag: chassis.identifier("asset_tag"),
            form_factor,
            classification,
        }],
    }
}

fn build_firmware_section(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventorySection<FirmwareProfile> {
    let sources = [BIOS, FIRMWARE, SECURE_BOOT, TPM, SYSTEM];
    if !sources.iter().any(|section| snapshot.has_record(section)) {
        return empty_section(
            combined_query_state(snapshot, &sources),
            "firmware and security-hardware sources",
            collected_at_unix,
        );
    }

    let bios = SourceRecord::new(snapshot, BIOS, 0, "Win32_BIOS", collected_at_unix);
    let firmware = SourceRecord::new(
        snapshot,
        FIRMWARE,
        0,
        "GetFirmwareType via Get-ComputerInfo",
        collected_at_unix,
    );
    let secure_boot = SourceRecord::new(
        snapshot,
        SECURE_BOOT,
        0,
        "Confirm-SecureBootUEFI",
        collected_at_unix,
    );
    let tpm = SourceRecord::new(snapshot, TPM, 0, "Win32_Tpm", collected_at_unix);
    let mode = firmware_mode_field(&firmware);
    let (secure_boot_present, secure_boot_enabled) =
        secure_boot_fields(&secure_boot, mode.value, collected_at_unix);

    InventorySection {
        status: CollectionStatus::Reported,
        provenance: source_provenance(
            "BIOS, firmware type, Secure Boot, and TPM sources",
            collected_at_unix,
            PermissionState::NotRequired,
        ),
        records: vec![FirmwareProfile {
            vendor: bios.text("manufacturer", PrivacyClass::NonSensitive),
            version: bios.text("version", PrivacyClass::OperationalMetadata),
            release_date: bios.text("release_date", PrivacyClass::OperationalMetadata),
            smbios_version: smbios_version_field(&bios),
            mode,
            secure_boot_present,
            secure_boot_enabled,
            tpm_present: tpm.boolean("present"),
            tpm_specification_version: tpm.text("spec_version", PrivacyClass::OperationalMetadata),
            virtualization_indicator: derive_virtualization_indicator(snapshot, collected_at_unix),
        }],
    }
}

fn build_processor_section(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventorySection<ProcessorProfile> {
    let indices = snapshot.record_indices(PROCESSOR);
    if indices.is_empty() {
        return empty_section(
            snapshot.query_state(PROCESSOR),
            "Win32_Processor",
            collected_at_unix,
        );
    }
    let package_count = u32::try_from(indices.len()).ok();

    let records = indices
        .into_iter()
        .map(|index| {
            let source = SourceRecord::new(
                snapshot,
                PROCESSOR,
                index,
                "Win32_Processor",
                collected_at_unix,
            );
            ProcessorProfile {
                manufacturer: source.text("manufacturer", PrivacyClass::NonSensitive),
                model: source.text("model", PrivacyClass::NonSensitive),
                architecture: processor_architecture_field(&source),
                physical_package_count: derived_field(
                    package_count,
                    Confidence::High,
                    PrivacyClass::OperationalMetadata,
                    "number of Win32_Processor records",
                    collected_at_unix,
                ),
                physical_core_count: source.positive_u32("cores"),
                logical_processor_count: source.positive_u32("logical_processors"),
                maximum_clock_mhz: source.positive_u32("maximum_clock_mhz"),
                current_clock_mhz: source.positive_u32("current_clock_mhz"),
                address_width_bits: source.positive_u32("address_width_bits"),
                virtualization_capable: processor_virtualization_field(&source),
            }
        })
        .collect();

    InventorySection {
        status: CollectionStatus::Reported,
        provenance: source_provenance(
            "Win32_Processor",
            collected_at_unix,
            PermissionState::NotRequired,
        ),
        records,
    }
}

fn build_memory_inventory(snapshot: &RawSnapshot, collected_at_unix: u64) -> MemoryInventory {
    let summary_sources = [SYSTEM, OPERATING_SYSTEM, MEMORY_ARRAY, MEMORY_MODULE];
    let has_summary_source = summary_sources
        .iter()
        .any(|section| snapshot.has_record(section));

    let summary = if has_summary_source {
        let system = SourceRecord::new(
            snapshot,
            SYSTEM,
            0,
            "Win32_ComputerSystem",
            collected_at_unix,
        );
        let operating_system = SourceRecord::new(
            snapshot,
            OPERATING_SYSTEM,
            0,
            "Win32_OperatingSystem",
            collected_at_unix,
        );
        InventorySection {
            status: CollectionStatus::Reported,
            provenance: source_provenance(
                "computer system, operating system, and SMBIOS memory sources",
                collected_at_unix,
                PermissionState::NotRequired,
            ),
            records: vec![MemorySummary {
                installed_physical_bytes: system.positive_u64("total_physical_memory"),
                visible_physical_bytes: visible_memory_field(&operating_system),
                physical_slot_count: physical_slot_count_field(snapshot, collected_at_unix),
                populated_slot_count: populated_slot_count_field(snapshot, collected_at_unix),
                error_correction_capability: error_correction_field(snapshot, collected_at_unix),
            }],
        }
    } else {
        empty_section(
            combined_query_state(snapshot, &summary_sources),
            "memory summary sources",
            collected_at_unix,
        )
    };

    let module_indices = snapshot.record_indices(MEMORY_MODULE);
    let modules = if module_indices.is_empty() {
        empty_section(
            snapshot.query_state(MEMORY_MODULE),
            "Win32_PhysicalMemory",
            collected_at_unix,
        )
    } else {
        let records = module_indices
            .into_iter()
            .map(|index| build_memory_module(snapshot, index, collected_at_unix))
            .collect();
        InventorySection {
            status: CollectionStatus::Reported,
            provenance: source_provenance(
                "Win32_PhysicalMemory",
                collected_at_unix,
                PermissionState::NotRequired,
            ),
            records,
        }
    };

    MemoryInventory { summary, modules }
}

fn build_memory_module(
    snapshot: &RawSnapshot,
    index: usize,
    collected_at_unix: u64,
) -> MemoryModule {
    let source = SourceRecord::new(
        snapshot,
        MEMORY_MODULE,
        index,
        "Win32_PhysicalMemory",
        collected_at_unix,
    );
    MemoryModule {
        locator: source.text("locator", PrivacyClass::OperationalMetadata),
        capacity_bytes: source.positive_u64("capacity_bytes"),
        speed_mhz: source.positive_u32("speed_mhz"),
        configured_speed_mhz: source.positive_u32("configured_speed_mhz"),
        memory_type: memory_type_field(&source),
        form_factor: memory_form_factor_field(&source),
        manufacturer: source.text("manufacturer", PrivacyClass::NonSensitive),
        part_number: source.text("part_number", PrivacyClass::OperationalMetadata),
        serial_number: source.identifier("serial_number"),
    }
}

fn firmware_mode_field(source: &SourceRecord<'_>) -> InventoryField<FirmwareMode> {
    let value = source
        .snapshot
        .value(source.section, source.index, "mode")
        .and_then(|value| match value.to_ascii_lowercase().as_str() {
            "uefi" => Some(FirmwareMode::Uefi),
            "bios" => Some(FirmwareMode::LegacyBios),
            _ => None,
        });

    match value {
        Some(mode) => InventoryField::reported(
            mode,
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            source.provenance(),
        ),
        None if source
            .snapshot
            .value(source.section, source.index, "mode")
            .is_some() =>
        {
            InventoryField::unknown(PrivacyClass::OperationalMetadata, source.provenance())
        }
        None => source.unavailable(PrivacyClass::OperationalMetadata),
    }
}

fn secure_boot_fields(
    source: &SourceRecord<'_>,
    firmware_mode: Option<FirmwareMode>,
    collected_at_unix: u64,
) -> (InventoryField<bool>, InventoryField<bool>) {
    if firmware_mode == Some(FirmwareMode::LegacyBios) {
        return (
            InventoryField::derived(
                false,
                Confidence::High,
                PrivacyClass::OperationalMetadata,
                derived_provenance("legacy BIOS mode", collected_at_unix),
            ),
            InventoryField::not_applicable(
                PrivacyClass::OperationalMetadata,
                derived_provenance("legacy BIOS mode", collected_at_unix),
            ),
        );
    }

    let enabled = source.boolean("enabled");
    if enabled.value.is_some() {
        (
            InventoryField::observed(
                true,
                Confidence::High,
                PrivacyClass::OperationalMetadata,
                source.provenance(),
            ),
            enabled,
        )
    } else {
        let state = source.snapshot.query_state(source.section);
        (
            missing_field(
                state.status(),
                PrivacyClass::OperationalMetadata,
                source.provenance_for_state(state),
            ),
            enabled,
        )
    }
}

fn smbios_version_field(source: &SourceRecord<'_>) -> InventoryField<String> {
    let major = source
        .snapshot
        .value(source.section, source.index, "smbios_major")
        .and_then(|value| value.parse::<u16>().ok());
    let minor = source
        .snapshot
        .value(source.section, source.index, "smbios_minor")
        .and_then(|value| value.parse::<u16>().ok());

    match (major, minor) {
        (Some(major), Some(minor)) if major > 0 => InventoryField::reported(
            format!("{major}.{minor}"),
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            source.provenance(),
        ),
        _ if source
            .snapshot
            .value(source.section, source.index, "smbios_major")
            .is_some()
            || source
                .snapshot
                .value(source.section, source.index, "smbios_minor")
                .is_some() =>
        {
            InventoryField::unknown(PrivacyClass::OperationalMetadata, source.provenance())
        }
        _ => source.unavailable(PrivacyClass::OperationalMetadata),
    }
}

fn processor_architecture_field(source: &SourceRecord<'_>) -> InventoryField<String> {
    let raw = source
        .snapshot
        .value(source.section, source.index, "architecture");
    let architecture =
        raw.and_then(|value| value.parse::<u16>().ok())
            .and_then(|code| match code {
                0 => Some("x86"),
                1 => Some("MIPS"),
                2 => Some("Alpha"),
                3 => Some("PowerPC"),
                5 => Some("ARM"),
                6 => Some("Itanium"),
                9 => Some("x64"),
                12 => Some("ARM64"),
                _ => None,
            });

    match architecture {
        Some(value) => InventoryField::derived(
            value.to_string(),
            Confidence::High,
            PrivacyClass::OperationalMetadata,
            derived_provenance("Win32_Processor.Architecture", source.collected_at_unix),
        ),
        None if raw.is_some() => {
            InventoryField::unknown(PrivacyClass::OperationalMetadata, source.provenance())
        }
        None => source.unavailable(PrivacyClass::OperationalMetadata),
    }
}

fn processor_virtualization_field(source: &SourceRecord<'_>) -> InventoryField<bool> {
    let monitor_extensions = source.boolean("vm_monitor_extensions");
    if monitor_extensions.value.is_some() {
        return monitor_extensions;
    }
    source.boolean("virtualization_firmware_enabled")
}

fn visible_memory_field(source: &SourceRecord<'_>) -> InventoryField<u64> {
    match source
        .snapshot
        .value(source.section, source.index, "visible_memory_kib")
    {
        Some(value) => match value
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .and_then(|value| value.checked_mul(1024))
        {
            Some(bytes) => InventoryField::derived(
                bytes,
                Confidence::High,
                PrivacyClass::OperationalMetadata,
                derived_provenance(
                    "Win32_OperatingSystem.TotalVisibleMemorySize KiB to bytes",
                    source.collected_at_unix,
                ),
            ),
            None => InventoryField::unknown(PrivacyClass::OperationalMetadata, source.provenance()),
        },
        None => source.unavailable(PrivacyClass::OperationalMetadata),
    }
}

fn physical_slot_count_field(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventoryField<u32> {
    let indices = snapshot.record_indices(MEMORY_ARRAY);
    if indices.is_empty() {
        let source = SourceRecord::new(
            snapshot,
            MEMORY_ARRAY,
            0,
            "Win32_PhysicalMemoryArray",
            collected_at_unix,
        );
        return source.unavailable(PrivacyClass::OperationalMetadata);
    }

    let mut total = 0_u32;
    for index in indices {
        let Some(value) = snapshot
            .value(MEMORY_ARRAY, index, "slot_count")
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value > 0)
        else {
            return InventoryField::unknown(
                PrivacyClass::OperationalMetadata,
                source_provenance(
                    "Win32_PhysicalMemoryArray.MemoryDevices",
                    collected_at_unix,
                    PermissionState::NotRequired,
                ),
            );
        };
        let Some(next) = total.checked_add(value) else {
            return InventoryField::unknown(
                PrivacyClass::OperationalMetadata,
                source_provenance(
                    "Win32_PhysicalMemoryArray.MemoryDevices",
                    collected_at_unix,
                    PermissionState::NotRequired,
                ),
            );
        };
        total = next;
    }

    InventoryField::derived(
        total,
        Confidence::High,
        PrivacyClass::OperationalMetadata,
        derived_provenance(
            "sum of Win32_PhysicalMemoryArray.MemoryDevices",
            collected_at_unix,
        ),
    )
}

fn populated_slot_count_field(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventoryField<u32> {
    let indices = snapshot.record_indices(MEMORY_MODULE);
    if indices.is_empty() {
        let source = SourceRecord::new(
            snapshot,
            MEMORY_MODULE,
            0,
            "Win32_PhysicalMemory",
            collected_at_unix,
        );
        return source.unavailable(PrivacyClass::OperationalMetadata);
    }

    derived_field(
        u32::try_from(indices.len()).ok(),
        Confidence::High,
        PrivacyClass::OperationalMetadata,
        "number of Win32_PhysicalMemory records",
        collected_at_unix,
    )
}

fn error_correction_field(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventoryField<String> {
    let indices = snapshot.record_indices(MEMORY_ARRAY);
    if indices.is_empty() {
        let source = SourceRecord::new(
            snapshot,
            MEMORY_ARRAY,
            0,
            "Win32_PhysicalMemoryArray",
            collected_at_unix,
        );
        return source.unavailable(PrivacyClass::OperationalMetadata);
    }

    let values = indices
        .into_iter()
        .filter_map(|index| {
            snapshot
                .value(MEMORY_ARRAY, index, "error_correction")
                .and_then(|value| value.parse::<u16>().ok())
                .and_then(memory_error_correction_name)
        })
        .collect::<BTreeSet<_>>();

    let value = if values.len() == 1 {
        values.first().map(|value| (*value).to_string())
    } else if values.len() > 1 {
        Some("Mixed".to_string())
    } else {
        None
    };
    derived_field(
        value,
        Confidence::Medium,
        PrivacyClass::OperationalMetadata,
        "Win32_PhysicalMemoryArray.MemoryErrorCorrection",
        collected_at_unix,
    )
}

fn memory_type_field(source: &SourceRecord<'_>) -> InventoryField<String> {
    mapped_u16_field(source, "memory_type", memory_type_name)
}

fn memory_form_factor_field(source: &SourceRecord<'_>) -> InventoryField<String> {
    mapped_u16_field(source, "form_factor", memory_form_factor_name)
}

fn mapped_u16_field(
    source: &SourceRecord<'_>,
    name: &str,
    mapper: fn(u16) -> Option<&'static str>,
) -> InventoryField<String> {
    let raw = source.snapshot.value(source.section, source.index, name);
    let mapped = raw
        .and_then(|value| value.parse::<u16>().ok())
        .and_then(mapper);
    match mapped {
        Some(value) => InventoryField::derived(
            value.to_string(),
            Confidence::Medium,
            PrivacyClass::OperationalMetadata,
            derived_provenance(source.source_detail, source.collected_at_unix),
        ),
        None if raw.is_some() => {
            InventoryField::unknown(PrivacyClass::OperationalMetadata, source.provenance())
        }
        None => source.unavailable(PrivacyClass::OperationalMetadata),
    }
}

const fn memory_type_name(code: u16) -> Option<&'static str> {
    match code {
        18 => Some("DDR"),
        19 => Some("DDR2"),
        20 => Some("DDR2 FB-DIMM"),
        24 => Some("DDR3"),
        26 => Some("DDR4"),
        27 => Some("LPDDR"),
        28 => Some("LPDDR2"),
        29 => Some("LPDDR3"),
        30 => Some("LPDDR4"),
        34 => Some("DDR5"),
        35 => Some("LPDDR5"),
        _ => None,
    }
}

const fn memory_form_factor_name(code: u16) -> Option<&'static str> {
    match code {
        8 => Some("DIMM"),
        9 => Some("TSOP"),
        10 => Some("PGA"),
        11 => Some("RIMM"),
        12 => Some("SODIMM"),
        13 => Some("SRIMM"),
        15 => Some("FB-DIMM"),
        16 => Some("Die"),
        _ => None,
    }
}

const fn memory_error_correction_name(code: u16) -> Option<&'static str> {
    match code {
        1 => Some("Other"),
        3 => Some("None"),
        4 => Some("Parity"),
        5 => Some("Single-bit ECC"),
        6 => Some("Multi-bit ECC"),
        7 => Some("CRC"),
        _ => None,
    }
}

fn derive_classification(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventoryField<DeviceClassification> {
    let manufacturer = snapshot.value(SYSTEM, 0, "manufacturer");
    let model = snapshot.value(SYSTEM, 0, "model");
    let product_vendor = snapshot.value(PRODUCT, 0, "vendor");
    let (classification, confidence) = if is_virtual_machine(manufacturer, model, product_vendor) {
        (DeviceClassification::Virtual, Confidence::High)
    } else if manufacturer.is_none() && model.is_none() {
        (DeviceClassification::Unknown, Confidence::Unknown)
    } else if manufacturer.is_none_or(|value| !is_meaningful_text(value))
        || model.is_none_or(|value| !is_meaningful_text(value))
    {
        (
            DeviceClassification::CustomOrUnidentified,
            Confidence::Medium,
        )
    } else if vendors_conflict(manufacturer, product_vendor) {
        (
            DeviceClassification::ConflictingFirmwareData,
            Confidence::Low,
        )
    } else {
        (DeviceClassification::OemReported, Confidence::Medium)
    };
    if classification == DeviceClassification::Unknown {
        InventoryField::unknown(
            PrivacyClass::NonSensitive,
            derived_provenance("firmware classification rules", collected_at_unix),
        )
    } else {
        InventoryField::derived(
            classification,
            confidence,
            PrivacyClass::NonSensitive,
            derived_provenance("firmware classification rules", collected_at_unix),
        )
    }
}

fn derive_form_factor(
    snapshot: &RawSnapshot,
    classification: Option<DeviceClassification>,
    collected_at_unix: u64,
) -> InventoryField<FormFactor> {
    if classification == Some(DeviceClassification::Virtual) {
        return InventoryField::derived(
            FormFactor::VirtualMachine,
            Confidence::High,
            PrivacyClass::NonSensitive,
            derived_provenance("virtual-machine classification", collected_at_unix),
        );
    }

    let chassis_codes = snapshot
        .value(CHASSIS, 0, "chassis_types")
        .map(parse_csv_u16)
        .unwrap_or_default();
    let has_desktop = chassis_codes
        .iter()
        .any(|code| matches!(code, 3 | 4 | 5 | 6 | 7 | 13 | 15 | 16 | 24 | 35 | 36));
    let has_laptop = chassis_codes
        .iter()
        .any(|code| matches!(code, 8 | 9 | 10 | 14));
    let has_tablet = chassis_codes
        .iter()
        .any(|code| matches!(code, 11 | 30 | 31 | 32));

    let form_factor = match (has_desktop, has_laptop, has_tablet) {
        (true, false, false) => Some(FormFactor::Desktop),
        (false, true, false) => Some(FormFactor::Laptop),
        (false, false, true) => Some(FormFactor::Tablet),
        (false, false, false) => snapshot
            .value(SYSTEM, 0, "pc_system_type")
            .and_then(|value| value.parse::<u16>().ok())
            .and_then(|code| match code {
                1 | 3 => Some(FormFactor::Desktop),
                2 => Some(FormFactor::Laptop),
                _ => None,
            }),
        _ => None,
    };

    derived_field(
        form_factor,
        Confidence::Medium,
        PrivacyClass::NonSensitive,
        "SMBIOS chassis type and PCSystemType",
        collected_at_unix,
    )
}

fn derive_virtualization_indicator(
    snapshot: &RawSnapshot,
    collected_at_unix: u64,
) -> InventoryField<bool> {
    let manufacturer = snapshot.value(SYSTEM, 0, "manufacturer");
    let model = snapshot.value(SYSTEM, 0, "model");
    let product_vendor = snapshot.value(PRODUCT, 0, "vendor");
    if manufacturer.is_none() && model.is_none() && product_vendor.is_none() {
        return InventoryField::unknown(
            PrivacyClass::OperationalMetadata,
            derived_provenance("firmware virtualization markers", collected_at_unix),
        );
    }
    InventoryField::derived(
        is_virtual_machine(manufacturer, model, product_vendor),
        Confidence::Medium,
        PrivacyClass::OperationalMetadata,
        derived_provenance("firmware virtualization markers", collected_at_unix),
    )
}

fn is_virtual_machine(
    manufacturer: Option<&str>,
    model: Option<&str>,
    product_vendor: Option<&str>,
) -> bool {
    let combined = [manufacturer, model, product_vendor]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    [
        "virtual machine",
        "vmware",
        "virtualbox",
        "kvm",
        "qemu",
        "xen",
        "parallels",
        "bochs",
        "amazon ec2",
        "google compute engine",
    ]
    .iter()
    .any(|marker| combined.contains(marker))
}

fn vendors_conflict(manufacturer: Option<&str>, product_vendor: Option<&str>) -> bool {
    let Some(manufacturer) = manufacturer.filter(|value| is_meaningful_text(value)) else {
        return false;
    };
    let Some(product_vendor) = product_vendor.filter(|value| is_meaningful_text(value)) else {
        return false;
    };
    let manufacturer = canonical_vendor(manufacturer);
    let product_vendor = canonical_vendor(product_vendor);
    !manufacturer.is_empty()
        && !product_vendor.is_empty()
        && !manufacturer.contains(&product_vendor)
        && !product_vendor.contains(&manufacturer)
}

fn canonical_vendor(value: &str) -> String {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| {
            !word.is_empty()
                && !matches!(
                    word.to_ascii_lowercase().as_str(),
                    "inc"
                        | "incorporated"
                        | "corp"
                        | "corporation"
                        | "co"
                        | "company"
                        | "ltd"
                        | "limited"
                        | "llc"
                )
        })
        .collect::<String>()
        .to_ascii_lowercase()
}

fn parse_csv_u16(value: &str) -> Vec<u16> {
    value
        .split(',')
        .filter_map(|part| part.trim().parse::<u16>().ok())
        .collect()
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

fn is_meaningful_text(value: &str) -> bool {
    let canonical = canonical_placeholder(value);
    !canonical.is_empty()
        && !matches!(
            canonical.as_str(),
            "unknown"
                | "none"
                | "na"
                | "notapplicable"
                | "notspecified"
                | "defaultstring"
                | "tobefilledbyoem"
                | "systemmanufacturer"
                | "systemproductname"
                | "type1productconfigid"
                | "invalid"
        )
}

fn is_meaningful_identifier(value: &str) -> bool {
    if !is_meaningful_text(value) {
        return false;
    }
    let canonical = canonical_placeholder(value);
    if matches!(
        canonical.as_str(),
        "systemserialnumber" | "noassettag" | "noserialnumber"
    ) {
        return false;
    }
    !canonical.chars().all(|character| character == '0')
        && !canonical.chars().all(|character| character == 'f')
}

fn canonical_placeholder(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn combined_query_state(snapshot: &RawSnapshot, sections: &[&str]) -> QueryState {
    let states = sections
        .iter()
        .map(|section| snapshot.query_state(section))
        .collect::<Vec<_>>();
    if states.contains(&QueryState::PermissionDenied) {
        QueryState::PermissionDenied
    } else if states.contains(&QueryState::CollectionError) {
        QueryState::CollectionError
    } else if states.contains(&QueryState::Unsupported) {
        QueryState::Unsupported
    } else if states.contains(&QueryState::Available) {
        QueryState::Available
    } else {
        QueryState::NotReported
    }
}

fn empty_section<T>(
    state: QueryState,
    source_detail: &'static str,
    collected_at_unix: u64,
) -> InventorySection<T> {
    InventorySection {
        status: state.status(),
        provenance: source_provenance(source_detail, collected_at_unix, state.permission()),
        records: Vec::new(),
    }
}

fn missing_field<T>(
    status: CollectionStatus,
    privacy_class: PrivacyClass,
    provenance: Provenance,
) -> InventoryField<T> {
    InventoryField {
        value: None,
        status,
        confidence: Confidence::Unknown,
        privacy_class,
        provenance,
    }
}

fn derived_field<T>(
    value: Option<T>,
    confidence: Confidence,
    privacy_class: PrivacyClass,
    source_detail: &'static str,
    collected_at_unix: u64,
) -> InventoryField<T> {
    match value {
        Some(value) => InventoryField::derived(
            value,
            confidence,
            privacy_class,
            derived_provenance(source_detail, collected_at_unix),
        ),
        None => InventoryField::unknown(
            privacy_class,
            derived_provenance(source_detail, collected_at_unix),
        ),
    }
}

fn source_provenance(
    source_detail: &'static str,
    collected_at_unix: u64,
    permission: PermissionState,
) -> Provenance {
    Provenance {
        source: CollectionSource::PowerShellCimFallback,
        source_detail: Some(source_detail.to_string()),
        parser_version: env!("CARGO_PKG_VERSION").to_string(),
        collected_at_unix,
        permission,
    }
}

fn derived_provenance(source_detail: &'static str, collected_at_unix: u64) -> Provenance {
    Provenance {
        source: CollectionSource::Derived,
        source_detail: Some(source_detail.to_string()),
        parser_version: env!("CARGO_PKG_VERSION").to_string(),
        collected_at_unix,
        permission: PermissionState::NotRequired,
    }
}

fn failed_inventory(collected_at_unix: u64, error_kind: CollectorErrorKind) -> HardwareInventoryV1 {
    let (status, permission) = match error_kind {
        CollectorErrorKind::Unsupported => {
            (CollectionStatus::Unsupported, PermissionState::Unknown)
        }
        CollectorErrorKind::PermissionDenied => {
            (CollectionStatus::PermissionDenied, PermissionState::Denied)
        }
        CollectorErrorKind::CommandFailed
        | CollectorErrorKind::ParseFailed
        | CollectorErrorKind::Cancelled
        | CollectorErrorKind::TimedOut
        | CollectorErrorKind::OutputLimitExceeded
        | CollectorErrorKind::Internal => {
            (CollectionStatus::CollectionError, PermissionState::Unknown)
        }
    };
    let provenance = source_provenance("bounded hardware collector", collected_at_unix, permission);
    let mut inventory = HardwareInventoryV1::not_collected(collected_at_unix);
    inventory.device_and_chassis = InventorySection {
        status,
        provenance: provenance.clone(),
        records: Vec::new(),
    };
    inventory.firmware = InventorySection {
        status,
        provenance: provenance.clone(),
        records: Vec::new(),
    };
    inventory.processors = InventorySection {
        status,
        provenance: provenance.clone(),
        records: Vec::new(),
    };
    inventory.memory = MemoryInventory {
        summary: InventorySection {
            status,
            provenance: provenance.clone(),
            records: Vec::new(),
        },
        modules: InventorySection {
            status,
            provenance,
            records: Vec::new(),
        },
    };
    inventory
}

fn collector_error(safe_message: &'static str) -> CollectorError {
    typed_collector_error(CollectorErrorKind::ParseFailed, safe_message)
}

fn typed_collector_error(kind: CollectorErrorKind, safe_message: &'static str) -> CollectorError {
    CollectorError {
        collector: CollectorName::HardwareInventory,
        kind,
        safe_message: safe_message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FixtureOutput {
        text: String,
    }

    impl FixtureOutput {
        fn query(&mut self, section: &str) -> &mut Self {
            self.field(section, 0, "query_ok", "True")
        }

        fn record(&mut self, section: &str, index: usize) -> &mut Self {
            self.field(section, index, "record_present", "True")
        }

        fn field(&mut self, section: &str, index: usize, name: &str, value: &str) -> &mut Self {
            self.text.push_str(section);
            self.text.push('\t');
            self.text.push_str(&index.to_string());
            self.text.push('\t');
            self.text.push_str(name);
            self.text.push('\t');
            self.text.push_str(&hex_encode(value));
            self.text.push('\n');
            self
        }

        fn snapshot(&self) -> RawSnapshot {
            let mut complete = self.text.clone();
            for section in [
                SYSTEM,
                PRODUCT,
                CHASSIS,
                BASEBOARD,
                BIOS,
                FIRMWARE,
                SECURE_BOOT,
                TPM,
                PROCESSOR,
                OPERATING_SYSTEM,
                MEMORY_ARRAY,
                MEMORY_MODULE,
            ] {
                let has_query_result = self.text.lines().any(|line| {
                    let mut parts = line.split('\t');
                    parts.next() == Some(section)
                        && parts.next() == Some("0")
                        && matches!(parts.next(), Some("query_ok" | "query_status"))
                });
                if !has_query_result {
                    complete.push_str(section);
                    complete.push_str("\t0\tquery_ok\t");
                    complete.push_str(&hex_encode("True"));
                    complete.push('\n');
                }
            }
            parse_snapshot(complete.as_bytes()).expect("fixture protocol must parse")
        }
    }

    fn hex_encode(value: &str) -> String {
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn branded_laptop_fixture() -> FixtureOutput {
        let mut fixture = FixtureOutput::default();
        fixture
            .query(SYSTEM)
            .record(SYSTEM, 0)
            .field(SYSTEM, 0, "manufacturer", "Dell Inc.")
            .field(SYSTEM, 0, "model", "Latitude 7420")
            .field(SYSTEM, 0, "family", "Latitude")
            .field(SYSTEM, 0, "total_physical_memory", "17179869184")
            .field(SYSTEM, 0, "number_of_processors", "1")
            .field(SYSTEM, 0, "pc_system_type", "2")
            .field(SYSTEM, 0, "hypervisor_present", "False")
            .query(PRODUCT)
            .record(PRODUCT, 0)
            .field(PRODUCT, 0, "vendor", "Dell Inc.")
            .field(PRODUCT, 0, "identifying_number", "LOCAL-SERIAL-01")
            .field(PRODUCT, 0, "uuid", "11111111-2222-3333-4444-555555555555")
            .query(CHASSIS)
            .record(CHASSIS, 0)
            .field(CHASSIS, 0, "manufacturer", "Dell Inc.")
            .field(CHASSIS, 0, "chassis_types", "10")
            .field(CHASSIS, 0, "serial_number", "CHASSIS-01")
            .field(CHASSIS, 0, "asset_tag", "ASSET-01")
            .query(BASEBOARD)
            .record(BASEBOARD, 0)
            .field(BASEBOARD, 0, "manufacturer", "Dell Inc.")
            .field(BASEBOARD, 0, "product", "0ABC12")
            .field(BASEBOARD, 0, "version", "A01")
            .field(BASEBOARD, 0, "serial_number", "BOARD-01")
            .query(BIOS)
            .record(BIOS, 0)
            .field(BIOS, 0, "manufacturer", "Dell Inc.")
            .field(BIOS, 0, "version", "1.28.0")
            .field(BIOS, 0, "release_date", "2026-01-15")
            .field(BIOS, 0, "smbios_major", "3")
            .field(BIOS, 0, "smbios_minor", "3")
            .field(BIOS, 0, "serial_number", "LOCAL-SERIAL-01")
            .query(FIRMWARE)
            .record(FIRMWARE, 0)
            .field(FIRMWARE, 0, "mode", "Uefi")
            .query(SECURE_BOOT)
            .record(SECURE_BOOT, 0)
            .field(SECURE_BOOT, 0, "enabled", "True")
            .query(TPM)
            .record(TPM, 0)
            .field(TPM, 0, "present", "True")
            .field(TPM, 0, "spec_version", "2.0")
            .query(PROCESSOR)
            .record(PROCESSOR, 0)
            .field(PROCESSOR, 0, "manufacturer", "GenuineIntel")
            .field(PROCESSOR, 0, "model", "Intel Fixture CPU")
            .field(PROCESSOR, 0, "architecture", "9")
            .field(PROCESSOR, 0, "cores", "4")
            .field(PROCESSOR, 0, "logical_processors", "8")
            .field(PROCESSOR, 0, "maximum_clock_mhz", "2800")
            .field(PROCESSOR, 0, "current_clock_mhz", "1800")
            .field(PROCESSOR, 0, "address_width_bits", "64")
            .field(PROCESSOR, 0, "vm_monitor_extensions", "True")
            .field(PROCESSOR, 0, "virtualization_firmware_enabled", "True")
            .query(OPERATING_SYSTEM)
            .record(OPERATING_SYSTEM, 0)
            .field(OPERATING_SYSTEM, 0, "visible_memory_kib", "16500000")
            .query(MEMORY_ARRAY)
            .record(MEMORY_ARRAY, 0)
            .field(MEMORY_ARRAY, 0, "slot_count", "2")
            .field(MEMORY_ARRAY, 0, "error_correction", "3")
            .query(MEMORY_MODULE)
            .record(MEMORY_MODULE, 0)
            .field(MEMORY_MODULE, 0, "locator", "DIMM A")
            .field(MEMORY_MODULE, 0, "capacity_bytes", "8589934592")
            .field(MEMORY_MODULE, 0, "speed_mhz", "3200")
            .field(MEMORY_MODULE, 0, "configured_speed_mhz", "3200")
            .field(MEMORY_MODULE, 0, "memory_type", "26")
            .field(MEMORY_MODULE, 0, "form_factor", "12")
            .field(MEMORY_MODULE, 0, "manufacturer", "Fixture Memory")
            .field(MEMORY_MODULE, 0, "part_number", "PART-A")
            .field(MEMORY_MODULE, 0, "serial_number", "RAM-01")
            .record(MEMORY_MODULE, 1)
            .field(MEMORY_MODULE, 1, "locator", "DIMM B")
            .field(MEMORY_MODULE, 1, "capacity_bytes", "8589934592")
            .field(MEMORY_MODULE, 1, "speed_mhz", "3200")
            .field(MEMORY_MODULE, 1, "configured_speed_mhz", "3200")
            .field(MEMORY_MODULE, 1, "memory_type", "26")
            .field(MEMORY_MODULE, 1, "form_factor", "12")
            .field(MEMORY_MODULE, 1, "manufacturer", "Fixture Memory")
            .field(MEMORY_MODULE, 1, "part_number", "PART-B")
            .field(MEMORY_MODULE, 1, "serial_number", "RAM-02");
        fixture
    }

    #[test]
    fn branded_laptop_fixture_builds_complete_first_slice() {
        let inventory = build_inventory(&branded_laptop_fixture().snapshot(), 1_000);
        let device = &inventory.device_and_chassis.records[0];
        let firmware = &inventory.firmware.records[0];
        let processor = &inventory.processors.records[0];
        let memory = &inventory.memory.summary.records[0];

        assert_eq!(
            device.classification.value,
            Some(DeviceClassification::OemReported)
        );
        assert_eq!(device.form_factor.value, Some(FormFactor::Laptop));
        assert_eq!(firmware.mode.value, Some(FirmwareMode::Uefi));
        assert_eq!(firmware.secure_boot_enabled.value, Some(true));
        assert_eq!(firmware.tpm_present.value, Some(true));
        assert_eq!(processor.architecture.value.as_deref(), Some("x64"));
        assert_eq!(processor.physical_core_count.value, Some(4));
        assert_eq!(memory.installed_physical_bytes.value, Some(17_179_869_184));
        assert_eq!(memory.physical_slot_count.value, Some(2));
        assert_eq!(memory.populated_slot_count.value, Some(2));
        assert_eq!(inventory.memory.modules.records.len(), 2);
        assert_eq!(
            inventory.memory.modules.records[0]
                .memory_type
                .value
                .as_deref(),
            Some("DDR4")
        );
        assert_eq!(
            inventory.storage.devices.status,
            CollectionStatus::NotReported
        );
    }

    #[test]
    fn custom_or_assembled_fixture_is_not_claimed_as_oem() {
        let mut fixture = FixtureOutput::default();
        fixture
            .query(SYSTEM)
            .record(SYSTEM, 0)
            .field(SYSTEM, 0, "manufacturer", "System manufacturer")
            .field(SYSTEM, 0, "model", "System Product Name")
            .query(PRODUCT)
            .record(PRODUCT, 0)
            .field(PRODUCT, 0, "vendor", "To Be Filled By O.E.M.")
            .query(BASEBOARD)
            .record(BASEBOARD, 0)
            .field(BASEBOARD, 0, "manufacturer", "ASUSTeK COMPUTER INC.")
            .field(BASEBOARD, 0, "product", "PRIME-Z790")
            .query(CHASSIS)
            .record(CHASSIS, 0)
            .field(CHASSIS, 0, "chassis_types", "3");

        let inventory = build_inventory(&fixture.snapshot(), 1_000);
        let device = &inventory.device_and_chassis.records[0];
        assert_eq!(
            device.classification.value,
            Some(DeviceClassification::CustomOrUnidentified)
        );
        assert_eq!(device.system_manufacturer.value, None);
        assert_eq!(device.system_manufacturer.status, CollectionStatus::Unknown);
        assert_eq!(device.form_factor.value, Some(FormFactor::Desktop));
    }

    #[test]
    fn branded_desktop_fixture_is_oem_reported() {
        let mut fixture = FixtureOutput::default();
        fixture
            .query(SYSTEM)
            .record(SYSTEM, 0)
            .field(SYSTEM, 0, "manufacturer", "HP")
            .field(SYSTEM, 0, "model", "EliteDesk 800 G9")
            .field(SYSTEM, 0, "pc_system_type", "1")
            .query(PRODUCT)
            .record(PRODUCT, 0)
            .field(PRODUCT, 0, "vendor", "HP")
            .query(CHASSIS)
            .record(CHASSIS, 0)
            .field(CHASSIS, 0, "chassis_types", "6");

        let inventory = build_inventory(&fixture.snapshot(), 1_000);
        let device = &inventory.device_and_chassis.records[0];
        assert_eq!(
            device.classification.value,
            Some(DeviceClassification::OemReported)
        );
        assert_eq!(device.form_factor.value, Some(FormFactor::Desktop));
    }

    #[test]
    fn virtual_machine_fixture_is_classified_without_hardware_guessing() {
        let mut fixture = FixtureOutput::default();
        fixture
            .query(SYSTEM)
            .record(SYSTEM, 0)
            .field(SYSTEM, 0, "manufacturer", "Microsoft Corporation")
            .field(SYSTEM, 0, "model", "Virtual Machine")
            .field(SYSTEM, 0, "hypervisor_present", "True")
            .query(PRODUCT)
            .record(PRODUCT, 0)
            .field(PRODUCT, 0, "vendor", "Microsoft Corporation")
            .query(BIOS)
            .record(BIOS, 0)
            .field(BIOS, 0, "manufacturer", "Microsoft Corporation");

        let inventory = build_inventory(&fixture.snapshot(), 1_000);
        let device = &inventory.device_and_chassis.records[0];
        let firmware = &inventory.firmware.records[0];
        assert_eq!(
            device.classification.value,
            Some(DeviceClassification::Virtual)
        );
        assert_eq!(device.form_factor.value, Some(FormFactor::VirtualMachine));
        assert_eq!(firmware.virtualization_indicator.value, Some(true));
    }

    #[test]
    fn generic_smbios_identifiers_remain_unknown_and_redacted() {
        let mut fixture = FixtureOutput::default();
        fixture
            .query(SYSTEM)
            .record(SYSTEM, 0)
            .field(SYSTEM, 0, "manufacturer", "Default string")
            .field(SYSTEM, 0, "model", "Type1ProductConfigId")
            .query(PRODUCT)
            .record(PRODUCT, 0)
            .field(PRODUCT, 0, "identifying_number", "To Be Filled By O.E.M.")
            .field(PRODUCT, 0, "uuid", "00000000-0000-0000-0000-000000000000")
            .query(BIOS)
            .record(BIOS, 0)
            .field(BIOS, 0, "serial_number", "System Serial Number");

        let inventory = build_inventory(&fixture.snapshot(), 1_000);
        let device = &inventory.device_and_chassis.records[0];
        assert_eq!(device.serial_number.value, None);
        assert_eq!(device.serial_number.status, CollectionStatus::Unknown);
        assert_eq!(device.system_uuid.value, None);
        assert!(format!("{device:?}").contains("value: None"));
        assert!(!format!("{device:?}").contains("System Serial Number"));
    }

    #[test]
    fn secure_boot_access_denial_is_explicit_and_scan_continues() {
        let mut fixture = FixtureOutput::default();
        fixture
            .query(BIOS)
            .record(BIOS, 0)
            .field(BIOS, 0, "manufacturer", "Fixture Firmware")
            .query(FIRMWARE)
            .record(FIRMWARE, 0)
            .field(FIRMWARE, 0, "mode", "Uefi")
            .field(SECURE_BOOT, 0, "query_status", "permission_denied");

        let inventory = build_inventory(&fixture.snapshot(), 1_000);
        let firmware = &inventory.firmware.records[0];
        assert_eq!(
            firmware.secure_boot_present.status,
            CollectionStatus::PermissionDenied
        );
        assert_eq!(
            firmware.secure_boot_enabled.status,
            CollectionStatus::PermissionDenied
        );
        assert_eq!(
            firmware.secure_boot_enabled.provenance.permission,
            PermissionState::Denied
        );
        assert_eq!(firmware.vendor.value.as_deref(), Some("Fixture Firmware"));
    }

    #[test]
    fn malformed_protocol_is_rejected_without_partial_inventory() {
        let malformed = b"system\t0\tmanufacturer\tzz\n";
        let error = match parse_snapshot(malformed) {
            Ok(_) => panic!("invalid hex unexpectedly parsed"),
            Err(error) => error,
        };
        assert_eq!(error.kind, CollectorErrorKind::ParseFailed);
    }

    #[test]
    fn failed_collection_marks_only_the_requested_first_slice() {
        let inventory = failed_inventory(1_000, CollectorErrorKind::TimedOut);
        assert_eq!(
            inventory.device_and_chassis.status,
            CollectionStatus::CollectionError
        );
        assert_eq!(inventory.firmware.status, CollectionStatus::CollectionError);
        assert_eq!(
            inventory.memory.modules.status,
            CollectionStatus::CollectionError
        );
        assert_eq!(
            inventory.storage.devices.status,
            CollectionStatus::NotReported
        );
        assert_eq!(inventory.sensors.status, CollectionStatus::NotReported);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn live_windows_snapshot_obeys_the_fixed_protocol() {
        let inventory = try_collect(&CancellationToken::new())
            .unwrap_or_else(|error| panic!("live collector failed: {:?}", error.kind));
        assert_eq!(
            inventory.schema_version,
            crate::hardware_inventory_v1::SCHEMA_VERSION
        );
        assert!(inventory.device_and_chassis.is_consistent());
        assert!(inventory.firmware.is_consistent());
        assert!(inventory.processors.is_consistent());
        assert!(inventory.memory.summary.is_consistent());
        assert!(inventory.memory.modules.is_consistent());
        assert_eq!(
            inventory.storage.devices.status,
            CollectionStatus::NotReported
        );
    }
}
