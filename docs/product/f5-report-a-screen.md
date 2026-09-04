# F5: On-screen Report A matches the 14-section template

Taken after WINDOWS F4 OK (`9C9EE05B`, merge `3be9a17`, PR #63).

Wipe, B2 and Authenticode stay out.
`grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.
Live USB / live charger overlays stay removed.

## What F5 changes

The PDF already printed Report A sections 1–14. The on-screen Report
only showed the snapshot plus sections 5–7. F5 uses one shared builder
so the screen and the PDF print the same 14 sections. Custody fields
stay “Not recorded in this version”. Report S is named only, not generated.

## Guards

- Five destinations
- Seven application commands
- No new collectors
- No wipe, B2, Authenticode, or live USB/charger overlays
- No “Authenticated by CYVORIQ”
