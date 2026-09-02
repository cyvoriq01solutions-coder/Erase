# A4: Storage identity and SMART

A4 is the storage-health Advance scan slice. It reads disk identity and the
SMART / reliability telemetry an ITAD buyer actually needs, then prints it on
Report D. It does not erase, TRIM, format, sanitize or update firmware.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.

## What A4 adds

| Layer | Addition |
| --- | --- |
| Probe | `agent-windows/src/storage_health.rs` — `Win32_DiskDrive`, `Get-PhysicalDisk`, `Get-StorageReliabilityCounter`, SMART predict-failure and ATA attributes 5 / 9 / 12 / 197 |
| Engine | Storage health scores **up to 20 of 20** when a scoring path exists (NVMe wear or ATA realloc/pending). Predict-failure, NVMe critical warning or pending sectors force **0** and a measured F |
| Report D | Model, serial, bus, power-on hours, percentage used, spare when Windows returns it, predicted failure. TBW stays not collected when Windows does not expose it |
| Progress | Stage “Reading storage identity and health” names that nothing is written, erased or trimmed |

## Why this is not an elevated helper yet

Slice design mentioned `cyvra-advance-probe.rs` and a UAC prompt. A4 keeps the
A2/A3 pattern: one bounded, application-owned PowerShell script through
`run_fixed_powershell`. The unsigned installer must not pop UAC (Authenticode
is still frozen). If Windows refuses a class, Report D prints **Refused without
administrator rights**, storage stays not assessable, and the scan still
finishes.

The script is allow-listed in a unit test so it cannot name Secure Erase,
Sanitize, Format-Volume, Clear-Disk, TRIM or `IOCTL_STORAGE_PROTOCOL_COMMAND`.

## Scoring (rubric CG-1.0)

Printed: **Graded by CYVRA Grading Engine**, rubric CG-1.0.

NVMe / SSD wear:

- percentage used ≤ 5 **and** spare ≥ 95 → 20
- percentage used ≤ 20 (spare ≥ 90 when spare is present) → 16
- Missing spare is **never** assumed to be 100%, so a healthy wear figure without spare caps at 16
- percentage used > 80, spare < 50, media errors or a critical warning → 0

ATA:

- realloc 0 and pending 0 → 20
- realloc 1–10 and pending 0 → 11
- pending > 0 or realloc > 10 → 0

A measured predict-failure, NVMe critical warning or pending-sector count
forces grade **F** even when coverage is still below the 70% floor. Missing
SMART still **withholds** the grade. Those are different things.

On a laptop with a healthy battery, enumerated USB and a healthy NVMe,
coverage is about **42%**. The grade stays withheld until later slices fill
processor, memory, display and technician checks.

USB sticks are ignored for scoring when an internal disk is scorable.

## Still off

- Display EDID and radios (A5)
- Consent-gated benchmarks and interactive checks (A6–A7)
- Live camera preview and microphone record (A10)
- Issuance, cloud authentication, kernel sensors
- Destructive operations, WinPE, Authenticode, unsigned B2 upload
- CYVRA Purge / Report B (plan only: see the stored ITAD wipe guidance)
