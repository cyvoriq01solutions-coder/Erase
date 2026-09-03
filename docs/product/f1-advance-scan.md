# F1: USB 1–USB 4 ticks, installer polish, operator guide

F1 sits on top of A10 live intake and A9 local Report D integrity seal.
A technician ticks USB 1, USB 2, USB 3 and USB 4 because a PC can have
more than one socket. The teal **Check USB ports** button guides
“Insert a USB stick into USB 1”. Windows speed is telemetry.

Basic scan and Report A stay unchanged except the closing physical
verification block (no handwritten underscore line). Navigation still
has exactly five destinations. Wipe, B2 upload, and Authenticode stay
out. `grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.

Customer-facing copy does not name construction tools.

## What F1 adds

| Layer | Addition |
| --- | --- |
| UI | Tick boxes beside USB 1–USB 4; teal Check USB ports; guided insert; speed label |
| Live intake | Removable letter plus USB speed class when Windows reports it |
| Scoring | Four ticks derive physically verified ports. Not on this PC is not a fail |
| Report D | USB 1–USB 4 rows; Physical verification block; local seal from A9 |
| Setup | Welcome / Terms / Activate progress; CYVORIQ brand header |
| Help | Registration → Report D download with expected result and derivation |
| Guide | `docs/product/operator-guide.md` |

## USB scoring

- `absent` / Not on this PC — socket not on this chassis
- `skip` — on chassis, not attempted
- `pass` / `fail` — technician result after insert
- All on-chassis pass → `all_passed`
- Any fail → `any_failed`
- Some pass → `partial`
- None attempted → `skip`

Speed labels (telemetry only): USB 3.2 SuperSpeed+, USB 3.0 SuperSpeed,
USB 2.0 High Speed, USB 1.1 Full Speed, or Not reported by Windows.

CYVRA does not write to the stick.

## Guards

- Five destinations
- Still seven application commands (`probe_live_intake` unchanged)
- `desktop/src-tauri/src/lib.rs` still has no `std::fs` / `std::process`
- No keystroke log, no stored snapshot, no microphone recording
- No invented charging points
- Issuance stays off
- Installer licence still says assessment and not a customer release
- No NSIS / construction-tool names in licence or customer screens

## Still off

- Cloud authentication / Worker counter-sign (A9 remains a local seal)
- Authenticode, unsigned object-store upload, WinPE
- Purge execution
