import { useEffect, useState } from "react";
import { NavLink } from "react-router";
import { downloadSetupPackage, getDownloadStatus, getSession, logout, type DownloadStatusResponse, type SessionUser } from "../services/authApi";

const gates = [
  ["01", "Create or sign in to your CYVORIQ account", "Your email becomes the verified account identity; no separate username is required."],
  ["02", "Verify email ownership", "A one-time verification code confirms that the customer controls the email address."],
  ["03", "Accept privacy and licence terms", "Commercial access is recorded only after the required notices and licence terms are acknowledged."],
  ["04", "Receive an approved entitlement", "The server checks account, order/payment approval, licence state and download entitlement."],
  ["05", "Download the authorised package", "The Windows package is never exposed as an unrestricted public file."],
  ["06", "Bind the licence to one device", "First activation securely binds the licence to the authorised device; reuse on another device is rejected."],
] as const;

type AccessState = "checking" | "anonymous" | "authenticated" | "unavailable";

export default function DownloadPage() {
  const [accessState, setAccessState] = useState<AccessState>("checking");
  const [user, setUser] = useState<SessionUser | null>(null);
  const [download, setDownload] = useState<DownloadStatusResponse | null>(null);
  const [signingOut, setSigningOut] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [downloadError, setDownloadError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;

    getSession()
      .then(async (session) => {
        if (!active) return;
        if (session.authenticated) {
          setUser(session.user);
          try {
            const status = await getDownloadStatus();
            if (active) setDownload(status);
          } catch {
            if (active) setDownload(null);
          }
          setAccessState("authenticated");
        } else {
          setUser(null);
          setDownload(null);
          setAccessState("anonymous");
        }
      })
      .catch(() => {
        if (!active) return;
        setUser(null);
        setAccessState("unavailable");
      });

    return () => {
      active = false;
    };
  }, []);

  async function handleDownload() {
    setDownloadError(null);
    setDownloading(true);
    try {
      await downloadSetupPackage();
    } catch (error) {
      setDownloadError(
        error instanceof Error
          ? error.message
          : "Download could not start. No public installer URL was exposed.",
      );
    } finally {
      setDownloading(false);
    }
  }

  async function handleSignOut() {
    setSigningOut(true);
    try {
      await logout();
    } finally {
      setUser(null);
      setAccessState("anonymous");
      setSigningOut(false);
    }
  }

  return (
    <main className="download-page">
      <section className="download-hero">
        <div>
          <span className="eyebrow">PROTECTED SOFTWARE ACCESS</span>
          <h1>Download CYVRA Erase</h1>
          <p>
            Secure access starts with a verified identity. CYVRA Erase is licensed per authorised device and the
            package is released only after the server confirms the customer and commercial entitlement.
          </p>
        </div>

        <div className="download-access-card">
          {accessState === "checking" && (
            <>
              <span className="status-pill">CHECKING SECURE SESSION</span>
              <h2>Checking your CYVRA account</h2>
              <p>The server is verifying whether this browser already has an authenticated customer session.</p>
            </>
          )}

          {accessState === "anonymous" && (
            <>
              <span className="status-pill">ACCOUNT VERIFICATION · ACTIVE</span>
              <h2>Create Account &amp; Continue</h2>
              <p>
                Create or sign in to your CYVORIQ account and verify your email. Product download remains protected until
                the server confirms the required commercial entitlement and licence state.
              </p>
              <NavLink className="button button-orange button-primary-cta" to="/account?mode=register">
                Create Account &amp; Continue
              </NavLink>
              <NavLink className="download-signin" to="/account?mode=signin">Already have an account? Sign In</NavLink>
              <small>One licence · One authorised device · Server-verified activation</small>
            </>
          )}

          {accessState === "authenticated" && user !== null && (
            <>
              <span className="status-pill">SIGNED IN · EMAIL VERIFIED</span>
              <h2>Your CYVRA access</h2>
              <p className="download-session-identity">
                Signed in as <strong>{user.email}</strong>
              </p>
              <div className="download-session-status" aria-label="Customer access status">
                <div><span>Identity</span><strong>Verified</strong></div>
                <div><span>Account</span><strong>Active</strong></div>
                <div>
                  <span>Commercial entitlement</span>
                  <strong>
                    {download?.accessStatus === "approved"
                      ? "Approved"
                      : download?.accessStatus === "rejected"
                        ? "Not approved"
                        : "Waiting for CYVRA approval"}
                  </strong>
                </div>
                <div>
                  <span>CYVRA Erase package</span>
                  <strong>
                    {download?.entitled && download.packageAvailable
                      ? "Ready · authorised download"
                      : download?.entitled
                        ? "Approved · installer not in private store yet"
                        : "Locked until entitlement is approved"}
                  </strong>
                </div>
              </div>
              {download?.accessStatus === "rejected" && download.rejectReason && (
                <small>Reason: {download.rejectReason}</small>
              )}
              {download?.entitled && download.packageAvailable ? (
                <button
                  className="button button-orange button-primary-cta"
                  type="button"
                  onClick={() => void handleDownload()}
                  disabled={downloading}
                >
                  {downloading ? "Preparing download..." : "Download CYVRA Erase"}
                </button>
              ) : (
                <button className="button button-orange button-primary-cta" type="button" disabled>
                  {download?.entitled
                    ? "Download Locked · Package Not Released"
                    : "Download Locked · Entitlement Required"}
                </button>
              )}
              {downloadError && <small>{downloadError}</small>}
              <button className="download-signout" type="button" onClick={handleSignOut} disabled={signingOut}>
                {signingOut ? "Signing out..." : "Sign Out"}
              </button>
              <small>
                {download?.message ||
                  "Email verification confirms identity only. Administration approval and package release remain server-controlled."}
              </small>
            </>
          )}

          {accessState === "unavailable" && (
            <>
              <span className="status-pill">SESSION CHECK UNAVAILABLE</span>
              <h2>Secure account check unavailable</h2>
              <p>
                We could not verify your account session. No protected download has been exposed. Please retry shortly or sign in again.
              </p>
              <NavLink className="button button-orange button-primary-cta" to="/account?mode=signin">
                Sign In Again
              </NavLink>
            </>
          )}
        </div>
      </section>

      <section className="section-block download-gates-section">
        <div className="section-intro">
          <span className="eyebrow">BEFORE THE PACKAGE IS RELEASED</span>
          <h2>Six server-verified gates protect every download.</h2>
          <p>
            The Download action never points to an unrestricted EXE. It starts a controlled commercial workflow designed
            to prevent anonymous downloads, entitlement bypass and licence sharing.
          </p>
        </div>
        <div className="download-gates-grid">
          {gates.map(([number, title, description]) => (
            <article key={number} className="download-gate-card">
              <span>{number}</span>
              <h3>{title}</h3>
              <p>{description}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="download-privacy-strip">
        <div>
          <strong>Privacy-conscious account creation</strong>
          <span>
            We will collect only the account and device information needed to provide, secure and evidence the service.
            Mobile number, Aadhaar and unrelated personal details are not required simply to create a CYVRA account.
          </span>
        </div>
        <div>
          <strong>Current Windows release</strong>
          <span>Assessment &amp; verification focused · Non-destructive · Public executable not yet released.</span>
        </div>
      </section>
    </main>
  );
}
