# CYVORIQ Erase — Cloudflare Pages deployment freeze

## Purpose

This document freezes the frontend deployment target for CYVORIQ Erase Phase 2.

- Frontend/static delivery: Cloudflare Pages
- API/control plane: dedicated Cloudflare Worker in a later phase
- Database: Neon through Hyperdrive in a later phase
- The browser must never receive database credentials.

## Cloudflare Pages project settings

Use the GitHub repository as the source and configure the Pages project as follows:

- Production branch: `main`
- Root directory: `frontend`
- Framework preset: React (Vite), or no preset with the same explicit values below
- Build command: `npm run build`
- Build output directory: `dist`

Do not use `npx wrangler deploy` for the frontend. Wrangler/Worker deployment belongs to the dedicated Worker phase, not this Pages frontend phase.

## Build contract

From the repository root:

```bash
cd frontend
npm ci
npm run build
```

The production artifact must be generated in:

```text
frontend/dist/
```

## SPA routing

No top-level `404.html` is shipped. Cloudflare Pages therefore uses its default SPA behavior and routes unknown frontend paths to the root application, allowing React Router to resolve routes such as `/platform`, `/security`, and `/app/dashboard`.

## Static response headers

`frontend/public/_headers` is copied by Vite into the build output and adds conservative browser security headers. A Content-Security-Policy is intentionally deferred until the production Worker/API hostname and any approved third-party origins are frozen.

## Environment boundary

Frontend environment variables are public at build/runtime when exposed through Vite. Never place any of the following in `VITE_*` variables:

- Neon connection strings
- database passwords
- Cloudflare API tokens
- Worker secrets
- private signing keys

Only public browser configuration may use `VITE_*` variables.

## Preview deployments

Keep preview deployments enabled for non-production branches and pull requests. Production remains `main` only.

## Phase 2 acceptance checks

```bash
cd frontend
node --version
npm --version
npm ci
npm run build
test -f dist/index.html
test -f dist/_headers
```

Expected result: all commands succeed and `dist/index.html` plus `dist/_headers` exist.
