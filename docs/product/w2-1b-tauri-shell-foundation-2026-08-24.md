# CYVORIQ ERASE — W2.1B Tauri Shell Foundation

> **Status:** IMPLEMENTED — OWNER REVIEW REQUIRED
>
> **Date:** 2026-08-24
>
> **Branch:** `w2-1b-tauri-shell-foundation`
>
> **Repository baseline:** `848e61a819f6214356c33341c373a3ce79176ff8`
>
> **Scope:** A least-privilege Tauri 2 + React/TypeScript desktop shell implementing the approved W2.1/W2.1A application frame without enabling customer operations.
>
> **Change boundary:** No deployment, database mutation, secret change, code-signing operation, installer publication, production activation, hardware collection, grading issuance, authenticated report or destructive erasure is authorized by this package.

## 1. Outcome

W2.1B establishes the compiled customer desktop application boundary required before live services and collectors are integrated.

The package provides:

- a Tauri 2 native Windows application crate;
- a React 19 + TypeScript 5 customer shell;
- the approved CYVORIQ/CYVRA visual foundation;
- an exact package-local copy of the repository's approved CYVORIQ logo asset;
- the five frozen primary navigation destinations;
- one read-only frontend-to-Rust bootstrap command;
- a direct typed dependency on the reusable `cyvra_core` library;
- explicit fail-closed capability flags;
- least-privilege Tauri capabilities and a restrictive content security policy;
- contract tests guarding the safety boundary; and
- Windows CI that compiles an internal executable without producing an installer.

This is not a customer-installable release.

## 2. Frozen inputs honored

The implementation follows:

- `docs/product/w1-1-desktop-installer-hardware-freeze-2026-08-21.md`;
- `docs/product/w2-1-shared-customer-gui-ux-contract-2026-08-24.md`;
- `docs/product/w2-2-cyvra-qc-grading-contract-2026-08-24.md`; and
- `docs/product/w2-1a-gui-visual-design-package-2026-08-24.md`.

No implementation detail in this branch amends those contracts.

## 3. Customer entry remains server-authoritative

The desktop shell does not replace or weaken the protected website journey.

The required customer path remains:

1. Customer selects Download CYVRA Erase on the main website.
2. Customer creates an account or signs in.
3. Customer verifies email ownership by OTP.
4. Customer selects the account type and accepts the required legal documents.
5. The server atomically verifies first-500 or later commercial entitlement.
6. The authorized customer receives a short-lived private URL for the approved signed installer.
7. First activation verifies the entitlement and binds it to one privacy-preserving device identity.
8. A separate revocable device token authorizes later desktop communication.
9. The shared desktop journey presents CYVRA QC and CYVRA Erase results.
10. The final service produces an authenticated, tamper-verifiable report.

Registration alone must never reveal a raw installer URL. W2.1B implements none of the server decisions above and cannot bypass them.

## 4. Desktop architecture

### 4.1 Frontend

The `desktop` package uses:

- React `19.2.8`;
- React DOM `19.2.8`;
- TypeScript `5.9.3`;
- Vite `8.2.0`;
- `@tauri-apps/api` `2.11.1`; and
- `@tauri-apps/cli` `2.11.4`.

All dependency versions are exact and the generated package lock is required before commit.

### 4.2 Native application

The Rust application uses:

- package name `cyvra-desktop`;
- application version `0.3.0`;
- Rust `1.97.1`, edition 2024;
- Tauri `2.11.5`;
- `tauri-build` `2.6.3`; and
- a direct path dependency on the existing `cyvoriq-erase-agent` package under the library alias `cyvra_core`.

The temporary application identifier is `in.co.cyvra.erase`. It is suitable only for this internal foundation and must be confirmed by the signing/release contract before a customer package is built.

### 4.3 Typed core boundary

The native crate links `cyvra_core` as a Rust library. It does not:

- invoke the engineering command-line agent;
- parse CLI JSON;
- spawn a process;
- expose generic shell execution;
- grant generic filesystem access; or
- duplicate collector logic in the GUI.

W2.1B links a stable core type to prove the compile-time relationship while leaving collector execution disabled.

## 5. Narrow command surface

The only Tauri command is:

`get_shell_bootstrap`

It returns:

- application version;
- runtime mode;
- core-boundary identity; and
- five explicit capability flags.

Every capability flag is `false`:

- destructive operations;
- live activation;
- live collection;
- grading issuance; and
- report authentication.

The TypeScript adapter validates the shape and rejects the bootstrap if any protected capability is unexpectedly enabled.

## 6. Browser review adapter

The frontend can be reviewed in a normal browser without pretending that Tauri is available.

The browser adapter:

- is marked `browser_design_adapter`;
- reports the native bridge as not loaded;
- keeps every protected capability disabled;
- performs no network request;
- persists no customer state; and
- displays only truthful empty and pending states.

The adapter is for internal design review only.

## 7. Frozen navigation implemented

The customer shell includes exactly:

1. Overview
2. Verification
3. Results
4. Report
5. Help

The screens use the approved shared-product model:

- CYVRA QC presents the future evidence-based condition outcome;
- CYVRA Erase presents the future privacy-exposure assessment;
- both remain inside one application, one verification journey and one combined report; and
- no obsolete `XCQC` name appears.

## 8. Truthful foundation states

W2.1B never creates sample device data that could be mistaken for evidence.

The customer-facing state is:

- activation: `Not connected`;
- device binding: `Not started`;
- verification: `Not started`;
- CYVRA QC: `Grade pending`;
- CYVRA Erase: `Not assessed`;
- report: `Not generated`; and
- erasure status: `No data was erased`.

The primary verification button is disabled until consent, orchestration, typed progress and cancellation contracts are implemented.

## 9. Security and privacy posture

The main-window capability grants only `core:default`.

The Tauri configuration:

- disables global Tauri injection;
- freezes the JavaScript prototype;
- disables the asset protocol;
- restricts script, style, image, connection, object, form and frame sources with CSP;
- disables file drag-and-drop; and
- disables application bundling.

The frontend source contains no:

- `fetch` client;
- `XMLHttpRequest` client;
- WebSocket client;
- local storage usage; or
- session storage usage.

No activation key, raw device identifier, personal content or customer evidence is collected or retained.

## 10. Accessibility foundation

The shell includes:

- semantic primary navigation;
- visible keyboard focus;
- a skip link;
- focus transfer when the primary destination changes;
- programmatic current-page state;
- accessible notice roles;
- reduced-motion handling;
- forced-colors handling; and
- responsive layouts for the approved minimum window size.

Formal keyboard, screen-reader, scaling and Windows high-contrast validation remains required before customer release.

## 11. Build and test gates

The branch adds checks for:

- exact frontend dependencies;
- exact approved-logo blob identity;
- TypeScript compilation;
- production web-asset compilation;
- exactly five frozen navigation destinations;
- exactly one frontend invocation and one Rust command;
- direct typed core dependency;
- fail-closed capability flags;
- absence of network and persistent-browser clients;
- least-privilege Tauri capabilities;
- restrictive CSP;
- absence of destructive customer actions; and
- absence of the obsolete `XCQC` name.

Windows CI must also pass:

- Rust formatting;
- strict Clippy with warnings denied;
- Rust unit tests; and
- Tauri debug compilation with `--no-bundle`.

No artifact is uploaded and no installer is built in W2.1B.

## 12. Files introduced or changed

- root `package.json` desktop scripts;
- root `.gitignore` Tauri build exclusions;
- `.github/workflows/desktop-shell-build.yml`;
- `desktop/package.json` and lockfile;
- `desktop/index.html`;
- `desktop/tsconfig.json`;
- `desktop/vite.config.ts`;
- `desktop/src/**`;
- `desktop/tests/shell-contract.test.mjs`;
- `desktop/src-tauri/**`;
- `desktop/README.md`; and
- this evidence document.

## 13. Deployment and environment impact

- Production deployment: not performed.
- Cloudflare deployment: not performed.
- Neon database migration: not performed.
- R2 package publication: not performed.
- Resend configuration: not changed.
- Secrets: none added or changed.
- Code signing: not performed.
- Customer installer: not produced.
- Existing Windows collector behavior: not changed.

## 14. Rollback

Before merge, delete or close the branch without merging.

After merge, revert the W2.1B merge commit. The change introduces no database, deployment, entitlement or external-state migration.

## 15. Open gates

W2.1B does not authorize the next integration automatically. The following remain separately frozen or required:

- Agent-to-Worker API authentication and replay protection;
- privacy-preserving device binding and recovery;
- activation and first-500 entitlement transactions;
- typed verification orchestration, progress and cancellation;
- remaining passive collectors and physical Windows 11 validation;
- PDEM evidence rules;
- server-authoritative grading issuance;
- authenticated report generation and verification;
- code-signing custody;
- signed NSIS packaging;
- private R2 release delivery;
- secure updates and rollback; and
- complete Windows acceptance and internal pilot evidence.

## 16. Approval gate

Before commit and push, the owner must review:

- the shell visual fidelity;
- the truthful empty and pending states;
- the one-command native boundary;
- the customer website-to-desktop separation;
- the CI gates; and
- the explicit absence of customer-release capability.

Commit, push, PR creation and merge each remain explicit owner-controlled actions.
