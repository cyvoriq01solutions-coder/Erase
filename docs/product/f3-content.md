# F3: World-class customer copy and Report A/D structure

Taken after F2 merged (`327488b`, PR #61). Content only.

Wipe, B2 and Authenticode stay out.
`grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.
Live USB / live charger overlays stay removed.

## What F3 changes

Customer screens use the polished wording pack (Welcome through
Save Report). Report A PDF follows the intake / pre-sanitization
section order. Report D PDF follows the technical diagnostic
section order. Fields we do not collect stay “Not recorded in this
version”.

## Guards

- Five destinations
- Seven application commands
- No new collectors
- No invented numbers or custody data
- No “Authenticated by CYVORIQ”
- Purge stays fail-closed
