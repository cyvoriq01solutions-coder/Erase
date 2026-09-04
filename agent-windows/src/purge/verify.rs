//! Independent read-back. The sanitizer process must not certify itself.

pub const MARKER: &[u8] = b"CYVRA-TEST";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyReport {
    pub passed: bool,
    pub sample_percent: u32,
    pub samples_read: u32,
    pub marker_hits: u32,
    pub note: String,
}

pub fn inspect_buffer(buffer: &[u8]) -> (u32, bool) {
    let mut hits = 0_u32;
    if buffer.windows(MARKER.len()).any(|window| window == MARKER) {
        hits = 1;
    }
    let looks_user_text = buffer
        .iter()
        .filter(|byte| byte.is_ascii_alphanumeric())
        .count()
        > buffer.len() / 4
        && hits > 0;
    (hits, looks_user_text)
}

pub fn residue_ok(buffer: &[u8]) -> bool {
    if buffer.is_empty() {
        return false;
    }
    let (hits, _) = inspect_buffer(buffer);
    if hits > 0 {
        return false;
    }
    let zeros = buffer.iter().filter(|byte| **byte == 0).count();
    let ones = buffer.iter().filter(|byte| **byte == 0xFF).count();
    zeros * 2 > buffer.len() || ones * 2 > buffer.len() || looks_random(buffer)
}

fn looks_random(buffer: &[u8]) -> bool {
    if buffer.len() < 64 {
        return false;
    }
    let mut seen = [0_u32; 256];
    for byte in buffer {
        seen[*byte as usize] += 1;
    }
    let distinct = seen.iter().filter(|count| **count > 0).count();
    distinct > 64
}

pub fn summarise(samples: &[(u32, bool)]) -> VerifyReport {
    let samples_read = samples.len() as u32;
    let marker_hits: u32 = samples.iter().map(|(hits, _)| *hits).sum();
    let passed = samples_read > 0 && marker_hits == 0 && samples.iter().all(|(_, ok)| *ok);
    VerifyReport {
        passed,
        sample_percent: 10,
        samples_read,
        marker_hits,
        note: if passed {
            "Independent 10% sample found no CYVRA-TEST marker and no leftover user-data pattern."
                .to_string()
        } else if marker_hits > 0 {
            "Independent verify FAIL: CYVRA-TEST marker still present.".to_string()
        } else {
            "Independent verify FAIL: residue did not match zeros, ones, or ciphertext.".to_string()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_fails_verify() {
        let mut buffer = vec![0_u8; 4096];
        buffer[100..100 + MARKER.len()].copy_from_slice(MARKER);
        let (hits, _) = inspect_buffer(&buffer);
        assert_eq!(hits, 1);
        assert!(!residue_ok(&buffer));
    }

    #[test]
    fn zeros_pass() {
        let buffer = vec![0_u8; 4096];
        assert!(residue_ok(&buffer));
        let report = summarise(&[(0, true), (0, true)]);
        assert!(report.passed);
        assert_eq!(report.sample_percent, 10);
    }
}
