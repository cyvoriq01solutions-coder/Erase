# P-SECONDARY — Mode S extra-disk wipe

Mode S sanitises **extra disks only** after a bound CYVRA Purge licence, this PC’s name, and `ERASE` in capital letters.

## What this slice does

- Ninth desktop command: `run_mode_s_purge`.
- Elevated helper `cyvra-purge-helper` issues the method. The desktop crate does not spawn processes or open raw volumes.
- Independent 10% sample looks for the `CYVRA-TEST` marker. Report S / Report B is issued only on verify **PASS**. Status is **VERIFIED** or **FAILED**, never CERTIFIED SECURE.
- Extra internal magnetic HDD uses a single-pass overwrite labelled **NIST Clear**, not Purge.
- Extra internal SATA SSD → ATA Secure Erase. Extra internal NVMe → NVMe Sanitize. If firmware sanitize is unavailable, Mode S **fails closed**. Host overwrite is not called Purge on flash.
- Optical, network, unknown, USB, removable media, USB enclosures, and the Windows system disk are refused. There is no USB opt-in. See **P-USB-LOCK**.

## What stays locked

- Bootstrap `destructive_operations_enabled` stays **false**. Assessment engines are not the wipe path.
- Bootstrap `purge_licence_bound` stays **false**. Bind is session-only from P-LICENCE.
- `grading_issuance_enabled` stays **false**. `report_authentication_enabled` stays **false**.
- No Authenticode. No B2. No sixth navigation item. No “Authenticated by CYVORIQ”.

## Lab only

Run Mode S on spare disks on a lab PC. Never the daily OS disk. After a FAIL, cancel, crash, or missing helper, no Report S is written.
