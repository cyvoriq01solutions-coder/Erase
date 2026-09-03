# CYVRA Erase — operator guide

From first registration to Report D download. Each step lists the **expected
result** and **what to derive if that result is missing**. This version
assesses a Windows PC. It does not erase files.

Do not print or guess internal construction details of the application.
Customer documents name CYVRA Erase, CYVORIQ Solutions, Windows, and the
report identifiers only.

---

## 1. Account registration

**What you do.** An organisation administrator invites or approves the
operator on https://www.cyvra.co.in. The operator verifies identity on the
website if asked.

**Expected result.** The account shows as approved. Approval is not a
licence. The website login does not unlock the Windows application.

**If not.** The operator cannot receive an activation key. Do not invent a
key. Ask the administrator to finish approval. A pending or rejected
account means the PC must not be assessed under CYVRA.

---

## 2. Licence issuance and email

**What you do.** An administrator issues a licence for one Windows PC. CYVRA
emails the key from auth@cyvra.co.in, subject “Your CYVRA Erase activation
key”.

**Expected result.** One key in that mail. Store it. CYVORIQ does not keep
the full key after issuance.

**If not.** No mail means no licence was issued, the mailbox is wrong, or
the message is in junk. Do not reuse a key from another PC. A key for a
different operator is not valid here.

---

## 3. Install on the assessed PC

**What you do.** Run the Windows setup package on the PC that will be
assessed. Walk Welcome → Terms → Activate.

**Expected result.** The setup window shows the CYVORIQ mark, three steps
(Welcome, Terms, Activate), and the assessment-only licence. Uninstall
later uses Windows Programs and Features, entry CYVRA Erase.

**If not.** If Windows SmartScreen warns, this build may still be unsigned.
That is a signing gap, not proof the PC failed. If setup will not start,
Windows or the package is blocked — do not continue the assessment. If the
licence text is missing, stop; do not click through an empty agreement.

---

## 4. Activate this Windows PC

**What you do.** Paste the emailed key. Choose Activate, then Open CYVRA
Erase.

**Expected result.** The first successful activation binds the licence to
this PC. The same key cannot bind a second PC.

**If not.** `invalid_key` means the key is mistyped, already bound, or not
issued. Do not retry endlessly. Online binding needs a path to
api.cyvra.co.in; if the PC is offline, activation cannot finish. A preview
session without live binding is not a licensed assessment.

---

## 5. Overview and safety banner

**What you do.** Confirm the banner: local assessment; purge and grading
issuance stay off. Home shows three coloured workstreams:

1. **01 Standard assessment** (orange) — verification, Report A, Back to main.
2. **02 Advance diagnostic** (purple) — Advance scan, Report D, Back to main.
3. **03 Data purge** (red) — wipe record only; not enabled.

**Expected result.** Five destinations only: Overview, Verification,
Results, Report, Help. Each workstream returns to Overview after the
report is saved.

**If not.** If the application will not load the safety boundary, no
customer operation has started. Restart. Do not treat a failed start as a
hardware fault on the assessed PC.

---

## 6. Drive selection (Verification)

**What you do.** Leave the Windows system drive selected. Leave USB sticks
and backup disks unchecked unless they must appear on Report A.

**Expected result.** The list shows internal disks and any already-mounted
removable volumes. Large extra disks make verification take longer.

**If not.** If a USB stick you need is missing from the list, Windows has
not mounted it. Insert it in File Explorer first if it must appear on
Report A. Missing letters are not proof the socket is dead. USB topology
for Report D comes from Advance scan, not from a live insertion check.

---

## 7. Report A — basic assessment

**What you do.** Run verification, then Generate report / Save as PDF.

**Expected result.** Report A is a local pre-sanitization assessment:
hardware identity, serials Windows actually returned, and where documents
appear to live. It is not a wipe certificate. Battery health and connector
counts print only when collected. “No data was erased.”

**If not.** A serial printed as not reported means Windows did not supply
it — derive “not collected”, never zero. If the PDF will not save, the
chosen folder is blocked; the scan itself may still be valid on screen. If
verification stops with an error, read the on-screen message; do not
re-run as a wipe.

---

## 8. Advance scan consent

**What you do.** Optionally allow benchmarks. The write test is a second,
separate permission and writes one temporary file that should be deleted.
Predicted-failure disks are not exercised.

**Expected result.** Both permissions start off. Declining benchmarks
leaves those areas not assessable. They are never scored as zero.

**If not.** If a write is reported when you did not allow the write test,
stop and treat the result as unsafe. Do not call that a completed Report D.

---

## 9. Technician checks — display, keyboard, camera

Order on the PC:

1. Colour wash (red, green, blue, white, black)
2. Keyboard
3. Camera live capture
4. Trackpad and speakers
5. Camera/microphone presence attestation

USB sockets and charger/AC state are **not** live buttons. They are read
once during Advance scan (USB controllers, hubs, attached devices, battery
status) and printed on Report D. This version removed live USB insertion
and live charger overlays because they opened repeating command windows
and froze some laptops.

Skipped checks stay not assessable. Keystrokes, tones, washes, snapshots
and clips are not stored.

### USB topology and charger on Report D

**What you do.** Run Advance scan. Do not look for Check USB ports or
Start charger check — those controls are gone.

**Expected result.** Report D lists USB topology Windows exposed in that
one pass, and battery/charger state from the same pass. Charging is
telemetry, not a grading point. USB 1–USB 4 technician ticks stay
not-attempted unless a later version reintroduces attestation without a
live process.

**If not.**

| What you see | What to derive |
| --- | --- |
| USB group says not collected | Windows did not return controller/device data in this pass. That is not a live-insert failure. |
| Battery group empty on a desktop | No battery pack is expected. That is not a failed charger. |
| Status is on mains, not charging | AC is present. Do not write “charging”. |

### Camera

**Expected.** Webcam light on. Photo or 5s clip stays in the window and is
discarded. Then attest presence for the two points.

**If not.** Windows denied the camera: check Privacy settings. Do not
attest Pass. No stored image is success, not a fault.

---

## 10. Report D — Advance scan result

**What you do.** Run Advance scan, open Report, Save Report D as PDF.
Optionally Verify this report.

**Expected result.** Report D shows coverage, technician rows, USB
topology and battery/charger state from that one Advance scan pass, method
and limitations, and a Physical verification block (technician name, date,
result — not a handwritten underscore line). A local integrity seal
(SHA-256 and Ed25519, QR) proves the JSON was not altered after this scan
on this PC. Printed: Graded by CYVRA Grading Engine, rubric CG-1.0. Issued
by CYVORIQ Solutions Pvt. Ltd. The document is computer-generated and not
cloud-authenticated. Save the PDF, then Back to main.

**If not.**

| What you see | What to derive |
| --- | --- |
| Grade withheld | Too little of the device was assessed, or a required area is missing. Read Not assessable. Do not invent a band. |
| Coverage below the floor | Same: withheld, not an F for missing evidence. |
| Confirmed measured fault | That can force F. That is evidence held, not evidence missing. |
| Verify this report fails | The PDF/JSON no longer matches the seal from this scan. Do not treat it as authentic. Re-run on the PC if the original is gone. |
| No seal | Older build, or the scan did not finish. Not a CYVORIQ certificate either way. |
| “Authenticated by CYVORIQ” | Must not appear. If you see it, the build is wrong — stop. |

This is not an issued CYVORIQ grading certificate. It is not Authenticode.
It does not certify sanitization, destruction, resale grade, or legal
compliance.

---

## 11. Email and copies

**What you do.** If emailing a report, tick the consent warning. Keep PDFs
off disks you may later erase.

**Expected result.** The operator chose a destination. CYVRA does not open
private file contents to build the report.

**If not.** Mail failure does not invalidate a saved PDF on this PC. Do not
copy the PDF onto the USB stick you just used for port tests if that stick
will leave the site untrusted.

---

## 12. What this version must never do

- Erase, overwrite, encrypt, or destroy customer files
- Collect passwords, recovery keys, or email bodies
- Write to the technician USB stick
- Store keystrokes or camera frames
- Award USB port points from volume listing alone
- Treat charging as a CG-1.0 domain
- Present a sixth navigation item
- Claim cloud authentication or a grading certificate

Purge stays off until a separate, explicit licence to purge is approved
outside this guide.
