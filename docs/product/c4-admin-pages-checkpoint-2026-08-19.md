# C4 Admin Portal Checkpoint — 2026-08-19

Status: ACTIVE DEVELOPMENT CHECKPOINT

This document extends the earlier frozen checkpoint and records the current C4 state.

## Cloudflare Pages admin project

Dedicated Pages project:

- Project name: `cyvra-admin`
- Pages hostname: `cyvra-admin.pages.dev`
- Git repository: `cyvoriq01solutions-coder/Erase`
- Production branch: `admin-portal-v1`
- Root directory: `admin-frontend`
- Build command: `npm run build`
- Build output: `dist`
- Framework: React + Vite + TypeScript
- Automatic deployments: enabled
- Build watch include path: `admin-frontend/*`
- Latest C4 Admin Foundation Pages build: successful via Cloudflare Git integration

The existing public/customer project `erase` and Worker `cyvoriq-erase-api` remain separate applications.

## Admin custom domain

`admin.cyvra.co.in` has been added through the `cyvra-admin` Pages Custom Domains workflow.

Cloudflare created the required relationship:

- Type: CNAME
- Name: `admin`
- Target: `cyvra-admin.pages.dev`
- TTL: Auto

Current observed dashboard state at this checkpoint: `Initializing` / DNS and TLS provisioning in progress.

Do not manually replace the DNS record while Pages is provisioning the custom domain.

## Admin frontend foundation

The former static provisioning shell has been upgraded to the same frozen frontend stack as the customer site:

- React 19.2.8
- React DOM 19.2.8
- React Router 8.3.0
- TypeScript 5.9.3
- Vite 8.2.0

Admin UI foundation includes:

- internal email OTP entry
- CEO / Accounts identity restriction in the UI
- server-session verification
- server-authoritative admin-session confirmation before rendering the control panel
- protected navigation shells for Customers, Orders, Payments, Approvals, Licences, Download Entitlements, Activations / Devices, Audit Events, Management Report and Accounts Report
- Super Administrator-only Internal Users / Roles area
- no live customer/payment/licence/device data loaded yet
- CSP/security headers
- `noindex,nofollow,noarchive`
- SPA fallback

## Admin backend authority added on branch

New Worker routes are implemented on `admin-portal-v1`:

- `GET /api/v1/admin/session`
- `POST /api/v1/admin/roles/accounts/approve`
- `POST /api/v1/admin/roles/accounts/revoke`

Authority rules:

- `ceo@cyvoriq.com` becomes active `super_admin` after successful email OTP, as already implemented by the auth challenge service.
- `accounts@cyvra.co.in` remains pending after email verification until CEO approval.
- Accounts approval/revocation requires the authenticated bootstrap CEO Super Administrator session.
- Role changes are transactional in Neon.
- Approval records `ADMIN_ROLE_APPROVED` in `audit_events`.
- Revocation records `ADMIN_ROLE_REVOKED` in `audit_events`.

## Admin browser origins

The branch Worker configuration allows the dedicated admin origins in addition to the customer portal origins:

- `https://admin.cyvra.co.in`
- `https://cyvra-admin.pages.dev`

Runtime CORS verification is still required before production freeze.

## GitHub state

Draft PR:

- PR #15 — `C4 Admin Portal Foundation V1`
- Base: `main`
- Head: `admin-portal-v1`
- State: draft / not merged

Cloudflare Pages build for the dedicated `cyvra-admin` project passed on the Admin Foundation frontend.

Do not merge PR #15 yet.

## Security gates still required

1. Wait for `admin.cyvra.co.in` to become Active with valid TLS.
2. Put Cloudflare Access in front of `admin.cyvra.co.in`.
3. Initial Access allow identities: `ceo@cyvoriq.com`, `accounts@cyvra.co.in` only.
4. Configure and verify `api.cyvra.co.in` for the Worker control plane.
5. Verify Worker branch deployment/typecheck with the new admin routes.
6. Run real CEO OTP -> admin session -> control-panel test.
7. Run real Accounts OTP -> pending-role state.
8. CEO approves Accounts role from the admin control panel.
9. Verify Accounts can then enter with `accounts_admin` but cannot use Super Administrator role controls.
10. Verify Neon `user_roles` state and audit events for approval/revocation.
11. Test logout and manual protected-route access.

## C3 dependency

Customer Auth C3 remains separate and open in draft PR #14. It must still pass the previously frozen real OTP/session/Neon/logout/protected-route acceptance gates before it is treated as production-complete.

## Merge order warning

PR #14 should be completed first. After C3 is merged, rebase/update `admin-portal-v1` from the new `main` and preserve the combined CORS/origin changes before PR #15 is considered for merge.
