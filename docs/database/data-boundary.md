# CYVORIQ Database Data Boundary

The PostgreSQL database stores structured platform data.

Raw personal device content should not become the normal cloud data model.

The cloud model should favor structured facts such as:

- device identity
- storage/device metadata
- assessment state
- evidence metadata
- hashes/fingerprints
- verification status
- audit events

Secrets and database credentials must never be exposed to frontend code.
