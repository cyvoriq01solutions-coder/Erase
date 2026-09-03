//! Local Report D integrity seal (A9).
//!
//! Canonical JSON of the customer-visible diagnostic fields is hashed with
//! SHA-256 and signed with an ephemeral Ed25519 key generated on this PC for
//! this scan. The public key travels with the report, so anyone can check
//! that the JSON was not altered. That is integrity, not identity: it does
//! not prove CYVORIQ issued the document, is not cloud-authenticated, and is
//! not Authenticode.
//!
//! `grading_issuance_enabled` and `report_authentication_enabled` stay false.

use crate::NamedValue;
use crate::diagnostics::{CustomerAdvanceScan, DomainCoverageRow, TelemetryGroup};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use qrcode::QrCode;
use qrcode::render::svg::Color as SvgColor;
use rand_core::OsRng;
use sha2::{Digest, Sha256};

/// Printed on Report D next to the QR. Honest about what the seal is not.
pub const LOCAL_SEAL_NOTICE: &str = "Local integrity seal. This proves the Report D JSON was not altered after this scan on this PC. It is not cloud-authenticated, not Authenticode, and not a CYVORIQ certificate.";

pub const SEAL_SCHEME: &str = "cyvra-erd-ed25519-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegritySeal {
    pub scheme: &'static str,
    pub digest_hex: String,
    pub public_key_hex: String,
    pub signature_hex: String,
    pub qr_payload: String,
    pub qr_svg: String,
    pub canonical_json: String,
    pub notice: &'static str,
}

/// Compact JSON of the documentary Report D fields. The seal itself is omitted
/// so the encoding cannot depend on the signature it is about to produce.
#[must_use]
pub fn canonical_json(scan: &CustomerAdvanceScan) -> String {
    let mut out = String::from("{");
    append_pair(
        &mut out,
        "schemaVersion",
        json_str(scan.schema_version),
        true,
    );
    append_pair(
        &mut out,
        "elevationState",
        json_str(scan.elevation_state),
        false,
    );
    append_pair(
        &mut out,
        "elevationLabel",
        json_str(scan.elevation_label),
        false,
    );
    append_pair(
        &mut out,
        "benchmarksConsented",
        json_bool(scan.benchmarks_consented),
        false,
    );
    append_pair(
        &mut out,
        "writeBenchmarkConsented",
        json_bool(scan.write_benchmark_consented),
        false,
    );
    append_pair(
        &mut out,
        "bytesWritten",
        json_u64(scan.bytes_written),
        false,
    );
    append_pair(
        &mut out,
        "destructiveOperationsEnabled",
        json_bool(scan.destructive_operations_enabled),
        false,
    );
    append_pair(
        &mut out,
        "contentInspected",
        json_bool(scan.content_inspected),
        false,
    );
    append_pair(
        &mut out,
        "boundaryNote",
        json_str(&scan.boundary_note),
        false,
    );
    append_pair(
        &mut out,
        "temporaryFilesNote",
        json_str(&scan.temporary_files_note),
        false,
    );
    append_pair(
        &mut out,
        "coveragePercent",
        json_u64(u64::from(scan.coverage_percent)),
        false,
    );
    append_pair(
        &mut out,
        "indexPercent",
        json_opt_u32(scan.index_percent),
        false,
    );
    append_pair(&mut out, "provisional", json_bool(scan.provisional), false);
    append_pair(&mut out, "gradeLabel", json_str(scan.grade_label), false);
    append_pair(
        &mut out,
        "gradeCondition",
        json_str(scan.grade_condition),
        false,
    );
    append_pair(
        &mut out,
        "gradeObservation",
        json_opt_str(scan.grade_observation),
        false,
    );
    append_pair(
        &mut out,
        "gradeWithheld",
        json_bool(scan.grade_withheld),
        false,
    );
    append_pair(
        &mut out,
        "gradeWithheldReason",
        json_opt_owned(scan.grade_withheld_reason.as_deref()),
        false,
    );
    append_pair(
        &mut out,
        "gradingEngine",
        json_str(scan.grading_engine),
        false,
    );
    append_pair(
        &mut out,
        "gradingRubric",
        json_str(scan.grading_rubric),
        false,
    );
    append_pair(
        &mut out,
        "issuanceNotice",
        json_str(scan.issuance_notice),
        false,
    );
    append_pair(
        &mut out,
        "coverageRows",
        json_named_values(&scan.coverage_rows),
        false,
    );
    append_pair(
        &mut out,
        "coverageDomains",
        json_domains(&scan.coverage_domains),
        false,
    );
    append_pair(
        &mut out,
        "telemetryGroups",
        json_groups(&scan.telemetry_groups),
        false,
    );
    append_pair(
        &mut out,
        "notAssessable",
        json_string_array(&scan.not_assessable),
        false,
    );
    append_pair(
        &mut out,
        "methodRows",
        json_named_values(&scan.method_rows),
        false,
    );
    append_pair(
        &mut out,
        "rubricRows",
        json_named_values(&scan.rubric_rows),
        false,
    );
    out.push('}');
    out
}

/// Hash, sign and QR-encode one Report D payload.
#[must_use]
pub fn seal(scan: &CustomerAdvanceScan) -> IntegritySeal {
    let canonical = canonical_json(scan);
    let digest = Sha256::digest(canonical.as_bytes());
    let digest_hex = to_hex(&digest);
    let signing_key = SigningKey::generate(&mut OsRng);
    let signature = signing_key.sign(canonical.as_bytes());
    let public_key_hex = to_hex(signing_key.verifying_key().as_bytes());
    let signature_hex = to_hex(&signature.to_bytes());
    let qr_payload = format!("CYVRA-ERD:1:{digest_hex}");
    IntegritySeal {
        scheme: SEAL_SCHEME,
        digest_hex,
        public_key_hex,
        signature_hex,
        qr_svg: qr_svg(&qr_payload),
        qr_payload,
        canonical_json: canonical,
        notice: LOCAL_SEAL_NOTICE,
    }
}

/// Re-check digest and Ed25519 over the stored canonical JSON.
#[must_use]
pub fn verify(seal: &IntegritySeal) -> bool {
    verify_detail(seal).is_ok()
}

pub fn verify_detail(seal: &IntegritySeal) -> Result<(), &'static str> {
    if seal.scheme != SEAL_SCHEME {
        return Err("unknown seal scheme");
    }
    let digest = Sha256::digest(seal.canonical_json.as_bytes());
    if to_hex(&digest) != seal.digest_hex {
        return Err("digest does not match canonical JSON");
    }
    let public_key = decode_hex(&seal.public_key_hex).ok_or("public key is not hex")?;
    let signature = decode_hex(&seal.signature_hex).ok_or("signature is not hex")?;
    let pk_bytes: [u8; 32] = public_key
        .as_slice()
        .try_into()
        .map_err(|_| "public key must be 32 bytes")?;
    let sig_bytes: [u8; 64] = signature
        .as_slice()
        .try_into()
        .map_err(|_| "signature must be 64 bytes")?;
    let verifying_key =
        VerifyingKey::from_bytes(&pk_bytes).map_err(|_| "public key is not a valid Ed25519 key")?;
    let signature = Signature::from_bytes(&sig_bytes);
    verifying_key
        .verify(seal.canonical_json.as_bytes(), &signature)
        .map_err(|_| "signature does not match this report")?;
    Ok(())
}

fn qr_svg(payload: &str) -> String {
    match QrCode::new(payload.as_bytes()) {
        Ok(code) => code
            .render::<SvgColor>()
            .min_dimensions(168, 168)
            .dark_color(SvgColor("#0b1f3a"))
            .light_color(SvgColor("#ffffff"))
            .build(),
        Err(_) => "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 168 168\"><rect width=\"168\" height=\"168\" fill=\"#ffffff\"/><text x=\"8\" y=\"84\" font-size=\"9\" fill=\"#0b1f3a\">QR unavailable</text></svg>".to_string(),
    }
}

fn append_pair(out: &mut String, key: &str, value: String, first: bool) {
    if !first {
        out.push(',');
    }
    out.push_str(&json_str(key));
    out.push(':');
    out.push_str(&value);
}

fn json_bool(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn json_u64(value: u64) -> String {
    value.to_string()
}

fn json_opt_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_string(), |n| n.to_string())
}

fn json_opt_str(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_string(), json_str)
}

fn json_opt_owned(value: Option<&str>) -> String {
    json_opt_str(value)
}

fn json_str(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_named_values(rows: &[NamedValue]) -> String {
    let mut out = String::from("[");
    for (index, row) in rows.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('{');
        out.push_str("\"label\":");
        out.push_str(&json_str(&row.label));
        out.push_str(",\"value\":");
        out.push_str(&json_str(&row.value));
        out.push('}');
    }
    out.push(']');
    out
}

fn json_string_array(items: &[String]) -> String {
    let mut out = String::from("[");
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&json_str(item));
    }
    out.push(']');
    out
}

fn json_domains(domains: &[DomainCoverageRow]) -> String {
    let mut out = String::from("[");
    for (index, domain) in domains.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('{');
        append_pair(&mut out, "domain", json_str(&domain.domain), true);
        append_pair(
            &mut out,
            "awarded",
            json_u64(u64::from(domain.awarded)),
            false,
        );
        append_pair(
            &mut out,
            "assessed",
            json_u64(u64::from(domain.assessed)),
            false,
        );
        append_pair(
            &mut out,
            "notAssessable",
            json_u64(u64::from(domain.not_assessable)),
            false,
        );
        append_pair(
            &mut out,
            "weight",
            json_u64(u64::from(domain.weight)),
            false,
        );
        append_pair(&mut out, "state", json_str(&domain.state), false);
        append_pair(&mut out, "confidence", json_str(&domain.confidence), false);
        append_pair(&mut out, "note", json_str(&domain.note), false);
        out.push('}');
    }
    out.push(']');
    out
}

fn json_groups(groups: &[TelemetryGroup]) -> String {
    let mut out = String::from("[");
    for (index, group) in groups.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('{');
        append_pair(&mut out, "title", json_str(&group.title), true);
        append_pair(
            &mut out,
            "note",
            json_opt_owned(group.note.as_deref()),
            false,
        );
        append_pair(&mut out, "rows", json_named_values(&group.rows), false);
        out.push('}');
    }
    out.push(']');
    out
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::AdvanceScanRequest;

    #[test]
    fn a_fresh_scan_seal_verifies_and_tampering_fails() {
        let scan = crate::diagnostics::run_advance_scan(&AdvanceScanRequest::default());
        let mut sealed = scan.clone();
        let seal = seal(&scan);
        assert_eq!(seal.scheme, SEAL_SCHEME);
        assert_eq!(seal.digest_hex.len(), 64);
        assert_eq!(seal.public_key_hex.len(), 64);
        assert_eq!(seal.signature_hex.len(), 128);
        assert!(seal.qr_payload.starts_with("CYVRA-ERD:1:"));
        assert!(seal.qr_svg.contains("<svg"));
        assert!(seal.notice.contains("not cloud-authenticated"));
        assert!(seal.notice.contains("not Authenticode"));
        assert!(verify(&seal));
        assert_eq!(canonical_json(&scan), seal.canonical_json);

        sealed.boundary_note.push_str(" tampered");
        let tampered = IntegritySeal {
            canonical_json: canonical_json(&sealed),
            digest_hex: seal.digest_hex.clone(),
            ..seal.clone()
        };
        assert!(!verify(&tampered));
    }

    #[test]
    fn digest_is_sha256_of_canonical_json() {
        let scan = crate::diagnostics::run_advance_scan(&AdvanceScanRequest::default());
        let json = canonical_json(&scan);
        let digest = to_hex(&Sha256::digest(json.as_bytes()));
        let sealed = seal(&scan);
        assert_eq!(sealed.digest_hex, digest);
        assert_eq!(canonical_json(&scan), json);
    }
}
