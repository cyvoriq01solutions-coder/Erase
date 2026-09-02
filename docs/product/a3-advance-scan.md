# A3: Cameras, microphones, and USB topology

A3 is the second Advance scan collection slice. It enumerates cameras and
microphones across the PnP classes Windows actually uses, and it walks USB
controller / hub / device topology instead of guessing connectors from SMBIOS
labels.

Basic scan and Report A stay unchanged. Navigation still has exactly five
destinations. Wipe, B2 upload, and Authenticode stay out.

## What A3 adds

| Layer | Addition |
| --- | --- |
| Probe | `agent-windows/src/capture_probe.rs` — Camera, Image, USB video and media classes, plus audio endpoints. No frame, no sample. |
| Probe | `agent-windows/src/usb_topology.rs` — Win32 USB controllers, hubs and attached devices, with negotiated speed when Windows reports it |
| Engine | Ports and connectivity scores **2 of 10** when controllers are enumerated. Screen-domain camera/mic presence stays **0** until a technician attests (A7) |
| Report D | Camera/mic list with the class that answered; USB controllers, hubs and attached devices; physically verified ports stay not attempted |

## Why several camera classes

Report A asked a single Camera ClassGuid and an AudioEndpoint filter. On at
least one laptop that printed "None enumerated by Windows" even though a UVC
webcam was present. The Camera ClassGuid misses some USB Video Class devices.

A3 unions:

- PnP Camera class
- PnP Image class
- USB video service (`USBVideo`)
- Media class, name-filtered so scanners are not turned into cameras
- Audio endpoints and sound devices, name-filtered so speakers are dropped

Each listed camera records which class answered. Enumeration is not content
inspection: `frames_captured` and `audio_recorded` stay false, and
`content_inspected` stays false.

## USB topology, not plastic connectors

Report A counted SMBIOS port-connector labels. Those labels often miss HDMI and
invent USB counts that do not match the sockets on the chassis.

A3 prints what Windows can see:

- USB controllers
- USB hubs, including root hubs
- Attached devices, with port index and negotiated speed when reported

Empty sockets are invisible. Report D therefore keeps **physically verified
ports** as not attempted until a technician inserts a device (A7). Wi-Fi,
Bluetooth and Ethernet stay not collected until A5.

## Scoring

Rubric CG-1.0, still **Graded by CYVRA Grading Engine**.

- Battery (A2): up to 20 points when firmware reports two real capacities.
- USB topology (A3): **2 of 10** Ports points when the controller class
  answers. The remaining 8 stay not assessable (radios + physical insertion).
- Cameras and microphones: listed on Report D, **0** Screen-domain points.
  Presence confirmation is an operator attestation, not a PnP listing.

On a laptop with a healthy battery and enumerated USB controllers, coverage is
about **22%**. The grade stays **withheld** because storage SMART is mandatory
and unread. That is intended. Collection for storage is A4.

## Still off

- Storage SMART (A4)
- Display EDID and radios (A5)
- Consent-gated benchmarks and interactive checks (A6–A7)
- Live camera preview and microphone record (A10)
- Issuance, cloud authentication, kernel sensors
- Destructive operations, WinPE, Authenticode, unsigned B2 upload
