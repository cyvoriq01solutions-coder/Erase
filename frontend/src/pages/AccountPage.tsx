import { FormEvent, useState } from "react";
import { NavLink, useNavigate, useSearchParams } from "react-router";
import {
  ApiError,
  registerCustomer,
  requestLoginCode,
  verifyLoginCode,
  type AccountType,
} from "../services/authApi";

type AuthMode = "register" | "signin";
type AuthStep = "details" | "verify";

export default function AccountPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const initialMode: AuthMode = searchParams.get("mode") === "signin" ? "signin" : "register";

  const [mode, setMode] = useState<AuthMode>(initialMode);
  const [step, setStep] = useState<AuthStep>("details");
  const [accountType, setAccountType] = useState<AccountType>("individual");
  const [displayName, setDisplayName] = useState("");
  const [organizationName, setOrganizationName] = useState("");
  const [email, setEmail] = useState("");
  const [challengeId, setChallengeId] = useState("");
  const [code, setCode] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  function switchMode(nextMode: AuthMode) {
    setMode(nextMode);
    setStep("details");
    setChallengeId("");
    setCode("");
    setError("");
    setNotice("");
    const next = new URLSearchParams(searchParams);
    next.set("mode", nextMode);
    next.delete("returnTo");
    setSearchParams(next, { replace: true });
  }

  async function handleDetailsSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    setNotice("");

    try {
      const normalizedEmail = email.trim().toLowerCase();
      const response =
        mode === "register"
          ? await registerCustomer({
              email: normalizedEmail,
              displayName: displayName.trim() || undefined,
              accountType,
              organizationName:
                accountType === "enterprise" ? organizationName.trim() || undefined : undefined,
            })
          : await requestLoginCode(normalizedEmail);

      setEmail(normalizedEmail);
      setChallengeId(response.challengeId);
      setStep("verify");
      setNotice("Verification code sent. It is valid for 10 minutes.");
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "We could not start email verification. Please try again.",
      );
    } finally {
      setBusy(false);
    }
  }

  async function handleVerifySubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");

    try {
      await verifyLoginCode(challengeId, code.trim());
      navigate("/download", { replace: true });
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "The verification code could not be confirmed. Please try again.",
      );
    } finally {
      setBusy(false);
    }
  }

  async function handleResend() {
    setBusy(true);
    setError("");
    setNotice("");

    try {
      const response = await requestLoginCode(email);
      setChallengeId(response.challengeId);
      setCode("");
      setNotice("A new verification code has been sent. Older unused codes are no longer valid.");
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "We could not resend the verification code. Please try again.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="auth-page">
      <section className="auth-shell">
        <div className="auth-intro">
          <span className="eyebrow">SECURE CYVRA ACCESS</span>
          <h1>{mode === "register" ? "Create your CYVRA account" : "Sign in to CYVRA"}</h1>
          <p>
            Your email is your CYVORIQ identity. We verify it with a one-time code and create a secure browser session.
            After verification you return to the CYVRA download page, where product access remains separately protected
            by payment, approval, licence and entitlement checks.
          </p>
          <div className="auth-trust-list">
            <span>Email OTP verification</span>
            <span>No password required</span>
            <span>Server-verified session</span>
            <span>Protected download entitlement</span>
          </div>
        </div>

        <div className="auth-card">
          <div className="auth-mode-tabs" role="tablist" aria-label="Account access">
            <button
              type="button"
              className={mode === "register" ? "active" : ""}
              onClick={() => switchMode("register")}
              disabled={busy}
            >
              Create Account
            </button>
            <button
              type="button"
              className={mode === "signin" ? "active" : ""}
              onClick={() => switchMode("signin")}
              disabled={busy}
            >
              Sign In
            </button>
          </div>

          {step === "details" ? (
            <form className="auth-form" onSubmit={handleDetailsSubmit}>
              <div>
                <span className="status-pill">STEP 1 OF 2</span>
                <h2>{mode === "register" ? "Account details" : "Enter your email"}</h2>
                <p>
                  {mode === "register"
                    ? "We only ask for the identity information needed to create and secure your account."
                    : "We will send a six-digit verification code to your registered email address."}
                </p>
              </div>

              {mode === "register" && (
                <>
                  <fieldset className="auth-account-type">
                    <legend>Account type</legend>
                    <label>
                      <input
                        type="radio"
                        name="accountType"
                        value="individual"
                        checked={accountType === "individual"}
                        onChange={() => setAccountType("individual")}
                        disabled={busy}
                      />
                      <span>
                        <strong>Individual</strong>
                        <small>For your own device</small>
                      </span>
                    </label>
                    <label>
                      <input
                        type="radio"
                        name="accountType"
                        value="enterprise"
                        checked={accountType === "enterprise"}
                        onChange={() => setAccountType("enterprise")}
                        disabled={busy}
                      />
                      <span>
                        <strong>Enterprise</strong>
                        <small>For an organisation or business</small>
                      </span>
                    </label>
                  </fieldset>

                  <label className="auth-field">
                    <span>Full name</span>
                    <input
                      type="text"
                      value={displayName}
                      onChange={(event) => setDisplayName(event.target.value)}
                      autoComplete="name"
                      maxLength={160}
                      placeholder="Your name"
                      disabled={busy}
                    />
                  </label>

                  {accountType === "enterprise" && (
                    <label className="auth-field">
                      <span>Organisation name</span>
                      <input
                        type="text"
                        value={organizationName}
                        onChange={(event) => setOrganizationName(event.target.value)}
                        maxLength={160}
                        placeholder="Company or organisation"
                        required
                        disabled={busy}
                      />
                    </label>
                  )}
                </>
              )}

              <label className="auth-field">
                <span>Email address</span>
                <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  autoComplete="email"
                  placeholder="name@example.com"
                  required
                  disabled={busy}
                />
              </label>

              {error && <div className="auth-message auth-message-error">{error}</div>}

              <button className="button button-orange button-primary-cta" type="submit" disabled={busy}>
                {busy ? "Please wait..." : "Send Verification Code"}
              </button>

              <small className="auth-privacy-note">
                Email verification confirms account ownership only. CYVRA Erase download and activation remain subject to
                separate commercial approval and licence controls.
              </small>
            </form>
          ) : (
            <form className="auth-form" onSubmit={handleVerifySubmit}>
              <div>
                <span className="status-pill">STEP 2 OF 2</span>
                <h2>Verify your email</h2>
                <p>
                  Enter the six-digit code sent to <strong>{email}</strong>.
                </p>
              </div>

              <label className="auth-field auth-code-field">
                <span>Verification code</span>
                <input
                  type="text"
                  inputMode="numeric"
                  pattern="[0-9]{6}"
                  maxLength={6}
                  value={code}
                  onChange={(event) => setCode(event.target.value.replace(/\D/g, "").slice(0, 6))}
                  autoComplete="one-time-code"
                  placeholder="000000"
                  required
                  disabled={busy}
                  autoFocus
                />
              </label>

              {notice && <div className="auth-message auth-message-success">{notice}</div>}
              {error && <div className="auth-message auth-message-error">{error}</div>}

              <button
                className="button button-orange button-primary-cta"
                type="submit"
                disabled={busy || code.length !== 6}
              >
                {busy ? "Verifying..." : "Verify & Return to Download"}
              </button>

              <div className="auth-secondary-actions">
                <button type="button" onClick={handleResend} disabled={busy}>Resend code</button>
                <button
                  type="button"
                  onClick={() => {
                    setStep("details");
                    setChallengeId("");
                    setCode("");
                    setError("");
                    setNotice("");
                  }}
                  disabled={busy}
                >
                  Change email
                </button>
              </div>
            </form>
          )}

          <NavLink className="auth-back-link" to="/download">Back to CYVRA Erase download</NavLink>
        </div>
      </section>
    </main>
  );
}
