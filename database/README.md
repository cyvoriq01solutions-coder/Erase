# CYVORIQ Database Foundation

Database:
PostgreSQL hosted on Neon.

Architecture:
Cloudflare Worker -> Hyperdrive -> Neon PostgreSQL

The browser must never connect directly to Neon.

## Phase 4 scope

This phase establishes:

- PostgreSQL schema source files
- migration structure
- core organization/user/asset/device tables
- assessments
- evidence
- verification results
- audit events

No production database credentials are committed to Git.

No Hyperdrive binding is added in this phase.
