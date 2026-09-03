# A7: Interactive technician checks

A7 adds phase-one technician checks to Advance scan. Colour wash, keyboard,
trackpad, speakers, camera/microphone presence, and physical port insertion
are operator-attested. Live USB listing, charger sensing and in-session
camera capture arrive in A10, immediately after the keyboard check.
Keystrokes, tones and washes are not stored.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.
`grading_issuance_enabled` stays false.

## What A7 adds

| Layer | Addition |
| --- | --- |
| UI | Colour wash overlay, keyboard check, trackpad canvas, left/right speaker tones, USB insertion attestation, camera/mic presence attestation |
| Engine | Screen domain can score **15** when every subject is attested pass. Physical insertion adds **up to 4** of the 10 ports points |
| Report D | Technician checks group prints Pass / Fail / Not attempted. Skip is never scored as zero |
| Method | Keyboard Fn/OEM limit disclosed. Insertion is not a write to the stick or to an assessed drive |

## Scoring (rubric CG-1.0)

Printed: **Graded by CYVRA Grading Engine**, rubric CG-1.0.

Screen, keyboard and peripherals (15), awarded only after a human attests:

- Display colour wash inspected → 4
- Keyboard: attempted keys registered → 4
- Trackpad: movement, clicks and gestures → 3
- Speakers: both channels heard → 2
- Camera / microphone present and confirmed → 2

Skip or not attempted → that subject's points stay **not assessable**.
Fail → the subject is assessed and awarded **0**.

Ports physical insertion (4 of 10):

- All attempted ports passed → 4
- Partial pass → 2
- Any attempted port failed → 0 assessed
- Not attempted → not assessable

A laptop with battery + identity + SMART + USB + radios + all A7 passes
and no benches reaches about **74%** coverage, so a **provisional** band
may print. Issuance stays off.

## Guards

- Default for every subject is not attempted
- Keystroke log is not written to Report D
- No write to a USB stick and no write to an assessed drive for insertion
- No sixth navigation destination
- `desktop/src-tauri/src/lib.rs` still has no `std::fs` / `std::process`

## Still off

- Grading issuance (A8)
- Report signing (A9)
- Live USB / charger / camera intake (A10)
- Authenticode, unsigned B2 upload, WinPE
- CYVRA Purge / Report B (plan only)
