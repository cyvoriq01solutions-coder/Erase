# A9: Local Report D integrity seal

A9 attaches a **local integrity seal** to Report D after Advance scan finishes:
canonical JSON of the customer-visible diagnostic fields, SHA-256 digest,
ephemeral Ed25519 signature, QR of the digest, and an in-app verify page.

This is integrity, not identity. It proves the JSON was not altered after the
scan on this PC. It does **not** mean CYVORIQ authenticated the document, is
not cloud-authenticated, is not Authenticode, and is not a certificate.

`grading_issuance_enabled` stays **false**.
`report_authentication_enabled` stays **false**, so the product still does not
print “Authenticated by CYVORIQ Solutions Pvt. Ltd.”

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.

## What A9 adds

| Layer | Addition |
| --- | --- |
| Agent | `report_signing.rs` — canonical JSON, SHA-256, Ed25519, QR SVG |
| Report D | Local seal card with digest, QR, Verify this report |
| PDF | Scheme, digest, QR payload, public key, signature, honest notice |
| Help | Article 04 — verify on this PC; no cyvra.co.in call |

The key is generated for this scan and travels with the report. Anyone with
the JSON can check the signature. Anyone can also generate a new key, so this
is not an organisational identity.

## Printed (customer)

- **Issued by CYVORIQ Solutions Pvt. Ltd.** (publisher wording, unchanged)
- **Local integrity seal** · scheme `cyvra-erd-ed25519-v1`
- SHA-256 digest and QR payload `CYVRA-ERD:1:<digest>`
- “This is not an issued CYVORIQ grading certificate.”
- Never “Certified”. Never “Authenticated by CYVORIQ” in this slice.

## Guards

- Five destinations. Still seven Tauri commands (no eighth verify command;
  the web view re-checks SHA-256 and Ed25519)
- `desktop/src-tauri/src/lib.rs` still has no `std::fs` / `std::process`
- No upload. No cloud counter-sign. No verify.cyvra.co.in
- No invented numbers. Absent stays absent
- No NIST / CERT-In / ADISA / DPDP certification claims
- No AI grade

## Still off

- Cloud authentication / Worker counter-sign
- Authenticode, unsigned B2 upload, WinPE
- CYVRA Purge / Report B (plan only)
- Grading issuance (the flag stays false)
