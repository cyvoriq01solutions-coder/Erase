# Private R2 release store (slice B)

Customer installers are not public Pages files and must not use a permanent
R2 URL. The Worker binding `RELEASES` streams one allowlisted object after
C5 admin approval.

## Create the bucket (Cloudflare Dashboard)

1. Cloudflare Dashboard → R2 Object Storage → Create bucket
2. Name: `cyvra-erase-releases` (must match `worker/wrangler.jsonc`)
3. Location: leave default unless you already standardised on a region
4. Public access: **Off** (do not connect a custom public domain)
5. Create bucket

Do **not** upload an unsigned GitHub Actions EXE as the customer package.
Object key when a signed (or explicitly approved) build exists:

`releases/0.3.0/CYVRA-Erase-0.3.0-x64-setup.exe`

Until that object exists:

- `GET /api/v1/auth/download-status` reports `packageAvailable: false`
- `GET /api/v1/auth/download/setup` returns 404 `package_not_released` for
  approved customers
- Anonymous callers get 401; unapproved sessions get 403

## Worker binding

`worker/wrangler.jsonc` already declares:

```json
"r2_buckets": [
  { "binding": "RELEASES", "bucket_name": "cyvra-erase-releases" }
]
```

Git deploy of the Worker fails until the bucket exists in the same account.

## Gates

```bash
curl -sS -o /dev/null -w "%{http_code}\n" https://api.cyvra.co.in/api/v1/auth/download/setup
# 401

# Signed-in unapproved customer: 403
# Signed-in approved customer, empty bucket: 404 package_not_released
```
