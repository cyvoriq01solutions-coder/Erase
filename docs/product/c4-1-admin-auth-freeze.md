# C4.1 Freeze — Dedicated Admin Authentication + Internal User Management

Status: FROZEN DESIGN DECISION
Date: 2026-08-19

## 1. Trust boundary

`admin.cyvra.co.in` is a separate internal control-plane application. It must not reuse the customer browser session as its authorization credential.

Customer realm:
- public site: `cyvra.co.in` / `www.cyvra.co.in`
- customer cookie: `cyvoriq_session`
- customer endpoints: `/api/v1/auth/*`

Admin realm:
- internal site: `admin.cyvra.co.in`
- admin cookie: `cyvoriq_admin_session`
- admin endpoints: `/api/v1/admin/auth/*`
- server-authoritative `user_roles` RBAC in Neon
- Cloudflare Access is an additional outer gate and does not replace Worker/Neon authorization.

A customer session token must never authorize an Admin endpoint.

## 2. Dedicated Admin auth contract

- `POST /api/v1/admin/auth/request-code`
- `POST /api/v1/admin/auth/verify-code`
- `GET /api/v1/admin/auth/session`
- `POST /api/v1/admin/auth/logout`

Admin OTP rules:
- 6 digits
- 10 minute expiry
- maximum 5 verification attempts
- issuing a new challenge consumes older unconsumed Admin challenges for the same identity
- plaintext OTP is never stored in Neon

Admin session rules:
- separate `admin_sessions` table
- separate `cyvoriq_admin_session` cookie
- HttpOnly
- Secure
- SameSite=Lax
- token plaintext is never stored in Neon
- session lifetime target for C4.1: 4 hours
- revoked/expired Admin sessions are rejected server-side

## 3. Admin identity eligibility

No public Admin self-registration exists.

Bootstrap identities:
- `ceo@cyvra.co.in` -> `super_admin`
- `accounts@cyvra.co.in` -> `accounts_admin`

CEO bootstrap:
- the server may create the CEO internal identity if it does not yet exist
- email ownership must still be proven by OTP
- first successful Admin OTP may activate the bootstrap `super_admin` role

Accounts bootstrap:
- the server may create the Accounts internal identity if it does not yet exist
- email ownership must be proven by OTP
- the `accounts_admin` role remains pending until an active Super Administrator approves it
- a pending Accounts identity receives no Admin session

Additional internal administrators:
- must be created/invited from inside the protected Admin Portal by authorized management
- must use a CYVORIQ/CYVRA corporate email identity for MVP
- cannot self-create from the public web or Admin sign-in screen
- email ownership verification alone does not activate the role
- Super Administrator approval is required before an Admin session can be issued

## 4. Internal user lifecycle

Super Admin -> Internal Users -> Create/Invite Administrator -> name + corporate email + role -> pending email verification -> email OTP verification -> pending role approval -> Super Admin approval -> active administrator.

For MVP, the create/invite UI may create `accounts_admin` identities only. `super_admin` remains bootstrap-only until a later explicit policy change.

Required server-authoritative operations:
- list internal administrators
- create/invite internal administrator
- approve pending administrator role
- revoke administrator role

Material audit events:
- `ADMIN_USER_CREATED`
- `ADMIN_EMAIL_VERIFIED`
- `ADMIN_ROLE_APPROVED`
- `ADMIN_ROLE_CHANGED`
- `ADMIN_ROLE_REVOKED`
- `ADMIN_LOGIN`
- `ADMIN_LOGOUT`

## 5. Data persistence

C4.1 adds:
- `admin_login_challenges`
- `admin_sessions`

The existing `users`, `organizations`, and `user_roles` tables remain the identity and RBAC authority.

The dedicated Admin tables are intentionally separate from:
- `login_challenges`
- `customer_sessions`

## 6. Browser authority

The browser does not decide Admin eligibility or role activation.

The Admin frontend may display server-returned role state, but must not:
- grant a role from local state
- trust a customer session
- create a Super Administrator
- bypass pending approval
- expose Neon credentials

Manual navigation to protected Admin modules must still be rejected unless the Worker validates an active Admin session and role.

## 7. Visual identity

Admin must be visually unmistakable from the public/customer CYVRA experience.

Frozen palette:
- graphite / charcoal navigation and authority surfaces
- emerald / teal primary actions and selected states
- neutral grey / off-white work surfaces
- red only for destructive, revoke, or error actions

Do not use the public CYVRA navy/orange palette as the dominant Admin identity.

## 8. C4.1 acceptance gates

Before PR #15 may merge:
1. Admin branch is based on the merged C3.1 `main`.
2. Dedicated Admin challenge/session migration is applied to the verified development database.
3. Worker typecheck/build passes.
4. Admin frontend build passes.
5. `admin.cyvra.co.in` remains isolated.
6. Cloudflare Access is configured and verified.
7. CEO OTP creates a dedicated Admin session and reaches Super Admin UI.
8. Customer session cannot authorize Admin endpoints.
9. General customer/Gmail identity cannot enter the Admin realm.
10. Accounts OTP verifies ownership but remains pending until CEO approval.
11. CEO approval enables Accounts Admin login.
12. Internal administrator invite/verify/approve/revoke passes.
13. Neon Admin session/role/audit evidence is verified.
14. Admin visual identity uses the frozen graphite/emerald palette.

Do not merge PR #15 without explicit approval.
