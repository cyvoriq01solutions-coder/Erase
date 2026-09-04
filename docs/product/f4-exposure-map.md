# F4: Privacy exposure map (where + what)

Taken after WINDOWS F3 OK (`5C592AF0`, merge `5807882`, PR #62).

Wipe, B2 and Authenticode stay out.
`grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.
Live USB / live charger overlays stay removed.

## What F4 changes

Standard Assessment already mapped approved folders and file types
without opening contents. The GUI only showed counts. F4 prints the
map: folder path, type, file count, size, and classification.

Results shows the table. Report A section 7 includes the same rows.
Individual file names are not recorded. Contents stay unopened.

## Guards

- Five destinations
- Seven application commands
- No new wipe, B2, Authenticode, or live USB/charger overlays
- No `std::fs` / `std::process` in `desktop/src-tauri/src/lib.rs`
- No “Authenticated by CYVORIQ”
