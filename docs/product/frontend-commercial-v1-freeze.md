# CYVORIQ / CYVRA Erase — Frontend Commercial V1 Freeze

Status: FROZEN
Date: 2026-08-19
Scope: Public website, commercial entry flow, protected download positioning, device-bound licensing.

## Brand and Product Hierarchy

- Company brand: CYVORIQ Solutions.
- Product brand: CYVRA Erase.
- Public presentation: `CYVRA Erase — by CYVORIQ Solutions`.
- Use the approved CYVORIQ Solutions logo in the public header.
- Remove the existing `Open Platform` public CTA.
- Primary public CTA is an orange `DOWNLOAD CYVRA ERASE` button.

## Public Positioning

Primary promise: secure, evidence-backed device retirement.

Primary audiences:
- Individuals preparing a device for buy-back, trade-in, resale, donation or handover.
- Enterprises, OEMs and ITAD programs managing retired, returned, leased, refurbished or resold devices.

Core message:
- A retired device may still contain sensitive information.
- CYVRA Erase is designed to assess the device, map potential personal-data exposure, collect structured evidence, verify the available evidence and report the outcome.
- The current Windows release is assessment and verification focused and remains non-destructive.

Core lifecycle:
`ASSESS -> PDEM -> EVIDENCE -> VERIFY -> REPORT`

## Compliance Positioning

- Public wording must use `Designed to support DPDP readiness` or equivalent evidence-based wording.
- Do not claim government DPDP certification.
- Do not claim that use of CYVRA alone establishes legal compliance.
- Explain that CYVRA supports secure data-lifecycle controls, documented evidence, verification and auditability.
- Public standards references should be reviewed against the current standard before release. Avoid obsolete certification language.

## Download Access Model

The public `DOWNLOAD CYVRA ERASE` button must route to a gated download flow, not directly to an executable.

Required flow:
1. Create account or sign in.
2. Verify email ownership using OTP.
3. Server creates/uses the internal CYVORIQ user identity; customers do not need a separate username.
4. Customer chooses individual or enterprise account type; enterprise registration captures organisation name.
5. Present and record acceptance of Privacy Notice, Terms and Licence Terms before commercial download entitlement.
6. Purchase/payment/approval generates an authorised entitlement.
7. Server verifies account status, order/payment state, licence state and download entitlement.
8. Only an authorised user receives package download access.
9. First activation binds the licence to the authorised device.

Data minimisation rule: do not require mobile number, address, Aadhaar or other unnecessary personal data merely to create an account. Collect billing/tax information only when required by the commercial transaction.

## Licence and Device Binding

Commercial rule:
`ONE LICENCE. ONE DEVICE. ONE ACTIVATION BINDING.`

- Default licence device allowance is one device.
- The serial/licence key is used for initial activation and must not act as a reusable bearer credential for normal API traffic.
- First successful activation records and binds the licence to the device identity/fingerprint.
- Activation of the same licence on a different device must be rejected unless CYVORIQ performs an authorised, auditable reset or reissue.
- Legitimate reactivation on the same verified device may be supported.
- After successful device binding, use a separate revocable device/agent token for ongoing authenticated communication.

Target flow:
`PAYMENT APPROVED -> LICENSE GENERATED -> DOWNLOAD ENTITLEMENT -> FIRST ACTIVATION -> DEVICE FINGERPRINT VERIFIED -> LICENSE BOUND -> DEVICE TOKEN ISSUED`

## Release Safety

- Do not expose a raw public Windows executable before the commercial entitlement and activation controls are complete.
- Production package release requires protected download, server-side entitlement checks, device binding and signed Windows binaries.
- Current Windows Verification Agent remains non-destructive until the sanitization engine and independent validation stages are explicitly completed and frozen.

## Visual Direction

- Premium, security-first, Indian enterprise technology presentation.
- CYVORIQ navy/blue visual foundation.
- Orange reserved for high-intent commercial CTAs such as `DOWNLOAD CYVRA ERASE`.
- Clear typography, strong whitespace, no generic startup-dashboard styling on the public website.
- Logo must be legible and proportionally displayed; do not recreate it as a letter icon.

## Change Control

Any change to the decisions above requires an explicit freeze review before implementation.
