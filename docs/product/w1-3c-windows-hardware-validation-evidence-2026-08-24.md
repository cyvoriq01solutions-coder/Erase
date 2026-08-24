# CYVORIQ ERASE — W1.3C Windows Hardware Validation Evidence

**Evidence date:** 2026-08-24
**Checkpoint:** W1.3C — Windows 10 and Windows 11 hardware validation
**Overall status:** PARTIALLY VALIDATED — NOT FROZEN

## 1. Acceptance status

| Validation target | Status | Evidence |
|---|---|---|
| Windows 10 22H2 x64 | PASS | Physical standard-user validation completed |
| Windows 11 x64 | PENDING | Physical Windows 11 computer unavailable |
| W1.3C overall | NOT FROZEN | Windows 11 validation remains required |

The Windows 10 pass does not authorize a customer release. W1.3C remains open until the required Windows 11 physical validation is completed.

## 2. Implementation under test

- Repository: `cyvoriq01solutions-coder/Erase`
- Pull request: `#26`
- Feature head: `7ea604fd3ae42d31909827cc6891d00ddcb92b00`
- Main merge commit: `39e383fff26d61fc228a6e4fa91c1b11078e2392`
- Validator version: `0.2.1`
- Validator name: `cyvoriq_w1_3c_hardware_validation`
- GitHub Actions run: `#15`
- Workflow run ID: `32696238538`
- Workflow result: PASS
- Build target: Windows x64
- Release status: Internal unsigned validation artifact only

## 3. Artifact integrity

### Downloaded validation artifact

- Artifact: `CYVORIQ-Internal-Hardware-Validation-x64.zip`
- Size: `198045` bytes
- SHA-256:

  `f145b6402a8bb88f308dd563d7d45b822d66c30e0143077c4037cc12d1a39146`

### Extracted validator executable

- File: `CYVORIQ-Hardware-Validation-v0.2.1-windows-x64.exe`
- Size: `461824` bytes
- SHA-256:

  `8a6d456341cb12a41360902c53e373d8e0201a350d5fa3774f3c9fcf6d11fe1c`

The independently calculated executable hash matched the checksum packaged by GitHub Actions.

## 4. Windows 10 physical test environment

- Operating system: Windows 10 Pro
- Version: 22H2
- OS build: `19045.7563`
- Execution context: Standard user
- Administrator-role check: `False`
- Collection mode: Passive and read-only
- Destructive operations: Disabled
- Identifiers: Redacted
- Schema: `hardware_inventory_v1`

## 5. Windows 10 execution result

- Collection timestamp: `1787553737`
- Collection timestamp UTC: `2026-08-24 06:42:17 UTC`
- Validator result: PASS
- Process exit code: `0`
- Requested sections consistent: `true`
- Requested fields consistent: `true`
- Requested coverage complete: `true`
- Deferred sections untouched: `true`

Successfully reported or derived data included:

- Device manufacturer and model
- OEM device classification
- Laptop form factor
- Firmware vendor and UEFI mode
- Processor manufacturer, model, architecture and core counts
- Installed and visible physical memory
- Memory-slot and memory-module information

## 6. Explicit limited-result observations

The standard-user scan reported:

- Secure Boot presence: `permission_denied`
- Secure Boot enabled state: `permission_denied`
- TPM presence: `collection_error`
- TPM specification: `collection_error`

These states were explicit and localized. They did not crash or terminate the remaining passive collection. They demonstrate failure isolation but do not prove successful Secure Boot or TPM value collection on every supported Windows device.

## 7. Privacy verification

The generated evidence report was checked for the following prohibited identifier labels:

- `serial`
- `uuid`
- `asset_tag`
- `mac_address`

The privacy scan returned no matches.

### Saved evidence report

- File: `windows-10-22h2-validation.txt`
- Size: `3398` bytes
- SHA-256:

  `b73854124cd319bf53927354097f6d084d7b2961495e87f8ff6c14bc556bb269`

## 8. Governance note

PR #26 was merged before the complete Windows 10 and Windows 11 physical validation matrix was finished. The merge records implementation availability; it does not represent final W1.3C acceptance or release authorization.

The Windows 10 physical test is now complete and passed. Windows 11 remains an open acceptance gate.

## 9. Required next validation

Run the same hash-verified validator artifact on a physical Windows 11 x64 computer under a standard, non-administrator account.

The Windows 11 run must record:

- Windows edition, version and OS build
- Validator artifact and executable hashes
- Standard-user confirmation
- Complete validator output
- Exit code
- Privacy scan result
- Evidence-file SHA-256

W1.3C may be frozen only after the Windows 11 evidence is reviewed and accepted.