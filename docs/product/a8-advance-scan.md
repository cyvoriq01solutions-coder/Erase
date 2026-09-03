# A8: Report D grading card

A8 polishes the CG-1.0 grading card on Report D. The arithmetic already
ships. This slice makes the card honest about coverage, confidence, and
issuance. `grading_issuance_enabled` stays **false**. Every grade is
provisional.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.

## What A8 adds

| Layer | Addition |
| --- | --- |
| Engine | `ISSUANCE_NOTICE`, software-observed tag on banded grades, confidence on each domain row |
| Report D | Coverage statement **precedes** the grade card |
| Grade card | "This is not an issued CYVORIQ grading certificate." Physical verification required |
| PDF | Identity, coverage, then grading summary. Confidence printed per area |
| Rubric table | Issuance row |

## Printed (customer)

- **Graded by CYVRA Grading Engine** · rubric **CG-1.0**
- Assessed Health Index only beside Coverage
- Banded grade: condition **— software-observed**
- Withheld: never rendered as F
- Issued by CYVORIQ Solutions Pvt. Ltd. Never "Certified"

## Still off

- Grading issuance (the flag stays false; A8 is the card, not a certificate)
- Report signing / QR digest (A9)
- Live camera preview and microphone record (A10)
- Authenticode, unsigned B2 upload, WinPE
- CYVRA Purge / Report B (plan only)

## Guards

- Five destinations, six Tauri commands
- `desktop/src-tauri/src/lib.rs` still has no `std::fs` / `std::process`
- No invented numbers. Absent stays absent
- No NIST / CERT-In / ADISA / DPDP certification claims
- No AI grade
