import { useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import logoUrl from "../assets/cyvoriq-logo.webp";

const licenceText = `CYVRA Erase is licensed for one authorised Windows device after CYVRA approves the account and issues a licence. Signing in on the website is not a licence.

This version performs assessment and diagnostics only. It does not erase, overwrite, encrypt, move or destroy files. CYVRA Erase does not collect passwords, recovery keys, email content or private file contents. It must not bypass Windows, BitLocker, BIOS, UEFI or corporate controls.

CYVRA emails the activation key from auth@cyvra.co.in when an administrator issues a licence. Store the key; CYVORIQ does not keep the full key after issuance. One licence binds to one authorised PC.`;

type SetupStep = "welcome" | "licence" | "ready";

const STEPS: { id: SetupStep; label: string }[] = [
  { id: "welcome", label: "Welcome" },
  { id: "licence", label: "Terms" },
  { id: "ready", label: "Activate" },
];

export function InstallerSetup({
  liveActivationEnabled,
  onFinished,
}: {
  liveActivationEnabled: boolean;
  onFinished: () => void;
}) {
  const [step, setStep] = useState<SetupStep>("welcome");
  const [accepted, setAccepted] = useState(false);
  const [activationKey, setActivationKey] = useState("");
  const [activating, setActivating] = useState(false);
  const [activationError, setActivationError] = useState<string | null>(null);
  const [activationNote, setActivationNote] = useState<string | null>(null);

  async function handleActivate() {
    setActivationError(null);
    setActivationNote(null);
    if (!liveActivationEnabled || !isTauri()) {
      setActivationError("Online device binding is not available in this preview.");
      return;
    }
    setActivating(true);
    try {
      const result = await invoke<{ ok: boolean; message: string; keyPrefix?: string }>(
        "activate_license",
        { activationKey: activationKey.trim() },
      );
      if (!result.ok) {
        setActivationError(result.message);
        return;
      }
      setActivationNote(result.message);
    } catch (error) {
      setActivationError(
        error instanceof Error ? error.message : "Activation could not be completed.",
      );
    } finally {
      setActivating(false);
    }
  }

  return (
    <main className="setup-shell" aria-labelledby="setup-title">
      <div className="setup-card">
        <header className="setup-brand">
          <img src={logoUrl} alt="CYVORIQ Solutions" width="128" height="78" />
          <span>
            <strong>CYVORIQ SOLUTIONS</strong>
            <small>CYVRA Erase · Windows assessment workstation</small>
          </span>
        </header>
        <div className="setup-body">
          <ol className="setup-progress" aria-label="Setup progress">
            {STEPS.map((item) => (
              <li key={item.id} aria-current={item.id === step ? "step" : undefined}>
                {item.label}
              </li>
            ))}
          </ol>

          {step === "welcome" && (
            <>
              <h1 id="setup-title">Install CYVRA Erase on This PC</h1>
              <p>
                CYVRA Erase performs a secure, non-destructive assessment of this PC and selected data
                locations.
              </p>
              <ol className="setup-steps" aria-label="What happens next">
                <li>Review and accept the licence and privacy terms.</li>
                <li>Activate this PC using your CYVRA activation key.</li>
                <li>Choose the assessment you want to run.</li>
              </ol>
              <p className="setup-note">
                CYVRA Erase does not erase, overwrite or open your personal files in this version.
              </p>
              <div className="setup-actions">
                <button className="button button-primary" type="button" onClick={() => setStep("licence")}>
                  Continue
                </button>
              </div>
            </>
          )}

          {step === "licence" && (
            <>
              <h1 id="setup-title">Licence and Privacy</h1>
              <p>Please review the terms before using CYVRA Erase.</p>
              <pre className="setup-licence">{licenceText}</pre>
              <label className="setup-accept">
                <input
                  type="checkbox"
                  checked={accepted}
                  onChange={(event) => setAccepted(event.target.checked)}
                />
                I have read and accept the Licence and Privacy Terms. I understand that this version
                does not erase files.
              </label>
              <div className="setup-actions">
                <button className="button button-secondary" type="button" onClick={() => setStep("welcome")}>
                  Back
                </button>
                <button
                  className="button button-primary"
                  type="button"
                  disabled={!accepted}
                  onClick={() => setStep("ready")}
                >
                  Accept and Continue
                </button>
              </div>
            </>
          )}

          {step === "ready" && (
            <>
              <h1 id="setup-title">Activate This PC</h1>
              <p>
                Enter the activation key provided by CYVRA to activate this authorised PC. Your first
                successful activation securely links this licence to this PC. The same activation key
                cannot be used to activate another PC unless the licence is reset or transferred.
              </p>
              <label className="setup-key-field" htmlFor="activation-key">
                Activation Key
              </label>
              <input
                id="activation-key"
                type="text"
                disabled={!liveActivationEnabled || activating}
                value={activationKey}
                placeholder="CYVRA-XXXX-XXXX-XXXX-XXXX"
                autoComplete="off"
                spellCheck={false}
                onChange={(event) => setActivationKey(event.target.value)}
              />
              {activationError ? <p className="setup-note">{activationError}</p> : null}
              {activationNote ? <p className="setup-note">{activationNote}</p> : null}
              <div className="setup-actions">
                <button className="button button-secondary" type="button" onClick={() => setStep("licence")}>
                  Back
                </button>
                {liveActivationEnabled ? (
                  <button
                    className="button button-primary"
                    type="button"
                    disabled={activating || activationKey.trim().length < 20}
                    onClick={() => void handleActivate()}
                  >
                    {activating ? "Binding this PC…" : "Activate"}
                  </button>
                ) : null}
                <button
                  className="button button-primary"
                  type="button"
                  disabled={liveActivationEnabled && !activationNote}
                  onClick={onFinished}
                >
                  Open CYVRA Erase
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </main>
  );
}
