# A2: Battery collection, live progress, and Report D download

A2 is the first Advance scan collection slice. It reads battery capacity from
every source Windows offers, shows live progress while it works, and lets the
operator save Report D as a PDF.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations.

## What A2 adds

| Layer | Addition |
| --- | --- |
| Probe | `agent-windows/src/battery_probe.rs` — bounded PowerShell/CIM plus `powercfg /batteryreport` fallback |
| Engine | `diagnostics.rs` scores Battery and power from measured wear; other domains stay not assessable |
| Progress | `advance-scan-progress` events, one stage per subsystem, honest detail per stage |
| UI | Violet progress ring with live percent and the exact stage being read |
| Report D | Per-domain coverage table, method and rubric, temporary-file disclosure, Save as PDF |

## Why not a raw IOCTL crate

The design asked for `windows-sys` IOCTL as the primary path. A2 instead uses
the existing bounded PowerShell runtime (`run_fixed_powershell`) for three
reasons that keep the report honest:

1. The same transport already proved itself on Report A. Parsing and derivation
   stay fixture-tested off Windows.
2. `Win32_Battery` and the `root/WMI` capacity classes are the documented
   user-mode surface over the same battery firmware the IOCTL would read.
   When those classes withhold design capacity, Windows' own
   `powercfg /batteryreport /xml` is consulted and then deleted.
3. A thin IOCTL shim cannot be executed on the Linux build host. Shipping an
   untested syscall would hide failures. The probe records *why* each source
   failed instead.

`windows-sys` stays available for A4 (elevated SMART) if a syscall is then
required. A2 does not add that dependency.

## Battery rules

- Wear is `1 − (full charge / design capacity)`, and only when both numbers
  are real and the ratio is sane (5%–150%).
- A charge level is never reused as health. The report labels it
  "charge level, not health".
- A desktop with no battery is **not applicable**, so the 20 points leave the
  denominator. A laptop that reports no pack is **not assessable**.
- Relative (unitless) firmware capacities are printed without an mWh label.
- One temporary XML file, if written, is disclosed on Report D and is not
  counted as bytes written to an assessed drive.

## Progress

Eleven stages, reported as `(percent, stage index, stage, detail)`:

Preparing → battery → processor → memory → storage → ports → display →
cameras → benchmarks → scoring → Report D.

The shell draws a bright circular indicator in violet / fuchsia, distinct
from the basic assessment's blue bar, and lists every stage with its
current state.

## Report D

Document number `ERD-YYYYMMDD-HOST`. Device identity is copied from the
basic assessment when one has run. Coverage is printed both as a statement
and as a six-row table (awarded / assessed / not assessable / weight).
Method, rubric CG-1.0, gaps, and the operator signature sit at the end.
**Save Report D as PDF** writes `CYVRA-Erase-diagnostic-ERD-….pdf`.

The grade remains **Graded by CYVRA Grading Engine**. With only battery
scored, coverage is 20% on a laptop and the grade stays withheld because
storage is mandatory and unread. That is intended.

## Still off

- Cameras, USB topology, SMART, EDID, radios (A3–A5)
- Consent-gated benchmarks and interactive checks (A6–A7)
- Issuance, cloud authentication, kernel sensors
- Destructive operations, WinPE, Authenticode, unsigned B2 upload
