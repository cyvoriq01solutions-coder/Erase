# CYVRA desktop shell

This directory contains the W2.1B Tauri 2 + React/TypeScript customer desktop shell foundation.

It implements the frozen application frame and five primary destinations:

- Overview
- Verification
- Results
- Report
- Help

## Safety boundary

W2.1B is an internal engineering foundation, not a customer release. It deliberately keeps the following capabilities disabled:

- live activation and device binding;
- hardware or privacy collection;
- CYVRA QC grade issuance;
- authenticated report generation;
- destructive erasure; and
- installer bundling or publication.

The frontend has one read-only Tauri command, `get_shell_bootstrap`. The Rust crate links the reusable `cyvra_core` library directly; it does not invoke the engineering CLI as a subprocess. The only Tauri capability granted to the main window is `core:default`.

The browser adapter exists only for safe visual review. It returns a fixed fail-closed foundation state and never represents native integration as active.

## Local checks

From this directory:

```bash
npm ci
npm run check
npm test
npm run build:web
```

With the Windows Rust and WebView2 build prerequisites installed:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri -- build --debug --no-bundle
```

The final command compiles the internal shell executable without creating a distributable installer. Signed NSIS packaging remains a later, separately approved release-pipeline gate.

## Customer entry boundary

This shell does not replace the protected website download journey. The customer must still enter through the CYVRA website, verify identity, satisfy server-authoritative entitlement and legal gates, receive a short-lived private signed-installer download, and activate/bind the authorized device before live desktop functions become available.
