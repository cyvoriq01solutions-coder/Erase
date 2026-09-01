import { useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";

const licenceText = `CYVRA Erase is licensed for one authorised Windows device after CYVRA approves the account and issues a licence. Signing in on the website is not a licence.

This version assesses the PC only. It must not erase, overwrite, encrypt, move or destroy files. It must not collect passwords, recovery keys, email bodies or private file contents. It must not bypass Windows, BitLocker, BIOS, UEFI or corporate controls.

Until Authenticode signing is complete, this installer is an unsigned engineering build. Customer download remains on https://www.cyvra.co.in/download after the signed package is stored. CYVRA emails the activation key from auth@cyvra.co.in when an administrator issues a licence. Store the key; CYVORIQ does not keep the full key after issuance.`;

type SetupStep = "welcome" | "licence" | "ready";

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
        <span className="eyebrow">CYVRA ERASE · WINDOWS SETUP</span>
        {step === "welcome" && (
          <>
            <h1 id="setup-title">Welcome to CYVRA Erase</h1>
            <p>
              This application prepares a local assessment of this Windows PC. It records hardware identity
              and where documents appear to live. It does not erase files and it does not open private
              contents.
            </p>
            <ol className="setup-steps" aria-label="First-run steps">
              <li>Review what this version does and does not do.</li>
              <li>Accept the licence and privacy terms.</li>
              <li>Enter the activation key emailed from auth@cyvra.co.in.</li>
              <li>Choose drives, run verification, then generate the report.</li>
            </ol>
            <p className="setup-note">
              Binding contacts api.cyvra.co.in from this application. One licence is valid for one authorised
              Windows PC.
            </p>
            <button className="button button-primary" type="button" onClick={() => setStep("licence")}>
              Continue
            </button>
          </>
        )}

        {step === "licence" && (
          <>
            <h1 id="setup-title">Licence and privacy</h1>
            <pre className="setup-licence">{licenceText}</pre>
            <label className="setup-accept">
              <input
                type="checkbox"
                checked={accepted}
                onChange={(event) => setAccepted(event.target.checked)}
              />
              I have read and accept these terms. I understand this version does not erase files.
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
                Continue
              </button>
            </div>
          </>
        )}

        {step === "ready" && (
          <>
            <h1 id="setup-title">Activate this Windows PC</h1>
            <p>
              Paste the key from the email subject “Your CYVRA Erase activation key”. The first successful
              activation binds the licence to this PC. The same key cannot be used on a second PC.
            </p>
            <label className="setup-key-field" htmlFor="activation-key">
              Activation key
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
    </main>
  );
}
