# P-USB-LOCK — one extra internal disk; USB always refused

Mode S still needs a bound CYVRA Purge licence, this PC’s name, and `ERASE`. This slice changes only the picker and refuse rules.

## What this slice does

- USB, removable media and USB enclosures are always Mode S refused. There is no opt-in.
- The helper refuses USB even if the desktop is bypassed.
- Exactly one extra internal disk per job. Data purge stays disabled until that disk is chosen.
- After PASS or FAIL, the operator may choose another extra internal disk and run again.
- Blocked devices stay visible with a reason (system disk, USB/removable, optical, network, unknown).
- Method preview for the chosen disk shows model, serial, capacity, media class, planned method, and READY or BLOCKED.
- A failed job stays FAILED. No Report S. The failed-job note can be saved. Status is never VERIFIED on FAIL.
- The Purge-key field names `CYVRA-PRG-`. Assessment keys cannot Activate Purge.
- Physical verification is optional. It does not block Mode S or Report A.

## Honest methods (unchanged)

- Extra internal HDD → single-pass overwrite labelled NIST Clear, not Purge.
- Extra internal SATA SSD → ATA Secure Erase, or fail closed.
- Extra internal NVMe → NVMe Sanitize, or fail closed.

## What stays locked

- Five destinations. Nine commands. No sixth navigation item.
- Report A and Report D section lists are not rewritten. Report A still says **No data was erased**.
- Bootstrap `destructive_operations_enabled` stays **false**.
- Bootstrap `purge_licence_bound` stays **false**.
- `grading_issuance_enabled` stays **false**. `report_authentication_enabled` stays **false**.
- The desktop crate does not spawn processes or open raw volumes.
- No Authenticode. No B2. No “Authenticated by CYVORIQ”. No CERTIFIED SECURE.
