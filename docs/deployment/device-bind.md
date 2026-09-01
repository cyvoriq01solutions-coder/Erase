# Device bind (slice C6)

Online activation is a Worker operation. The desktop webview never calls
`fetch`. Tauri command `activate_license` reads the Windows MachineGuid and
POSTs to `https://api.cyvra.co.in/api/v1/auth/activate`.

Body:

```json
{ "activationKey": "CYVRA-…", "machineGuid": "…", "hostname": "PC-NAME" }
```

The Worker HMAC-hashes the key (`cyvoriq-erase:license:v1:`) and the device
id (`cyvoriq-erase:device:v1:`) with `AUTH_PEPPER`. Rows go into existing
`licenses`, `devices`, and `device_activations` tables. No new migration.

First success binds one device (`max_devices = 1`). The same PC revalidates
(`already_bound`). A different MachineGuid returns `409 device_mismatch`.

Anonymous curl without a valid key is `400`. Collection, grading, and
destructive flags stay false.

Finish in first-run stays disabled until Activate succeeds on Windows.
