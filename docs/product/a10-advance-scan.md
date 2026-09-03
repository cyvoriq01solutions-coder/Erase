# A10: Live USB, charger and camera intake

A10 sits in Interactive Checks **immediately after the keyboard**. A
technician who is already on the laptop can insert a USB stick, plug the
charger, and force-open the camera for a still or a short clip, then
finish the remaining A7 attestations.

This is not extra CG-1.0 points. USB listing is not the four physical-port
points. Charging is not a rubric domain. Camera Pass/Fail is still the
operator attestation (2 points). Snapshots and clips are discarded.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.
`grading_issuance_enabled` stays false.

## What A10 adds

| Layer | Addition |
| --- | --- |
| UI | After keyboard: USB insertion overlay, charger overlay, live camera overlay (photo or 5s clip) |
| Command | Seventh Tauri command `probe_live_intake` — removable volumes + Win32_Battery only |
| CSP | `img-src` allows `blob:` / `data:`; `media-src` allows `blob:` / `mediastream:` |
| Report D | Technician rows: USB insertion sense, charger status, live camera session. No image stored |
| Method | Honest: listing a letter ≠ scoring a port; BatteryStatus 2 ≠ charging |

## Order on the laptop

1. Colour wash
2. Keyboard
3. **USB insertion** — insert any stick; wait for a new removable letter
4. **Charger** — plug in; wait for Charging (status 6–9) or On mains (status 2)
5. **Camera** — Open camera, Take photo or Record 5s clip, then attest presence
6. Trackpad, speakers
7. Physically verified ports (the scored insertion)

## Guards

- Five destinations
- Seven Tauri commands (`probe_live_intake` is the A10 exception to the A8 six)
- `desktop/src-tauri/src/lib.rs` still has no `std::fs` / `std::process`
- No write to a USB stick and no write to an assessed drive
- No keystroke log, no stored snapshot, no microphone recording
- No invented charging points
- Webcam LED will turn on; that is expected
- Issuance stays off

## Still off

- Report signing (A9)
- Authenticode, unsigned B2 upload, WinPE
- CYVRA Purge / Report B (plan only)
