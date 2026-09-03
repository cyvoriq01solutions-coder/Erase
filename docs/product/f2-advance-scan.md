# F2: Remove live USB/charger overlays, three workstreams

Taken after WINDOWS F1 OK (`308BB157`, main `27741ce`).

Live USB insertion and live charger overlays froze some laptops: each
poll started a new command window, polls overlapped, and the desktop
filled until Windows locked. Those buttons are removed. USB topology
and battery/charger state stay on Report D from the one-shot Advance
scan collectors.

Home after installation is three coloured workstreams:

1. Standard assessment → Report A → Back to main
2. Advance diagnostic → Report D → Back to main
3. Wipe (fail-closed) → wipe report → Back to main

The CYVORIQ logo is larger in the title bar and first-run setup.
`grading_issuance_enabled` stays false.
`report_authentication_enabled` stays false.
Wipe, B2 and Authenticode stay out.

## What F2 changes

| Layer | Change |
| --- | --- |
| UI | No Check USB ports, no USB 1–4 live ticks, no live charger overlay |
| UI | Three coloured workstream cards on Overview |
| UI | Report screen is three coloured sections, each with Back to main |
| UI | Larger logo (title bar and setup) |
| Collectors | Hidden command window on one-shot PowerShell (`CREATE_NO_WINDOW`) |
| Report D | USB/battery from Advance scan pass; physical-port ticks stay not attempted |
| Help / guide | USB and charger are described as scan telemetry, not live buttons |

## Why live USB/charger froze the PC

Interactive Checks polled `probe_live_intake` every 1.5s without waiting
for the previous call. Each tick started a new command process (volume
list, USB speed associates, battery status). Timeout is 12s, so processes
overlapped. The runner did not hide the console, so Windows opened a
visible command window per tick.

Advance scan collectors use the same runner once per stage, in series.
Those one-shot reads stay.

## Guards

- Five destinations
- Still seven application commands (`probe_live_intake` unused by the UI)
- `desktop/src-tauri/src/lib.rs` still has no `std::fs` / `std::process`
- No keystroke log, no stored snapshot, no write to USB sticks
- Charging is telemetry, not a CG-1.0 domain
- Issuance stays off
- Purge execution stays off

## Still off

- Cloud authentication / Authenticode / unsigned object-store upload
- Purge execution
- Live USB insertion and live charger overlays (intentionally removed)
