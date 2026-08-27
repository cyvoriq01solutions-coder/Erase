# CYVRA — Approved Frozen Architecture

**Status:** APPROVED AND FROZEN  
**Project:** CYVRA Erase  
**Repository:** Erase  
**Architecture Decision Date:** 2026-08-27  

---

# 1. Purpose

This document defines the approved high-level architecture for CYVRA Erase.

The purpose of this architecture is to provide a clean, secure, maintainable foundation for the first official CYVRA user journey:

1. Visit CYVRA.
2. Create an account or sign in.
3. Verify email ownership.
4. Accept the required legal and licence terms.
5. Receive eligibility and entitlement validation.
6. Download the CYVRA installer through a protected delivery flow.
7. Install and launch CYVRA Erase.
8. Activate the product.
9. Bind the entitlement to one approved device.
10. Perform the approved erase workflow.
11. Submit feedback.

This architecture preserves the existing CYVRA core and avoids unnecessary changes to the erase engine.

---

# 2. Frozen Architectural Principles

The following principles are approved and frozen.

## 2.1 One Central CYVRA API Boundary

All sensitive application operations must pass through one central CYVRA API boundary.

The public frontend and admin frontend must not directly communicate with infrastructure services that contain sensitive credentials or privileged access.

The central API boundary is responsible for controlling access to:

- authentication operations
- email verification
- entitlement validation
- installer delivery
- activation
- device binding
- revocation
- administrative operations
- database operations
- email delivery operations

The initial implementation will use a Cloudflare Worker as the CYVRA Control Plane.

---

## 2.2 No Direct Frontend Access to Neon

The public frontend must not directly connect to the Neon PostgreSQL database.

The admin frontend must also not directly connect to Neon.

The database connection and privileged database operations must remain behind the CYVRA Control Plane.

Architecture:

```text
Frontend
    |
    v
CYVRA Control Plane
    |
    v
Neon PostgreSQL
2.3 No Direct Frontend Access to Resend

The frontend must not contain or expose Resend credentials.

All email operations must pass through the CYVRA Control Plane.

Architecture:

Frontend
    |
    v
CYVRA Control Plane
    |
    v
Resend

The Worker is responsible for securely initiating approved transactional email operations.

2.4 No Infrastructure Credentials in the Frontend

The frontend must never contain:

Neon database credentials
Resend API keys
Cloudflare API tokens
Cloudflare deployment credentials
R2 credentials
privileged administrative credentials
device activation secrets

Browser applications must only contain public configuration that is safe to expose.

3. Approved High-Level Architecture
                         ┌─────────────────────┐
                         │   CYVRA CUSTOMER    │
                         │      BROWSER        │
                         └──────────┬──────────┘
                                    │
                                    v
                         ┌─────────────────────┐
                         │   PUBLIC FRONTEND   │
                         │     Cloudflare      │
                         └──────────┬──────────┘
                                    │ HTTPS API
                                    v
┌───────────────────────────────────────────────────────────────┐
│                    CYVRA CONTROL PLANE                        │
│                                                               │
│                 Cloudflare Worker API                         │
│                                                               │
│  Authentication                                              │
│  Email Verification                                          │
│  Legal Acceptance                                            │
│  Eligibility                                                 │
│  Entitlements                                                │
│  Protected Downloads                                         │
│  Activation                                                   │
│  Device Binding                                              │
│  Revocation                                                   │
│  Feedback                                                     │
│  Administrative APIs                                         │
└───────────────┬────────────────────┬───────────────────┬──────┘
                │                    │                   │
                v                    v                   v
        ┌───────────────┐    ┌───────────────┐   ┌───────────────┐
        │ Neon          │    │ Resend        │   │ Cloudflare R2 │
        │ PostgreSQL    │    │ Email         │   │ Installer     │
        │               │    │               │   │ Storage       │
        └───────────────┘    └───────────────┘   └───────────────┘
                                                            │
                                                            v
                                                   ┌───────────────┐
                                                   │ CYVRA Windows │
                                                   │ Installer     │
                                                   └───────────────┘
                                                            │
                                                            v
                                                   ┌───────────────┐
                                                   │ CYVRA Desktop │
                                                   │ Application   │
                                                   └───────────────┘
                                                            │
                                                            v
                                                   ┌───────────────┐
                                                   │ CYVRA Erase   │
                                                   │ Core          │
                                                   └───────────────┘
4. CYVRA Control Plane

The CYVRA Control Plane is the central security and application boundary.

For the initial architecture, it will be implemented as one Cloudflare Worker application.

However, this does not mean one giant source file.

The implementation must follow a modular monolith architecture.

Example logical structure:

worker/
├── src/
│   ├── index.ts
│   ├── auth/
│   ├── email/
│   ├── legal/
│   ├── eligibility/
│   ├── entitlements/
│   ├── downloads/
│   ├── activation/
│   ├── devices/
│   ├── feedback/
│   ├── admin/
│   └── shared/

The exact source structure may evolve, but the architectural principle remains frozen:

One central CYVRA API boundary implemented as a modular application.

5. Cloudflare R2 — Approved Installer Storage Direction

Cloudflare R2 is approved as the intended storage layer for CYVRA installer artifacts.

The installer must not primarily live as:

a public frontend asset
a publicly exposed static download
source code inside the Worker
an unrestricted GitHub Release download

The intended architecture is:

Build and Sign Installer
        |
        v
Upload Approved Release Artifact
        |
        v
Cloudflare R2
        |
        v
CYVRA Control Plane validates request
        |
        v
Protected installer delivery
        |
        v
Eligible customer download

R2 is intended to separate installer storage from the application frontend and control plane.

The exact Cloudflare plan, free-tier limits, pricing, and production storage configuration must be verified against the current Cloudflare documentation before production use.

No production assumption about free-tier capacity is frozen by this document.

6. Protected Download Policy

The installer must not simply be exposed as a permanently public URL.

The CYVRA Control Plane must validate the download request.

Before an installer is delivered, the system should be able to evaluate:

authenticated user identity
account status
email verification status
accepted legal and licence terms
eligibility status
entitlement status
release availability
download authorization
expiry of the authorization
revocation status

The exact policy values are not yet frozen.

The following must be decided during the entitlement and download implementation:

Is authorization linked to one user?
Is authorization linked to one entitlement?
How many download attempts are allowed?
How long does a download authorization remain valid?
What happens if a download fails?
Can a customer download again after installation?
Can an authorization be revoked before download?
How are installer versions controlled?

The implementation must be driven by this policy.

7. Neon PostgreSQL

Neon PostgreSQL is the approved relational data layer.

The CYVRA Control Plane is the application layer that communicates with Neon.

The browser must not directly communicate with the database.

The database will support application records including, as required:

users
email verification state
legal acceptance records
eligibility records
entitlements
installer releases
download authorizations
activation records
device bindings
revocation records
feedback records
administrative audit information

The detailed database schema remains a separate implementation concern and must not be changed casually.

Database migrations must be version-controlled.

8. Resend Email Architecture

Resend is approved for transactional email delivery.

Email operations must pass through the CYVRA Control Plane.

Potential email flows include:

account verification
OTP or verification codes
sign-in confirmation where required
activation-related notifications
download-related notifications
administrative notifications

The Resend API key must remain secret.

The frontend must never send email directly using privileged Resend credentials.

9. Customer Journey Architecture

The approved first-user journey is:

Customer
   |
   v
Visit CYVRA
   |
   v
Create Account / Sign In
   |
   v
Verify Email
   |
   v
Accept Privacy / Terms / Licence
   |
   v
Eligibility Check
   |
   +--------------------------+
   |                          |
   v                          v
Eligible                 Not Eligible
   |                          |
   v                          v
Entitlement              Clear Response
   |
   v
Protected Installer Authorization
   |
   v
Download Installer
   |
   v
Install CYVRA Erase
   |
   v
First Launch
   |
   v
Verify Entitlement
   |
   v
Explain One-Device Binding
   |
   v
Activate
   |
   v
Device Binding
   |
   v
Issue Revocable Device Authorization
   |
   v
Use CYVRA Erase
   |
   v
Submit Feedback

The obsolete payment gate must not interfere with the approved initial eligible-user journey.

10. Device Binding

CYVRA will support controlled device binding.

The intended initial model is:

Valid Entitlement
        |
        v
First Successful Activation
        |
        v
Bind Approved Device
        |
        v
Issue Device Authorization
        |
        v
Allow Application Operation

The system must support future revocation and administrative control.

The exact device fingerprinting and security implementation must be designed carefully before production.

The architecture does not approve insecure or irreversible device identification shortcuts.

11. Admin Architecture

The administrative frontend is not a direct infrastructure client.

The admin frontend communicates with the CYVRA Control Plane.

Admin Browser
      |
      v
Admin Frontend
      |
      v
CYVRA Control Plane
      |
      +---- Neon
      |
      +---- Resend
      |
      +---- R2

Administrative authorization must be enforced by the server-side control plane.

Hiding a frontend page is not considered authorization.

12. Existing CYVRA Core

The existing CYVRA erase core remains protected from unnecessary architectural disruption.

The architecture separates:

CYVRA Platform
    |
    +-- Customer and Admin Frontends
    |
    +-- CYVRA Control Plane
    |
    +-- Identity and Entitlements
    |
    +-- Protected Installer Delivery
    |
    +-- Activation and Device Binding
    |
    +-- CYVRA Desktop Application
            |
            +-- CYVRA Erase Core

The objective is to build the surrounding product platform without casually rewriting the existing erase functionality.

13. Security Boundaries

The following boundaries are mandatory:

Component	Direct Browser Access	Privileged Credentials
Public Frontend	Yes	No
Admin Frontend	Yes, authorized users only	No
CYVRA Control Plane	API only	Yes, server-side only
Neon	No direct browser access	Server-side only
Resend	No direct browser access	Server-side only
R2	No unrestricted installer access	Controlled by delivery design
CYVRA Desktop App	Controlled application access	No master infrastructure credentials
14. Environment and Secret Management

Sensitive values must be managed as secrets or protected environment configuration.

Examples include:

DATABASE_URL
RESEND_API_KEY
R2-related configuration
JWT or session secrets
activation secrets
administrative secrets

Secret values must not be committed to Git.

Secret values must not be placed in public frontend source files.

Development, preview, and production environments should be separated where practical.

15. Deployment Direction

The repository may contain multiple deployable components.

These should be configured according to their actual deployment responsibilities.

Conceptually:

Repository
│
├── frontend
│     └── Public CYVRA website
│
├── admin-frontend
│     └── CYVRA administration interface
│
├── worker
│     └── CYVRA Control Plane
│
├── agent-windows
│     └── CYVRA Windows application and erase core
│
└── docs
      └── Architecture and operational documentation

Deployment configuration must not blindly treat the entire repository root as one static asset directory.

Each deployment target must have an explicitly verified responsibility.

16. Build and Release Direction

The Windows application release pipeline should eventually support:

Source
   |
   v
Build
   |
   v
Automated Verification
   |
   v
Release Candidate
   |
   v
Code Signing
   |
   v
Timestamping
   |
   v
Approved Installer Artifact
   |
   v
Release Storage
   |
   v
Controlled Customer Delivery

The current engineering foundation is not automatically equivalent to a final production installer.

Production signing and installer release remain separate release milestones.

17. What Is Frozen

The following architectural decisions are frozen:

One central CYVRA API boundary.
Cloudflare Worker as the initial CYVRA Control Plane.
Modular monolith implementation.
No direct frontend-to-Neon access.
No direct frontend-to-Resend access.
No infrastructure credentials in browser applications.
Neon as the relational database direction.
Resend as the transactional email direction.
Cloudflare R2 as the approved installer storage direction.
Protected installer delivery through the CYVRA Control Plane.
Controlled entitlement and device-binding architecture.
Separation between CYVRA platform services and the CYVRA erase core.
Admin authorization enforced server-side.
Version-controlled database migrations.
No unnecessary rewrite of the existing CYVRA erase core.
18. What Is Not Yet Frozen

The following implementation details remain intentionally open:

exact authentication mechanism
exact session or token format
OTP format and expiry
entitlement policy details
download retry limits
download authorization expiry
installer version retention policy
exact R2 bucket configuration
production Cloudflare plan selection
production R2 cost configuration
database schema details
device fingerprint implementation
device transfer policy
exact activation protocol
final Windows signing provider
timestamping provider
final installer packaging strategy
detailed production CI/CD configuration

These items must be designed and approved before they are frozen.

19. Immediate Implementation Order

The approved implementation order is:

Phase A — Preserve and Verify Existing Work
Inspect the repository state.
Preserve all useful Codespaces work.
Verify the current branch and working tree.
Do not casually overwrite existing work.
Establish a known-good baseline.
Phase B — Architecture Documentation
Store this architecture document.
Document implementation decisions as they are approved.
Keep architecture decisions separate from experimental notes.
Phase C — Control Plane Foundation
Verify the existing Worker structure.
Establish the CYVRA Control Plane module structure.
Verify environment configuration.
Establish secure secret handling.
Create health and baseline API verification.
Phase D — Neon Integration
Verify the Neon project and database access.
Define the application schema.
Add version-controlled migrations.
Establish Worker-to-Neon connectivity.
Verify database access without exposing credentials.
Phase E — Resend Integration
Verify the Resend account and sending domain configuration.
Configure the API secret securely.
Implement the approved verification email flow.
Test delivery.
Add controlled failure handling.
Phase F — Identity and Entitlements
Account creation.
Sign in.
Email verification.
Legal acceptance recording.
Eligibility evaluation.
Entitlement creation and validation.
Phase G — Protected Installer Delivery
Define installer artifact storage.
Configure Cloudflare R2.
Define installer release records.
Implement download authorization.
Implement controlled download delivery.
Test expiry, retry, and revocation behavior.
Phase H — Windows Activation
Complete desktop application integration.
Implement entitlement verification.
Implement first-launch activation.
Implement device binding.
Implement revocation handling.
Phase I — First Official User

The target is:

One official CYVRA user can successfully create an account, verify email ownership, receive eligibility and entitlement approval, download the authorized installer, install the application, activate one device, use CYVRA Erase, and provide detailed feedback.

Only after this complete journey is verified should broader rollout be considered.

20. Change Control

This architecture is approved and frozen.

Future changes must not be introduced casually.

Any proposed change should identify:

What architectural decision is changing.
Why the current design is insufficient.
Security implications.
Database implications.
Deployment implications.
Customer journey implications.
Whether existing CYVRA core functionality is affected.

Approved changes should be recorded as explicit amendments to this architecture.

21. Current Starting Point

The project will resume implementation from this architecture.

The immediate priority is not to redesign the system again.

The priority is to proceed step by step:

Preserve and verify the existing repository state.
Verify the existing Cloudflare deployment configuration.
Establish the CYVRA Control Plane correctly.
Integrate Neon securely.
Integrate Resend securely.
Implement identity and entitlement flow.
Configure protected installer storage and delivery.
Complete activation and device binding.
Test the complete first-user journey.
Collect detailed feedback from the first official user.