# CYVORIQ Cloudflare Worker — Phase 3

## Purpose

This Worker is the dedicated CYVORIQ edge API/control plane. The React frontend must call the Worker over HTTPS. The browser must never receive Neon credentials, Hyperdrive credentials, or destructive device authority.

## Frozen Phase 3 scope

- Worker project exists independently under `/worker`.
- API is versioned from day one under `/api/v1/...`.
- Health endpoint: `GET /api/v1/health`.
- Development, staging, and production Worker names are separated.
- No Neon or Hyperdrive binding yet.
- No R2 binding yet.
- No Render dependency yet.
- No device-agent destructive endpoint.

## Local verification

From repository root:

```bash
cd worker
npm install
npm run typecheck
npm run deploy:dry-run
npm run dev
```

Test locally:

```bash
curl http://localhost:8787/api/v1/health
```

Expected JSON contains `status: "ok"` and `apiVersion: "v1"`.

## Cloudflare deployment

Do not deploy until local typecheck and dry-run both pass.

When approved:

```bash
npm run deploy
```

Staging:

```bash
npx wrangler deploy --env staging
```

Production:

```bash
npx wrangler deploy --env production
```

Secrets must be stored with Wrangler/Cloudflare secret management, never committed to Git.
