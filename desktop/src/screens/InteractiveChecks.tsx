import { useEffect, useRef, useState, type PointerEvent, type ReactNode } from "react";
import type { AdvanceInteractive, AttestationValue } from "../types/shell";

const WASH_COLOURS = [
  { name: "Red", value: "#c62828" },
  { name: "Green", value: "#2e7d32" },
  { name: "Blue", value: "#1565c0" },
  { name: "White", value: "#f5f7fb" },
  { name: "Black", value: "#11141a" },
] as const;

interface InteractiveChecksProps {
  value: AdvanceInteractive;
  disabled: boolean;
  onChange: (next: AdvanceInteractive) => void;
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

  function setLive(field: "liveCamera", text: string) {
    onChange({ ...valueRef.current, [field]: text });
  }

  function setSubject(
    field: "colourWash" | "keyboard" | "trackpad" | "speakers" | "capture",
    next: AttestationValue,
  ) {
    onChange({ ...valueRef.current, [field]: next });
  }

  function playTone(pan: number, label: string) {
    const AudioContextClass =
      window.AudioContext || (window as Window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioContextClass) {
      window.alert(`This window cannot play a ${label} tone.`);
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
      <legend>Technician checks — optional</legend>
      <p className="interactive-lead">
        Use these optional checks to verify selected hardware functions. Test results are recorded,
        but personal content is not stored. USB sockets and charger state are read once during Advance scan
        and printed on Report D. This version does not run a live USB or live charger check. Skipped
        checks stay not assessable. Keystrokes, tones, colour washes, snapshots and clips are not stored.
      </p>

      <Subject
        title="Display Colour Test"
        copy="Start the colour wash: view each test colour and check for dead pixels, lines or uneven brightness. Select Pass if the display appears normal, or Fail if you identify a visible issue."
        value={value.colourWash}
        onChange={(next) => setSubject("colourWash", next)}
        actionLabel={washOpen ? "Close Colour Test" : "Start Colour Test"}
        onAction={() => {
          setWashIndex(0);
          setWashOpen((open) => !open);
        }}
      />
      <Subject
        title="Keyboard Check"
        copy="Press the requested keys and confirm that each key responds correctly. Keystrokes are not stored. Fn combinations and some OEM hotkeys will not register. Select Pass if the tested keys respond correctly."
        value={value.keyboard}
        onChange={(next) => setSubject("keyboard", next)}
        actionLabel={keysOpen ? "Finish Keyboard Check" : "Start Keyboard Check"}
        onAction={() => {
          seenKeys.current.clear();
          setKeysTried(0);
          setKeysOpen((open) => !open);
        }}
        extra={
          keysOpen
            ? `${keysTried} distinct keys registered in this session. That count is not written to Report D.`
            : null
        }
      />
      <Subject
        title="Camera Check"
        copy="Open the camera and confirm that a live preview is available. No image or video is recorded or stored by CYVRA. Select Pass if the camera preview works correctly. Attest presence separately."
        value="skip"
        onChange={() => undefined}
        hideChoices
        actionLabel={cameraOpen ? "Finish Camera Check" : "Open Camera"}
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
        title="Trackpad Check"
        copy="Move the pointer, test clicking and try a two-finger gesture. Select Pass if movement, clicking and gestures work correctly."
        value={value.trackpad}
        onChange={(next) => setSubject("trackpad", next)}
        actionLabel={padOpen ? "Finish Trackpad Check" : "Open Trackpad Test"}
        onAction={() => setPadOpen((open) => !open)}
        extra={padMoved ? "Movement was seen on the canvas. Report D records only your attestation." : null}
      />
      <Subject
        title="Speaker Check"
        copy="Play the test tones and confirm that you can hear both audio channels. Audio is played for testing only and nothing is recorded. Select Pass if both test channels are heard clearly."
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
