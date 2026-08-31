# Private Backblaze B2 release store

Customer installers are not public Pages files and must not use a permanent
B2 or R2 URL. The Worker signs a private S3 GET after C5 admin approval and
streams the bytes. There is no `r2_buckets` binding — Cloudflare R2 is not
enabled on this account (API error 10042).

## Bucket (already created)

- Name: `cyvra-erase-releases`
- Type: Private
- Endpoint: `s3.us-east-005.backblazeb2.com`
- Region: `us-east-005`
- Encryption: SSE-B2
- Leave empty until a release is intentionally published

Do **not** upload an unsigned GitHub Actions EXE as the customer package.
Object key when a signed (or explicitly approved) build exists:

`releases/0.3.0/CYVRA-Erase-0.3.0-x64-setup.exe`

## Worker secrets (Cloudflare Dashboard)

Worker `cyvoriq-erase-api` → Settings → Variables and Secrets → Encrypt:

| Name | Value |
| --- | --- |
| `B2_KEY_ID` | Backblaze application keyID |
| `B2_APPLICATION_KEY` | applicationKey (shown once) |

`B2_BUCKET`, `B2_ENDPOINT`, and `B2_REGION` are non-secret `vars` in
`worker/wrangler.jsonc`. Do not commit the applicationKey.

Restricted app key: this bucket only, Read and Write. Never use the master key.

Until secrets exist, `GET /api/v1/auth/download/setup` returns 503
`release_store_unconfigured` for approved customers. Anonymous callers still
get 401.

Until the object exists:

- `GET /api/v1/auth/download-status` reports `packageAvailable: false`
- `GET /api/v1/auth/download/setup` returns 404 `package_not_released` for
  approved customers (after secrets are set)

## Gates

```bash
curl -sS -o /dev/null -w "%{http_code}\n" https://api.cyvra.co.in/api/v1/auth/download/setup
# 401
```
