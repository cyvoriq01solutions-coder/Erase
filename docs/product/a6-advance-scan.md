# A6: Consent-gated Advance scan workloads

A6 collects processor and memory **identity** on every Advance scan, then
runs CPU, memory and storage workloads **only** when the operator ticks
Allow benchmarks. A second tick is required before any temporary write.
It does not erase disks, does not print “memory verified”, and does not
invent package temperature.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.
`grading_issuance_enabled` stays false.

## What A6 adds

| Layer | Addition |
| --- | --- |
| Identity | `agent-windows/src/cpu_memory.rs` — one bounded PowerShell script: `Win32_Processor`, `Win32_PhysicalMemory`, `Win32_OperatingSystem` |
| Workloads | `agent-windows/src/advance_bench.rs` — in-process CPU loop, 32 MiB pattern spot check, sequential/random **read** of an existing Windows file, optional 8 MiB TEMP write |
| Engine | Processor can score identity **4** without a workload. The **16** clock points wait for a consented loop. Memory inventory **5**; pattern **7**; bandwidth **3** |
| Report D | Processor model/cores/cache, module list, benchmark statuses. Idle clock is labelled as an idle sample |
| Progress | Stages 2–3 name identity. Stage 8 names consented workloads or “none were run” |

## Why identity is a collector and benches are in-process

A4’s Windows agent tests failed when several large PowerShell collectors
ran next to process-limit tests. A6 therefore:

- adds **one** identity script (not a seventh collector for each workload)
- runs CPU/memory/storage benches **in-process** after consent
- samples current/max megahertz with a tiny read-only script **only** when
  benchmarks were permitted
- never calls `powershell.exe` from the process-limit tests (`cmd.exe` stays)

The agent workflow already runs `cargo test -- --test-threads=1`. Do not
add more parallel `powershell.exe` tests.

`desktop/src-tauri/src/lib.rs` still must not use `std::fs` or
`std::process`. The agent crate may write the consented TEMP file.

## Scoring (rubric CG-1.0)

Printed: **Graded by CYVRA Grading Engine**, rubric CG-1.0.

Processor and thermal stability (20):

- Model + cores + cache → 4 (no workload)
- Sustained-to-maximum clock after the CPU loop → 16 / 11 / 6 / 0
- Package temperature stays **not collected** (no kernel sensor driver)

Memory integrity and speed (15):

- Slot/capacity/speed inventory → 5
- Pattern spot check (capped at 32 MiB) → 7, or domain 0 + critical F on mismatch
- User-mode copy bandwidth ≥ 100 MiB/s → 3
- Never printed as memory verified

Storage SMART stays the scored storage domain. Sequential/random **reads**
of an existing file (`%SystemRoot%\System32\ntdll.dll` when present) are
printed, not scored. The write test is printed as `bytes_written` on
Report D. A disk whose SMART predicts failure is not exercised.

Without consent, a laptop with battery + identity + SMART + USB + radios
reaches about **55%** coverage. The grade stays withheld (floor 70%).

With consent and passing benches, coverage can reach about **81%**. A
**provisional** band may print. Issuance stays off until A8.

## Guards

- Both permission boxes start off
- CPU loop refuses to start at battery charge ≤ 5%
- Write needs the second tick; file is 8 MiB in the Windows TEMP folder, then deleted
- No ATA/NVMe wipe IOCTL, no Format-Volume, no Clear-Disk
- No Ring 0 sensor driver
- Clock points are never awarded from an idle WMI sample

## Still off

- Interactive technician checks and physical port insertion (A7)
- Grading issuance (A8)
- Live camera preview and microphone record (A10)
- Authenticode, unsigned B2 upload, WinPE
- CYVRA Purge / Report B (plan only)
