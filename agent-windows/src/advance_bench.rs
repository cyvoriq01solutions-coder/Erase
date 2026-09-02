//! Consent-gated Advance scan workloads.
//!
//! These run in-process inside the agent crate (never in `desktop/src-tauri`).
//! Default Advance scan does not call them. Windows CI therefore does not add
//! another PowerShell collector for the declined-benchmark path.
//!
//! A user-mode CPU loop cannot read package temperature. Clock points are
//! scored only from Windows current/max megahertz sampled after the loop.
//! Memory is a spot check on a capped region, never "memory verified".

use crate::battery_probe::BatteryProbe;
use crate::collector_runtime::{
    CancellationToken, CollectorLimits, TrustedPowerShellScript, run_fixed_powershell,
};
use crate::cpu_memory::CpuMemoryProbe;
use crate::storage_health::StorageProbe;
use crate::{CollectorName, hardware_diagnostics_v1::CriticalFault};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const CPU_WORKLOAD: Duration = Duration::from_millis(1500);
const PATTERN_CAP_BYTES: usize = 32 * 1024 * 1024;
const READ_BYTES: usize = 8 * 1024 * 1024;
const WRITE_BYTES: u64 = 8 * 1024 * 1024;
const CLOCK_SAMPLE_SCRIPT: TrustedPowerShellScript = TrustedPowerShellScript::application_owned(
    r#"
$ErrorActionPreference = 'Stop'
$p = Get-CimInstance -ClassName Win32_Processor | Select-Object -First 1
$cur = [Convert]::ToString($p.CurrentClockSpeed, [Globalization.CultureInfo]::InvariantCulture)
$max = [Convert]::ToString($p.MaxClockSpeed, [Globalization.CultureInfo]::InvariantCulture)
$c = -join ([Text.Encoding]::UTF8.GetBytes($cur) | ForEach-Object { $_.ToString('x2') })
$m = -join ([Text.Encoding]::UTF8.GetBytes($max) | ForEach-Object { $_.ToString('x2') })
[Console]::Out.WriteLine("cpu`t0`tcurrent_mhz`t$c")
[Console]::Out.WriteLine("cpu`t0`tmax_mhz`t$m")
"#,
);

#[derive(Debug, Clone, PartialEq)]
pub struct BenchResult {
    pub cpu: CpuBench,
    pub memory: MemoryBench,
    pub storage: StorageBench,
    pub bytes_written: u64,
    pub temporary_file_removed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuBench {
    pub status: String,
    pub current_mhz: Option<u32>,
    pub max_mhz: Option<u32>,
    pub ratio: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryBench {
    pub status: String,
    pub bytes_tested: Option<u64>,
    pub pattern_passed: Option<bool>,
    pub bandwidth_mib_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageBench {
    pub sequential_status: String,
    pub random_status: String,
    pub write_status: String,
    pub sequential_mib_s: Option<f64>,
    pub random_iops: Option<u32>,
}

impl BenchResult {
    #[must_use]
    pub fn declined() -> Self {
        Self {
            cpu: CpuBench {
                status: "Declined by the operator. No benchmark was run and nothing was written."
                    .to_string(),
                current_mhz: None,
                max_mhz: None,
                ratio: None,
            },
            memory: MemoryBench {
                status: "Declined by the operator. No benchmark was run and nothing was written."
                    .to_string(),
                bytes_tested: None,
                pattern_passed: None,
                bandwidth_mib_s: None,
            },
            storage: StorageBench {
                sequential_status:
                    "Declined by the operator. No benchmark was run and nothing was written."
                        .to_string(),
                random_status:
                    "Declined by the operator. No benchmark was run and nothing was written."
                        .to_string(),
                write_status:
                    "Declined by the operator. No benchmark was run and nothing was written."
                        .to_string(),
                sequential_mib_s: None,
                random_iops: None,
            },
            bytes_written: 0,
            temporary_file_removed: true,
        }
    }

    #[must_use]
    pub fn memory_critical(&self) -> Option<CriticalFault> {
        (self.memory.pattern_passed == Some(false))
            .then_some(CriticalFault::MemoryIntegrityMismatch)
    }
}

#[must_use]
pub fn run(
    benchmarks_consented: bool,
    write_consented: bool,
    battery: &BatteryProbe,
    storage: &StorageProbe,
    identity: &CpuMemoryProbe,
    cancellation: &CancellationToken,
) -> BenchResult {
    if !benchmarks_consented {
        return BenchResult::declined();
    }
    if cfg!(not(target_os = "windows")) {
        return skipped("Benchmarks run only on the installed Windows application.");
    }

    let mut result = skipped("Starting consented workloads.");

    result.cpu = cpu_workload(battery, identity, cancellation);
    result.memory = memory_workload(identity, cancellation);
    let (storage_bench, written, removed) =
        storage_workload(write_consented, storage, cancellation);
    result.storage = storage_bench;
    result.bytes_written = written;
    result.temporary_file_removed = removed;
    result
}

fn skipped(reason: &str) -> BenchResult {
    BenchResult {
        cpu: CpuBench {
            status: reason.to_string(),
            current_mhz: None,
            max_mhz: None,
            ratio: None,
        },
        memory: MemoryBench {
            status: reason.to_string(),
            bytes_tested: None,
            pattern_passed: None,
            bandwidth_mib_s: None,
        },
        storage: StorageBench {
            sequential_status: reason.to_string(),
            random_status: reason.to_string(),
            write_status: reason.to_string(),
            sequential_mib_s: None,
            random_iops: None,
        },
        bytes_written: 0,
        temporary_file_removed: true,
    }
}

fn cpu_workload(
    battery: &BatteryProbe,
    identity: &CpuMemoryProbe,
    cancellation: &CancellationToken,
) -> CpuBench {
    if let Some(charge) = battery.primary().and_then(|reading| reading.charge_percent)
        && charge <= 5
    {
        return CpuBench {
            status:
                "CPU workload was not started because the battery charge is critical (5% or below)."
                    .to_string(),
            current_mhz: None,
            max_mhz: identity.processor.as_ref().and_then(|cpu| cpu.max_mhz),
            ratio: None,
        };
    }

    let deadline = Instant::now() + CPU_WORKLOAD;
    let mut acc: u64 = 1;
    while Instant::now() < deadline {
        if cancellation.is_cancelled() {
            return CpuBench {
                status: "CPU workload was cancelled.".to_string(),
                current_mhz: None,
                max_mhz: None,
                ratio: None,
            };
        }
        acc = acc.wrapping_mul(1_000_003).wrapping_add(17);
    }
    let _ = acc;

    let (current, max) = sample_clocks(identity, cancellation);
    let ratio = match (current, max) {
        (Some(cur), Some(top)) if top > 0 => Some(((u64::from(cur) * 100) / u64::from(top)) as u32),
        _ => None,
    };
    CpuBench {
        status: match ratio {
            Some(percent) => format!(
                "CPU workload finished. Windows reported {current} MHz after the loop against a maximum of {max} MHz ({percent}% of maximum). Package temperature is not collected.",
                current = current.unwrap_or(0),
                max = max.unwrap_or(0),
            ),
            None => {
                "CPU workload finished. Windows did not return both current and maximum clock, so the sustained-clock ratio stays not assessable. Package temperature is not collected."
                    .to_string()
            }
        },
        current_mhz: current,
        max_mhz: max,
        ratio,
    }
}

fn sample_clocks(
    identity: &CpuMemoryProbe,
    cancellation: &CancellationToken,
) -> (Option<u32>, Option<u32>) {
    let limits = CollectorLimits::new(
        Duration::from_secs(8),
        4 * 1024,
        2 * 1024,
        Duration::from_millis(10),
    );
    let sampled = run_fixed_powershell(
        CollectorName::HardwareInventory,
        CLOCK_SAMPLE_SCRIPT,
        limits,
        cancellation,
    )
    .ok()
    .and_then(|output| String::from_utf8(output.stdout().to_vec()).ok())
    .map(|text| {
        let mut current = None;
        let mut max = None;
        for line in text.lines() {
            let mut parts = line.split('\t');
            let (_section, _index, name, encoded) =
                match (parts.next(), parts.next(), parts.next(), parts.next()) {
                    (Some(section), Some(index), Some(name), Some(encoded))
                        if section == "cpu" && index == "0" =>
                    {
                        (section, index, name, encoded)
                    }
                    _ => continue,
                };
            if let Some(value) = decode_simple(encoded).and_then(|text| text.parse::<u32>().ok()) {
                match name {
                    "current_mhz" => current = Some(value),
                    "max_mhz" => max = Some(value),
                    _ => {}
                }
            }
        }
        (current, max)
    });

    sampled.unwrap_or_else(|| {
        let cpu = identity.processor.as_ref();
        (
            cpu.and_then(|item| item.current_mhz),
            cpu.and_then(|item| item.max_mhz),
        )
    })
}

fn memory_workload(identity: &CpuMemoryProbe, cancellation: &CancellationToken) -> MemoryBench {
    let available = identity.available_bytes.unwrap_or(64 * 1024 * 1024);
    let target = ((available / 2) as usize).clamp(1024 * 1024, PATTERN_CAP_BYTES);
    if cancellation.is_cancelled() {
        return MemoryBench {
            status: "Memory pattern check was cancelled.".to_string(),
            bytes_tested: None,
            pattern_passed: None,
            bandwidth_mib_s: None,
        };
    }

    let mut buffer = vec![0_u8; target];
    let start = Instant::now();
    for (index, byte) in buffer.iter_mut().enumerate() {
        *byte = (index as u8).wrapping_mul(31).wrapping_add(0xA5);
    }
    let fill_secs = start.elapsed().as_secs_f64().max(0.000_001);
    let bandwidth = (target as f64 / (1024.0 * 1024.0)) / fill_secs;

    let mut passed = true;
    for (index, byte) in buffer.iter().enumerate() {
        if cancellation.is_cancelled() {
            return MemoryBench {
                status: "Memory pattern check was cancelled.".to_string(),
                bytes_tested: Some(target as u64),
                pattern_passed: None,
                bandwidth_mib_s: Some(bandwidth),
            };
        }
        let expected = (index as u8).wrapping_mul(31).wrapping_add(0xA5);
        if *byte != expected {
            passed = false;
            break;
        }
    }

    MemoryBench {
        status: if passed {
            format!(
                "Memory pattern spot check passed on {:.1} MiB. This is not full-coverage memory testing; kernel-resident memory was not included.",
                target as f64 / (1024.0 * 1024.0)
            )
        } else {
            "Memory pattern spot check failed: the tested region did not match the pattern that was written.".to_string()
        },
        bytes_tested: Some(target as u64),
        pattern_passed: Some(passed),
        bandwidth_mib_s: Some(bandwidth),
    }
}

fn storage_workload(
    write_consented: bool,
    storage: &StorageProbe,
    cancellation: &CancellationToken,
) -> (StorageBench, u64, bool) {
    if storage
        .scoring_drives()
        .iter()
        .any(|drive| drive.predicts_failure == Some(true))
    {
        let blocked =
            "Storage workloads were not started because firmware reports a predicted failure."
                .to_string();
        return (
            StorageBench {
                sequential_status: blocked.clone(),
                random_status: blocked.clone(),
                write_status: blocked,
                sequential_mib_s: None,
                random_iops: None,
            },
            0,
            true,
        );
    }

    let read_path = existing_read_target();
    let sequential = sequential_read(&read_path, READ_BYTES);
    let random = random_read(&read_path, READ_BYTES);

    let (write_status, written, removed) = if write_consented {
        write_temp(cancellation)
    } else {
        (
            "Declined by the operator. No benchmark was run and nothing was written.".to_string(),
            0,
            true,
        )
    };

    (
        StorageBench {
            sequential_status: sequential.0,
            random_status: random.0,
            write_status,
            sequential_mib_s: sequential.1,
            random_iops: random.2,
        },
        written,
        removed,
    )
}

fn existing_read_target() -> PathBuf {
    if let Some(root) = std::env::var_os("SystemRoot") {
        let ntdll = PathBuf::from(root).join("System32").join("ntdll.dll");
        if ntdll.is_file() {
            return ntdll;
        }
    }
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."))
}

fn sequential_read(path: &PathBuf, bytes: usize) -> (String, Option<f64>, Option<u32>) {
    let mut file = match OpenOptions::new().read(true).open(path) {
        Ok(file) => file,
        Err(error) => {
            return (
                format!("Sequential read could not open {path:?} ({error})."),
                None,
                None,
            );
        }
    };
    let mut buf = vec![0_u8; 64 * 1024];
    let mut done = 0_usize;
    let start = Instant::now();
    while done < bytes {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => done += n,
            Err(error) => {
                return (format!("Sequential read failed ({error})."), None, None);
            }
        }
    }
    let secs = start.elapsed().as_secs_f64().max(0.000_001);
    let mib_s = (done as f64 / (1024.0 * 1024.0)) / secs;
    (
        format!(
            "Sequential read {mib_s:.0} MiB/s of an existing Windows system file. No file was created."
        ),
        Some(mib_s),
        None,
    )
}

fn random_read(path: &PathBuf, _bytes: usize) -> (String, Option<f64>, Option<u32>) {
    let mut file = match OpenOptions::new().read(true).open(path) {
        Ok(file) => file,
        Err(error) => {
            return (
                format!("Random read could not open {path:?} ({error})."),
                None,
                None,
            );
        }
    };
    let mut buf = vec![0_u8; 4096];
    let start = Instant::now();
    let mut ops = 0_u32;
    let span = file
        .metadata()
        .map(|meta| meta.len().saturating_sub(4096))
        .unwrap_or(1)
        .max(1);
    for step in 0..256_u64 {
        if file.seek(SeekFrom::Start((step * 4096) % span)).is_err() {
            break;
        }
        if file.read(&mut buf).is_err() {
            break;
        }
        ops += 1;
    }
    let secs = start.elapsed().as_secs_f64().max(0.000_001);
    let iops = (f64::from(ops) / secs) as u32;
    (
        format!(
            "Random 4 KiB read {iops} IOPS of an existing Windows system file. No file was created."
        ),
        None,
        Some(iops),
    )
}

fn write_temp(cancellation: &CancellationToken) -> (String, u64, bool) {
    if cancellation.is_cancelled() {
        return ("Write benchmark was cancelled.".to_string(), 0, true);
    }
    let path = std::env::temp_dir().join("cyvra-advance-bench.write.bin");
    let chunk = vec![0x3C_u8; 64 * 1024];
    let mut written = 0_u64;
    let mut removed = false;
    let started = Instant::now();
    let outcome = (|| {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|error| error.to_string())?;
        while written < WRITE_BYTES {
            file.write_all(&chunk).map_err(|error| error.to_string())?;
            written += chunk.len() as u64;
        }
        file.flush().map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })();
    if path.exists() {
        removed = fs::remove_file(&path).is_ok();
    }
    match outcome {
        Ok(()) => {
            let secs = started.elapsed().as_secs_f64().max(0.000_001);
            let mib_s = (WRITE_BYTES as f64 / (1024.0 * 1024.0)) / secs;
            (
                format!(
                    "Write benchmark {mib_s:.0} MiB/s. {WRITE_BYTES} bytes were written to the Windows temporary folder and then {}.",
                    if removed {
                        "deleted"
                    } else {
                        "could not be confirmed as deleted"
                    }
                ),
                WRITE_BYTES,
                removed,
            )
        }
        Err(error) => (
            format!("Write benchmark did not complete ({error})."),
            written,
            removed,
        ),
    }
}

fn decode_simple(encoded: &str) -> Option<String> {
    if !encoded.len().is_multiple_of(2) {
        return None;
    }
    let mut decoded = Vec::new();
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = match pair[0] {
            b'0'..=b'9' => pair[0] - b'0',
            b'a'..=b'f' => pair[0] - b'a' + 10,
            b'A'..=b'F' => pair[0] - b'A' + 10,
            _ => return None,
        };
        let low = match pair[1] {
            b'0'..=b'9' => pair[1] - b'0',
            b'a'..=b'f' => pair[1] - b'a' + 10,
            b'A'..=b'F' => pair[1] - b'A' + 10,
            _ => return None,
        };
        decoded.push((high << 4) | low);
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector_runtime::CancellationToken;

    #[test]
    fn declined_consent_writes_nothing() {
        let result = run(
            false,
            false,
            &BatteryProbe::unavailable("off"),
            &StorageProbe::unavailable("off"),
            &CpuMemoryProbe::unavailable("off"),
            &CancellationToken::new(),
        );
        assert_eq!(result.bytes_written, 0);
        assert!(result.cpu.status.contains("Declined"));
        assert!(result.memory.pattern_passed.is_none());
    }

    #[test]
    fn pattern_mismatch_is_a_critical_fault() {
        let mut result = BenchResult::declined();
        result.memory.pattern_passed = Some(false);
        assert_eq!(
            result.memory_critical(),
            Some(CriticalFault::MemoryIntegrityMismatch)
        );
    }

    #[test]
    fn clock_sample_script_is_read_only() {
        assert!(CLOCK_SAMPLE_SCRIPT.as_str().contains("CurrentClockSpeed"));
        assert!(!CLOCK_SAMPLE_SCRIPT.as_str().contains("Format-Volume"));
    }
}
