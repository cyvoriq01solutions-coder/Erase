# C3 Temporary Acceptance-Test Rollback Checklist

Status: REQUIRED before final production freeze of PR #14.

Purpose: document temporary infrastructure and data changes created only to validate the C3 customer authentication flow safely before merge.

## Temporary items created for testing

### 1. `auth-test.cyvra.co.in`

Purpose: same-site acceptance hostname for branch `portal-auth-ui-v1`.

Temporary mapping:

`auth-test.cyvra.co.in` -> proxied CNAME -> `portal-auth-ui-v1.erase-e93.pages.dev`

Rollback after C3.1 production acceptance:

- Remove `auth-test.cyvra.co.in` from the `erase` Pages project Custom Domains.
- Delete the `auth-test` CNAME from the `cyvra.co.in` DNS zone.
- Verify `cyvra.co.in` remains mapped to the production `erase` Pages project.

### 2. Temporary Worker CORS origin

Runtime Worker setting `PORTAL_ORIGINS` was manually extended with:

`https://auth-test.cyvra.co.in`

Rollback:

- Remove only `https://auth-test.cyvra.co.in` after final C3.1 production verification.
- Preserve the production customer origins and all permanent admin/API settings.

### 3. Branch preview hostname

`portal-auth-ui-v1.erase-e93.pages.dev` is a preview hostname only. Do not publish it as a customer URL.

### 4. Test customer records

Real browser testing created development customer/session/challenge/audit data. Before production reporting is treated as business reporting:

- identify/deactivate or otherwise clearly exclude test customer identities from business metrics;
- expire/revoke test sessions/challenges where appropriate;
- preserve material audit evidence according to the audit-retention rule rather than rewriting it as genuine business activity.

## Permanent items - never remove during this rollback

- `cyvra.co.in` / `www.cyvra.co.in`
- `api.cyvra.co.in`
- `admin.cyvra.co.in`
- `cyvra-admin` Pages project
- `cyvoriq-erase-api` Worker
- Hyperdrive binding
- Neon migrations/schema
- Resend OTP DNS and sender configuration
- `AUTH_EMAIL_TOKEN`
- `AUTH_PEPPER`

## Rollback verification

After cleanup, verify:

1. `https://cyvra.co.in` loads the production site.
2. `https://api.cyvra.co.in/api/v1/health` returns `status: ok`.
3. `https://api.cyvra.co.in/api/v1/db/health` remains healthy.
4. `https://admin.cyvra.co.in` still resolves to the isolated Admin Pages project.
5. `auth-test.cyvra.co.in` no longer resolves as a customer test surface.
6. Production `PORTAL_ORIGINS` no longer contains the temporary test hostname.
