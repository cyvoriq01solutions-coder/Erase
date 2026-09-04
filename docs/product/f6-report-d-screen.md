# F6: On-screen Report D numbered 1–20

Taken after WINDOWS F5 OK (`08631AF0`, merge `05c63ce`, PR #64).

Wipe, B2 and Authenticode stay out.
`grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.
Live USB / live charger overlays stay removed.

## What F6 changes

The PDF already printed Report D sections 1–20. The on-screen Report
had coverage, the grade card, the local seal and telemetry tables, but
not the numbered letterhead sections. F6 uses one shared builder so the
screen and the PDF print the same numbered sections. Grade card and
Verify this report stay. Report S is named only, not generated.

## Guards

- Five destinations
- Seven application commands
- No new collectors
- No wipe, B2, Authenticode, or live USB/charger overlays
- No “Authenticated by CYVORIQ”
