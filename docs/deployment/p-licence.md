# P-LICENCE deploy

1. Neon: run `database/migrations/0008_purge_licence.sql` **before** merging Worker code that inserts `CYVORIQ_PURGE`.
2. Merge this PR. Deploy Worker. Confirm `/api/v1/auth/activate-purge` exists.
3. Run **CYVRA Erase — desktop NSIS** (`desktop-shell-build.yml`) on `main`.
4. Laptop: enter a purge key on Data purge. Wipe must still fail closed.
5. Reply `WINDOWS P-LICENCE OK` plus the installer SHA256 prefix.
