# CYVORIQ Commercial Access Policy

Status: FROZEN for MVP control-plane implementation.

## Identity and customer registration

- A customer may register with any valid email address.
- The email address is the customer's user ID and must be verified by email OTP.
- Email identity is case-insensitive and globally unique in the MVP.
- OTP verification proves control of the email address only. It does not grant paid-product download rights.

## CYVORIQ control panel authority

### Super user

`ceo@cyvoriq.com` is the bootstrap `super_admin` identity.

After successful email OTP verification, this identity may receive the active `super_admin` role through the server-side bootstrap policy. No other email address may self-bootstrap as `super_admin`.

The `super_admin` may:

- approve or revoke control-panel administrators;
- approve customer purchases;
- confirm or override payment decisions;
- approve, issue, suspend, or revoke licenses;
- enable or revoke download entitlements;
- manage device/license relationships;
- view protected audit records.

### Accounts authority

`accounts@cyvoriq.com` is eligible for the `accounts_admin` role, but control-panel access is not automatic. The role must first be approved by an active `super_admin`.

An active `accounts_admin` may:

- confirm customer payment;
- approve or reject a customer purchase;
- enable a paid-product entitlement after required checks;
- view customer, order, payment, entitlement, and license records needed for accounts operations.

An `accounts_admin` may not:

- create, approve, revoke, or replace a `super_admin`;
- promote itself or another user to `super_admin`;
- remove the CEO super-user authority.

## Customer paid-product gate

A verified customer account alone is not sufficient to download CYVORIQ Erase.

Protected download requires all of the following to be true server-side:

1. email verification is complete;
2. customer account is active;
3. an order exists for the requested product;
4. payment is recorded as received;
5. the purchase is approved by an active `super_admin` or active `accounts_admin`;
6. an active license has been issued for the approved purchase;
7. a download entitlement is enabled and not revoked or expired.

The browser must never decide whether these conditions are satisfied. Every protected download request is authorized by the Worker against Neon.

## Admin access gate

No ordinary customer, unapproved employee, or user with a CYVORIQ email address receives control-panel access merely because of their email domain.

Protected control-panel routes must reject unauthorized sessions server-side, including direct/manual navigation to admin URLs.

## Audit policy

The following actions must produce `audit_events` records when implemented:

- `EMAIL_VERIFIED`
- `ADMIN_ROLE_REQUESTED`
- `ADMIN_ROLE_APPROVED`
- `ADMIN_ROLE_REVOKED`
- `PAYMENT_CONFIRMED`
- `PURCHASE_APPROVED`
- `PURCHASE_REJECTED`
- `LICENSE_ISSUED`
- `LICENSE_REVOKED`
- `DOWNLOAD_AUTHORIZED`
- `DOWNLOAD_REVOKED`

## Security invariants

- No plaintext OTP is stored in Neon.
- No plaintext customer session token is stored in Neon.
- No plaintext license key is stored in Neon after issuance.
- Role and entitlement checks are performed by the Worker, never trusted from client-supplied fields.
- The CEO super-user email is a policy constant; possession must still be proven by OTP before a session can exercise authority.
