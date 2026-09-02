# A5: Display panel and radios

A5 is the display-and-radio Advance scan slice. It reads panel identity and
native resolution from EDID, plus Wi-Fi, Bluetooth and Ethernet adapter
state, then prints them on Report D. It does not guess HDR, does not print
a MAC address, and does not award Screen-domain points.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.

## What A5 adds

| Layer | Addition |
| --- | --- |
| Probe | `agent-windows/src/display_radio.rs` — one bounded PowerShell script: `WmiMonitorID`, `WmiMonitorRawEEdidV1Block`, `Win32_VideoController`, `Get-NetAdapter`, `netsh wlan show interfaces`, `Get-PnpDevice -Class Bluetooth` |
| Engine | Ports domain can now score **up to 6 of 10**: USB topology 2 + Wi-Fi 2 + Bluetooth 1 + Ethernet 1. Physical insertion (4) stays A7 |
| Report D | Native resolution from the EDID preferred timing, current desktop mode printed separately, Wi-Fi / Bluetooth / Ethernet without MAC addresses |
| Progress | Stage “Reading ports and connectivity” names radios; stage “Reading display panel” names EDID preferred timing |

## Why one PowerShell script

A4’s Windows agent tests failed when several large PowerShell collectors ran
next to process-limit tests. A5 therefore uses **one** combined display+radio
script, not a sixth collector process. The agent workflow already runs
`cargo test -- --test-threads=1`. Do not add more parallel `powershell.exe`
tests.

Slice design mentioned `display_edid.rs` plus `radio.rs` and a UAC helper.
A5 keeps the A2–A4 pattern: one bounded, application-owned PowerShell script
through `run_fixed_powershell`. The unsigned installer must not pop UAC.
MAC-like strings are dropped in PowerShell and again in Rust.

## Scoring (rubric CG-1.0)

Printed: **Graded by CYVRA Grading Engine**, rubric CG-1.0.

Ports and connectivity (10):

- USB controller topology enumerated → 2
- Wi-Fi adapter present and reporting state or signal → 2
- Bluetooth radio present → 1
- Ethernet link state readable (a disconnected cable still counts) → 1
- Physical insertion → 4, still technician-only (A7)

Display identity is printed. Screen-domain points stay **0** until a
technician attests a colour wash (A7). HDR is not inferred from the current
desktop colour profile.

On a laptop with a healthy battery, enumerated USB, a healthy NVMe, and
Wi-Fi + Bluetooth + Ethernet, coverage is about **46%**. The grade stays
withheld (CG-1.0 floor is 70%).

## Still off

- Consent-gated benchmarks (A6)
- Interactive technician checks and physical port insertion (A7)
- Grading issuance (A8)
- Live camera preview and microphone record (A10)
- Issuance, cloud authentication, kernel sensors
- Destructive operations, WinPE, Authenticode, unsigned B2 upload
- CYVRA Purge / Report B (plan only: see the stored ITAD wipe guidance)
