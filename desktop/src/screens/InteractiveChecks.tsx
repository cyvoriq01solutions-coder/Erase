import { useEffect, useRef, useState, type PointerEvent, type ReactNode } from "react";
import type { AdvanceInteractive, AttestationValue, PortAttestationValue } from "../types/shell";

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

  function setSubject(field: keyof AdvanceInteractive, next: AttestationValue | PortAttestationValue) {
    onChange({ ...value, [field]: next });
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

  return (
    <fieldset className="advance-consent interactive-checks" disabled={disabled}>
      <legend>Technician checks (optional)</legend>
      <p className="interactive-lead">
        Leave every subject as not attempted unless you inspect it. Skipped checks stay not assessable;
        they are never scored as zero. Keystrokes, tones and colour washes are not stored. Live camera
        and microphone capture is not part of this scan.
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
        copy="Confirm the enumerated camera and microphone are physically there. No frame is captured and no audio is recorded."
        value={value.capture}
        onChange={(next) => setSubject("capture", next)}
      />

      <div className="interactive-subject">
        <strong>Physically verified ports</strong>
        <small>
          Plug a technician test device into plastic connectors you want to confirm. This is an
          insertion attestation, not a write to the stick and not a write to an assessed drive.
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
}: {
  title: string;
  copy: string;
  value: AttestationValue;
  onChange: (next: AttestationValue) => void;
  actionLabel?: string;
  onAction?: () => void;
  extra?: ReactNode;
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
      {extra}
      <div className="interactive-choices" role="radiogroup" aria-label={title}>
        <Choice checked={value === "skip"} label="Not attempted" onSelect={() => onChange("skip")} />
        <Choice checked={value === "pass"} label="Pass" onSelect={() => onChange("pass")} />
        <Choice checked={value === "fail"} label="Fail" onSelect={() => onChange("fail")} />
      </div>
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
