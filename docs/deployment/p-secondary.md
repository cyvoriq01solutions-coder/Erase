# P-SECONDARY deploy

1. Merge this PR after Desktop Shell is green.
2. Run **CYVRA Erase — desktop NSIS** (`desktop-shell-build.yml`) on `main`.
3. Install the unsigned NSIS on a **lab PC** only.
4. Bind the Purge key from P-LICENCE. Run Report A and save the PDF.
5. Mode S: extra disks only, USB opt-in if needed, type the PC name, type `ERASE`.
6. Confirm the system letter is refused. Confirm no key → helper does not start.
7. On verify PASS, Save Report S. On FAIL, no Report S.
8. Reply `WINDOWS P-SECONDARY OK` plus the installer SHA256 prefix.

Do not start Authenticode until that line. Do not upload to B2.
