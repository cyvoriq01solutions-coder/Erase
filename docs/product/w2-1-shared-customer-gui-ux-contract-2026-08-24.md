# CYVORIQ ERASE — W2.1 Shared Customer GUI/UX Contract — 2026-08-24

> **Status:** PROPOSED — OWNER REVIEW REQUIRED
>
> **Branch:** `w2-1-shared-customer-gui-ux-contract`
>
> **Repository baseline:** `39e383fff26d61fc228a6e4fa91c1b11078e2392`
>
> **Scope:** Shared Windows customer GUI for CYVRA Erase and CYVRA QC, including navigation, customer journey, visual direction, state handling, accessibility and implementation boundaries.
>
> **Change boundary:** This document authorizes no deployment, database mutation, secret change, code-signing operation, destructive erasure, customer release or production GUI implementation.

## 1. Purpose and precedence

This contract defines how customers will experience CYVRA on Windows before Tauri GUI implementation begins.

It must be read with:

- [Current Product Freeze & Team Handoff — 2026-08-21](current-product-freeze-handoff-2026-08-21.md)
- [W1.1 Desktop, Installer and Passive Hardware Inventory Freeze — 2026-08-21](w1-1-desktop-installer-hardware-freeze-2026-08-21.md)
- [Frontend Commercial V1 Freeze](frontend-commercial-v1-freeze.md)

The existing safety, privacy, installer, activation, device-binding, report and release requirements remain in force.

If this document conflicts with an approved safety or security requirement, implementation must stop and the conflict must be reviewed. GUI convenience must never weaken the frozen product boundary.

## 2. Shared application decision

CYVRA V1 will present one customer application, not two separate customer programs.

| Customer-visible concept | V1 decision |
| --- | --- |
| Windows installer | One installer |
| Installed application | One desktop application |
| Account and activation | One shared flow |
| Device binding | One shared binding |
| Consent | One clear consent sequence |
| Hardware and privacy collection | One coordinated assessment |
| Results | Two clearly separated result areas |
| Customer report | One combined authenticated report |
| Customer command line | None |

The approved V1 installer identity remains:

`CYVRA Erase — by CYVORIQ Solutions`

The approved installer naming pattern remains:

`CYVRA-Erase-{version}-{architecture}-setup.exe`

Changing the installed product to a broader suite name requires a separate brand and packaging amendment. This contract does not silently rename the frozen installer.

CYVRA QC is presented inside the same application as the device-verification and grading domain. It is not a second installer, second activation, second scan, or separate customer login.

## 3. Two result domains inside one experience

### CYVRA QC

Customer purpose:

- identify the device;
- present passive hardware facts;
- show evidence provenance and collection confidence;
- distinguish reported, derived, unavailable and restricted information;
- support device verification; and
- produce an evidence-based CYVRA QC device grade when the approved evidence threshold is met.

CYVRA QC grading is a core product capability, not an optional future feature. CYVRA QC must not be approved for customer release until the grading rules are frozen, implemented and validated.

QC must never claim that hardware works merely because it was detected. A device grade must be supported by the approved combination of automated evidence, functional evidence, cosmetic evidence and any authorized operator review.

The separate W2.2 CYVRA QC Grading Contract must define:

- the grade scale and customer-facing grade names;
- grading inputs;
- mandatory and optional evidence;
- automated, functional, cosmetic and operator-reviewed evidence boundaries;
- treatment of customer-uploaded images and videos;
- weights and thresholds;
- minimum evidence required to issue a grade;
- missing-data and permission-denied treatment;
- exception and manual-review rules;
- evidence provenance;
- grade recalculation and versioning;
- audit requirements; and
- dispute or correction handling.

Customer-facing grade states are:

- before assessment completion: `Grade pending`;
- sufficient approved evidence: display the actual CYVRA QC grade and supporting evidence summary;
- insufficient evidence: `Unable to grade — insufficient evidence`, with the missing or limited inputs identified; and
- grading failure: `Unable to grade — grading error`, with a safe retry or support action.

`Grade not issued` may appear only in an unfinished internal development build. It must not be the default result in a customer release.

A CYVRA QC device grade describes assessed device condition under the approved grading rules. It is not automatically a resale price or monetary valuation.

The W2.2 grading contract must be approved before the final QC results screen, grading engine or customer-release implementation is completed.


### CYVRA Erase

Customer purpose in V1:

- assess potential personal-data exposure;
- map relevant data and application locations without reading personal content;
- present evidence and coverage;
- explain erasure readiness;
- generate an authenticated verification report.

The V1 GUI must state clearly:

`Assessment only — this release does not erase data.`

The following customer actions are prohibited in V1:

- `Erase now`
- `Wipe device`
- `Delete data`
- `Bypass password`
- any control that implies destructive sanitization has occurred

Future destructive erasure remains a separately authorized lifecycle.

## 4. Frozen customer journey

The target journey is:

1. Customer receives an authorized protected download.
2. Customer installs the signed CYVRA setup package.
3. CYVRA launches as a standard-user desktop application.
4. Customer signs in or enters a server-issued activation key.
5. The server verifies eligibility and entitlement.
6. The customer reviews the device-binding disclosure.
7. The first successful activation binds the entitlement to one device.
8. CYVRA displays the detected device for confirmation.
9. The customer reviews the privacy and scan scope.
10. The customer gives explicit consent.
11. The customer starts one coordinated device verification.
12. CYVRA displays truthful progress and supports cancellation.
13. CYVRA presents an overall results summary.
14. CYVRA presents the QC device and hardware results.
15. CYVRA presents the Erase privacy-exposure and readiness results.
16. The customer previews and saves one authenticated report.
17. Later launches revalidate the same authorized device and entitlement.

The customer must never need PowerShell, Command Prompt, JSON, a developer console or an internal validator.

## 5. Experience principles

The GUI must be:

- premium and enterprise-ready;
- calm, clear and evidence-led;
- understandable to an individual customer without technical training;
- credible for enterprise, OEM and ITAD workflows;
- explicit about what is and is not being tested;
- transparent about permissions and unavailable information;
- usable without administrator privileges during normal operation;
- accessible by keyboard and assistive technology; and
- consistent with the CYVORIQ public portal.

The interface must prioritize trust, evidence and next actions over decorative dashboards.

It must not use fear-based messaging, exaggerated security claims, fake progress, invented hardware values or unsupported compliance claims.

## 6. Visual direction

### Brand hierarchy

- Company: CYVORIQ Solutions
- Product: CYVRA Erase
- Verification domain: CYVRA QC
- Public product presentation: `CYVRA Erase — by CYVORIQ Solutions`

Use the approved CYVORIQ logo asset. Do not redraw the logo as a letter icon or substitute an unofficial mark.

### Colour roles

- CYVORIQ navy and blue form the primary visual foundation.
- White and restrained neutral surfaces provide clarity and whitespace.
- Orange is reserved for the highest-intent primary action.
- Success, warning, information and error states use accessible semantic colours.
- Colour must never be the only way a status is communicated.
- Exact colour tokens require approval through the visual-design package.

### Typography and layout

- Use a Windows-appropriate, highly legible interface typeface.
- Maintain clear heading, body, label and evidence hierarchies.
- Use generous spacing and a consistent spacing scale.
- Avoid dense card walls and generic startup-dashboard styling.
- Keep the primary action visually obvious.
- Long evidence values must wrap or truncate safely without breaking layout.
- Raw JSON must never appear in the customer interface.

### Motion

- Motion must be restrained and functional.
- Animations may explain progress or transitions but must not simulate work.
- Respect the Windows reduced-motion preference.
- Do not use flashing, continuous decorative movement or distracting effects.

### Theme

The initial V1 design target is a polished light interface with Windows high-contrast compatibility. A dark theme is optional and must not delay the accessible V1 customer journey.

## 7. Application structure

After activation, the primary navigation is:

- **Overview**
- **Verification**
- **Results**
- **Report**
- **Help**

Settings may be available through a secondary menu. It must not compete with the main verification journey.

The interface must not present CYVRA Erase and CYVRA QC as two unrelated applications. Their results appear as two named sections within the same verification record.

## 8. Screen contract

| Screen | Purpose | Primary action | Required content |
| --- | --- | --- | --- |
| Welcome and Trust | Establish publisher and product trust | Continue | Product identity, publisher, version and non-destructive V1 statement |
| Sign In or Activate | Establish authorized entitlement | Continue securely | Email sign-in and/or activation-key route, privacy-safe errors |
| Device-Binding Disclosure | Explain one-key/one-device binding | Agree and bind | Data categories used, pseudonymous server treatment and support-recovery notice |
| Device Detected | Confirm the intended device | Confirm device | Manufacturer/model where reported, architecture and masked identifiers |
| Consent and Scope | Explain what the scan reads and excludes | Give consent | Hardware scope, PDEM metadata scope, exclusions and permission behavior |
| Ready to Verify | Present final scan summary | Run Device Verification | Device, scope, estimated stages and cancellation availability |
| Verification Progress | Show truthful collection progress | Cancel scan | Current stage, completed stages, limitations and safe cancellation |
| Results Overview | Summarize the completed assessment | Review results | Coverage, limitations, QC summary and Erase summary |
| QC Results | Present device and hardware evidence | Continue to privacy results | Hardware overview, provenance, statuses, confidence and grade readiness |
| Erase Results | Present privacy exposure and readiness | Continue to report | PDEM summary, evidence coverage and explicit no-erasure statement |
| Report Preview | Review the customer evidence package | Generate or save report | Device summary, both result domains, limitations and authenticity status |
| Completion | Confirm safe completion | Finish | Report location, verification method and recommended next action |
| Help and Recovery | Support recoverable problems | Retry or contact support | Privacy-safe diagnostics and approved support path |

## 9. Overview screen

The Overview screen must provide a calm starting point rather than a generic analytics dashboard.

It must show:

- current device summary;
- entitlement status;
- device-binding status;
- last verification date, if one exists;
- report availability;
- important unresolved limitations; and
- one dominant `Run Device Verification` action.

It must not show an `Erase Now` action in V1.

## 10. Consent and transparency

Before scanning, the customer must be told:

- which hardware categories will be queried;
- that personal-data location metadata may be assessed;
- that private file contents will not be read or uploaded;
- that cameras, microphones and sensor measurements will not be activated;
- that passwords, tokens, keys and recovery material will not be collected;
- that the scan is passive and non-destructive;
- that some information may require permission or may be unavailable;
- that cancellation is available; and
- how evidence and report metadata will be used.

Consent must be explicit. It must not be preselected or hidden inside general terms.

## 11. Verification progress

The progress screen must show stages rather than a misleading fixed animation.

Required stages include:

1. Preparing verification
2. Confirming device identity
3. Collecting passive hardware information
4. Assessing personal-data locations
5. Building the Privacy Exposure Map
6. Preparing evidence
7. Verifying consistency
8. Preparing results

The GUI may display a percentage only when it is derived from known completed work. It must not advance a fake percentage while an unknown-duration collector is running.

Each stage supports:

- pending;
- running;
- completed;
- completed with limitations;
- cancelled;
- or collection error.

Cancellation must:

- change the visible state immediately to `Cancelling safely`;
- signal the Rust scan orchestrator;
- prevent new collectors from starting;
- wait for bounded collector shutdown;
- preserve only approved partial state;
- avoid producing a final passed report from incomplete work; and
- return the customer to a clear retry or exit decision.

Closing the application during a scan must invoke the same safe-cancellation contract.

## 12. Result language

The GUI must preserve the Hardware Inventory V1 status meanings:

- Reported
- Observed
- Derived
- Unknown
- Not reported
- Not applicable
- Permission denied
- Unsupported
- Collection error

The interface must never translate `unknown`, `not_reported`, `permission_denied`, `unsupported` or `collection_error` into `failed hardware`.

Every material result must retain access to:

- source;
- collection timestamp;
- status;
- confidence;
- permission state where relevant; and
- schema version.

Customer summaries may simplify the presentation, but the evidence view must preserve provenance.

## 13. QC results experience

The QC result area must organize evidence into understandable groups:

- Device identity and chassis
- Firmware and security hardware
- Processor
- Memory
- Storage and volumes
- Graphics and displays
- Battery and power
- Ports and controllers
- Sensors and presence-only devices
- Network and communications
- Relevant peripherals

Each group must show:

- collection coverage;
- reported facts;
- explicit limitations;
- evidence source;
- timestamp;
- confidence where applicable; and
- permission or support status.

Sensitive hardware identifiers are masked by default. A locally authorized reveal action may be added only when the privacy contract permits it.

Until the grading contract is approved, the interface must not show an A/B/C grade, resale grade, condition score or monetary valuation.

## 14. CYVRA Erase results experience

The Erase result area must show privacy exposure without exposing personal content.

It may present:

- data-location categories;
- application-data categories;
- profile and volume coverage;
- metadata-based findings;
- evidence completeness;
- areas not assessed;
- permission limitations;
- verification status; and
- erasure-readiness guidance.

It must not present:

- personal file contents;
- email or message bodies;
- browser-history content;
- passwords, tokens or recovery keys;
- screenshots;
- raw sensitive content samples; or
- a claim that data was erased.

The permanent V1 result notice is:

`No data was erased. This assessment identifies exposure and prepares evidence for an authorized next step.`

## 15. Combined report experience

The GUI targets one combined customer report containing:

- CYVORIQ and CYVRA product identity;
- application and schema versions;
- verification identifier;
- collection time;
- masked device identity;
- entitlement and binding status where appropriate;
- QC hardware evidence summary;
- grading status or `Grade not issued`;
- CYVRA Erase privacy-exposure summary;
- scope, exclusions and limitations;
- evidence and provenance hashes;
- authenticity information; and
- customer verification instructions.

The exact report file format, signature mechanism, retention policy and verification endpoint remain governed by the future authenticated-report contract.

The GUI must not display a report as authenticated until server or cryptographic verification has actually succeeded.

## 16. Activation and device-binding experience

The application consumes server-authoritative decisions. It never determines first-500 eligibility locally.

The activation experience must:

- support the approved email sign-in and/or activation-key path;
- avoid revealing whether an unrelated identity or key exists;
- mask the activation key after entry;
- avoid writing activation keys to logs;
- explain one-device binding before consent;
- show which identifier categories contribute to binding;
- send only the approved pseudonymous binding value;
- handle same-device revalidation;
- reject different-device reuse with a support-safe message;
- distinguish expired, revoked and unavailable entitlement states; and
- never bypass entitlement because the network is unavailable.

Eligible first-500 users must not encounter the obsolete paid-approval gate.

## 17. Error and recovery contract

Errors must explain what happened, what remains safe and what the customer can do next.

| Condition | Required treatment |
| --- | --- |
| No network before activation | Explain that entitlement cannot be verified; offer retry |
| Worker unavailable | Preserve local safety; offer bounded retry |
| Invalid or unavailable entitlement | Use privacy-safe wording; provide approved support path |
| Different device detected | Explain binding conflict without exposing fingerprint data |
| Permission declined | Continue where safe and mark affected fields `Permission denied` |
| Collector timeout | Isolate the collector, continue when permitted and record limitation |
| Collector parse error | Record `Collection error`; never invent a value |
| Customer cancellation | Stop safely and do not issue a completed report |
| Report-authentication failure | Preserve local results but label the report unverified |
| Unexpected application error | Show a privacy-safe error code and recovery action |

Retry must not duplicate activation, device binding, evidence or report transactions. Server requests require the idempotency and replay controls defined by the future Agent-to-Worker contract.

## 18. Accessibility requirements

The V1 GUI targets WCAG 2.2 AA principles where applicable to a Windows desktop application.

It must support:

- complete keyboard navigation;
- visible keyboard focus;
- logical focus order;
- accessible names and descriptions;
- screen-reader announcements for progress and errors;
- text resizing and Windows display scaling;
- layouts usable at 100%, 125%, 150% and 200% scaling;
- sufficient text and control contrast;
- non-colour status indicators;
- reduced motion;
- no keyboard traps;
- usable validation messages; and
- accessible report preview controls.

The design must be tested on Windows 10 22H2 and supported Windows 11 releases.

## 19. Window and layout behavior

The application must:

- remain usable on common 1366 × 768 business laptops;
- adapt safely to larger desktop displays;
- support high-DPI rendering;
- preserve critical actions without horizontal scrolling;
- avoid placing the only primary action below an unreachable viewport;
- retain state during safe window resizing;
- warn before closing an active scan; and
- never hide security, privacy or limitation notices behind decorative elements.

## 20. Technical implementation boundary

The implementation must use:

- Tauri 2 stable;
- React and TypeScript;
- the shared typed Rust core;
- architecture-neutral schemas;
- a narrow allowlisted Tauri command surface; and
- explicit typed success, progress, cancellation and error events.

The GUI must not:

- parse command-line report text or JSON output as its internal API;
- launch the existing engineering CLI as an untrusted child process;
- include the internal hardware validator in the customer package;
- expose generic shell, filesystem or arbitrary-process execution;
- execute commands supplied by the frontend or server;
- store secrets in browser storage;
- treat frontend state as entitlement authority;
- weaken standard-user operation; or
- enable destructive functionality.

Normal runtime remains non-elevated. Any future one-shot elevated helper requires a separately frozen command and consent contract.

## 21. Privacy-safe diagnostics

Customer support diagnostics must be opt-in and understandable.

Diagnostics must not include:

- raw serial numbers;
- UUIDs;
- MAC addresses;
- activation keys;
- device tokens;
- passwords;
- recovery material;
- personal filenames or contents;
- email or message content; or
- browser-history content.

The GUI may show a privacy-safe support code and allow the customer to preview approved diagnostic categories before export or upload.

## 22. Design deliverables before implementation

The W2.1A design package must include:

- application information architecture;
- complete screen inventory;
- low-fidelity wireframes;
- approved visual tokens;
- high-fidelity designs for the core journey;
- component-state definitions;
- progress and cancellation prototype;
- QC result presentation;
- Erase result presentation;
- combined report preview;
- error and recovery matrix;
- customer-facing copy deck;
- accessibility checklist; and
- Windows 10 and Windows 11 layout review.

The owner must approve the design package before production GUI implementation begins.

## 23. W2 implementation acceptance criteria

A W2 implementation is not complete until:

- the customer can complete the journey without a terminal;
- one application presents both result domains clearly;
- activation and binding states consume typed server contracts;
- consent precedes collection;
- progress reflects real collector state;
- cancellation is safe and bounded;
- every unavailable value remains explicit;
- no unsupported hardware-health or grade claim appears;
- no destructive action appears;
- no prohibited private content is collected or displayed;
- reports are not called authenticated before verification succeeds;
- keyboard and accessibility tests pass;
- Windows 10 and Windows 11 layout tests pass;
- internal CLI and validator tools are excluded from customer packaging; and
- implementation evidence is reviewed through a dedicated pull request.

W2 completion does not itself authorize customer release. W3 through W6 gates remain mandatory.

## 24. Explicit non-goals

This contract does not authorize:

- destructive data erasure;
- password or security-control bypass;
- Windows Server customer support;
- offline retirement mode;
- public executable distribution;
- unsigned customer installation;
- a second CYVRA customer application;
- separate QC activation or device binding;
- hardware stress or functional testing;
- an unapproved commercial grade;
- resale valuation;
- auction functionality;
- production deployment; or
- customer go-live.

## 25. Deferred owner decisions

The following require later approval:

- whether a future release renames the shell from CYVRA Erase to a broader CYVRA suite;
- the QC grading model, thresholds and evidence rules;
- exact desktop colour and typography tokens;
- final report file format and verification experience;
- support-assisted device rebind experience;
- legal and consent wording;
- localisation and language sequence;
- enterprise branding or tenant customisation;
- support contact routes;
- update notification and rollback experience; and
- destructive lifecycle UX for a later authorized product.

Until amended, the existing CYVRA Erase product and installer identity remains authoritative.

## 26. Change control and handoff

Every GUI implementation handoff must state:

- branch and base commit;
- approved screen or requirement addressed;
- files and schemas changed;
- privacy and security impact;
- Tauri capabilities added or changed;
- API, binding or report contracts consumed;
- tests and accessibility checks run;
- packaging performed or explicitly not performed;
- deployment performed or explicitly not performed;
- rollback method;
- open risks; and
- next approval gate.

Visual convenience must not silently redefine activation, binding, grading, evidence, report authenticity or destructive behavior.

## 27. Immediate next action after approval

After this contract is reviewed and merged:

1. Create the W2.1A visual-design branch.
2. Produce the complete wireframe and high-fidelity design package.
3. Review the shared GUI with the owner before writing production Tauri screens.
4. Continue the remaining W1 Agent-to-Worker, device-binding, signing, report, first-500 and observability contracts in parallel.
5. Begin the least-privilege Tauri shell only after the relevant UI and technical contracts are approved.

No production deployment, customer installer or release is authorized by this document.