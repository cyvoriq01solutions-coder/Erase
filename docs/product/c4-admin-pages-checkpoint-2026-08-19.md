# C4 Admin Pages Checkpoint — 2026-08-19

Status: FROZEN CHECKPOINT

This checkpoint records the exact state reached before pausing work.

## Cloudflare Pages admin project

A dedicated Cloudflare Pages project has been created successfully:

- Project name: `cyvra-admin`
- Pages hostname: `cyvra-admin.pages.dev`
- Git repository: `cyvoriq01solutions-coder/Erase`
- Production branch: `admin-portal-v1`
- Latest deployed commit: `c68a02b` (`admin: add isolated Pages build contract`)
- Deployment status: successful
- Automatic deployments: enabled

## Admin build isolation

The admin Pages project is isolated from the existing customer/public frontend and Worker.

Build configuration:

- Build command: `npm run build`
- Build output: `dist`
- Root directory: `admin-frontend`
- Framework preset: `None`
- Build system version: Version 3
- Build cache: disabled
- Build comments: enabled

The existing public/customer project `erase` and Worker `cyvoriq-erase-api` are not replaced by this admin Pages project.

## Current admin shell

The branch `admin-portal-v1` contains a minimal provisioning shell under `admin-frontend/` with:

- `index.html`
- `_headers`
- `package.json`

The shell exposes no customer, commercial, licence, activation, management, or account data. It is only a safe provisioning target.

## Custom domain status

`admin.cyvra.co.in` has NOT yet been attached.

Cloudflare Pages is currently ready at:

`cyvra-admin` -> Custom domains -> `Set up a custom domain`

The next intended domain action is:

`admin.cyvra.co.in` -> dedicated Pages project `cyvra-admin`

Do not point `admin.cyvra.co.in` to the existing `erase` Pages project and do not create a placeholder DNS record manually.

## Security sequence after custom-domain attachment

After `admin.cyvra.co.in` is attached and validated:

1. Add Cloudflare Access in front of the admin hostname.
2. Initial Access identities: `ceo@cyvra.co.in` and `accounts@cyvra.co.in` only.
3. Keep CYVRA email-OTP authentication as a second identity check.
4. Enforce Worker-side RBAC from Neon:
   - `ceo@cyvra.co.in` -> `super_admin`
   - `accounts@cyvra.co.in` -> CEO-approved `accounts_admin`
5. Never rely on the subdomain being hidden as the security boundary.

## Build watch path follow-up

Current Cloudflare build watch path is still broad (`*`).

After resuming, narrow it to the admin application path so unrelated repository changes do not unnecessarily rebuild the admin Pages project. Intended scope: `admin-frontend/*` or the current Cloudflare-supported equivalent.

## C3 dependency remains open

Customer Auth C3 is still separate from C4. Draft PR #14 must not be treated as production-complete until the previously frozen C3 acceptance gates pass, including real OTP/session/Neon/logout/protected-route checks.

## Resume point

When work resumes, continue from this exact order:

1. Verify `cyvra-admin.pages.dev` opens the isolated admin provisioning shell.
2. Narrow the admin Pages build watch path.
3. Attach `admin.cyvra.co.in` through `cyvra-admin` -> Custom domains.
4. Verify DNS/TLS and that only the admin shell is served.
5. Configure Cloudflare Access for the admin hostname.
6. Then continue C3 production API/OTP acceptance before treating authentication as production-ready.

No further changes should be made until work resumes.
