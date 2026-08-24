# W2.1A CYVRA GUI Visual Design Package

This folder contains the repository-local, non-production visual prototype for the shared CYVRA Windows customer application.

## Contents

- `prototype/index.html` — clickable screen prototype
- `prototype/styles.css` — approved-token candidate visual layer
- `prototype/app.js` — local-only prototype navigation and state simulation
- `../../product/w2-1a-gui-visual-design-package-2026-08-24.md` — complete design specification and review checklist

## Review locally

From the repository root:

```bash
python3 -m http.server 4173 --directory /workspaces/Erase
```

Open the forwarded port and navigate to:

```text
/docs/design/w2-1a/prototype/
```

The prototype loads the approved logo from `frontend/public/cyvoriq-logo.webp`.

## Safety boundary

This prototype:

- uses static sample data only;
- calls no API;
- stores no credentials or customer data;
- runs no collector;
- performs no activation or device binding;
- generates no authenticated report;
- performs no destructive action; and
- is excluded from customer packaging.

It exists only for owner review before Tauri production implementation.
