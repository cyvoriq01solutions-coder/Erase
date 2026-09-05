//! Mode S helper I/O. Runs in cyvra-purge-helper, not in the Tauri crate.

use std::fs;
use std::path::Path;

use super::media::MethodClass;
use super::plan::PlannedTarget;

#[derive(Debug, Clone)]
pub struct HelperResult {
    pub job_id: String,
    pub letter: String,
    pub method: String,
    pub ok: bool,
    pub message: String,
    pub bytes_processed: u64,
}

pub fn write_plan_file(path: &Path, job_id: &str, target: &PlannedTarget) -> Result<(), String> {
    if helper_must_refuse(target.media_class.as_key(), &target.bus, "") {
        return Err(
            "Attached USB or removable media cannot be sanitised by this application.".to_string(),
        );
    }
    if !target.allowed || target.method == MethodClass::Refused {
        return Err(target
            .refuse_reason
            .clone()
            .unwrap_or_else(|| "Mode S refused this volume.".to_string()));
    }
    let body = format!(
        "job_id={}\nletter={}\ndisk_index={}\nmethod={}\nsize_bytes={}\nserial={}\nmodel={}\nmedia_class={}\nbus={}\n",
        job_id,
        target.letter,
        target.disk_index,
        method_key(target.method),
        target.size_bytes,
        target.serial.replace('\n', " "),
        target.model.replace('\n', " "),
        target.media_class.as_key(),
        target.bus.replace('\n', " "),
    );
    fs::write(path, body).map_err(|_| "CYVRA could not write the purge plan.".to_string())
}

pub fn helper_must_refuse(media_class: &str, bus: &str, drive_kind: &str) -> bool {
    let media = media_class.to_ascii_lowercase();
    let bus_l = bus.to_ascii_lowercase();
    let kind = drive_kind.to_ascii_lowercase();
    media.contains("usb")
        || media == "usb_hdd"
        || media == "usb_flash"
        || bus_l.contains("usb")
        || kind == "removable"
        || kind == "optical"
        || kind == "network"
        || matches!(
            media.as_str(),
            "optical" | "network" | "system_disk" | "unknown"
        )
}

pub fn read_result_file(path: &Path) -> Result<HelperResult, String> {
    let body = fs::read_to_string(path)
        .map_err(|_| "The purge helper did not write a result. The job is FAILED.".to_string())?;
    parse_result(&body)
}

fn parse_result(body: &str) -> Result<HelperResult, String> {
    let mut job_id = String::new();
    let mut letter = String::new();
    let mut method = String::new();
    let mut ok = false;
    let mut message = String::new();
    let mut bytes_processed = 0_u64;
    for line in body.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key {
                "job_id" => job_id = value.to_string(),
                "letter" => letter = value.to_string(),
                "method" => method = value.to_string(),
                "ok" => ok = value == "true",
                "message" => message = value.to_string(),
                "bytes_processed" => {
                    bytes_processed = value.parse().unwrap_or(0);
                }
                _ => {}
            }
        }
    }
    if job_id.is_empty() {
        return Err("Purge helper result was not readable.".to_string());
    }
    Ok(HelperResult {
        job_id,
        letter,
        method,
        ok,
        message,
        bytes_processed,
    })
}

fn method_key(method: MethodClass) -> &'static str {
    match method {
        MethodClass::OverwriteClear => "overwrite_clear",
        MethodClass::AtaSecureErase => "ata_secure_erase",
        MethodClass::NvmeSanitize => "nvme_sanitize",
        MethodClass::Refused => "refused",
    }
}

pub fn run_helper_plan(plan_path: &Path, result_path: &Path) -> i32 {
    match run_helper_plan_inner(plan_path, result_path) {
        Ok(()) => 0,
        Err(message) => {
            let _ = fs::write(
                result_path,
                format!(
                    "job_id=unknown\nletter=\nmethod=\nok=false\nmessage={message}\nbytes_processed=0\n"
                ),
            );
            1
        }
    }
}

fn run_helper_plan_inner(plan_path: &Path, result_path: &Path) -> Result<(), String> {
    let body = fs::read_to_string(plan_path)
        .map_err(|_| "Purge helper could not read the plan.".to_string())?;
    let mut job_id = String::new();
    let mut letter = String::new();
    let mut method = String::new();
    let mut size_bytes = 0_u64;
    let mut media_class = String::new();
    let mut bus = String::new();
    for line in body.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key {
                "job_id" => job_id = value.to_string(),
                "letter" => letter = value.to_string(),
                "method" => method = value.to_string(),
                "size_bytes" => size_bytes = value.parse().unwrap_or(0),
                "media_class" => media_class = value.to_string(),
                "bus" => bus = value.to_string(),
                _ => {}
            }
        }
    }
    if letter.is_empty() || job_id.is_empty() {
        return Err("Purge plan is incomplete.".to_string());
    }
    let live_kind = live_drive_kind(&letter);
    if helper_must_refuse(&media_class, &bus, &live_kind) {
        return Err(
            "Attached USB or removable media cannot be sanitised by this application.".to_string(),
        );
    }
    if method == "ata_secure_erase" || method == "nvme_sanitize" {
        let message = "Firmware sanitize is not available on this controller. Mode S fails closed rather than calling host overwrite Purge on flash.";
        fs::write(
            result_path,
            format!(
                "job_id={job_id}\nletter={letter}\nmethod={method}\nok=false\nmessage={message}\nbytes_processed=0\n"
            ),
        )
        .ok();
        return Ok(());
    }
    if method != "overwrite_clear" {
        return Err("Purge helper refused an unknown method.".to_string());
    }
    let processed = overwrite_volume(&letter, size_bytes)?;
    fs::write(
        result_path,
        format!(
            "job_id={job_id}\nletter={letter}\nmethod=overwrite_clear\nok=true\nmessage=Single-pass overwrite completed.\nbytes_processed={processed}\n"
        ),
    )
    .map_err(|_| "Purge helper could not write the result.".to_string())
}

fn overwrite_volume(letter: &str, size_bytes: u64) -> Result<u64, String> {
    #[cfg(not(windows))]
    {
        let _ = (letter, size_bytes);
        Err("The purge helper only runs on Windows.".to_string())
    }
    #[cfg(windows)]
    {
        overwrite_volume_windows(letter, size_bytes)
    }
}

#[cfg(windows)]
fn overwrite_volume_windows(letter: &str, size_bytes: u64) -> Result<u64, String> {
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};

    let path = format!(r"\\.\{letter}:");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|_| {
            "CYVRA could not open the selected volume for Mode S. Confirm it is not the system disk and that you accepted the elevation prompt."
                .to_string()
        })?;
    let length = if size_bytes > 0 {
        size_bytes
    } else {
        file.seek(SeekFrom::End(0))
            .map_err(|_| "CYVRA could not measure the selected volume.".to_string())?
    };
    if length == 0 {
        return Err("Selected volume reports zero size.".to_string());
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| "CYVRA could not rewind the selected volume.".to_string())?;
    let chunk = vec![0_u8; 1024 * 1024];
    let mut written = 0_u64;
    while written < length {
        let remain = (length - written) as usize;
        let take = remain.min(chunk.len());
        file.write_all(&chunk[..take])
            .map_err(|_| "CYVRA could not finish the overwrite. The job is FAILED.".to_string())?;
        written += take as u64;
    }
    file.flush()
        .map_err(|_| "CYVRA could not flush the overwrite.".to_string())?;
    Ok(written)
}

fn live_drive_kind(letter: &str) -> String {
    let key = letter.trim().trim_end_matches(':').to_ascii_uppercase();
    crate::volume::collect()
        .into_iter()
        .find(|item| {
            item.drive_letter
                .trim()
                .trim_end_matches(':')
                .eq_ignore_ascii_case(&key)
        })
        .map(|item| item.drive_kind)
        .unwrap_or_default()
}

pub fn sample_volume(
    letter: &str,
    size_bytes: u64,
    samples: u32,
) -> Result<super::verify::VerifyReport, String> {
    #[cfg(not(windows))]
    {
        let _ = (letter, size_bytes, samples);
        Err("Independent verify of Mode S media runs only on Windows.".to_string())
    }
    #[cfg(windows)]
    {
        sample_volume_windows(letter, size_bytes, samples)
    }
}

#[cfg(windows)]
fn sample_volume_windows(
    letter: &str,
    size_bytes: u64,
    samples: u32,
) -> Result<super::verify::VerifyReport, String> {
    use super::verify::{inspect_buffer, residue_ok, summarise, MARKER};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom};

    let path = format!(r"\\.\{letter}:");
    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)
        .map_err(|_| "Independent verify could not open the volume.".to_string())?;
    let length = if size_bytes > 0 {
        size_bytes
    } else {
        file.seek(SeekFrom::End(0))
            .map_err(|_| "Independent verify could not measure the volume.".to_string())?
    };
    if length < 4096 {
        return Err("Volume is too small to verify.".to_string());
    }
    let count = samples.max(8);
    let mut findings = Vec::new();
    let mut buffer = vec![0_u8; 4096];
    for index in 0..count {
        let offset = ((index as u64).saturating_mul(length / count as u64)).min(length - 4096);
        file.seek(SeekFrom::Start(offset))
            .map_err(|_| "Independent verify could not seek.".to_string())?;
        file.read_exact(&mut buffer)
            .map_err(|_| "Independent verify could not read a sample.".to_string())?;
        let (hits, _) = inspect_buffer(&buffer);
        findings.push((
            hits,
            residue_ok(&buffer) && !buffer.windows(MARKER.len()).any(|w| w == MARKER),
        ));
    }
    Ok(summarise(&findings))
}

#[cfg(test)]
mod tests {
    use super::read_result_file;
    use crate::purge::verify::{residue_ok, MARKER};
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;

    fn temp(name: &str) -> PathBuf {
        env::temp_dir().join(format!("cyvra-purge-test-{name}"))
    }

    #[test]
    fn result_round_trip() {
        let path = temp("result.txt");
        fs::write(
            &path,
            "job_id=abc\nletter=E\nmethod=overwrite_clear\nok=true\nmessage=done\nbytes_processed=10\n",
        )
        .unwrap();
        let parsed = read_result_file(&path).unwrap();
        assert!(parsed.ok);
        assert_eq!(parsed.letter, "E");
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn file_overwrite_clears_marker() {
        let path = temp("volume.bin");
        let mut data = vec![b'A'; 64 * 1024];
        data[32..32 + MARKER.len()].copy_from_slice(MARKER);
        fs::write(&path, &data).unwrap();
        {
            let mut file = File::create(&path).unwrap();
            file.write_all(&vec![0_u8; data.len()]).unwrap();
        }
        let read_back = fs::read(&path).unwrap();
        assert!(!read_back
            .windows(MARKER.len())
            .any(|window| window == MARKER));
        assert!(residue_ok(&read_back));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn helper_refuses_usb_even_if_plan_asks() {
        assert!(super::helper_must_refuse("usb_hdd", "SATA", "internal"));
        assert!(super::helper_must_refuse("magnetic_hdd", "USB", "internal"));
        assert!(super::helper_must_refuse(
            "magnetic_hdd",
            "SATA",
            "removable"
        ));
        assert!(!super::helper_must_refuse(
            "magnetic_hdd",
            "SATA",
            "internal"
        ));
        assert!(!super::helper_must_refuse("nvme", "NVMe", "internal"));
    }
}
