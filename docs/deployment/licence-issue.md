# Licence issuance (slice C)

C5 approval is not a paid licence. A Super Administrator or Accounts
Administrator issues a key with **Issue licence** on an already-approved
customer.

## What is stored

Neon `licenses`: `key_prefix` (e.g. `CYVRA-AB3K`) and HMAC-SHA256 `key_hash`
using `AUTH_PEPPER`. The full key is never written to the database.

Format: `CYVRA-XXXX-XXXX-XXXX-XXXX` (Crockford-style alphabet, no 0/O/1/I).

## What the customer sees

- Email: full key once
- `/download`: prefix only + “full key emailed”
- Windows first-run activation remains disabled (`live_activation_enabled: false`)

## API

`POST /api/v1/admin/customers/:userId/issue-license` (admin cookie)

- 200 `{ customer, activationKey }` once
- 409 `access_not_approved` or `license_already_issued`
- 503 `license_email_unavailable` — licence still created; copy `activationKey`

Anonymous `GET /api/v1/auth/download/setup` remains 401.
Approved without a licence: download stays locked (`license_required` if they
call setup). Package still must exist in private B2 to unlock the button.
