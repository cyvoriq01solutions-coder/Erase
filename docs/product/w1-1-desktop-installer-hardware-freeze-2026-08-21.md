# CYVRA Erase — W1.1 Desktop, Installer and Passive Hardware Inventory Freeze — 2026-08-21

> **Status:** Proposed consolidated freeze. It becomes authoritative when approved and merged into main.
>
> **Repository baseline:** cyvoriq01solutions-coder/Erase at main commit 7ef12871b44012d538d6283de47533432b759cd1.
>
> **Scope:** Windows customer product foundation, installer, passive hardware inventory, future Windows Server provision, and future authorized offline retired-device provision.
>
> **Safety:** This document authorizes no destructive erasure, credential bypass, deployment, database mutation, secret change, code-signing operation, or customer release.

## 1. Why this is the consolidated freeze

The project has evolved through several approved checkpoints:

1. The original Windows freeze established a non-destructive CYVRA engineering agent and the sequence IDENTIFY → DISCOVER → ASSESS → MAP → EVIDENCE → VERIFY → REPORT.
2. The control-plane freezes added customer/admin authentication, Cloudflare Worker, Neon, private release distribution, first-500 activation, one-key/one-device binding, audit and protected reports.
3. The current product handoff established that agent 0.2.1 is an engineering command-line foundation, not a customer installer or GUI.
4. This W1.1 revision freezes the desktop technology, installer direction, Windows release sequence, passive hardware inventory, Windows Server provision and future authorized offline retired-device mode.

Earlier documents remain historical evidence. Where an older Windows product statement conflicts with this document, this document governs after approval and merge. Control-plane and security requirements that do not conflict remain in force.

## 2. Frozen product release sequence

### CYVRA V1 — current product target

- Windows 10 22H2 and Windows 11.
- Customer desktop GUI.
- Protected installer download.
- Server-authoritative first-500 activation.
- One activation key and one bound device.
- Passive device, hardware and personal-data-location discovery.
- Non-destructive PDEM, evidence, verification and authenticated report.
- No password bypass.
- No destructive erasure.

### CYVRA Server — next product version

- Headless and centrally managed Windows Server assessment.
- Initial planned targets: Windows Server 2019, 2022 and 2025.
- Server 2016 may be evaluated as compatibility-only; it is not promised by this freeze.
- No dependency on a desktop WebView for core collectors.
- Enterprise deployment, proxy, Group Policy and managed-environment provisions.
- Exact server support matrix requires its own approved freeze and test evidence.

### CYVRA Authorized Offline Retirement Mode — later advanced version

- Signed bootable media for an organization-owned retired device when the installed Windows account cannot be used.
- Operates outside the installed Windows login; it does not crack, reveal, reset or bypass a password.
- Does not unlock BitLocker data without legitimate recovery material.
- Requires a separate destructive-lifecycle freeze, asset authorization, chain of custody, step-up authentication, audit, verification and rollback/recovery policy.
- BIOS or UEFI password recovery remains an OEM or authorized corporate process.
- This mode is an architectural provision only and is not implemented or authorized for use by W1.1.

## 3. Frozen safety and privacy boundary

The V1 customer sequence remains:

ASSESS → PDEM → EVIDENCE → VERIFY → REPORT

The future destructive sequence remains separate:

AUTHORIZE → ERASE → VERIFY → CERTIFY

V1 must not:

- read or transmit personal file contents;
- read email bodies, message contents or browser-history contents;
- collect passwords, recovery keys, tokens or secrets;
- activate cameras or microphones;
- sample private sensor data;
- modify, move, encrypt, overwrite or delete customer files;
- run storage stress tests or erase commands;
- install a driver or permanent privileged service;
- bypass Windows, BitLocker, BIOS, UEFI or corporate security controls; or
- claim a component works merely because it is present.

## 4. Desktop technology decision

The V1 desktop product is frozen as:

- **Tauri 2 stable** for the Windows desktop shell;
- **React and TypeScript** for the customer interface;
- **Rust** for collectors, domain rules, evidence preparation and trusted commands;
- **WebView2 Evergreen** for the desktop renderer;
- a narrow capability and command boundary between the WebView and Rust; and
- pinned lockfiles and reviewed stable dependency updates.

Latest technology means the latest stable, security-supported version that passes the release test matrix. Preview, beta and unpinned dependencies are not permitted in a customer release.

### Required Rust refactor

The existing agent must not be wrapped as an untrusted child process that prints JSON. It must be refactored into a shared core:

- cyvra-core: platform-neutral models, scan orchestration, PDEM, evidence and report inputs;
- cyvra-windows: Windows 10/11 hardware and data-location adapters;
- cyvra-desktop: Tauri commands and desktop lifecycle;
- cyvra-cli: engineering diagnostics using the same core;
- future cyvra-server: headless Windows Server adapter; and
- future cyvra-offline: separately authorized retired-device environment.

The core must expose typed results and errors. The GUI must never parse the current report text as an internal API.

## 5. Installer decision

### V1 customer artifact

- Product name: CYVRA Erase.
- Primary artifact: Authenticode-signed NSIS setup executable.
- Naming pattern: CYVRA-Erase-{version}-{architecture}-setup.exe.
- Install scope: per-machine.
- Installer elevation: one explicit UAC prompt.
- Application runtime: standard user.
- Default application location: Program Files under the CYVORIQ publisher directory.
- Normal uninstall registration and publisher identity.
- In-place semantic-version upgrades.
- Downgrade blocked unless an approved rollback package is used.
- No automatic startup, driver, kernel component or permanent background service in V1.
- WebView2 Evergreen bootstrapper included or invoked safely when the runtime is absent.
- An enterprise MSI package is deferred to the Windows Server/enterprise distribution freeze.

### Architecture outputs

The code and schemas must remain architecture-neutral.

- x64 is the primary launch artifact.
- ARM64 is a required Windows 11 release target after native-device validation.
- x86 is a Windows 10 compatibility artifact only if every collector and installer acceptance test passes.
- The download service must return the correct signed artifact for the declared architecture and entitlement.
- An architecture is not advertised as supported until its signed build passes the complete release matrix.

### Signing and distribution

No customer release may be unsigned.

- Sign executable and installer with an approved CYVORIQ Authenticode identity.
- Timestamp signatures.
- Generate and verify SHA-256 artifact hashes.
- Store artifacts in private R2.
- Neon stores release metadata, hashes, architecture, signer reference, status and audit.
- The Worker issues only short-lived authorized downloads.
- Signing keys must not be stored in source, application binaries or ordinary repository variables.
- Signed updater manifests, staged update rings and rollback are required before automatic application updates are enabled.

## 6. Passive hardware inventory purpose

Hardware inventory is part of the same customer scan as the planned PDEM discovery. It provides device identity and a factual hardware overview for retirement, verification, grading and later asset-lifecycle workflows.

It is passive discovery only.

CYVRA records what Windows, firmware, ACPI, PnP, drivers and supported native APIs expose. It does not prove that a discovered component works.

## 7. Hardware Inventory V1 data contract

The schema name is hardware_inventory_v1.

Every collected field must carry, directly or through its containing record:

- source;
- collection timestamp;
- status;
- confidence;
- permission state where relevant; and
- schema version.

Allowed status values:

- reported;
- observed;
- derived;
- unknown;
- not_reported;
- not_applicable;
- permission_denied;
- unsupported; and
- collection_error.

Unknown values must never be replaced with invented defaults.

### 7.1 Device and chassis identity

Collect where exposed:

- system manufacturer;
- system product/model;
- system family;
- device serial number;
- system UUID;
- chassis manufacturer, type and serial;
- baseboard manufacturer, product, version and serial;
- asset tag;
- desktop, laptop, tablet, virtual-machine or unknown form factor; and
- source and confidence for every classification.

#### Branded versus custom/assembled rule

CYVRA must not conclusively label a device assembled merely because manufacturer data is blank or generic.

Allowed classification:

- OEM-reported;
- custom-or-unidentified;
- virtual;
- conflicting-firmware-data; or
- unknown.

The report must show the firmware facts supporting the classification.

### 7.2 BIOS, UEFI and security hardware

Collect where exposed:

- BIOS or firmware vendor;
- version;
- release date;
- SMBIOS version;
- legacy BIOS or UEFI mode;
- Secure Boot presence/state;
- TPM presence and reported specification version;
- firmware serial/identifiers already included in device identity; and
- virtualization firmware indicators.

Do not collect TPM private material, BitLocker recovery material, Secure Boot private keys or firmware passwords.

### 7.3 Processor

Collect:

- manufacturer/vendor;
- model/name;
- architecture;
- physical package count;
- physical core count;
- logical processor count;
- reported maximum/current clock where reliable;
- address width; and
- virtualization capability if reported.

No CPU benchmark or stress test is permitted.

### 7.4 Memory

Collect:

- total installed physical memory;
- total visible memory;
- physical slot count when firmware reports it;
- populated slot count;
- module capacity;
- speed and configured speed where exposed;
- memory type/form factor;
- manufacturer;
- part number;
- module serial number locally; and
- error-correction capability when reported.

Do not run a memory test. A listed module means reported presence, not proven health.

### 7.5 Storage and volumes

Collect:

- physical drive index;
- manufacturer/model;
- serial number locally;
- firmware revision;
- size;
- bus/interface type;
- HDD, SSD, NVMe, removable or unknown media classification;
- logical volumes;
- drive letter or mount point;
- filesystem;
- capacity and free space;
- Windows-reported operational/health status;
- BitLocker protection metadata without keys; and
- device capability information required later to select an approved sanitization technique.

W1.1 does not authorize SMART self-tests, write tests, sanitize commands or destructive operations.

### 7.6 Graphics and display

Collect where exposed:

- graphics adapter manufacturer/model;
- adapter memory as reported;
- driver version;
- attached display count;
- display manufacturer/model and serial when EDID exposes it;
- native/current resolution; and
- internal versus external display classification when reliable.

Do not capture screenshots or display contents.

### 7.7 Battery and power

For battery-powered devices collect where exposed:

- battery presence;
- manufacturer;
- model/device name;
- serial number locally;
- chemistry;
- designed capacity;
- last/full-charge capacity;
- remaining capacity at collection time;
- cycle count;
- reported charge/status; and
- derived health ratio: full-charge capacity divided by designed capacity.

Battery health is an estimate from firmware/driver-reported capacity, not a functional, runtime or safety certification. Missing design/full-charge capacity produces unknown, not zero or failed.

No charge/discharge test is permitted.

### 7.8 USB and other ports

Collect passive connector and controller information where exposed:

- USB host controllers;
- USB hubs;
- USB4/Thunderbolt controllers;
- declared USB-A/USB-C connectors when firmware or topology exposes them;
- HDMI, DisplayPort, Ethernet, audio, serial, parallel, docking and card-reader connectors when declared;
- currently attached device classes; and
- PnP/driver identifiers required to distinguish controllers, connectors and attached devices.

The report must distinguish:

- physical connector count;
- logical controller count;
- hub count; and
- currently attached device count.

Windows and firmware do not reliably expose every empty physical port. CYVRA reports the count only when a reliable source exists; otherwise it reports unknown or partial. It must not infer physical port count from controller count.

Friendly names of attached phones or removable devices may contain personal information. Prefer device class and vendor/product identifiers; redact user-assigned names unless explicitly required and consented.

### 7.9 Sensors and presence-only devices

CYVRA may inventory sensors exposed by Windows, ACPI, PnP or supported modern Windows device APIs, including:

- accelerometer;
- gyroscope;
- orientation;
- ambient-light;
- proximity;
- magnetometer/compass;
- location capability;
- lid, tablet-mode and other platform sensors;
- biometric-reader presence;
- camera presence;
- microphone presence; and
- other sensor categories reported by the operating system.

For V1 collect only:

- presence;
- sensor/device category;
- manufacturer/model where exposed;
- hardware or PnP identifier;
- driver/provider version; and
- permission/availability status.

Do not request private sensor permission merely to inventory presence. Do not read sensor measurements, location, biometric data, camera frames or microphone audio. The deprecated COM Sensor API must not be introduced for new implementation when a supported modern API or PnP presence record is sufficient.

“All sensors” means all sensors exposed to supported Windows interfaces. Hidden, disabled, undocumented or vendor-private components may not be discoverable and must not be guessed.

### 7.10 Network and communications hardware

Collect presence and model information for:

- Ethernet;
- Wi-Fi;
- Bluetooth;
- cellular/WWAN;
- NFC where exposed; and
- other network adapters.

Exact MAC addresses and subscriber identifiers are device identifiers. They remain local unless a later approved contract defines encryption, HMAC pseudonymization and purpose.

### 7.11 Other useful passive inventory

Collect where exposed:

- audio controllers;
- camera and imaging-device presence;
- keyboard, mouse, touch and pen capability;
- optical drives;
- card readers;
- docking station;
- TPM and smart-card reader presence;
- virtualization/hypervisor indicators; and
- Windows device-driver inventory limited to fields needed for identification and support.

## 8. Collection-source policy

Preferred order:

1. Supported native Windows APIs, SetupAPI/PnP and modern Windows device interfaces.
2. CIM/WMI and SMBIOS/ACPI data exposed by Windows.
3. Built-in PowerShell/CIM as a compatibility fallback.
4. Vendor-specific read-only adapters only after security review.

Rules:

- Do not download or execute a script during a scan.
- Do not invoke untrusted vendor utilities.
- Use fixed commands and typed parsers.
- Apply timeouts, cancellation and output-size limits.
- Record the source and parsing version.
- Handle malformed firmware strings safely.
- A collector failure must not crash or corrupt the overall scan.

## 9. Privilege model

- Desktop application runs as a standard user.
- Most hardware inventory must operate without elevation.
- BitLocker collection may use a narrow, signed, one-shot elevated helper after an explicit explanation and UAC consent.
- Declined elevation produces permission_denied and continues the scan.
- The WebView receives no generic shell, filesystem or process execution capability.
- No arbitrary command supplied by frontend or server may reach PowerShell or a native command runner.

## 10. Privacy treatment of hardware identifiers

Hardware serials, UUIDs, MAC addresses and similar identifiers can track a device.

- Exact values may appear locally in an owner-authorized device report.
- Exact values must be encrypted if retained.
- The server-side device binding uses a domain-separated HMAC or equivalent pseudonymous identifier, not a raw concatenated fingerprint.
- Reports support masking/redaction.
- Logs must not contain raw serial numbers, MAC addresses, activation keys or recovery material.
- The data-retention contract must define deletion and support access.
- The customer sees what hardware identifiers will be used before activation/binding.

## 11. Scan and customer experience integration

The frozen V1 flow becomes:

1. Protected download.
2. Signed installer.
3. Activation and one-device binding.
4. Device identity confirmation.
5. Consent and scan-scope explanation.
6. Passive hardware discovery.
7. Personal-data-location and application-data discovery.
8. PDEM construction.
9. Evidence and verification.
10. Hardware Overview and Privacy Map presentation.
11. Authenticated report.
12. Same-device revalidation and controlled application updates.

The GUI must show progress by collector, allow cancellation, and distinguish:

- found;
- not reported;
- permission denied;
- unsupported; and
- collection error.

It must never display “failed hardware” because a component was absent, inaccessible or not reported.

## 12. First-500 and control-plane requirements retained

The first-500 revision remains compulsory:

- server-authoritative eligibility;
- one issued key/entitlement per accepted user;
- one device per key;
- atomic first binding;
- same-device revalidation;
- different-device rejection and audit;
- no manual payment gate for eligible trial users;
- protected installer download; and
- later conversion to the paid workflow.

Hardware inventory must not make the Windows client the authority for first-500 eligibility. The Worker and Neon remain authoritative for entitlement and binding.

The release path remains:

Customer/Admin UI → Worker authorization → Neon entitlement/audit → private R2 artifact or authenticated report

## 13. Windows Server provision retained now

To prevent a future rewrite:

- core models contain no WebView dependency;
- collectors use a platform-adapter interface;
- scan orchestration supports GUI and headless callers;
- output schemas are stable and versioned;
- proxy and corporate certificate-store behavior are abstracted;
- local interactive consent and enterprise job authorization are separate strategies;
- battery and consumer-only fields support not_applicable;
- no GUI assumption exists in evidence/report generation; and
- server-specific roles, services, remote execution and deployment are deferred to the Server freeze.

## 14. Authorized offline retirement provision retained now

W1.1 reserves interfaces for:

- signed offline job manifests;
- asset and drive identity matching;
- one-time authorization;
- chain-of-custody events;
- offline evidence capture;
- later synchronization;
- sanitization technique selection;
- verification and validation; and
- certificate generation.

No offline destructive implementation starts during W1.1. A future plan must align with NIST SP 800-88 Revision 2 and applicable device standards, licensing and law.

## 15. Quality and acceptance matrix

Before V1 support is claimed, test at minimum:

### Operating systems and architecture

- Windows 10 22H2 x64;
- Windows 11 x64 on supported current releases;
- Windows 11 ARM64;
- Windows 10 x86 only if shipped;
- clean install, upgrade, repair and uninstall;
- standard user and administrator account scenarios; and
- online, proxy, intermittent network and denied-elevation paths.

### Hardware types

- branded business laptop;
- branded business desktop;
- custom/assembled desktop;
- laptop with healthy battery;
- laptop with degraded or unreported battery;
- desktop with no battery;
- SATA HDD;
- SATA SSD;
- NVMe SSD;
- removable storage;
- BitLocker protected and unprotected volumes;
- USB-A/USB-C/Thunderbolt combinations;
- docked and undocked laptop;
- device with sensors;
- device with no exposed sensors;
- virtual machine; and
- malformed or generic SMBIOS data.

### Required assertions

- no private file contents read;
- no camera, microphone or sensor measurement captured;
- no password, token or recovery key captured;
- no destructive command executed;
- no raw sensitive identifiers in logs;
- unknown fields remain unknown;
- collector errors do not terminate the scan;
- report provenance is present;
- correct signed architecture artifact delivered; and
- app runs non-elevated after installation.

## 16. Explicitly frozen decisions

Approved by this freeze when merged:

- V1 supports Windows 10 22H2 and Windows 11.
- Windows Server is the next product version, not part of V1.
- Authorized Offline Retirement Mode is later and is not password bypass.
- Tauri 2, React/TypeScript and Rust are the desktop foundation.
- Rust collectors become a reusable core, not a JSON subprocess contract.
- The primary installer is a signed per-machine NSIS setup executable.
- Runtime is standard user with narrowly scoped optional elevation.
- Passive Hardware Inventory V1 is part of the normal scan.
- Presence is not a working-condition test.
- Exact hardware facts are reported only when exposed; no guessing.
- Private distribution, first-500 entitlement and one-device binding remain.
- Destructive operations remain disabled.

## 17. Deferred decisions

Require later contracts or owner approval:

- exact signing certificate/provider and key custody;
- final visual design and accessibility specification;
- authenticated report file format and verification UX;
- final server support matrix and server package technology;
- enterprise MSI timing;
- offline boot-environment licensing and implementation;
- sanitization techniques by media type;
- support-assisted device rebind;
- update-ring timing and rollback thresholds;
- retention periods for hardware identifiers; and
- production release date.

## 18. Implementation order after approval

1. Merge the current documentation/README gate.
2. Approve and merge this W1.1 freeze.
3. Refactor agent 0.2.1 into typed shared core and CLI crates without changing collection behavior.
4. Define hardware_inventory_v1 types, privacy classifications and fixtures.
5. Implement passive collectors with unit/fixture tests.
6. Build the least-privilege Tauri GUI shell.
7. Add protected activation and Agent-to-Worker contracts.
8. Build unsigned internal installers for test only.
9. Acquire and integrate approved signing custody.
10. Execute architecture and hardware acceptance matrix.
11. Integrate private R2 distribution and signed updates.
12. Run an internal pilot.
13. Request separate customer-release approval.

## 19. Authoritative references

- Current canonical handoff: docs/product/current-product-freeze-handoff-2026-08-21.md
- Control-plane freeze: docs/product/control-plane-c3-c7-freeze.md
- Admin/download freeze: docs/product/c4-c5-admin-download-freeze-2026-08-19.md
- Admin-auth freeze: docs/product/c4-1-admin-auth-freeze.md
- Microsoft WMI/CIM guidance: https://learn.microsoft.com/powershell/scripting/samples/getting-wmi-objects--get-ciminstance-
- Microsoft Windows Sensor APIs: https://learn.microsoft.com/windows/win32/api/_winsensors/
- Tauri architecture: https://v2.tauri.app/concept/architecture/
- Tauri Windows installer: https://v2.tauri.app/distribute/windows-installer/
- Tauri security: https://v2.tauri.app/security/
- Microsoft WebView2 distribution: https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution
- Microsoft Windows PE: https://learn.microsoft.com/windows-hardware/manufacture/desktop/winpe-intro
- Microsoft BitLocker recovery: https://learn.microsoft.com/windows/security/operating-system-security/data-protection/bitlocker/recovery-overview
- NIST SP 800-88 Revision 2: https://csrc.nist.gov/pubs/sp/800/88/r2/final
