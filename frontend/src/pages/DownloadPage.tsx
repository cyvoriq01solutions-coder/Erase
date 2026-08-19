import { NavLink } from "react-router";

const gates = [
  ["01", "Create or sign in to your CYVORIQ account", "Your email becomes the verified account identity; no separate username is required."],
  ["02", "Verify email ownership", "A one-time verification code confirms that the customer controls the email address."],
  ["03", "Accept privacy and licence terms", "Commercial access is recorded only after the required notices and licence terms are acknowledged."],
  ["04", "Receive an approved entitlement", "The server checks account, order/payment approval, licence state and download entitlement."],
  ["05", "Download the authorised package", "The Windows package is never exposed as an unrestricted public file."],
  ["06", "Bind the licence to one device", "First activation securely binds the licence to the authorised device; reuse on another device is rejected."],
] as const;

export default function DownloadPage() {
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
          <span className="status-pill">DOWNLOAD GATE · ACTIVE DESIGN</span>
          <h2>Create Account &amp; Continue</h2>
          <p>
            Authentication and entitlement controls are being connected to this screen. Until that release gate passes,
            no raw executable will be exposed publicly.
          </p>
          <button className="button button-orange button-primary-cta" type="button" disabled>
            Create Account &amp; Continue
          </button>
          <NavLink className="download-signin" to="/download">Already have an account? Sign In</NavLink>
          <small>One licence · One authorised device · Server-verified activation</small>
        </div>
      </section>

      <section className="section-block download-gates-section">
        <div className="section-intro">
          <span className="eyebrow">BEFORE THE PACKAGE IS RELEASED</span>
          <h2>Six server-verified gates protect every download.</h2>
          <p>
            The orange Download button does not point to an EXE. It starts a controlled commercial workflow designed
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
