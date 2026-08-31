# CYVORIQ Erase Control Plane C3-C7 Freeze

Status: FROZEN on 2026-08-19 and REVISED on 2026-08-19 after live C3 browser acceptance testing.

This document extends the existing CYVORIQ Erase architecture and commercial-access policy. The revision explicitly removes the customer dashboard concept. Customers authenticate only to establish identity and obtain protected software access from the CYVRA public website. The internal administration control plane remains a separate application on `admin.cyvra.co.in`.

## 1. Trust-zone freeze

Three distinct trust zones are required:

1. Public/customer website: `cyvra.co.in` / `www.cyvra.co.in`
2. Control-plane API: `api.cyvra.co.in`
3. Protected internal administration portal: `admin.cyvra.co.in`

There is NO customer administration/dashboard trust zone in the final product.

A customer may create an account or sign in from the public CYVRA website. Authentication creates a secure server-verified identity session only. The customer remains on the public/customer site and returns to `/download` after authentication.

The public website must never expose internal admin data. Direct/manual navigation to legacy `/app/*` customer routes must not expose a customer dashboard; those routes are removed from the final C3 implementation.

The Cloudflare Worker remains the control-plane authority. The browser does not decide roles, payment status, licence status, entitlement, activation eligibility, package release, or device binding.

## 2. Identity and role freeze

### Customer

- Any valid customer email may register.
- Email is the user identity; no separate username is required.
- Email ownership is verified automatically by 6-digit email OTP.
- Successful OTP verification activates the customer identity only.
- Successful OTP verification does NOT grant paid-product download entitlement.
- A customer does not receive a dashboard, operational portal, admin panel, device console, reports console, settings console, or other internal control-plane UI.

### CEO super administrator

- Bootstrap identity: `ceo@cyvoriq.com`
- Role: `super_admin`
- Possession of the email must still be proven through OTP.
- No other email may self-bootstrap as `super_admin`.
- The super administrator approves/revokes internal administrators and has final authority over purchase approval, payment override, licence issuance/revocation, download entitlement, software-release administration, device/licence relationships, and protected management reporting.

### Accounts administrator

- Internal identity: `accounts@cyvoriq.com`
- Intended role: `accounts_admin`
- The role is not self-activating. An authenticated active `super_admin` must approve it.
- Accounts verification means commercial verification after customer email verification: payment confirmation, purchase approval/rejection, and enabling an entitlement when policy conditions are satisfied.
- `accounts_admin` does not manually verify the customer's OTP/email ownership and may not create or promote a `super_admin`.

## 3. Customer Account + Protected Download Authentication V1

The customer journey starts and ends on the CYVRA public/customer website.

Create-account flow:

`/download` -> `Create Account & Continue` -> Individual/Enterprise -> name -> email -> enterprise organization name when applicable -> send OTP -> enter 6-digit OTP -> authenticated session -> return to `/download`.

Existing-account flow:

`/download` -> `Sign In` -> email -> send OTP -> enter 6-digit OTP -> authenticated session -> return to `/download`.

No password, separate username, mobile OTP, Aadhaar, or unrelated personal information is required for the MVP account-creation flow.

After authentication, `/download` must show server-derived customer/commercial state. Example states include:

- Email verified
- Account active
- Payment pending/confirmed
- Purchase approval pending/approved
- Licence pending/active
- Download entitlement locked/ready

The protected installer download action becomes available only after the Worker confirms every frozen commercial gate.

### Customer routes explicitly removed

The following browser routes are NOT part of the final customer product and must be removed from C3:

- `/app`
- `/app/dashboard`
- `/app/devices`
- `/app/assessments`
- `/app/evidence`
- `/app/verification`
- `/app/reports`
- `/app/certificates`
- `/app/settings`

A manually typed legacy `/app/*` URL must not expose a customer dashboard.

## 4. Admin portal freeze

`admin.cyvra.co.in` is a separate internal UI and deployment target.

The Admin Portal must NOT offer public self-registration or a customer-style `Create Account` flow.

Initial authority identities:

- `ceo@cyvoriq.com` -> `super_admin`
- `accounts@cyvoriq.com` -> `accounts_admin` only after super-admin approval

The final admin authentication realm is separate from the customer realm and must use dedicated admin endpoints and a dedicated admin session cookie. Target API contract:

- `POST /api/v1/admin/auth/request-code`
- `POST /api/v1/admin/auth/verify-code`
- `GET /api/v1/admin/auth/session`
- `POST /api/v1/admin/auth/logout`
- dedicated secure admin cookie such as `cyvoriq_admin_session`

Cloudflare Access is an additional outer gate for `admin.cyvra.co.in`; it does not replace Worker/Neon RBAC.

Required protected admin areas:

- Overview
- Customers
- Orders / Purchases
- Payments
- Approvals
- Licences
- Download Entitlements
- Software Releases
- Activations / Bound Devices
- Audit Events
- Management Report
- Accounts Report
- Internal Users / Role Administration (`super_admin` only)

No customer receives admin access because of their email domain, customer session, public account, or knowledge of the admin URL.

### Internal user creation freeze

New internal administrators are created/invited from inside the protected Admin Portal by authorized internal management. There is no public admin registration.

Target internal-user lifecycle:

`Super Admin -> Internal Users -> Create/Invite Administrator -> corporate email + name + role -> pending -> email ownership verification -> super-admin approval -> active administrator`.

Material internal-user events must be audit recorded, including creation/invitation, email verification, role approval/change/revocation, login and logout.

## 5. Admin visual identity freeze

Customer/public CYVRA UI continues to use the existing CYVRA visual identity.

The Admin Portal must be unmistakably different and must NOT reuse the public navy/orange look as its primary interface identity.

Frozen Admin visual direction:

- Graphite / charcoal
- Emerald / teal
- Neutral white / grey
- Internal-control-plane tone

## 6. Admin reporting freeze

Two first-class report groups are required.

### Management Report

Visible to `super_admin` and later-approved management roles.

Minimum scope:

- new / verified / active customer accounts
- individual vs enterprise accounts
- orders and purchase status
- payments confirmed / pending / rejected
- licences issued / active / suspended / revoked
- download entitlements enabled / revoked / expired
- software release/version state
- activations and bound devices
- activation failures / attempted key reuse on another device
- high-level operational exceptions
- date-range filtering and export-ready structure

### Accounts Report

Visible to active `accounts_admin` and `super_admin`.

Minimum scope:

- orders awaiting payment verification
- payments confirmed / pending / rejected
- purchases awaiting approval
- approvals performed, by whom, and when
- licence / entitlement state for paid purchases
- customer/order/payment references required for reconciliation
- activation state needed to answer customer/account queries
- date-range filtering and export-ready structure

All material admin approvals and changes must be audit-recorded server-side.

## 7. Protected installer distribution freeze

The `.exe` does NOT live inside the Admin Panel application bundle and is never an unrestricted public Pages asset.

Target architecture:

`Admin Portal release control -> private Cloudflare R2 -> Worker authorization -> entitled customer download from cyvra.co.in/download`.

The Admin Portal manages release metadata and authorized release actions. The signed Windows installer binary is stored in a private R2 bucket. The browser receives no R2 credentials.

A verified customer account alone is not sufficient to download CYVRA Erase.

Before package release, the Worker must confirm server-side:

1. email verified
2. account active
3. order exists
4. payment received
5. purchase approved by an active authorized administrator
6. active licence issued
7. download entitlement enabled and not revoked/expired

The customer uses the same approved signed installer release; per-customer/device control is enforced by the licence/activation system rather than generating a different `.exe` for every customer.

## 8. Product activation key and device-binding freeze

Each paid CYVRA Erase licence receives a unique server-generated activation key. The key must never be generated or authorized by browser JavaScript.

First successful activation is an atomic server operation:

`activation key + licensed customer + device identity -> Worker -> Neon -> bind licence to that device`.

After first successful activation:

- the licence is bound to one authorized device for the one-device product tier;
- the same key must not activate a different device;
- a different-device reuse attempt is rejected and audit-recorded;
- repeated validation from the already-bound device may be treated as idempotent revalidation rather than consuming a second device;
- device replacement/unbinding, if later supported, must require explicit authorized server-side action and audit history;
- activation count, first activation time, latest validation time, bound device identity, status, and failure/reuse events must be server-controlled.

No plaintext licence key should be stored after the secure issuance/recovery policy is implemented.

## 9. Windows installer experience freeze

The customer experience must look and feel like a professional Windows security-software installation, not a developer tool or command-line package.

Design goals:

- fast launch
- CYVORIQ / CYVRA branding
- Windows-native professional wizard feel
- clear `Back`, `Next`, `Install`, `Finish` progression
- sensible defaults; minimal technical choices
- short installation path
- progress indicator
- no terminal window required for the normal customer flow

Target installer flow:

1. Welcome to CYVRA Erase
2. Review / accept licence and privacy terms
3. Ready to Install
4. Installing CYVRA Erase
5. Launch CYVRA Erase
6. Sign in / enter activation key
7. Online activation and device binding
8. Device detected / activation successful
9. Continue into the CYVRA application

The installed application then follows the previously frozen professional GUI flow: Device detected -> Run Verification -> Scanning progress -> Personal Data Map -> Evidence / Verification -> Generate Report -> View / Save Report.

## 10. Delivery sequence freeze

### C3.1 - Customer Account + Protected Download Flow V1

Complete before merging PR #14:

- keep account registration/sign-in OTP and credentialed customer session
- remove the entire customer `/app/*` dashboard/navigation shell
- redirect successful create-account/sign-in verification to `/download`
- make `/download` session-aware
- show authenticated identity/account state on `/download`
- keep commercial/download controls server-authoritative
- add customer sign-out from the customer/download flow
- ensure manual `/app/*` navigation exposes no dashboard
- re-run real browser OTP/session/logout acceptance

### C4.1 - Dedicated Admin Authentication + Internal User Management

After C3.1:

- separate admin auth endpoints/session cookie from customer authentication
- Cloudflare Access outer gate
- CEO super-admin bootstrap
- accounts-admin approval workflow
- protected admin navigation
- internal Create/Invite Administrator lifecycle
- management and accounts report foundations
- graphite/emerald admin visual identity

### C5 - Commercial Approval & Protected Download Entitlement V1

Then:

- order/payment/approval workflow
- accounts verification
- entitlement checks
- private R2 release storage integration
- protected package-release endpoint
- software-release administration
- audit events

### C6 - Licence Activation & One-Device Binding V1

Then:

- secure server-generated activation key
- activation API
- atomic first-device binding
- same-device revalidation
- different-device reuse rejection
- activation/device audit trail

### C7 - Professional Windows Installer / GUI V1

Then:

- Windows wizard installer
- fast Next/Install/Finish flow
- CYVRA branded GUI
- activation-key entry / sign-in
- device detection and binding
- handoff to the existing verification application

## 11. Temporary acceptance-test configuration - MUST ROLLBACK

The following items were created only to test C3 securely before merging and must not be mistaken for final production architecture.

### Temporary hostname

`auth-test.cyvra.co.in`

Current purpose: same-site browser acceptance testing for branch `portal-auth-ui-v1`.

Current temporary routing:

`auth-test.cyvra.co.in` -> proxied CNAME -> `portal-auth-ui-v1.erase-e93.pages.dev`.

Rollback after C3.1 acceptance/merge:

1. remove `auth-test.cyvra.co.in` from the `erase` Pages project's Custom Domains;
2. remove the `auth-test` DNS CNAME from the `cyvra.co.in` zone;
3. remove `https://auth-test.cyvra.co.in` from the Worker's runtime `PORTAL_ORIGINS` value;
4. verify `cyvra.co.in` / `www.cyvra.co.in`, `api.cyvra.co.in`, and `admin.cyvra.co.in` remain unaffected.

### Temporary Worker CORS runtime change

During testing, `https://auth-test.cyvra.co.in` was manually appended to `PORTAL_ORIGINS` in the deployed Worker settings. This is temporary runtime configuration and must be removed after C3.1 is production-verified.

### Preview branch hostname

`portal-auth-ui-v1.erase-e93.pages.dev` is a branch preview hostname. It is not a customer-facing production hostname.

### Test customer data

Acceptance testing created customer identity/session/challenge/audit records in the currently connected development control-plane database. Before production reporting is treated as business data, test identities must be clearly excluded/deactivated/cleaned according to the audit-retention rule. Material audit evidence must not be silently rewritten as genuine business activity.

### Permanent items - DO NOT ROLLBACK

The following are permanent architecture and must NOT be removed as part of test cleanup:

- `cyvra.co.in` / `www.cyvra.co.in`
- `api.cyvra.co.in` -> `cyvoriq-erase-api`
- `admin.cyvra.co.in` -> `cyvra-admin`
- Resend OTP DNS/email configuration
- `AUTH_EMAIL_TOKEN`
- `AUTH_PEPPER`
- Hyperdrive binding
- Neon schema/migrations

## 12. Current implementation boundary

The active branch `portal-auth-ui-v1` remains the C3/C3.1 customer-auth branch and PR #14 must remain unmerged until the corrected download-only flow passes acceptance.

The Admin Portal remains a separate `admin-portal-v1` branch / `cyvra-admin` Pages project. Do not mix customer C3.1 UI changes into the Admin branch.

No C3.1 work may bypass or fake payment, entitlement, licence issuance, device binding, admin approval, protected installer release, or activation logic. Those controls stay server-authoritative and are implemented in the frozen sequence above.
