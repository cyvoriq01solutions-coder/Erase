# P-LICENCE — Purge product key bind

Wipe execution stays **fail-closed**. This slice only issues and binds a **CYVRA Purge** product key. Red Data purge still does not erase storage.

## Two products

| Product | Key shape | Desktop bind | Unlocks |
| --- | --- | --- | --- |
| Assessment (`CYVORIQ_ERASE`) | `CYVRA-XXXX-XXXX-XXXX-XXXX` | Help → Enter licence | Reports A–D. Does **not** unlock Purge. |
| Purge (`CYVORIQ_PURGE`) | `CYVRA-PRG-XXXX-XXXX-XXXX-XXXX` | Data purge workstream | Key bound. Wipe still off until **P-SECONDARY**. |

One PC may hold both binds. Assessment bind does not skip Report A. Purge bind does not skip Report A.

## Control plane

- Additive product on `licenses`: `CYVORIQ_ERASE` or `CYVORIQ_PURGE`.
- Run `database/migrations/0008_purge_licence.sql` on Neon **before** deploying Worker code that inserts `CYVORIQ_PURGE`.
- Admin **Issue purge key** is separate from **Issue licence**. Email subject: `Your CYVRA Purge activation key`.
- Full keys are not re-stored. Prefixes only after issue.

## Worker

- `POST /api/v1/auth/activate` stays assessment-only. A purge key returns `wrong_product`.
- `POST /api/v1/auth/activate-purge` binds a purge key only.
- Desktop `destructive_operations_enabled` stays **false**.

## Desktop

- Eighth command: `activate_purge_license`.
- Bootstrap `purge_licence_bound` is compile-time **false**. Bind is session-only until P-SECONDARY.
- Report S is named only. No Report S file. No ATA/NVMe.
