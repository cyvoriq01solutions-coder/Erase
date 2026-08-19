import { NavLink } from "react-router";

const stages = [
  ["01", "ASSESS", "Identify the device, operating system, storage architecture and relevant system state."],
  ["02", "PDEM", "Map potential personal-data exposure without unnecessarily collecting personal file contents."],
  ["03", "EVIDENCE", "Create structured, traceable technical evidence with timestamps, confidence and coverage."],
  ["04", "VERIFY", "Evaluate the available evidence independently instead of trusting an unverified success message."],
  ["05", "REPORT", "Produce a clear device-level result for the owner, IT team, auditor or retirement workflow."],
] as const;

export default function HomePage() {
  return (
    <main>
      <section className="hero hero-commercial">
        <div className="hero-copy">
          <span className="eyebrow">Secure Device Retirement · Evidence-Backed Data Protection</span>
          <h1>Before your device changes hands, make sure your data doesn't.</h1>
          <p>
            CYVRA Erase helps individuals and organisations assess retired devices, identify residual-data risk,
            build verifiable evidence and document the outcome before a laptop or storage device is traded in,
            returned, resold, refurbished or retired.
          </p>
          <div className="hero-actions">
            <NavLink className="button button-orange button-primary-cta" to="/download">
              Download CYVRA Erase
            </NavLink>
            <NavLink className="button button-secondary" to="/how-it-works">
              See How It Works
            </NavLink>
          </div>
          <div className="release-note">
            <strong>Current release:</strong> Windows Assessment &amp; Verification · Non-Destructive
          </div>
        </div>

        <div className="assurance-card assurance-card-premium" aria-label="CYVRA verification lifecycle">
          <span className="status-pill">CYVRA ERASE · VERIFICATION FIRST</span>
          <h2>Know before you let go.</h2>
          <p>
            Device retirement should be a controlled security event—not a guess. CYVRA creates a repeatable path
            from assessment to evidence, verification and report.
          </p>
          <div className="stage-grid compact-stage-grid">
            {stages.map(([number, stage]) => (
              <div className="stage" key={stage}>
                <span>{number}</span>
                <strong>{stage}</strong>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="trust-strip" aria-label="CYVRA product principles">
        <span>Privacy-conscious assessment</span>
        <span>Device-level evidence</span>
        <span>Independent verification model</span>
        <span>Designed to support DPDP readiness</span>
      </section>

      <section className="section-block problem-section">
        <div className="section-intro">
          <span className="eyebrow">THE RETIREMENT RISK</span>
          <h2>Your device may be retired. Your data isn't.</h2>
          <p>
            Deleting files, signing out of applications or preparing a device for resale does not itself create
            verifiable evidence that sensitive information is no longer exposed. CYVRA helps make the handover
            deliberate, traceable and evidence-backed.
          </p>
        </div>

        <div className="audience-grid">
          <article className="audience-card">
            <span className="card-kicker">FOR INDIVIDUALS</span>
            <h3>Trade in the device—not your private life.</h3>
            <p>
              Upgrading through a buy-back programme? Selling a laptop? Donating or handing it to someone else?
              Your device may still contain years of personal, financial and work-related information.
            </p>
            <NavLink to="/individuals" className="text-link">Protect my device before buy-back →</NavLink>
          </article>

          <article className="audience-card audience-card-dark">
            <span className="card-kicker">FOR ENTERPRISE &amp; OEM</span>
            <h3>Every retired endpoint is a data-security event.</h3>
            <p>
              Employee exits, refresh cycles, lease returns, OEM buy-backs, ITAD, refurbishment and resale all move
              devices beyond their original control. CYVRA turns retirement into a structured workflow with identity,
              evidence, verification and reporting.
            </p>
            <NavLink to="/enterprise" className="text-link text-link-light">Explore enterprise use →</NavLink>
          </article>
        </div>
      </section>

      <section className="section-block workflow-section">
        <div className="section-intro section-intro-centered">
          <span className="eyebrow">HOW CYVRA WORKS</span>
          <h2>Know what is there. Understand the risk. Verify the outcome.</h2>
        </div>

        <div className="workflow-grid">
          {stages.map(([number, stage, description]) => (
            <article className="workflow-card" key={stage}>
              <span>{number}</span>
              <h3>{stage}</h3>
              <p>{description}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="section-block dpdp-section">
        <div className="dpdp-panel">
          <div>
            <span className="eyebrow eyebrow-light">BUILT FOR THE DPDP ERA</span>
            <h2>Turn device-retirement controls into evidence.</h2>
            <p>
              CYVRA Erase is designed to support secure data-lifecycle practices and organisational DPDP readiness
              through authenticated access, data minimisation, device-level evidence, controlled workflows,
              verification and auditable reporting.
            </p>
            <NavLink className="button button-light" to="/dpdp-readiness">See the DPDP readiness model</NavLink>
          </div>
          <div className="dpdp-points">
            <div><strong>Security safeguards</strong><span>Controlled identities, sessions, device workflows and evidence.</span></div>
            <div><strong>Erasure readiness</strong><span>Assessment and verification foundation for the controlled sanitization roadmap.</span></div>
            <div><strong>Accountability</strong><span>Traceable device-level records rather than verbal assurance.</span></div>
            <div><strong>Data minimisation</strong><span>Collect only what the service requires; avoid unnecessary personal-content collection.</span></div>
          </div>
        </div>
        <p className="compliance-note">
          DPDP readiness notice: CYVRA Erase is a technology and evidence platform. Use of CYVRA does not by itself
          constitute legal compliance or government certification.
        </p>
      </section>

      <section className="section-block license-section">
        <div className="license-copy">
          <span className="eyebrow">COMMERCIAL SECURITY</span>
          <h2>One licence. One device. One activation binding.</h2>
          <p>
            The public download is gated behind a verified customer identity and an authorised entitlement. On first
            successful activation, the licence is bound to the authorised device. Attempts to reuse the same licence on
            another device are rejected unless CYVORIQ performs an authorised, auditable reset or reissue.
          </p>
        </div>
        <div className="license-flow" aria-label="CYVRA licence flow">
          <span>Verified account</span>
          <b>→</b>
          <span>Approved entitlement</span>
          <b>→</b>
          <span>Protected download</span>
          <b>→</b>
          <span>Device binding</span>
        </div>
      </section>

      <section className="final-cta">
        <span className="eyebrow">SECURE LIFECYCLE. TRUSTED FUTURE.</span>
        <h2>Ready to retire a device with evidence—not assumptions?</h2>
        <div className="hero-actions final-actions">
          <NavLink className="button button-orange button-primary-cta" to="/download">Download CYVRA Erase</NavLink>
          <NavLink className="button button-secondary" to="/enterprise">Talk to CYVORIQ Enterprise</NavLink>
        </div>
      </section>
    </main>
  );
}
