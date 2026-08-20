# C4/C5 Freeze — Admin Visual Identity + Secure Installer Distribution

Status: FROZEN DESIGN DECISION
Date: 2026-08-19

## 1. Admin Portal visual identity

The internal Admin Portal must be visually distinct from the public/customer CYVRA experience.

Frozen palette direction:
- Graphite / charcoal for primary shell and navigation
- Emerald / teal for primary actions, authority indicators and selected states
- Neutral grey / off-white for work surfaces
- Red reserved only for destructive/revoke/error actions

Do not reuse the public CYVRA navy + orange palette as the dominant Admin Portal palette.

Purpose: an operator must immediately know whether they are in the Customer Portal or the Admin Control Plane.

## 2. Installer binary storage

The CYVRA Erase Windows `.exe` must NOT be stored in the browser UI or embedded in the Admin Portal application bundle.

Frozen architecture:
- Binary object storage: private Cloudflare R2 bucket
- Admin Portal: release-management/control interface only
- Worker (`api.cyvra.co.in`): authoritative download gate
- Neon: release metadata, entitlement state, licence state, download/audit records

Recommended private bucket logical name: `cyvra-erase-releases-private`.

The bucket should remain non-public. No permanent public `.exe` URL is to be exposed.

## 3. Release management

The Admin Portal will later include a Super-Administrator release area for:
- upload/select a signed installer build
- version number
- release channel/status
- file size
- SHA-256 checksum
- publication/withdrawal state
- release notes
- created/published timestamps

For MVP, Accounts Administrator must not have installer-upload or release-publication authority.

## 4. Customer download authority

A customer download is permitted only after the existing frozen commercial gates pass server-side:
1. verified email
2. active customer account
3. valid order
4. payment recorded received
5. purchase approved by an active authorized admin
6. active licence issued
7. download entitlement enabled and not revoked/expired

The browser must never decide download eligibility.

The customer clicks Download in the protected Customer Portal. The portal calls the Worker. The Worker checks Neon and, only if authorized, returns/streams the installer from private R2 or issues a very short-lived authorized object download mechanism.

No R2 credentials are exposed to the browser.

## 5. Installer and activation key are separate

The same signed CYVRA Erase installer binary may be downloaded by many entitled customers. A unique installer binary per customer is not required.

Each approved customer receives a unique product activation/licence key issued server-side.

Frozen activation rule:
- plaintext licence key shown only when issued/delivered as required by the product flow
- only a hash is retained server-side after issuance according to the existing security invariant
- licence defaults to one device (`max_devices = 1`)
- first successful activation binds the licence to the authorized device/fingerprint
- the same key must not activate a second unrelated device
- any legitimate transfer/rebind must require an explicit server-authoritative admin workflow and audit evidence

This keeps download entitlement and device activation as separate security controls.

## 6. Intended customer flow

Account -> Email OTP -> Purchase/Payment -> Admin Approval -> Licence + Download Entitlement -> Download signed EXE -> Windows-style installer -> Enter activation key -> First-device binding -> Product use.

## 7. Windows installer UX freeze

Installer should feel like a professional Windows security product installer:
- familiar Windows wizard presentation
- short, fast sequence
- Next-driven flow
- licence/terms acknowledgement
- installation location/defaults where appropriate
- install progress
- completion/launch
- activation presented immediately after install or first launch

Avoid a developer-tool appearance, terminal-driven workflow or complicated setup screens for normal customers.
