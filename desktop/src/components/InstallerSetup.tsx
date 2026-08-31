import { useState } from "react";

const licenceText = `CYVRA Erase is licensed for one authorised Windows device after CYVRA approves the account and issues a licence. Website identity verification is not a licence.

V1 is assessment and verification only. It must not erase, overwrite, encrypt, move or destroy files. It must not collect passwords, recovery keys, email bodies or private file contents. It must not bypass Windows, BitLocker, BIOS, UEFI or corporate controls.

This build is an unsigned engineering installer. Customer download stays on https://www.cyvra.co.in/download after Authenticode signing and private Backblaze B2 storage. CYVRA emails the activation key when an administrator issues a licence. This build does not bind that key to this PC.`;

type SetupStep = "welcome" | "licence" | "ready";

export function InstallerSetup({ onFinished }: { onFinished: () => void }) {
  const [step, setStep] = useState<SetupStep>("welcome");
  const [accepted, setAccepted] = useState(false);

  return (
    <main className="setup-shell" aria-labelledby="setup-title">
      <div className="setup-card">
        <span className="eyebrow">CYVRA ERASE · WINDOWS SETUP</span>
        {step === "welcome" && (
          <>
            <h1 id="setup-title">Welcome to CYVRA Erase</h1>
            <p>
              This application is assessment-focused and non-destructive. Installation used a per-machine NSIS setup
              with a single elevation prompt. The program runs as a standard user.
            </p>
            <ol className="setup-steps" aria-label="Installer journey">
              <li>Welcome to CYVRA Erase</li>
              <li>Review licence and privacy terms</li>
              <li>Continue into the application</li>
            </ol>
            <p className="setup-note">
              An administrator may already have emailed your activation key. Online device binding is not enabled in this build.
            </p>
            <button className="button button-primary" type="button" onClick={() => setStep("licence")}>
              Next
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
              I have read and accept these terms
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
                Next
              </button>
            </div>
          </>
        )}

        {step === "ready" && (
          <>
            <h1 id="setup-title">Continue into CYVRA Erase</h1>
            <p>
              The desktop shell is ready. If CYVRA emailed an activation key, keep it. This build shows the field only as a preview; online device binding is not enabled yet.
            </p>
            <label className="setup-key-field" htmlFor="activation-key-preview">
              Activation key
            </label>
            <input
              id="activation-key-preview"
              type="text"
              disabled
              placeholder="Key is emailed — binding not enabled yet"
              autoComplete="off"
            />
            <div className="setup-actions">
              <button className="button button-secondary" type="button" onClick={() => setStep("licence")}>
                Back
              </button>
              <button className="button button-primary" type="button" onClick={onFinished}>
                Finish
              </button>
            </div>
          </>
        )}
      </div>
    </main>
  );
}
