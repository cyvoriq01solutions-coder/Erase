import { useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import logoUrl from "../assets/cyvoriq-logo.webp";

const licenceText = `SOFTWARE LICENSE TERMS
CYVRA Erase — Assessment Edition
CYVORIQ Solutions Pvt. Ltd.

IMPORTANT — READ CAREFULLY. These Software License Terms ("Terms")
are an agreement between CYVORIQ Solutions Pvt. Ltd. and the
entity or person who installs or runs the software ("you").
By checking "I accept" or by installing, copying or using the
software, you agree to these Terms. If you do not agree, do not
install or use the software.

1. PARTIES AND SOFTWARE
The software is licensed, not sold. CYVORIQ retains all rights
not expressly granted. The software includes the Windows
installer, the local application, and related documentation.

2. PRE-RELEASE / UNSIGNED ENGINEERING SOFTWARE
Until Authenticode code-signing is complete, this installer is
an unsigned engineering build. It may differ from the signed
customer package. CYVORIQ may change or withhold a generally
available release. Customer download of a signed package, when
issued, remains at https://www.cyvra.co.in/download. Use of an
unsigned build is at your own risk, including operating-system
SmartScreen or antivirus warnings.

3. GRANT OF LICENCE
Subject to these Terms and to CYVORIQ issuing a licence for your
account, CYVORIQ grants you a limited, non-exclusive,
non-transferable, revocable licence to install and run one copy
of the software on one authorised Windows device.

Website sign-in, OTP, or account approval is not a licence.
A licence exists only after an administrator issues an activation
key and the key is successfully bound to that device.

4. ACTIVATION
CYVRA emails the activation key from auth@cyvra.co.in when a
licence is issued. Store the key. CYVORIQ does not retain the
full key after issuance. The first successful activation binds
the licence to that Windows PC. The same key cannot be used on a
second PC. Binding contacts api.cyvra.co.in from this application.

5. WHAT THIS VERSION MAY DO
This version performs a local pre-sanitization assessment only:
hardware identity and firmware-reported inventory that this
assessment actually obtained, and document-location metadata
(names, types and sizes). It does not open private file contents.

6. RESTRICTIONS — YOU AND THE SOFTWARE MUST NOT
The software must not, and you must not use it to:
  (a) erase, overwrite, encrypt, format, move or destroy files
      or storage media;
  (b) collect passwords, recovery keys, operating-system activation
      keys, email bodies, or private file contents;
  (c) bypass Windows, BitLocker, BIOS, UEFI, Secure Boot, TPM
      ownership, or corporate management controls;
  (d) reverse engineer except to the extent applicable law
      expressly permits despite this limitation;
  (e) rent, sublicense, or run the software as a public bureau
      service except under a separate written CYVORIQ agreement;
  (f) represent the output as a Certificate of Sanitization,
      NIST SP 800-88 Purge proof, DPDP compliance certificate,
      or a cloud-authenticated legal instrument.

7. DATA
Collection is local unless you separately use a CYVORIQ online
service (activation). Hardware serials and document-location
counts on Report A are an operator copy on this PC. Unknown
hardware values stay unknown. CYVORIQ will not invent a battery
percentage, port count, or serial number.

8. OWNERSHIP
The software, marks CYVRA and CYVORIQ, and all copies, are
owned by CYVORIQ Solutions Pvt. Ltd. or its licensors.

9. DISCLAIMER OF WARRANTIES
THE SOFTWARE IS LICENSED "AS IS", "WITH ALL FAULTS", AND
"AS AVAILABLE". TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE
LAW, CYVORIQ DISCLAIMS ALL WARRANTIES AND CONDITIONS, EXPRESS
OR IMPLIED, INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR
PURPOSE, TITLE AND NON-INFRINGEMENT. YOU BEAR THE RISK OF USING
PRE-RELEASE AND UNSIGNED SOFTWARE. HARDWARE ROWS THAT WERE NOT
COLLECTED ARE NOT A STATEMENT THAT THE COMPONENT IS ABSENT.

10. LIMITATION OF LIABILITY
TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, CYVORIQ AND
ITS SUPPLIERS SHALL NOT BE LIABLE FOR INDIRECT, INCIDENTAL,
SPECIAL, CONSEQUENTIAL, PUNITIVE OR LOST-PROFIT DAMAGES, OR FOR
LOSS OF DATA, BUSINESS OR CONFIDENTIAL INFORMATION, EVEN IF
ADVISED OF THE POSSIBILITY. CYVORIQ'S AGGREGATE LIABILITY ARISING
OUT OF THESE TERMS OR THE SOFTWARE SHALL NOT EXCEED THE AMOUNT
YOU PAID TO CYVORIQ FOR THE LICENCE THAT WAS BOUND TO THE DEVICE
GIVING RISE TO THE CLAIM, OR ONE THOUSAND INDIAN RUPEES
(INR 1,000) IF NO FEE WAS PAID. SOME JURISDICTIONS DO NOT ALLOW
CERTAIN LIMITATIONS; IN THAT CASE THE LIMITATION APPLIES TO THE
MAXIMUM EXTENT PERMITTED.

11. TERMINATION
These Terms terminate if you breach them or if CYVORIQ revokes
the licence. On termination you must stop using the software and
destroy copies in your possession, except copies you are required
by law to retain.

12. GOVERNING LAW
These Terms are governed by the laws of India, without regard to
conflict-of-law rules. Courts at the registered office of
CYVORIQ Solutions Pvt. Ltd. have exclusive jurisdiction, subject
to any non-excludable consumer rights.

13. ENTIRE AGREEMENT
These Terms, together with any written licence issuance from
CYVORIQ, are the entire agreement for the software. They supersede
prior oral or written statements about this engineering build.

14. CONTACT
CYVORIQ Solutions Pvt. Ltd.
Product: CYVRA Erase
Activation mail: auth@cyvra.co.in
Customer site: https://www.cyvra.co.in

Uninstall uses the normal Windows Programs and Features entry for
CYVRA Erase.`;

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
                locations. The next step shows the binding Software License Terms.
              </p>
              <ol className="setup-steps" aria-label="What happens next">
                <li>Review and accept the Software License Terms.</li>
                <li>Activate this PC using your CYVRA activation key.</li>
                <li>Choose the assessment you want to run.</li>
              </ol>
              <p className="setup-note">
                One authorised Windows device. Signing in on the website is not a licence. This
                version does not erase files. This build is unsigned until Authenticode.
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
              <h1 id="setup-title">Software License Terms</h1>
              <p>
                Please scroll and review the terms before using CYVRA Erase. Welcome is a summary
                only. These Terms are binding.
              </p>
              <pre className="setup-licence">{licenceText}</pre>
              <label className="setup-accept">
                <input
                  type="checkbox"
                  checked={accepted}
                  onChange={(event) => setAccepted(event.target.checked)}
                />
                I accept these Software License Terms. I understand that this version does not erase
                files.
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
