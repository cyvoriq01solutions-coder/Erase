# S-setup: Software License Terms

Taken after WINDOWS F6 OK (`03B6F609`, merge `e3bf4cb`, PR #65).

Wipe, B2 and Authenticode stay out.
`grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.
Live USB / live charger overlays stay removed.

## What S-setup changes

Welcome keeps a short summary. The Terms step and the installer
licence page now show the same commercial Software License Terms:
grant, one authorised device, website login is not a licence,
assessment-only, no erase, unsigned until Authenticode, key from
auth@cyvra.co.in, disclaimer, and limitation of liability.

The installer page no longer says “not a customer release”. Unsigned
status is stated in section 2 of the Terms.

## Guards

- Five destinations
- Seven application commands
- No wipe, B2, Authenticode, or live USB/charger overlays
- No “Authenticated by CYVORIQ”
