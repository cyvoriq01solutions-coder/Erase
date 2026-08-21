# CYVORIQ Erase

CYVORIQ Erase is the engineering repository for the CYVRA Windows privacy-verification product, its customer and admin portals, Cloudflare API, and Neon database migrations.

> **Current customer-release status: NOT RELEASED.**
>
> The Rust agent at version 0.2.1 is an engineering command-line foundation that emits JSON. It is not yet the frozen signed installer, desktop GUI, activation flow, one-device binding, or authenticated-report product.

Read the [Current Product Freeze & Team Handoff — 2026-08-21](docs/product/current-product-freeze-handoff-2026-08-21.md) before starting or extending Windows, C5–C7, activation, download, evidence, or report work.

## Product safety boundary

The current customer MVP is non-destructive:

**ASSESS → PDEM → EVIDENCE → VERIFY → REPORT**

The agent may map privacy-relevant metadata but must not read personal file contents, email bodies, browser-history content, passwords, tokens, or secrets. It must not delete, overwrite, move, encrypt, wipe, or otherwise modify customer data.

Destructive erasure belongs to a separately approved future lifecycle and is not authorized by the current freeze.

## Repository structure

| Path | Purpose | Current status |
| --- | --- | --- |
| frontend/ | Public/customer React portal | Implemented foundation |
| admin-frontend/ | React admin portal | C4.1 foundation merged |
| worker/ | Cloudflare Worker API and Hyperdrive integration | Authentication/admin foundation merged |
| database/migrations/ | Versioned Neon PostgreSQL migrations | Development migrations through 0006 |
| agent-windows/ | Rust Windows metadata/PDEM engineering agent | Version 0.2.1 CLI foundation |
| docs/product/ | Product freezes, checkpoints, and handoffs | Current canonical handoff linked above |
| docs/deployment/ | Deployment, test, and rollback guidance | Operational references |

The intended service path is:

**Customer/Admin UI → Cloudflare Worker → Neon authorization and audit → private R2 artifact or authenticated report**

Private release artifacts must not be exposed through a permanent unauthenticated .exe URL.

## Current delivery status

- Customer authentication foundation is merged.
- Admin portal and authentication foundation are merged.
- C4.1A P0 hardening through durable OTP rate limiting is merged.
- Database migration 0006 and its health-contract correction are merged.
- The Windows customer application remains in the pre-implementation W1 contract stage.
- Private R2 installer distribution, activation/device binding, authenticated reports, signing, update, and customer pilot evidence remain pending.
- Main branch protection and required checks remain pending.

Code merged into GitHub does not by itself prove that Cloudflare, Neon, or a customer release was deployed.

## Development baseline

- Node.js 24
- npm 11
- TypeScript 5.9.3
- React 19.2.8
- Vite 8.2.0
- Wrangler 4.123.0
- Rust 1.97.1

Verify the local toolchain:

~~~bash
npm run verify:environment
rustc --version
cargo --version
~~~

## Install dependencies

Install each JavaScript package independently:

~~~bash
npm --prefix frontend install
npm --prefix admin-frontend install
npm --prefix worker install
~~~

Never commit .env files, database URLs, API tokens, signing material, or provider secrets.

## Customer frontend

~~~bash
npm --prefix frontend run build
npm --prefix frontend run dev -- --host 0.0.0.0
~~~

## Admin frontend

~~~bash
npm --prefix admin-frontend run build
npm --prefix admin-frontend run dev -- --host 0.0.0.0
~~~

## Cloudflare Worker

Run local/static verification:

~~~bash
npm --prefix worker run typecheck
npm --prefix worker run deploy:dry-run
~~~

A dry run validates packaging; it does not deploy.

Wrangler named environments do not automatically inherit every top-level binding, variable, or secret. Development, staging, and production configuration must be checked independently before any approved deployment.

## Windows engineering agent

Run Rust quality checks from the repository root:

~~~bash
cargo fmt --manifest-path agent-windows/Cargo.toml --check
cargo clippy --manifest-path agent-windows/Cargo.toml -- -D warnings
cargo test --manifest-path agent-windows/Cargo.toml
~~~

Running or compiling the agent does not create a customer-ready installer. The next Windows phase is W1: freeze the GUI, installer, Agent-to-Worker API, device-binding, code-signing/release, authenticated-report, first-500 entitlement, and observability contracts.

## Database migrations

Migrations live in database/migrations/ and must be applied in numeric order to an explicitly verified target database.

Before any migration:

1. Confirm the Neon project, branch, database, role, and active schema.
2. Confirm whether the approval is for development, staging, or production.
3. Use stop-on-error execution.
4. Verify schema and health-contract results.
5. Record rollback and evidence.

Migration 0006 was approved for Neon development only. This README does not authorize a production migration.

## Contribution workflow

1. Fetch current main.
2. Create one dedicated feature or documentation branch.
3. Keep the change bounded to its approved concern.
4. Run relevant checks and inspect the exact diff.
5. Open a Draft pull request.
6. Record migrations, environment impact, tests, deployment status, rollback, and open risks.
7. Obtain review before marking ready or merging.
8. Obtain separate approval for deployments, production data changes, secrets, or customer releases.

Repository roles:

- **cyvoriq01solutions-coder:** owner/admin and final repository approval authority.
- **mswaroop707-del (Swarup-Personal):** co-developer with write access.

GitHub permissions do not grant runtime CYVORIQ application roles.

## Source-of-truth documents

- [Current Product Freeze & Team Handoff — 2026-08-21](docs/product/current-product-freeze-handoff-2026-08-21.md)
- [Control-plane C3–C7 freeze](docs/product/control-plane-c3-c7-freeze.md)
- [C4.1 admin-auth freeze](docs/product/c4-1-admin-auth-freeze.md)
- [C4/C5 admin-download freeze](docs/product/c4-c5-admin-download-freeze-2026-08-19.md)
- [C3 authentication test and rollback](docs/deployment/c3-auth-test-rollback.md)

If an older checkpoint conflicts with the approved current handoff, follow the current handoff or raise the conflict before implementation.
