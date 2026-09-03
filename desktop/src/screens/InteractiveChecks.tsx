import { useEffect, useRef, useState, type PointerEvent, type ReactNode } from "react";
import { probeLiveIntake } from "../adapters/desktopBridge";
import type {
  AdvanceInteractive,
  AttestationValue,
  LivePowerStatus,
  LiveRemovableVolume,
  PortAttestationValue,
} from "../types/shell";

const WASH_COLOURS = [
  { name: "Red", value: "#c62828" },
  { name: "Green", value: "#2e7d32" },
  { name: "Blue", value: "#1565c0" },
  { name: "White", value: "#f5f7fb" },
  { name: "Black", value: "#11141a" },
] as const;

const EMPTY_POWER: LivePowerStatus = {
  present: false,
  onMains: false,
  charging: false,
  statusCode: null,
  statusLabel: "Not collected",
  chargePercent: null,
  available: false,
  detail: "Waiting for Windows battery status.",
};

interface InteractiveChecksProps {
  value: AdvanceInteractive;
  disabled: boolean;
  onChange: (next: AdvanceInteractive) => void;
}

function volumeKey(volume: LiveRemovableVolume): string {
  return volume.letter.trim().toUpperCase();
}

function describeVolumes(volumes: LiveRemovableVolume[]): string {
  if (volumes.length === 0) {
    return "";
  }
  return volumes
    .map((volume) => {
      const letter = `${volume.letter.replace(/:$/, "")}:`;
      return volume.label ? `${letter} ${volume.label}` : letter;
    })
    .join("; ");
}

function recorderMimeType(): string {
  if (typeof MediaRecorder === "undefined") {
    return "";
  }
  for (const type of ["video/webm;codecs=vp8", "video/webm", "video/mp4"]) {
    if (MediaRecorder.isTypeSupported(type)) {
      return type;
    }
  }
  return "";
}

export function InteractiveChecks({ value, disabled, onChange }: InteractiveChecksProps) {
  const [washOpen, setWashOpen] = useState(false);
  const [washIndex, setWashIndex] = useState(0);
  const [keysOpen, setKeysOpen] = useState(false);
  const [keysTried, setKeysTried] = useState(0);
  const seenKeys = useRef(new Set<string>());
  const [padOpen, setPadOpen] = useState(false);
  const [padMoved, setPadMoved] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const drawing = useRef(false);

  const [usbOpen, setUsbOpen] = useState(false);
  const [usbVolumes, setUsbVolumes] = useState<LiveRemovableVolume[]>([]);
  const [usbNew, setUsbNew] = useState<LiveRemovableVolume[]>([]);
  const [usbError, setUsbError] = useState<string | null>(null);
  const usbBaseline = useRef(new Set<string>());

  const [chargerOpen, setChargerOpen] = useState(false);
  const [power, setPower] = useState<LivePowerStatus>(EMPTY_POWER);
  const [powerError, setPowerError] = useState<string | null>(null);

  const [cameraOpen, setCameraOpen] = useState(false);
  const [cameraError, setCameraError] = useState<string | null>(null);
  const [snapshotUrl, setSnapshotUrl] = useState<string | null>(null);
  const [clipUrl, setClipUrl] = useState<string | null>(null);
  const [recording, setRecording] = useState(false);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const valueRef = useRef(value);
  valueRef.current = value;

  useEffect(() => {
    if (!keysOpen) return;
    function onKey(event: KeyboardEvent) {
      event.preventDefault();
      seenKeys.current.add(event.code || event.key);
      setKeysTried(seenKeys.current.size);
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [keysOpen]);

  useEffect(() => {
    if (!usbOpen && !chargerOpen) return;
    let cancelled = false;

    async function tick() {
      try {
        const probe = await probeLiveIntake();
        if (cancelled) return;
        const next = { ...valueRef.current };
        if (usbOpen) {
          setUsbError(null);
          setUsbVolumes(probe.removable);
          const fresh = probe.removable.filter((volume) => !usbBaseline.current.has(volumeKey(volume)));
          setUsbNew(fresh);
          const listed = describeVolumes(probe.removable);
          const appeared = describeVolumes(fresh);
          next.liveUsb = appeared
            ? `Windows listed a new removable volume ${appeared} during this check. CYVRA did not write to the stick.`
            : listed
              ? `Windows already listed removable volume ${listed} when this check opened. CYVRA did not write to the stick.`
              : "No removable volume was listed while this check was open.";
        }
        if (chargerOpen) {
          setPowerError(null);
          setPower(probe.power);
          const percent =
            probe.power.chargePercent === null ? "" : ` · ${probe.power.chargePercent}%`;
          next.livePower = probe.power.available
            ? `${probe.power.statusLabel}${percent}. ${probe.power.detail}`
            : probe.power.detail;
        }
        onChange(next);
      } catch (error) {
        if (cancelled) return;
        const message =
          error instanceof Error ? error.message : "CYVRA could not read live USB and charger status.";
        if (usbOpen) setUsbError(message);
        if (chargerOpen) setPowerError(message);
      }
    }

    void tick();
    const timer = window.setInterval(() => {
      void tick();
    }, 1500);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
    // Poll only while an overlay is open. Live fields are written through valueRef.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [usbOpen, chargerOpen]);

  useEffect(() => {
    if (!cameraOpen) return;
    const video = videoRef.current;
    const stream = streamRef.current;
    if (video && stream) {
      video.srcObject = stream;
      void video.play().catch(() => undefined);
    }
  }, [cameraOpen, cameraError]);

  useEffect(() => {
    return () => {
      streamRef.current?.getTracks().forEach((track) => track.stop());
    };
  }, []);

  function setLive(field: "liveUsb" | "livePower" | "liveCamera", text: string) {
    onChange({ ...valueRef.current, [field]: text });
  }

  function setSubject(field: keyof AdvanceInteractive, next: AttestationValue | PortAttestationValue) {
    onChange({ ...valueRef.current, [field]: next });
  }

  function playTone(pan: number, label: string) {
    const AudioContextClass = window.AudioContext || (window as Window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioContextClass) {
      window.alert(`This browser cannot play a ${label} tone.`);
      return;
    }
    const context = new AudioContextClass();
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    const panner = context.createStereoPanner();
    oscillator.frequency.value = pan < 0 ? 440 : 523;
    panner.pan.value = pan;
    gain.gain.value = 0.08;
    oscillator.connect(panner);
    panner.connect(gain);
    gain.connect(context.destination);
    oscillator.start();
    window.setTimeout(() => {
      oscillator.stop();
      void context.close();
    }, 700);
  }

  function onPadPointer(event: PointerEvent<HTMLCanvasElement>) {
    const canvas = canvasRef.current;
    if (!canvas || !drawing.current) return;
    const rect = canvas.getBoundingClientRect();
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.fillStyle = "#3d7ea6";
    ctx.beginPath();
    ctx.arc(event.clientX - rect.left, event.clientY - rect.top, 4, 0, Math.PI * 2);
    ctx.fill();
    setPadMoved(true);
  }

  async function openUsbCheck() {
    setUsbError(null);
    setUsbNew([]);
    try {
      const probe = await probeLiveIntake();
      usbBaseline.current = new Set(probe.removable.map(volumeKey));
      setUsbVolumes(probe.removable);
    } catch {
      usbBaseline.current = new Set();
      setUsbVolumes([]);
    }
    setUsbOpen(true);
  }

  async function openChargerCheck() {
    setPowerError(null);
    setPower(EMPTY_POWER);
    setChargerOpen(true);
  }

  function revokePreview(url: string | null) {
    if (url && url.startsWith("blob:")) {
      URL.revokeObjectURL(url);
    }
  }

  function stopCamera() {
    recorderRef.current?.stop();
    recorderRef.current = null;
    streamRef.current?.getTracks().forEach((track) => track.stop());
    streamRef.current = null;
    if (videoRef.current) {
      videoRef.current.srcObject = null;
    }
  }

  async function openCameraCheck() {
    setCameraError(null);
    revokePreview(snapshotUrl);
    revokePreview(clipUrl);
    setSnapshotUrl(null);
    setClipUrl(null);
    setCameraOpen(true);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: "user" },
        audio: false,
      });
      streamRef.current = stream;
      if (videoRef.current) {
        videoRef.current.srcObject = stream;
        await videoRef.current.play().catch(() => undefined);
      }
      onChange({
        ...valueRef.current,
        liveCamera: "Live camera preview opened. The webcam light is on. No image has been stored.",
      });
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : "Windows did not allow the camera. Check Privacy settings for camera access.";
      setCameraError(message);
      onChange({
        ...valueRef.current,
        liveCamera: "Live camera preview did not start. Windows did not allow the camera in this session.",
      });
    }
  }

  function closeCameraCheck() {
    stopCamera();
    setRecording(false);
    setCameraOpen(false);
  }

  function takeSnapshot() {
    const video = videoRef.current;
    if (!video || video.videoWidth === 0) {
      setCameraError("The camera preview is not ready yet.");
      return;
    }
    const canvas = document.createElement("canvas");
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      setCameraError("This window cannot capture a still frame.");
      return;
    }
    ctx.drawImage(video, 0, 0);
    revokePreview(snapshotUrl);
    const url = canvas.toDataURL("image/jpeg", 0.8);
    setSnapshotUrl(url);
    setLive(
      "liveCamera",
      "Live preview opened. A snapshot was taken in this session and was not stored.",
    );
  }

  function startClip() {
    const stream = streamRef.current;
    const mime = recorderMimeType();
    if (!stream || !mime) {
      setCameraError("This window cannot record a short clip.");
      return;
    }
    chunksRef.current = [];
    const recorder = new MediaRecorder(stream, { mimeType: mime });
    recorderRef.current = recorder;
    recorder.ondataavailable = (event) => {
      if (event.data.size > 0) {
        chunksRef.current.push(event.data);
      }
    };
    recorder.onstop = () => {
      setRecording(false);
      const blob = new Blob(chunksRef.current, { type: mime });
      chunksRef.current = [];
      revokePreview(clipUrl);
      const url = URL.createObjectURL(blob);
      setClipUrl(url);
      setLive(
        "liveCamera",
        "Live preview opened. A short clip was recorded in this session and was not stored.",
      );
    };
    recorder.start();
    setRecording(true);
    window.setTimeout(() => {
      if (recorderRef.current === recorder && recorder.state === "recording") {
        recorder.stop();
      }
    }, 5000);
  }

  function stopClip() {
    if (recorderRef.current && recorderRef.current.state === "recording") {
      recorderRef.current.stop();
    }
  }

  return (
    <fieldset className="advance-consent interactive-checks" disabled={disabled}>
      <legend>Technician checks (optional)</legend>
      <p className="interactive-lead">
        After the keyboard, insert a USB stick, plug the charger, then open the camera. Skipped checks
        stay not assessable; they are never scored as zero. Keystrokes, tones, colour washes, snapshots
        and clips are not stored. The webcam light will turn on during the camera check.
      </p>

      <Subject
        title="Display colour wash"
        copy="Full-screen red, green, blue, white and black. Look for dead pixels, lines and uneven backlight, then attest."
        value={value.colourWash}
        onChange={(next) => setSubject("colourWash", next)}
        actionLabel={washOpen ? "Close colour wash" : "Start colour wash"}
        onAction={() => {
          setWashIndex(0);
          setWashOpen((open) => !open);
        }}
      />
      <Subject
        title="Keyboard"
        copy="A webview cannot see Fn combinations and some OEM hotkeys. Attest only the keys you could try. Keystrokes are not stored."
        value={value.keyboard}
        onChange={(next) => setSubject("keyboard", next)}
        actionLabel={keysOpen ? "Finish keyboard check" : "Start keyboard check"}
        onAction={() => {
          seenKeys.current.clear();
          setKeysTried(0);
          setKeysOpen((open) => !open);
        }}
        extra={keysOpen ? `${keysTried} distinct keys registered in this session. That count is not written to Report D.` : null}
      />
      <Subject
        title="USB insertion"
        copy="Insert any USB stick. CYVRA asks Windows whether a removable volume appeared. This listing does not write to the stick and does not award the four physical-port points. Attest those ports separately."
        value="skip"
        onChange={() => undefined}
        hideChoices
        actionLabel={usbOpen ? "Finish USB check" : "Start USB insertion check"}
        onAction={() => {
          if (usbOpen) {
            setUsbOpen(false);
          } else {
            void openUsbCheck();
          }
        }}
        extra={
          value.liveUsb
            ? value.liveUsb
            : "Open this check, then insert a stick. Windows must list a letter before CYVRA can say it sensed the port."
        }
      />
      <Subject
        title="Charger and charging"
        copy="Plug the charger. CYVRA reads Win32_Battery only — no powercfg file. Charging is telemetry, not a CG-1.0 scoring item. Status 2 means AC is present, not that the pack is charging."
        value="skip"
        onChange={() => undefined}
        hideChoices
        actionLabel={chargerOpen ? "Finish charger check" : "Start charger check"}
        onAction={() => {
          if (chargerOpen) {
            setChargerOpen(false);
          } else {
            void openChargerCheck();
          }
        }}
        extra={value.livePower || "Open this check, then plug or unplug the charger."}
      />
      <Subject
        title="Camera live capture"
        copy="Opens the webcam now. Take a still or a five-second clip. The image stays in this window and is discarded when you close the check. Microphone audio is not recorded. Attest presence separately."
        value="skip"
        onChange={() => undefined}
        hideChoices
        actionLabel={cameraOpen ? "Finish camera check" : "Open camera"}
        onAction={() => {
          if (cameraOpen) {
            closeCameraCheck();
          } else {
            void openCameraCheck();
          }
        }}
        extra={value.liveCamera || "The webcam light will turn on. That is expected."}
      />
      <Subject
        title="Trackpad"
        copy="Move, click and try a two-finger gesture on the canvas, then attest movement, clicks and gestures."
        value={value.trackpad}
        onChange={(next) => setSubject("trackpad", next)}
        actionLabel={padOpen ? "Finish trackpad check" : "Open trackpad canvas"}
        onAction={() => setPadOpen((open) => !open)}
        extra={padMoved ? "Movement was seen on the canvas. Report D records only your attestation." : null}
      />
      <Subject
        title="Speakers"
        copy="Play a short left then right tone in memory. Attest only if you heard both channels. Nothing is recorded."
        value={value.speakers}
        onChange={(next) => setSubject("speakers", next)}
        extra={
          <span className="interactive-tone-row">
            <button type="button" className="button button-secondary" disabled={disabled} onClick={() => playTone(-1, "left")}>
              Play left
            </button>
            <button type="button" className="button button-secondary" disabled={disabled} onClick={() => playTone(1, "right")}>
              Play right
            </button>
          </span>
        }
      />
      <Subject
        title="Camera and microphone present"
        copy="Confirm the enumerated camera and microphone are physically there after you have tried the live preview. Two points are awarded only when you attest Pass."
        value={value.capture}
        onChange={(next) => setSubject("capture", next)}
      />

      <div className="interactive-subject">
        <strong>Physically verified ports</strong>
        <small>
          Plug a technician test device into plastic connectors you want to confirm. A live USB
          volume listing helps you see that Windows mounted a stick; it does not replace this
          attestation and it is not a write to the stick or to an assessed drive.
        </small>
        <div className="interactive-choices" role="radiogroup" aria-label="Physically verified ports">
          <Choice
            checked={value.physicalPorts === "skip"}
            label="Not attempted"
            onSelect={() => setSubject("physicalPorts", "skip")}
          />
          <Choice
            checked={value.physicalPorts === "all_passed"}
            label="All attempted ports passed"
            onSelect={() => setSubject("physicalPorts", "all_passed")}
          />
          <Choice
            checked={value.physicalPorts === "partial"}
            label="Some passed"
            onSelect={() => setSubject("physicalPorts", "partial")}
          />
          <Choice
            checked={value.physicalPorts === "any_failed"}
            label="A port failed"
            onSelect={() => setSubject("physicalPorts", "any_failed")}
          />
        </div>
      </div>

      {washOpen ? (
        <div className="colour-wash" style={{ background: WASH_COLOURS[washIndex].value }}>
          <p>
            {WASH_COLOURS[washIndex].name} wash {washIndex + 1} of {WASH_COLOURS.length}. Look at the
            panel, then next colour or close.
          </p>
          <div>
            <button
              type="button"
              className="button button-secondary"
              onClick={() => setWashIndex((index) => (index + 1) % WASH_COLOURS.length)}
            >
              Next colour
            </button>
            <button type="button" className="button" onClick={() => setWashOpen(false)}>
              Close wash
            </button>
          </div>
        </div>
      ) : null}

      {keysOpen ? (
        <div className="interactive-overlay" role="dialog" aria-label="Keyboard check">
          <p>
            Press keys now. Fn combinations and some OEM hotkeys will not register. Close when you are
            ready to attest.
          </p>
          <p>{keysTried} distinct keys registered. The list is discarded when you close this check.</p>
          <button type="button" className="button" onClick={() => setKeysOpen(false)}>
            Close keyboard check
          </button>
        </div>
      ) : null}

      {usbOpen ? (
        <div className="interactive-overlay" role="dialog" aria-label="USB insertion check">
          <p>Insert any USB stick now. CYVRA lists removable volumes Windows reports. Nothing is written to the stick.</p>
          {usbError ? <p>{usbError}</p> : null}
          <p>
            {usbNew.length > 0
              ? `New since this check opened: ${describeVolumes(usbNew)}.`
              : "Waiting for a new removable letter. If a stick is already in, Windows may already list it below."}
          </p>
          <p>
            {usbVolumes.length > 0
              ? `Currently listed: ${describeVolumes(usbVolumes)}.`
              : "No removable volume listed yet."}
          </p>
          <button type="button" className="button" onClick={() => setUsbOpen(false)}>
            Close USB check
          </button>
        </div>
      ) : null}

      {chargerOpen ? (
        <div className="interactive-overlay" role="dialog" aria-label="Charger check">
          <p>Plug the charger. Windows BatteryStatus is the only source. Charging is not a grading point.</p>
          {powerError ? <p>{powerError}</p> : null}
          <p>
            {power.charging
              ? "Charging — Windows says the pack is taking charge."
              : power.onMains
                ? "On mains — AC is present. That is not the same as charging."
                : power.present
                  ? "Not charging — plug the adapter and wait a few seconds."
                  : power.detail}
          </p>
          <p>
            {power.statusLabel}
            {power.chargePercent === null ? "" : ` · ${power.chargePercent}%`}
            {power.statusCode === null ? "" : ` · BatteryStatus ${power.statusCode}`}
          </p>
          <button type="button" className="button" onClick={() => setChargerOpen(false)}>
            Close charger check
          </button>
        </div>
      ) : null}

      {cameraOpen ? (
        <div className="interactive-overlay interactive-camera" role="dialog" aria-label="Camera check">
          <p>
            Live camera preview. The webcam light is on. Take a still or a five-second clip. The image
            is discarded when you close this check and is never written to Report D.
          </p>
          {cameraError ? <p>{cameraError}</p> : null}
          <video ref={videoRef} className="camera-preview" autoPlay playsInline muted />
          <div className="interactive-tone-row">
            <button type="button" className="button button-secondary" onClick={takeSnapshot}>
              Take photo
            </button>
            {recording ? (
              <button type="button" className="button button-secondary" onClick={stopClip}>
                Stop clip
              </button>
            ) : (
              <button type="button" className="button button-secondary" onClick={startClip}>
                Record 5s clip
              </button>
            )}
            <button type="button" className="button" onClick={closeCameraCheck}>
              Close camera
            </button>
          </div>
          {snapshotUrl ? (
            <img className="camera-still" src={snapshotUrl} alt="Snapshot taken in this session. Not stored on Report D." />
          ) : null}
          {clipUrl ? <video className="camera-preview" src={clipUrl} controls muted /> : null}
        </div>
      ) : null}

      {padOpen ? (
        <div className="interactive-overlay" role="dialog" aria-label="Trackpad check">
          <p>Move and click on the canvas. Gestures are attested by you, not measured by CYVRA.</p>
          <canvas
            ref={canvasRef}
            width={480}
            height={220}
            className="trackpad-canvas"
            onPointerDown={(event) => {
              drawing.current = true;
              event.currentTarget.setPointerCapture(event.pointerId);
              onPadPointer(event);
            }}
            onPointerMove={onPadPointer}
            onPointerUp={() => {
              drawing.current = false;
            }}
          />
          <button type="button" className="button" onClick={() => setPadOpen(false)}>
            Close trackpad check
          </button>
        </div>
      ) : null}
    </fieldset>
  );
}

function Subject({
  title,
  copy,
  value,
  onChange,
  actionLabel,
  onAction,
  extra,
  hideChoices,
}: {
  title: string;
  copy: string;
  value: AttestationValue;
  onChange: (next: AttestationValue) => void;
  actionLabel?: string;
  onAction?: () => void;
  extra?: ReactNode;
  hideChoices?: boolean;
}) {
  return (
    <div className="interactive-subject">
      <strong>{title}</strong>
      <small>{copy}</small>
      {onAction && actionLabel ? (
        <button type="button" className="button button-secondary" onClick={onAction}>
          {actionLabel}
        </button>
      ) : null}
      {extra ? <small>{extra}</small> : null}
      {hideChoices ? null : (
        <div className="interactive-choices" role="radiogroup" aria-label={title}>
          <Choice checked={value === "skip"} label="Not attempted" onSelect={() => onChange("skip")} />
          <Choice checked={value === "pass"} label="Pass" onSelect={() => onChange("pass")} />
          <Choice checked={value === "fail"} label="Fail" onSelect={() => onChange("fail")} />
        </div>
      )}
    </div>
  );
}

function Choice({
  checked,
  label,
  onSelect,
}: {
  checked: boolean;
  label: string;
  onSelect: () => void;
}) {
  return (
    <label>
      <input type="radio" checked={checked} onChange={onSelect} />
      {label}
    </label>
  );
}
