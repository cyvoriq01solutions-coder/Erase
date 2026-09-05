# P-USB-LOCK deploy

1. Merge this PR after Desktop Shell is green.
2. Run **CYVRA Erase — desktop NSIS** (`desktop-shell-build.yml`) on `main`.
3. Install the unsigned NSIS on a **lab PC** only.
4. Bind a `CYVRA-PRG-` key. Run Report A and save the PDF.
5. Mode S: choose **exactly one extra INTERNAL disk**. USB letters stay visible and blocked.
6. Confirm a USB letter cannot be chosen. Confirm two extra disks cannot be submitted in one job.
7. Confirm zero extra disks selected → Data purge stays disabled.
8. Type this PC’s name, type `ERASE`, run the job.
9. On verify PASS, Save Report S. On FAIL, save the failed-job note. No Report S.
10. Reply `WINDOWS P-USB-LOCK OK` plus the installer SHA256 prefix.

Lab media is an extra internal disk or extra SATA/NVMe. Not a USB stick. Not a USB enclosure. Not the OS disk.

Do not Authenticode the 31007edf installer as the customer file. Sign this USB-blocked NSIS only after that OK line. Do not upload to B2 until Authenticode.
