import { NavLink } from "react-router";

const stages = [
  ["01", "IDENTIFY", "Establish the device and storage identity before anything changes hands."],
  ["02", "DISCOVER", "Look for residual-data risk using a privacy-conscious, non-destructive assessment model."],
  ["03", "ASSESS", "Turn technical findings into a clear view of what still needs attention before handover."],
  ["04", "VERIFY", "Evaluate the evidence instead of relying on a reset, format or unverified success message."],
  ["05", "REPORT", "Create a device-level result that can support the owner, IT team, auditor or retirement workflow."],
] as const;

const residualExamples = [
  "Personal documents and downloaded files",
  "Photos, videos and private media",
  "Browser profiles, saved sessions and application data",
  "Work files, customer records and intellectual property",
  "Financial, identity and account-related information",
  "Local user profiles, caches and residual storage artefacts",
] as const;

export default function HomePage() {
  return (
    <main>
      <section className="hero hero-commercial">
        <div className="hero-copy">
          <span className="eyebrow">CYVRA ERASE · VERIFICATION-FIRST DATA PROTECTION</span>
          <h1>Before you sell the device, know your data is really gone.</h1>
          <p>
            You deleted your files. You reset Windows. You formatted the drive. But before a laptop, desktop or
            storage device enters buy-back, exchange, resale, refurbishment or retirement, one question still matters:
            can sensitive data still be exposed? CYVRA Erase is built to replace assumption with evidence.
          </p>
          <div className="hero-actions">
            <NavLink className="button button-orange button-primary-cta" to="/download">
              Start Device Verification
            </NavLink>
            <NavLink className="button button-secondary" to="/how-it-works">
              See How CYVRA Works
            </NavLink>
          </div>
          <div className="release-note">
            <strong>Current public release:</strong> Windows Assessment &amp; Verification · Non-Destructive
          </div>
        </div>

        <div className="assurance-card assurance-card-premium" aria-label="CYVRA verification lifecycle">
          <span className="status-pill">VERIFY BEFORE YOU LET GO</span>
          <h2>Formatting is not proof.</h2>
          <p>
            A device should not be treated as clean simply because files are no longer visible. CYVRA creates a
            controlled path from device identity to residual-data assessment, evidence, verification and reporting.
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
        <span>Verification before handover</span>
        <span>Privacy-conscious assessment</span>
        <span>Device-level evidence</span>
        <span>DPDP-aware design</span>
      </section>

      <section className="section-block problem-section">
        <div className="section-intro">
          <span className="eyebrow">THE BUY-BACK QUESTION</span>
          <h2>Your device can have a second life. Your data should not travel with it.</h2>
          <p>
            Devices entering exchange and resale programmes can carry years of personal and corporate information.
            Deleting files, reinstalling an operating system or performing a quick format should not automatically be
            treated as evidence that the underlying data risk has been resolved.
          </p>
        </div>

        <div className="audience-grid">
          <article className="audience-card">
            <span className="card-kicker">WHAT MAY STILL MATTER</span>
            <h3>Residual data can be more valuable than the hardware.</h3>
            <p>
              A retired device may still hold information connected to identity, finance, work, communications and
              private life. CYVRA is designed to determine the device state without turning verification into another
              unnecessary collection of personal content.
            </p>
            <ul>
              {residualExamples.map((item) => <li key={item}>{item}</li>)}
            </ul>
          </article>

          <article className="audience-card audience-card-dark">
            <span className="card-kicker">THE CYVRA PRINCIPLE</span>
            <h3>Do not assume a device is clean. Establish evidence.</h3>
            <p>
              CYVRA treats device retirement as a security event. The goal is to know which device is being processed,
              understand the relevant data-exposure state, record the findings and make the next action deliberate.
            </p>
            <NavLink to="/how-it-works" className="text-link text-link-light">Explore the verification model →</NavLink>
          </article>
        </div>
      </section>

      <section className="section-block workflow-section">
        <div className="section-intro section-intro-centered">
          <span className="eyebrow">HOW CYVRA WORKS</span>
          <h2>Identify. Discover. Assess. Verify. Report.</h2>
          <p>
            CYVRA separates device assessment from destructive action. That makes verification safer, easier to audit
            and suitable as the foundation for controlled sanitization when that capability is authorised and supported.
          </p>
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

      <section className="section-block problem-section">
        <div className="section-intro">
          <span className="eyebrow">VERIFICATION FIRST · SANITIZATION WHEN REQUIRED</span>
          <h2>The goal is not to make data invisible. The goal is to make the risk defensible.</h2>
          <p>
            CYVRA's architecture is being built around two distinct controls: first establish the device and evidence;
            then, where the relevant CYVRA release supports it and the user has explicitly authorised it, apply an
            appropriate sanitization method and validate the result. A generic format command is not treated as a
            universal sanitization method for every storage technology.
          </p>
        </div>

        <div className="audience-grid">
          <article className="audience-card">
            <span className="card-kicker">CURRENT RELEASE</span>
            <h3>Assessment and verification without destructive wiping.</h3>
            <p>
              The current public Windows release is intentionally non-destructive. It focuses on device identity,
              residual-data assessment, evidence and reporting so that the verification foundation can be proven before
              destructive sanitization is enabled.
            </p>
          </article>

          <article className="audience-card">
            <span className="card-kicker">SANITIZATION ROADMAP</span>
            <h3>Media-aware erasure with validation—not a one-method-fits-all wipe.</h3>
            <p>
              Controlled sanitization is being engineered as a gated capability for supported media and workflows,
              with method selection, authorisation, evidence and post-process validation considered part of the same
              trust chain.
            </p>
          </article>
        </div>
      </section>

      <section className="section-block dpdp-section">
        <div className="dpdp-panel">
          <div>
            <span className="eyebrow eyebrow-light">DESIGNED FOR INDIA'S DATA-PROTECTION ERA</span>
            <h2>Turn device retirement from a verbal assurance into a controlled record.</h2>
            <p>
              India's Digital Personal Data Protection Act, 2023 and the Digital Personal Data Protection Rules, 2025
              create a stronger accountability framework for digital personal data. The notified framework is being
              brought into force in phases. CYVRA is designed to support data-minimised, controlled and evidence-backed
              device-retirement practices as organisations prepare for and operate under those obligations.
            </p>
            <NavLink className="button button-light" to="/dpdp-readiness">See the DPDP readiness model</NavLink>
          </div>
          <div className="dpdp-points">
            <div><strong>Reasonable safeguards</strong><span>Controlled access, authenticated workflows and traceable evidence support stronger device handling.</span></div>
            <div><strong>Erasure lifecycle</strong><span>Verification provides the foundation for authorised sanitization and evidence of the outcome.</span></div>
            <div><strong>Accountability</strong><span>Device-level records are more useful than an undocumented promise that a device was wiped.</span></div>
            <div><strong>Data minimisation</strong><span>Collect only what is needed for the service; avoid unnecessary collection of personal-content data.</span></div>
          </div>
        </div>
        <p className="compliance-note">
          Compliance notice: CYVRA is a technology and evidence platform. Use of CYVRA does not by itself constitute
          government certification, legal advice or a guarantee of compliance with the DPDP Act or any other law.
        </p>
      </section>

      <section className="section-block workflow-section">
        <div className="section-intro section-intro-centered">
          <span className="eyebrow">STANDARDS-LED ENGINEERING</span>
          <h2>Modern storage needs more than an old wiping checklist.</h2>
          <p>
            NIST SP 800-88 Rev.2, published in September 2025, is the current NIST guidance for media sanitization and
            supersedes Rev.1. CYVRA's sanitization roadmap is being designed with reference to current media-sanitization
            principles, including appropriate technique selection, validation and evidence rather than relying on a
            single overwrite pattern for every device type.
          </p>
        </div>

        <div className="audience-grid">
          <article className="audience-card">
            <span className="card-kicker">LAPTOPS &amp; DESKTOPS</span>
            <h3>Protect the data before the endpoint leaves your control.</h3>
            <p>For personal upgrades, employee devices, refresh cycles, resale, donation and buy-back programmes.</p>
          </article>
          <article className="audience-card">
            <span className="card-kicker">HDD · SSD · NVMe</span>
            <h3>The media type matters.</h3>
            <p>Sanitization decisions should consider the actual storage technology and the assurance level required.</p>
          </article>
        </div>
      </section>

      <section className="section-block problem-section">
        <div className="section-intro">
          <span className="eyebrow">BUILT FOR INDIVIDUALS AND ENTERPRISE</span>
          <h2>Two users. One data-protection problem.</h2>
        </div>

        <div className="audience-grid">
          <article className="audience-card">
            <span className="card-kicker">FOR INDIVIDUALS</span>
            <h3>Trade in the device—not your private life.</h3>
            <p>
              Selling a laptop, exchanging it for a new one, donating it or passing it to another person? CYVRA gives
              you a verification-first path before the device leaves your possession.
            </p>
            <NavLink to="/individuals" className="text-link">Protect my device before buy-back →</NavLink>
          </article>

          <article className="audience-card audience-card-dark">
            <span className="card-kicker">FOR ENTERPRISE &amp; OEM</span>
            <h3>Every retired endpoint is a data-security event.</h3>
            <p>
              Employee exits, refresh cycles, lease returns, OEM buy-backs, ITAD, refurbishment and resale move
              devices beyond their original control. CYVRA is being built to bring identity, verification, evidence,
              controlled sanitization and reporting into one governed workflow.
            </p>
            <NavLink to="/enterprise" className="text-link text-link-light">Explore enterprise use →</NavLink>
          </article>
        </div>
      </section>

      <section className="section-block license-section">
        <div className="license-copy">
          <span className="eyebrow">CYVORIQ SOLUTIONS PVT. LTD.</span>
          <h2>Digital trust for the complete device-retirement lifecycle.</h2>
          <p>
            CYVORIQ Solutions is a purpose-built Digital Trust and Secure Data Lifecycle Management company focused on
            secure data sanitization, evidence-led device retirement and IT Asset Disposition. CYVRA provides the
            technology layer behind that philosophy: know the device, protect the data and prove the outcome.
          </p>
        </div>
        <div className="license-flow" aria-label="CYVRA trust flow">
          <span>Verified account</span>
          <b>→</b>
          <span>Authorised entitlement</span>
          <b>→</b>
          <span>Device evidence</span>
          <b>→</b>
          <span>Verified outcome</span>
        </div>
      </section>

      <section className="final-cta">
        <span className="eyebrow">SECURE LIFECYCLE. TRUSTED FUTURE.</span>
        <h2>Before the device changes hands, make verification the final step.</h2>
        <div className="hero-actions final-actions">
          <NavLink className="button button-orange button-primary-cta" to="/download">Start Device Verification</NavLink>
          <NavLink className="button button-secondary" to="/enterprise">Talk to CYVORIQ Enterprise</NavLink>
        </div>
      </section>
    </main>
  );
}
