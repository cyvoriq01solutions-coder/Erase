# CYVORIQ Erase Control Plane C3-C7 Freeze

Status: FROZEN on 2026-08-19 as an extension of the existing CYVORIQ Erase architecture and commercial-access policy.

This document does not replace the earlier freeze. It adds the customer-auth UI, internal admin-panel, commercial approval, one-device activation, and Windows installer experience required for production.

## 1. Trust-zone freeze

Three distinct browser trust zones are required:

1. Public website: `cyvra.co.in` / `www.cyvra.co.in`
2. Protected customer portal: authenticated `/app` routes on the public/customer domain
3. Protected internal administration portal: `admin.cyvra.co.in`

The public website must never expose protected customer or admin data. Direct/manual navigation to protected routes must fail without a valid server-verified session and authorization.

The Cloudflare Worker remains the control-plane authority. The browser does not decide roles, payment status, licence status, entitlement, activation eligibility, or device binding.

## 2. Identity and role freeze

### Customer

- Any valid customer email may register.
- Email is the user identity; no separate username is required.
- Email ownership is verified automatically by 6-digit email OTP.
- Successful OTP verification activates the customer identity only. It does not grant paid download entitlement.

### CEO super administrator

- Bootstrap identity: `ceo@cyvra.co.in`
- Role: `super_admin`
- Possession of the email must still be proven through OTP.
- No other email may self-bootstrap as `super_admin`.
- The super administrator approves/revokes internal administrators and has final authority over purchase approval, payment override, licence issuance/revocation, download entitlement, device/licence relationships, and protected management reporting.

### Accounts administrator

- Internal identity: `accounts@cyvra.co.in`
- Intended role: `accounts_admin`
- The role is not self-activating. An authenticated active `super_admin` must approve it.
- Accounts verification means commercial verification after customer email verification: payment confirmation, purchase approval/rejection, and enabling an entitlement when policy conditions are satisfied.
- `accounts_admin` does not manually verify the customer's OTP/email ownership and may not create or promote a `super_admin`.

## 3. Customer authentication UI V1

The `/download` page starts the account flow.

Create account flow:

`Create Account & Continue` -> Individual/Enterprise -> name -> email -> enterprise organization name when applicable -> send OTP -> enter 6-digit OTP -> authenticated session -> protected customer portal.

Existing account flow:

`Sign In` -> email -> send OTP -> enter 6-digit OTP -> authenticated session -> protected customer portal.

No password, separate username, mobile OTP, Aadhaar, or unrelated personal information is required for the MVP account-creation flow.

## 4. Admin portal freeze

`admin.cyvra.co.in` is a separate internal UI and deployment target.

Authentication uses the same server-controlled email-OTP identity system, followed by strict RBAC checks.

Required protected admin areas:

- Overview
- Customers
- Orders / Purchases
- Payments
- Approvals
- Licences
- Download Entitlements
- Activations / Bound Devices
- Audit Events
- Reports
- Internal User / Role Administration (super_admin only)

No customer receives admin access because of their email domain or because they know an admin URL.

## 5. Admin reporting freeze

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

## 6. Protected download and commercial gate freeze

A verified customer account alone is not sufficient to download CYVRA Erase.

Before package release, the Worker must confirm server-side:

1. email verified
2. account active
3. order exists
4. payment received
5. purchase approved by an active authorized administrator
6. active licence issued
7. download entitlement enabled and not revoked/expired

The Windows installer/package must never be exposed as an unrestricted public file.

## 7. Product activation key and device-binding freeze

Each paid CYVRA Erase licence receives a unique server-generated activation key. The key must never be generated or authorized by browser JavaScript.

First successful activation is an atomic server operation:

`activation key + authenticated/licensed customer + device identity -> Worker -> Neon -> bind licence to that device`

After first successful activation:

- the licence is bound to one authorized device for the one-device product tier;
- the same key must not activate a different device;
- a different-device reuse attempt is rejected and audit-recorded;
- repeated validation from the already-bound device may be treated as idempotent revalidation rather than consuming a second device;
- device replacement/unbinding, if later supported, must require explicit authorized server-side action and audit history;
- activation count, first activation time, latest validation time, bound device identity, status, and failure/reuse events must be server-controlled.

No plaintext licence key should be stored after the secure issuance/recovery policy is implemented.

## 8. Windows installer experience freeze

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

## 9. Delivery sequence freeze

### C3 - Customer Account & OTP Authentication UI V1

Build now:

- account UI
- register -> OTP -> verify
- sign-in -> OTP -> verify
- credentialed session handling
- authenticated `/app` route guard
- logout
- real browser test against Worker + Resend + Neon

### C4 - Admin Portal Foundation V1

Next:

- separate `admin.cyvra.co.in` frontend/deployment
- OTP login
- RBAC guard
- CEO super-admin bootstrap view
- accounts-admin approval workflow
- protected admin navigation
- management and accounts report shells

### C5 - Commercial Approval & Download Entitlement V1

Then:

- order/payment/approval workflow
- accounts verification
- entitlement checks
- protected package-release endpoint
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

## 10. Current implementation boundary

The active branch `portal-auth-ui-v1` is C3 only.

C3 must not bypass or fake payment, entitlement, licence issuance, device binding, admin approval, or installer logic. Those controls stay server-authoritative and are implemented in the frozen sequence above.
