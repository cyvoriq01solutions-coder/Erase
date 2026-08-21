# CYVORIQ Erase — Current Product Freeze & Team Handoff — 2026-08-21

> **Status:** Canonical product freeze and team handoff, merged through PR #19. Later approved amendments and linked contract freezes govern the scopes they expressly update.
>
> **Repository baseline:** cyvoriq01solutions-coder/Erase at main commit bf96431b6131eebfab6a5d3c18b568f1b1e93a0a.
>
> **Customer release status:** **NOT RELEASED.** Version 0.2.1 source is an engineering command-line agent, not the frozen customer-installable, signed Windows product.
>
> **Change boundary:** This document authorizes no deployment, database mutation, secret change, public release, or destructive erase capability.

## Amendment A — W1.1 Windows foundation and hardware scope

The detailed amendment is:

- [W1.1 Desktop, Installer and Passive Hardware Inventory Freeze — 2026-08-21](w1-1-desktop-installer-hardware-freeze-2026-08-21.md)

When that document is approved and merged, it freezes these updates:

- CYVRA V1 targets Windows 10 22H2 and Windows 11.
- Windows Server is the next product version and receives architectural provision now.
- Authorized Offline Retirement Mode is a later separately gated product; it works outside the installed Windows login and is not password bypass.
- Tauri 2, React/TypeScript and a reusable Rust core form the desktop foundation.
- The primary V1 package is a signed per-machine NSIS setup executable.
- The application runs as a standard user; only a narrow explicit helper may request elevation.
- Passive Hardware Inventory V1 joins the normal scan.
- Inventory includes firmware-reported device identity, BIOS/UEFI, CPU, memory, storage, graphics, battery, declared ports/controllers, sensors, network hardware and relevant peripherals.
- Presence does not prove working condition; unavailable information is unknown rather than guessed.
- Private distribution, first-500 entitlement, one-device binding, authenticated evidence and the non-destructive privacy boundary remain compulsory.
- No destructive operation or credential bypass is authorized.

This amendment consolidates the original Windows freeze, the control-plane/first-500 revision, the current product handoff and the latest retired-device/hardware decisions. Earlier conflicting Windows product statements are superseded when the linked W1.1 freeze is merged.

## 1. Purpose and precedence

This document gives every developer one current, testable understanding of:

- the product being built;
- what is implemented versus only planned;
- the safety and privacy rules that may not be weakened;
- the compulsory first-500 trial revision;
- the relation between Windows, Cloudflare, Neon, Resend, and GitHub;
- the remaining work before a customer-installable CYVRA .exe can be released; and
- how the team must hand off work without silently changing the freeze.

After approval and merge:

1. A later expressly approved and dated amendment overrides this document.
2. This document overrides conflicting product statements in earlier checkpoint, PR-comment, and freeze documents.
3. Source code and migrations remain the evidence of what is implemented; this document remains the authority for intended product behavior.
4. If code and this freeze disagree, the mismatch is product drift. Stop and raise it rather than treating either side as silently approved.

## 2. Verified repository and team baseline

The following was rechecked on GitHub on 2026-08-21.

| Item | Verified state |
| --- | --- |
| Repository | cyvoriq01solutions-coder/Erase |
| Default branch | main |
| Baseline commit | bf96431 — merge of PR #18 |
| Repository owner | cyvoriq01solutions-coder |
| Owner permission | Admin |
| Co-developer | mswaroop707-del, identified by the team as Swarup-Personal |
| Co-developer permission | Write |
| Co-developer evidence | Author of merged PRs #16 and #17 for admin hardening |
| Open pull requests | None at the time of this freeze |
| Main branch protection | Not enabled; no required checks are enforced |

GitHub permissions and CYVORIQ application roles are separate. Repository admin/write access does not grant CEO, Accounts, Support, or other runtime application authorization.

### Working agreement

- cyvoriq01solutions-coder is the repository owner and final approval authority for freeze changes, merges, releases, production deployment, secrets, and production data changes.
- mswaroop707-del is a co-developer with write access and works through dedicated branches and pull requests.
- Each branch should address one bounded concern and start from current main.
- Pull requests should be Draft until their scope, checks, and review evidence are ready.
- No direct production deployment, production migration, secret change, or release occurs merely because code is merged.
- Until branch protection is enabled, the team must enforce review and checks manually.

## 3. Frozen product definition

CYVRA Erase is a Windows privacy-verification and evidence product. The current customer MVP is deliberately non-destructive.

### Safety sequence

**Current MVP:**

ASSESS → PDEM → EVIDENCE → VERIFY → REPORT

**Separate future destructive lifecycle:**

AUTHORIZE → ERASE → VERIFY → CERTIFY

The future lifecycle is not enabled by this freeze. Destructive erase, wipe, delete, overwrite, or remediation must not be added to the current customer build without a separate security, legal, recovery, and product freeze.

### Non-negotiable privacy boundary

The current agent may collect only the metadata needed to build a Personal Data Exposure Map and verification report. It must not:

- read or transmit personal file contents;
- read email bodies or message content;
- read browser-history content;
- collect passwords, keys, tokens, or secrets;
- alter, delete, move, encrypt, or overwrite user files;
- execute destructive disk or account operations; or
- upload raw personal content to CYVORIQ services.

Metadata must be minimized, purpose-limited, auditable, and protected in transit and at rest.

## 4. Frozen customer experience

The target customer journey is:

1. Customer receives an authorized protected download.
2. Customer downloads the Windows installer through the Worker authorization gate.
3. Customer verifies publisher trust and installs CYVRA.
4. CYVRA launches a desktop GUI.
5. Customer signs in or enters a server-issued activation key.
6. The Worker validates entitlement and atomically binds the activation to one device.
7. The GUI confirms the detected device and privacy scope.
8. Customer starts a non-destructive verification.
9. CYVRA shows scanning progress without exposing private content.
10. CYVRA presents the Personal Data Exposure Map.
11. CYVRA presents evidence and verification results.
12. Customer generates, views, and saves an authenticated report.
13. Subsequent launches revalidate the same authorized device and entitlement.

A command prompt that prints JSON is an engineering interface, not the customer product.

### Frozen installer screens

The minimum installer/first-run sequence is:

Welcome → Terms → Ready to Install → Install → Launch → Sign in or Activation Key → Device Binding → Device Detected → Run Verification

The final wording and visual treatment may be refined without changing the contract above.

## 5. Compulsory first-500 trial revision

The launch trial is server-authoritative and applies to the first 500 eligible users.

- Trial eligibility and the count of accepted users are determined by the server, not by the Windows client.
- Each accepted user receives one server-issued activation entitlement/key.
- One activation is permitted per key and one bound device per key.
- The first successful activation atomically binds the entitlement to a privacy-preserving device fingerprint.
- Revalidation from the same device is allowed while the entitlement remains valid.
- A different-device activation is rejected and audited.
- There is no per-user payment or manual approval gate as a launch blocker for an eligible first-500 trial user.
- Installer access remains private and protected; the trial does not create a permanent public .exe URL.
- Authentication, rate limiting, audit, evidence integrity, report integrity, and release controls remain mandatory.
- Paid approval and payment workflows return after the trial or through a separately approved commercial amendment.

The Windows client must never decide that a user is within the first 500. It only consumes a signed or authenticated server decision.

## 6. System responsibilities

| Component | Frozen responsibility | Current position |
| --- | --- | --- |
| Windows CYVRA app | Installer, activation, one-device binding, GUI verification, PDEM, evidence, authenticated report | Engineering agent source exists; customer product incomplete |
| Cloudflare Worker at api.cyvra.co.in | Authentication, entitlement, activation, device binding, download authorization, report/evidence APIs, admin APIs | Foundation merged; customer product APIs incomplete |
| Neon Postgres | Users, entitlements, licences, activation/device binding, releases, audit, report metadata | Development database exists; required customer contracts incomplete |
| Private Cloudflare R2 | Store signed installer and release artifacts privately | Planned; integration and live bucket state not yet verified |
| Resend | Customer/admin OTP and transactional email | Admin OTP sender configured; broader release flow still requires verification |
| Public Cloudflare Pages | Marketing, account, and protected-download entry experience | Frontend exists; download copy still reflects the older paid-gate model |
| Admin Cloudflare Pages | Release, access, support, licence, audit, and operational control plane | C4.1 foundation merged; C5–C7 remain |
| GitHub | Source, review history, CI evidence, tagged release source | Active; main protection and required checks are missing |

### Required request path

The approved release path is:

Customer/Admin UI → Cloudflare Worker → Neon authorization/audit → private R2 artifact or authenticated report response

The browser must not receive database credentials, R2 credentials, email-provider secrets, signing secrets, or a permanent unauthenticated artifact URL.

## 7. Current Windows agent status

### Present in version 0.2.1 source

The engineering agent can collect or emit foundational information including:

- device, operating-system, CPU, storage, and volume metadata;
- BitLocker state;
- user-profile metadata;
- personal-data and application-data location metadata;
- a PDEM-shaped JSON result; and
- a foundation assessment result through a command-line execution path.

### Not yet customer-ready

The following remain incomplete or absent:

- desktop GUI and accessible customer flow;
- production installer;
- publisher code signing;
- protected download and release channel;
- activation and entitlement integration;
- privacy-preserving one-device fingerprint binding;
- authenticated Agent-to-Worker API;
- completed PDEM relationships;
- cryptographic evidence chain;
- substantive verification instead of a constant foundation-ready result;
- professional authenticated customer report;
- secure update and rollback mechanism;
- release telemetry and support diagnostics within the privacy boundary;
- installer, activation, upgrade, offline/failure, and uninstall testing; and
- customer pilot evidence.

Therefore, no team member may describe 0.2.1 as the latest customer-installable product. It is the latest known engineering foundation.

## 8. Control-plane and security status

### Merged foundation

- C3 customer authentication foundation is merged.
- C4.1 admin portal and authentication foundation is merged.
- C4.1A P0 hardening through P0-5 is merged.
- Admin and customer browser origins were separated.
- Admin session-cookie security was hardened.
- Admin identity enumeration was prevented.
- Durable, privacy-preserving admin OTP rate limits were added.
- Migration 0006 created admin_auth_rate_limits in Neon development only.
- PR #18 aligned the database health contract with migration 0006 and is merged into main.

### Security work still required

Before production readiness:

- enforce a 30-minute admin idle timeout with a four-hour absolute lifetime;
- enforce one active admin session per identity;
- audit suspicious and failed authentication events;
- validate Cloudflare Turnstile tokens on the server;
- define step-up OTP for high-risk admin actions;
- add passkeys only as a later optional enhancement;
- enable branch protection and required checks on main;
- verify every named Cloudflare environment has its own bindings, variables, and secrets;
- define production deployment and rollback evidence; and
- complete end-to-end authorization tests for download, activation, device binding, and reports.

## 9. Infrastructure audit snapshot

This section separates observed runtime state from repository state. It is a handoff snapshot, not a deployment authorization.

### Cloudflare

- Public Pages project erase and custom apex domain cyvra.co.in were observed working in the prior audit.
- The www.cyvra.co.in route was observed returning an error and requires a fresh DNS/Pages-route check.
- Admin Pages project cyvra-admin and admin.cyvra.co.in were observed working.
- The default pages.dev admin URL can show an API-origin failure because the Worker trusts the custom admin origin; that is expected unless the origin policy is intentionally expanded.
- Worker cyvoriq-erase-api has a Hyperdrive binding and repository configuration currently centered on development.
- Named staging and production Wrangler environments do not automatically inherit all top-level bindings and variables. Their Hyperdrive, origin, email, and secret configuration must be explicit before deployment.
- A direct production Worker deployment containing the latest PR #18 correction has not been verified by this freeze.
- Private R2 release distribution is not yet integrated.
- Turnstile is not yet implemented.
- Cloudflare Access was deferred by product choice; it is not recorded as a failed task.

### Neon

- Migration 0006 was approved and applied to the Neon development environment only.
- The development health contract expects 20 public tables, including admin_auth_rate_limits.
- The production Neon environment was not changed by the P0-5 work.
- Production schema state and release data must be reverified immediately before any approved production migration.

### Resend

- The admin OTP flow is configured to send from CYVORIQ <auth@otp.cyvra.co.in>.
- Domain authentication, delivery, bounce handling, and customer activation templates require release-stage verification.

## 10. W1 contract status before customer .exe implementation

W1.1 freezes the desktop framework, installer direction, Windows 10/11 scope, least-privilege model, reusable Rust core, future Server/offline provisions and passive hardware inventory through the linked W1.1 document.

The remaining W1 contracts must still define:

1. **Agent-to-Worker API:** request/response schemas, authentication, replay protection, idempotency, versioning, timeouts and privacy limits.
2. **Device binding:** privacy-preserving fingerprint inputs, normalization, salt/pepper ownership, change tolerance, rebind and recovery policy.
3. **Code signing and release:** certificate custody, signing environment, timestamping, artifact hashes, release approval, secure update and rollback.
4. **Authenticated report:** report format, claims, evidence hashes, verification endpoint or signature, retention, redaction and customer export.
5. **First-500 entitlement:** eligibility transaction, concurrency control, activation expiry, support reset, audit events and later paid conversion.
6. **Observability:** privacy-safe logs, failure codes, crash/support bundle policy, retention and access controls.

Implementation spikes may evaluate details, but a spike must not silently change a frozen product or security contract.

## 11. Delivery sequence

### H0 — Canonical handoff and governance

- Approve and merge this document.
- Correct the obsolete README.
- Correct historical checkpoint language that still calls merged PR #15 a Draft.
- Move the first-500 rule out of PR-comment-only history by retaining it here.
- Enable main branch protection and required checks.

### W1 — Windows and service contracts

- Decide the eight contracts in Section 10.
- Add versioned API and report schemas.
- Add threat model, privacy data map, and failure-state acceptance criteria.
- Freeze the first customer-build definition of done.

### W2 — Customer desktop shell

- Place the existing metadata collectors behind a GUI application boundary.
- Implement consent, scope, progress, cancellation, error, and safe-retry behavior.
- Complete PDEM relationships and substantive verification.
- Keep all destructive operations disabled.

### W3 — Activation, evidence, and report services

- Implement first-500 entitlement transaction.
- Implement one-device binding and audited recovery.
- Implement Agent-to-Worker authentication and replay protection.
- Implement evidence integrity and authenticated report generation/verification.

### W4 — Private release pipeline

- Create and verify the private R2 release store.
- Build the production installer.
- Sign and timestamp installer artifacts.
- Publish release metadata and hashes through the control plane.
- Issue only short-lived, authorized downloads.
- Implement secure update and rollback.

### W5 — End-to-end validation

- Test clean install, upgrade, repair, uninstall, activation, same-device revalidation, different-device rejection, expired/revoked entitlement, offline/network failure, and rollback.
- Test privacy boundaries and verify that no personal content or secret is collected.
- Test report authenticity and tamper detection.
- Complete security review and first-500 concurrency tests.
- Run a controlled internal pilot before customer release.

### W6 — Release decision

- Review evidence, unresolved risks, support readiness, and rollback readiness.
- Approve production database changes separately.
- Approve production Cloudflare deployment separately.
- Approve signed artifact publication separately.
- Record the exact release version, commit, hashes, signer, deployment IDs, and go/no-go decision.

C5–C7 control-plane work can proceed in bounded branches after the relevant contracts are frozen, but it must not invent Windows, activation, report, or trial behavior independently.

## 12. Customer-build definition of done

A build is not a customer-installable CYVRA release until all of the following are true:

- a signed installer is produced by the approved release pipeline;
- publisher identity and timestamp verification pass;
- the installer is stored privately and delivered only after authorization;
- first-500 eligibility is enforced atomically by the server;
- activation binds one key to one privacy-preserving device identity;
- the GUI completes the frozen non-destructive verification journey;
- the PDEM and evidence are substantive and internally consistent;
- the report is authenticated and tamper-verifiable;
- no prohibited personal content or secret is collected;
- clean install, upgrade, uninstall, failure, and rollback tests pass;
- Cloudflare and Neon production changes have separate approvals and evidence;
- support and audited recovery procedures exist; and
- the owner records an explicit go-live approval.

A compiled executable alone does not satisfy this definition.

## 13. Known documentation and product drift

The following must be corrected in separate, reviewable documentation work:

- The README incorrectly says the repository has no Worker or database.
- The C4 checkpoint still describes PR #15 as Draft even though it was merged.
- The public download experience reflects an older paid-gate model and must be reconciled with the first-500 trial.
- The first-500 revision previously lived primarily in a PR comment.
- The original Windows product freeze was not sufficiently represented in the repository.
- Runtime deployment state and code-merged state have sometimes been discussed as if they were identical.

Until corrected, this document governs the conflicting product statements.

## 14. Open owner decisions

The linked W1.1 freeze resolves the GUI framework, installer direction, Windows 10/11 V1 scope, Server-next sequence, offline-retirement provision, privilege model and passive hardware inventory scope.

These remain unauthorized assumptions until later contracts are approved:

- code-signing certificate/provider and custody;
- production API and release environment sequence;
- exact device-fingerprint and support-rebind policy;
- authenticated report format and verification experience;
- first-500 eligibility start event and trial duration;
- update-channel rollout and rollback thresholds;
- final Windows Server version/package matrix;
- offline boot-environment licensing and sanitization controls;
- retention and redaction periods for hardware identifiers; and
- production go-live date.

Each decision must be added by a dated amendment or an approved contract document linked from this freeze.

## 15. Change control and handoff checklist

Every implementation handoff must state:

- branch and base commit;
- frozen requirement or issue addressed;
- files and schemas changed;
- privacy/security impact;
- migrations and target environment;
- variables, bindings, and secrets required without exposing secret values;
- tests run and their results;
- deployment performed, or explicitly not performed;
- rollback method;
- open risks and decisions; and
- next action requiring approval.

A developer receiving work must compare it with this document before extending it. A conflict must be raised in the pull request, not resolved through an undocumented implementation choice.

## 16. Repository references

Earlier documents remain useful history and detailed checkpoints:

- docs/product/control-plane-c3-c7-freeze.md
- docs/product/c4-c5-admin-download-freeze-2026-08-19.md
- docs/product/c4-1-admin-auth-freeze.md
- docs/product/c4-admin-pages-checkpoint-2026-08-19.md
- docs/product/frontend-commercial-v1-freeze.md
- docs/deployment/c3-auth-test-rollback.md

Relevant merged pull requests:

- PR #15 — Admin portal foundation
- PR #16 — C4.1A admin hardening
- PR #17 — Durable admin OTP rate limits
- PR #18 — Database health-contract correction

## 17. Immediate next approved action

Complete the outstanding README documentation gate, then review and merge the linked W1.1 freeze. The first Windows implementation branch must refactor agent 0.2.1 into a typed reusable core and define hardware_inventory_v1 without changing the non-destructive boundary.

Do not create a customer release by merely wrapping the current command-line JSON output in an installer. Activation, API, device binding, signing, report, first-500, privacy and release contracts remain mandatory, and destructive or credential-bypass capability remains outside the authorized scope.
