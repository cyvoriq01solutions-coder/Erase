# W-collect — local hardware and document assessment

Approved freeze: both scan engines run **inside** `CYVRA-Erase-*-x64-setup.exe`.
They are not a second website download and are not spawned as extra console EXEs.

## Engines

1. Hardware validation (`hardware_inventory_v1`) — same logic as
   `CYVORIQ-Hardware-Validation-v0.2.1` / now `cyvra_w1_3c_hardware_validation`.
2. Verification / PDEM — same `run_scan()` path as the verification agent.

The Tauri command `run_device_verification` calls `cyvra_core::run_customer_verification()`.

## Flags

- `live_collection_enabled`: true
- `destructive_operations_enabled`: false
- grading and cloud report authentication: false

## Customer result

Report A: local hardware text + assessment JSON (document locations, not contents).
Report B (demo sanitization certificate) is **not** this slice.

Purge remains disabled.
