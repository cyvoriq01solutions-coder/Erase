import type { ReactNode } from "react";
import { NavLink } from "react-router";

type InfoPageProps = { title: string };

type Feature = {
  label: string;
  title: string;
  body: string;
};

function PageHero({ eyebrow, title, lead, children }: { eyebrow: string; title: string; lead: string; children?: ReactNode }) {
  return (
    <section className="info-hero">
      <div className="info-hero-copy">
        <span className="eyebrow">{eyebrow}</span>
        <h1>{title}</h1>
        <p>{lead}</p>
        {children}
      </div>
      <div className="info-hero-signal" aria-hidden="true">
        <span>CYVRA ERASE</span>
        <strong>VERIFY</strong>
        <b>BEFORE HANDOVER</b>
      </div>
    </section>
  );
}

function FeatureGrid({ items }: { items: Feature[] }) {
  return (
    <div className="info-card-grid">
      {items.map((item) => (
        <article className="info-card" key={item.title}>
          <span className="info-card-label">{item.label}</span>
          <h3>{item.title}</h3>
          <p>{item.body}</p>
        </article>
      ))}
    </div>
  );
}

function PageCTA({ title, body, primaryLabel = "Download CYVRA Erase", primaryTo = "/download" }: { title: string; body: string; primaryLabel?: string; primaryTo?: string }) {
  return (
    <section className="info-page-cta">
      <div>
        <span className="eyebrow eyebrow-light">SECURE LIFECYCLE. TRUSTED FUTURE.</span>
        <h2>{title}</h2>
        <p>{body}</p>
      </div>
      <div className="info-cta-actions">
        <NavLink className="button button-orange" to={primaryTo}>{primaryLabel}</NavLink>
        <NavLink className="button button-light" to="/">Return to Home</NavLink>
      </div>
    </section>
  );
}

function WhyCyvraPage() {
  const items: Feature[] = [
    { label: "01", title: "Verification before assumption", body: "Deleting files, resetting a device or completing a format does not itself create evidence that sensitive information is no longer exposed. CYVRA starts by establishing the device and its data-risk state." },
    { label: "02", title: "Privacy-conscious assessment", body: "The verification model is designed to minimise unnecessary collection of personal content. The goal is to understand exposure and sanitization requirements without turning the verification process into another privacy risk." },
    { label: "03", title: "Evidence at device level", body: "Identity, assessment state, timestamps, verification results and relevant processing evidence can be tied to the specific device instead of relying on a verbal assurance or a generic green tick." },
    { label: "04", title: "Built for the handover moment", body: "CYVRA is designed around buyback, exchange, resale, refurbishment and enterprise retirement—exactly when a device is about to move outside its previous owner's control." },
  ];

  return (
    <>
      <PageHero eyebrow="WHY CYVRA" title="Because ‘I formatted it’ should not be your final security answer." lead="CYVRA Erase is a verification-first device retirement platform for people and organisations that need confidence before a laptop, desktop or supported storage device changes hands.">
        <div className="info-hero-actions">
          <NavLink className="button button-orange" to="/download">Start with CYVRA</NavLink>
          <NavLink className="button button-secondary" to="/how-it-works">See How It Works</NavLink>
        </div>
      </PageHero>

      <section className="info-section">
        <div className="info-section-heading">
          <span className="eyebrow">THE DIFFERENCE</span>
          <h2>Move from assumption to evidence.</h2>
          <p>A device can look empty and still deserve a proper retirement check. CYVRA makes the final handover step deliberate, traceable and easier to explain.</p>
        </div>
        <FeatureGrid items={items} />
      </section>

      <section className="info-split-section">
        <div>
          <span className="eyebrow">CURRENT RELEASE</span>
          <h2>Verification first. Non-destructive by design.</h2>
        </div>
        <div className="info-rich-copy">
          <p>The current Windows release focuses on assessment, personal-data exposure mapping, evidence, independent verification and reporting.</p>
          <p>Controlled media sanitization is a separate gated capability on the product roadmap. We will not describe a destructive workflow as available until it has been implemented, validated and released.</p>
          <div className="info-note">This separation protects users and keeps product claims aligned with the software you can actually run today.</div>
        </div>
      </section>

      <PageCTA title="Know before you let go." body="Make verification part of the device handover—not an afterthought." />
    </>
  );
}

function HowItWorksPage() {
  const stages = [
    ["01", "Identify", "Establish the device, operating system and storage context so the evidence belongs to the correct asset."],
    ["02", "Discover", "Map relevant residual-data exposure indicators while avoiding unnecessary collection of the underlying personal content."],
    ["03", "Assess", "Turn the detected state into a clear risk view: what was checked, what was found and what still requires attention."],
    ["04", "Verify", "Evaluate the available evidence rather than trusting a format, reset or unverified success message."],
    ["05", "Report", "Create a device-level result that can be retained by the owner, IT team or retirement workflow."],
  ] as const;

  return (
    <>
      <PageHero eyebrow="HOW CYVRA WORKS" title="A controlled path from device handover to evidence." lead="CYVRA Erase turns a vague question—‘is my data really gone?’—into a repeatable verification workflow designed for buyback, exchange, resale and enterprise device retirement.">
        <div className="release-chip">Current release · Windows Assessment &amp; Verification · Non-Destructive</div>
      </PageHero>

      <section className="info-section info-section-soft">
        <div className="info-section-heading info-section-heading-centered">
          <span className="eyebrow">THE WORKFLOW</span>
          <h2>Five steps. One clear outcome.</h2>
        </div>
        <div className="process-rail">
          {stages.map(([number, stage, description]) => (
            <article className="process-step" key={stage}>
              <span>{number}</span>
              <h3>{stage}</h3>
              <p>{description}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="info-split-section">
        <div>
          <span className="eyebrow">WHAT HAPPENS TODAY</span>
          <h2>Assessment without destructive action.</h2>
        </div>
        <div className="info-rich-copy">
          <p>The current CYVRA Windows package is designed to inspect and document the device state without automatically wiping a disk. That allows us to establish the intelligence, evidence and verification layer first.</p>
          <ul className="info-check-list">
            <li>Device and storage identification</li>
            <li>Residual-data exposure assessment</li>
            <li>Privacy-conscious data mapping</li>
            <li>Structured evidence and timestamps</li>
            <li>Verification and reporting</li>
          </ul>
        </div>
      </section>

      <section className="info-section">
        <div className="info-section-heading">
          <span className="eyebrow">SANITIZATION ROADMAP</span>
          <h2>When erasure is enabled, it must be controlled and media-aware.</h2>
          <p>HDDs, SSDs, NVMe media and encrypted storage do not all behave the same way. CYVRA's sanitization architecture is being designed around appropriate methods, validation and evidence—not one generic wipe command for every device.</p>
        </div>
        <div className="standards-panel">
          <strong>NIST SP 800-88 Rev.2</strong>
          <span>Current NIST guidance for media sanitization programmes, techniques, controls and validation.</span>
        </div>
      </section>

      <PageCTA title="Verify first. Sanitize deliberately. Prove the outcome." body="Start with the current verification release and build a safer device-retirement habit." />
    </>
  );
}

function DpdpPage() {
  const items: Feature[] = [
    { label: "CONTROL", title: "Reasonable safeguards", body: "Device retirement should be treated as a security process with controlled identities, access, evidence and technical safeguards—not as an informal handover." },
    { label: "MINIMISE", title: "Data minimisation", body: "CYVRA is designed to collect only the information required for device processing, verification and reporting, avoiding unnecessary personal-content collection where technically possible." },
    { label: "EVIDENCE", title: "Accountability", body: "Device-level records, timestamps and verification results can help organisations demonstrate that a defined retirement process was followed." },
    { label: "ERASURE", title: "Erasure readiness", body: "The platform establishes the assessment and evidence foundation required for controlled sanitization workflows when the relevant capability is enabled and appropriate." },
  ];

  return (
    <>
      <PageHero eyebrow="DPDP READINESS" title="Turn device-retirement controls into defensible evidence." lead="India's DPDP framework raises the importance of appropriate technical and organisational measures, reasonable security safeguards and responsible handling of personal data. CYVRA is designed to support that discipline at the device-retirement stage.">
        <div className="info-hero-actions">
          <NavLink className="button button-orange" to="/enterprise">Enterprise Use</NavLink>
          <NavLink className="button button-secondary" to="/resources">View Resources</NavLink>
        </div>
      </PageHero>

      <section className="info-section info-section-soft">
        <div className="info-section-heading">
          <span className="eyebrow">WHAT CYVRA SUPPORTS</span>
          <h2>Technology controls for a better retirement process.</h2>
          <p>CYVRA does not replace an organisation's legal, governance or privacy programme. It gives the device-retirement workflow stronger technical structure and better evidence.</p>
        </div>
        <FeatureGrid items={items} />
      </section>

      <section className="info-split-section">
        <div>
          <span className="eyebrow">THE LEGAL CONTEXT</span>
          <h2>DPDP compliance is an organisational responsibility—not a software badge.</h2>
        </div>
        <div className="info-rich-copy">
          <p>The Digital Personal Data Protection Act, 2023 and the Digital Personal Data Protection Rules, 2025 form India's current personal-data protection framework, with implementation being brought into force in phases.</p>
          <p>For organisations processing personal data, device retirement can become part of the broader question of technical safeguards, retention, erasure and accountability.</p>
          <div className="info-note">CYVRA is a technology and evidence platform. Use of CYVRA does not by itself constitute legal compliance, legal advice, a government certification or a DPDP compliance certificate.</div>
        </div>
      </section>

      <section className="info-section">
        <div className="info-section-heading">
          <span className="eyebrow">PRACTICAL MODEL</span>
          <h2>What an enterprise should be able to answer.</h2>
        </div>
        <div className="question-grid">
          <div>Which device left our control?</div>
          <div>What storage media was present?</div>
          <div>What process was performed?</div>
          <div>Who or what authorised it?</div>
          <div>What evidence was captured?</div>
          <div>Can we verify the outcome later?</div>
        </div>
      </section>

      <PageCTA title="Build evidence into device retirement." body="Use CYVRA as the technical verification layer inside a broader privacy and governance programme." primaryLabel="Explore Enterprise" primaryTo="/enterprise" />
    </>
  );
}

function IndividualsPage() {
  const items: Feature[] = [
    { label: "BUYBACK", title: "Before an exchange", body: "Check your device before it leaves your possession for a retailer, exchange partner, doorstep pickup or trade-in programme." },
    { label: "RESALE", title: "Before you sell", body: "A clean desktop and an empty recycle bin are not the same as verified evidence. CYVRA helps you understand the device's residual-data risk first." },
    { label: "HANDOVER", title: "Before you give it away", body: "Donation, family handover, repair, refurbishment and reuse all move a device into someone else's control. Make data verification the final step." },
    { label: "PROOF", title: "After the check", body: "Keep a device-level verification result instead of relying on memory or assuming that a reset completed exactly as expected." },
  ];

  return (
    <>
      <PageHero eyebrow="FOR INDIVIDUALS" title="Trade in the device—not your private life." lead="You may have deleted files, signed out of accounts, reset Windows or formatted the drive. CYVRA helps you check the device before buyback, exchange, resale or handover so you are not relying on assumption alone.">
        <div className="info-hero-actions">
          <NavLink className="button button-orange" to="/download">Verify My Device</NavLink>
          <NavLink className="button button-secondary" to="/how-it-works">How It Works</NavLink>
        </div>
      </PageHero>

      <section className="info-section">
        <div className="info-section-heading">
          <span className="eyebrow">THE SIMPLE QUESTION</span>
          <h2>Is anything still there?</h2>
          <p>Years of documents, photos, work files, browser data, financial information and application traces can make an old device far more sensitive than it looks. The safest final step is verification.</p>
        </div>
        <FeatureGrid items={items} />
      </section>

      <section className="info-split-section info-split-dark">
        <div>
          <span className="eyebrow eyebrow-light">A BETTER HANDOVER HABIT</span>
          <h2>Reset. Check. Verify. Then hand it over.</h2>
        </div>
        <div className="info-rich-copy">
          <ol className="number-list">
            <li><span>1</span><div><strong>Back up what you need.</strong><p>Move important files and confirm you can access the backup.</p></div></li>
            <li><span>2</span><div><strong>Sign out and prepare the device.</strong><p>Follow the operating system and service-provider steps appropriate for the device.</p></div></li>
            <li><span>3</span><div><strong>Run CYVRA verification.</strong><p>Assess the device and review the evidence before the handover.</p></div></li>
            <li><span>4</span><div><strong>Keep the result.</strong><p>Retain the verification record for your own confidence and traceability.</p></div></li>
          </ol>
        </div>
      </section>

      <section className="info-section info-section-soft">
        <div className="info-section-heading info-section-heading-centered">
          <span className="eyebrow">CURRENT RELEASE</span>
          <h2>No automatic disk wiping.</h2>
          <p>Today's CYVRA Windows release is assessment and verification focused. If sanitization is required, the software will only describe capabilities that are actually released and validated.</p>
        </div>
      </section>

      <PageCTA title="Before the device changes hands, verify the data state." body="One final check can turn uncertainty into evidence." primaryLabel="Start Device Verification" />
    </>
  );
}

function EnterprisePage() {
  const items: Feature[] = [
    { label: "FLEET", title: "Device identity", body: "Tie processing to the specific asset, storage context and relevant identifiers before it moves through retirement, lease return, buyback or ITAD." },
    { label: "CONTROL", title: "Repeatable workflow", body: "Replace ad-hoc technician steps with a defined sequence for assessment, evidence, verification and reporting." },
    { label: "AUDIT", title: "Device-level evidence", body: "Create traceable records that can support internal governance, client assurance, audit preparation and exception management." },
    { label: "SCALE", title: "Enterprise integration", body: "The architecture is being built for centralised entitlements, device binding, controlled access and future high-volume processing workflows." },
  ];

  return (
    <>
      <PageHero eyebrow="ENTERPRISE & OEM" title="Every retired endpoint is a data-security event." lead="Refresh cycles, employee exits, lease returns, OEM exchange programmes, refurbishment and ITAD all move devices beyond their original control. CYVRA gives that transition a structured verification and evidence layer.">
        <div className="info-hero-actions">
          <NavLink className="button button-orange" to="/contact">Talk to CYVORIQ</NavLink>
          <NavLink className="button button-secondary" to="/dpdp-readiness">DPDP Readiness</NavLink>
        </div>
      </PageHero>

      <section className="info-section">
        <div className="info-section-heading">
          <span className="eyebrow">ENTERPRISE CONTROL</span>
          <h2>From a device pile to a governed process.</h2>
          <p>The challenge is not only wiping media. It is knowing which asset was processed, under whose authority, using what method, with what result, and whether that result can be demonstrated later.</p>
        </div>
        <FeatureGrid items={items} />
      </section>

      <section className="info-split-section info-split-dark">
        <div>
          <span className="eyebrow eyebrow-light">USE CASES</span>
          <h2>Designed for the points where devices leave trusted control.</h2>
        </div>
        <div className="enterprise-use-grid">
          <span>Corporate refresh cycles</span>
          <span>Employee exits</span>
          <span>Lease returns</span>
          <span>OEM buyback programmes</span>
          <span>Trade-in &amp; exchange</span>
          <span>ITAD processing</span>
          <span>Refurbishment</span>
          <span>Secondary-market resale</span>
        </div>
      </section>

      <section className="info-section info-section-soft">
        <div className="info-section-heading">
          <span className="eyebrow">THE EVIDENCE CHAIN</span>
          <h2>Device → authority → process → evidence → verification.</h2>
        </div>
        <div className="evidence-chain" aria-label="CYVRA enterprise evidence chain">
          <span>Device identity</span><b>→</b><span>Authorised session</span><b>→</b><span>Assessment</span><b>→</b><span>Evidence</span><b>→</b><span>Verification</span><b>→</b><span>Report</span>
        </div>
        <p className="info-disclaimer">Sanitization, certification and high-volume orchestration capabilities will be represented commercially only as they are implemented, validated and released.</p>
      </section>

      <PageCTA title="Retire devices with evidence—not assumptions." body="Talk to CYVORIQ about enterprise assessment, verification and the controlled sanitization roadmap." primaryLabel="Enterprise Enquiry" primaryTo="/contact" />
    </>
  );
}

function ResourcesPage() {
  const videos = [
    { tag: "INSTALLATION", title: "Install CYVRA Erase on Windows", body: "A short step-by-step walkthrough covering download access, installation, first launch and the beginning of device verification." },
    { tag: "GETTING STARTED", title: "Your first CYVRA verification", body: "Learn what CYVRA checks, what the assessment means and how to review the device-level result before handover." },
    { tag: "WHY CYVRA", title: "Why verification matters before buyback", body: "A customer-focused explanation of residual-data risk, formatting versus evidence, and why the final verification step matters." },
    { tag: "ENTERPRISE", title: "CYVRA for device retirement programmes", body: "An overview for IT, OEM and asset-lifecycle teams covering device identity, workflow control, evidence and reporting." },
  ];

  return (
    <>
      <PageHero eyebrow="CYVRA RESOURCES" title="Learn it. Run it. Understand the evidence." lead="Practical videos and guidance for installing CYVRA Erase, running your first verification and understanding how the platform supports safer device handover.">
        <div className="info-hero-actions">
          <NavLink className="button button-orange" to="/download">Download CYVRA Erase</NavLink>
          <NavLink className="button button-secondary" to="/how-it-works">Product Workflow</NavLink>
        </div>
      </PageHero>

      <section className="info-section info-section-soft">
        <div className="info-section-heading">
          <span className="eyebrow">VIDEO LIBRARY</span>
          <h2>Short guidance at the moment you need it.</h2>
          <p>The resource page is ready for your final videos. Until the approved video files or links are supplied, the player cards remain intentionally inactive rather than pointing to placeholder external content.</p>
        </div>
        <div className="video-grid">
          {videos.map((video, index) => (
            <article className="video-card" key={video.title}>
              <div className="video-placeholder" aria-label={`${video.title} video coming soon`}>
                <span className="video-play">▶</span>
                <small>VIDEO {String(index + 1).padStart(2, "0")} · COMING SOON</small>
              </div>
              <div className="video-copy">
                <span className="info-card-label">{video.tag}</span>
                <h3>{video.title}</h3>
                <p>{video.body}</p>
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="info-section">
        <div className="info-section-heading">
          <span className="eyebrow">QUICK GUIDES</span>
          <h2>What you should know before you begin.</h2>
        </div>
        <div className="resource-guide-grid">
          <article><strong>01</strong><h3>Before installation</h3><p>Use an authorised CYVRA account and entitlement. Download only from the official CYVRA flow.</p></article>
          <article><strong>02</strong><h3>Before verification</h3><p>Save important work, close applications where practical and ensure the device remains powered during assessment.</p></article>
          <article><strong>03</strong><h3>Understand the result</h3><p>CYVRA reports what was assessed and verified. The current release is non-destructive and should not be treated as proof of sanitization.</p></article>
          <article><strong>04</strong><h3>Need enterprise help?</h3><p>For fleet workflows, OEM programmes or structured ITAD use, engage CYVORIQ before defining the operating process.</p></article>
        </div>
      </section>

      <section className="info-split-section">
        <div>
          <span className="eyebrow">STANDARDS &amp; PRIVACY</span>
          <h2>Read the framework behind the product.</h2>
        </div>
        <div className="resource-links">
          <NavLink to="/dpdp-readiness"><strong>DPDP Readiness</strong><span>How CYVRA supports technical controls, evidence and data minimisation.</span></NavLink>
          <NavLink to="/how-it-works"><strong>How CYVRA Works</strong><span>The verification lifecycle and controlled sanitization roadmap.</span></NavLink>
          <NavLink to="/individuals"><strong>Individual Guide</strong><span>What to do before buyback, exchange, resale or handover.</span></NavLink>
          <NavLink to="/enterprise"><strong>Enterprise Guide</strong><span>Device retirement, fleet evidence and governed workflows.</span></NavLink>
        </div>
      </section>

      <PageCTA title="Need the software, not just the guide?" body="Use the protected download flow to access the current CYVRA Windows release." />
    </>
  );
}

function DefaultInfoPage({ title }: { title: string }) {
  return (
    <section className="content-page">
      <span className="eyebrow">CYVORIQ ERASE</span>
      <h1>{title}</h1>
      <p>This route is established as part of the frozen frontend architecture. Product content will be added in the appropriate build package.</p>
    </section>
  );
}

export default function InfoPage({ title }: InfoPageProps) {
  switch (title) {
    case "Why CYVRA": return <WhyCyvraPage />;
    case "How It Works": return <HowItWorksPage />;
    case "DPDP Readiness": return <DpdpPage />;
    case "For Individuals": return <IndividualsPage />;
    case "Enterprise & OEM": return <EnterprisePage />;
    case "Resources": return <ResourcesPage />;
    default: return <DefaultInfoPage title={title} />;
  }
}
