import { Notice } from "../components/Notice";
import type { BridgeState, NavigationId } from "../types/shell";

interface ShellScreenProps {
  current: NavigationId;
  bridge: BridgeState;
  onNavigate: (target: NavigationId) => void;
}

const verificationStages = [
  "Preparing verification",
  "Confirming device identity",
  "Collecting passive hardware information",
  "Assessing personal-data locations",
  "Building the Privacy Exposure Map",
  "Preparing evidence",
  "Verifying consistency",
  "Preparing results",
] as const;

function ScreenHeader({ eyebrow, title, copy }: { eyebrow: string; title: string; copy: string }) {
  return (
    <header className="screen-header">
      <span className="eyebrow">{eyebrow}</span>
      <h1 id="screen-title">{title}</h1>
      <p>{copy}</p>
    </header>
  );
}

function StatusCard({ label, value, copy }: { label: string; value: string; copy: string }) {
  return (
    <article className="status-card">
      <span className="card-label">{label}</span>
      <h2>{value}</h2>
      <p>{copy}</p>
    </article>
  );
}

function OverviewScreen({ onNavigate }: Pick<ShellScreenProps, "onNavigate">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="CUSTOMER APPLICATION FOUNDATION"
        title="Overview"
        copy="The trusted desktop frame is ready. Live customer services remain deliberately disconnected in W2.1B."
      />

      <Notice kind="information" title="Foundation state">
        This build proves the Tauri shell, typed Rust boundary and frozen navigation. It does not activate an entitlement,
        inspect a device, issue a grade or authenticate a report.
      </Notice>

      <section className="primary-panel" aria-labelledby="current-state-title">
        <div>
          <span className="card-label">CURRENT STATE</span>
          <h2 id="current-state-title">Ready for bounded integration work</h2>
          <p>
            No device has been confirmed and no verification has started. Customer data and hardware values are not
            simulated as real evidence.
          </p>
        </div>
        <button className="button button-primary" type="button" onClick={() => onNavigate("verification")}>
          Review verification scope
        </button>
      </section>

      <section className="status-grid" aria-label="Foundation capability status">
        <StatusCard label="ACTIVATION" value="Not connected" copy="Server-authoritative entitlement remains a later contract." />
        <StatusCard label="DEVICE BINDING" value="Not started" copy="No fingerprint or raw hardware identifier was created." />
        <StatusCard label="CYVRA QC" value="Grade pending" copy="No evidence exists and no grade can be issued." />
        <StatusCard label="REPORT" value="Not generated" copy="No report or authenticity claim exists." />
      </section>

      <Notice kind="warning" title="Assessment boundary">
        CYVRA Erase V1 is assessment-only. No destructive operation is exposed by this shell.
      </Notice>
    </div>
  );
}

function VerificationScreen({ bridge }: Pick<ShellScreenProps, "bridge">) {
  const nativeReady = bridge.status === "ready" && bridge.bootstrap.coreBoundary === "direct_typed_cyvra_core";

  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="PASSIVE ASSESSMENT"
        title="Verification"
        copy="One coordinated journey will eventually combine passive device evidence and privacy-exposure metadata."
      />

      <div className="two-column-grid">
        <section className="content-panel" aria-labelledby="scope-title">
          <span className="card-label">FROZEN SCOPE</span>
          <h2 id="scope-title">What the customer will review</h2>
          <ul className="check-list">
            <li>Detected device identity and architecture</li>
            <li>Explicit privacy and evidence scope</li>
            <li>Standard-user permission behavior</li>
            <li>Safe cancellation before final results</li>
          </ul>
        </section>

        <section className="content-panel" aria-labelledby="boundary-title">
          <span className="card-label">TRUSTED BOUNDARY</span>
          <h2 id="boundary-title">Typed Rust core</h2>
          <dl className="definition-list">
            <div>
              <dt>Native boundary</dt>
              <dd>{nativeReady ? "Linked" : "Available only in the Tauri runtime"}</dd>
            </div>
            <div>
              <dt>Frontend commands</dt>
              <dd>One read-only bootstrap command</dd>
            </div>
            <div>
              <dt>Collector execution</dt>
              <dd>Disabled</dd>
            </div>
          </dl>
        </section>
      </div>

      <section className="content-panel" aria-labelledby="stages-title">
        <div className="panel-heading">
          <div>
            <span className="card-label">EIGHT TRUTHFUL STAGES</span>
            <h2 id="stages-title">Verification has not started</h2>
          </div>
          <span className="status-pill status-neutral">Not started</span>
        </div>
        <ol className="stage-list">
          {verificationStages.map((stage, index) => (
            <li key={stage}>
              <span className="stage-number" aria-hidden="true">
                {index + 1}
              </span>
              <span>{stage}</span>
              <strong>Pending</strong>
            </li>
          ))}
        </ol>
      </section>

      <div className="action-row">
        <button className="button button-primary" type="button" disabled>
          Run Device Verification
        </button>
        <p>Unavailable until consent, orchestration and typed progress contracts are implemented.</p>
      </div>
    </div>
  );
}

function ResultsScreen({ onNavigate }: Pick<ShellScreenProps, "onNavigate">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="EVIDENCE-LED OUTCOMES"
        title="Results"
        copy="CYVRA QC condition and CYVRA Erase privacy exposure remain separate inside one verification record."
      />

      <Notice kind="warning" title="No results available">
        No verification has completed. The shell does not create sample results that could be mistaken for device
        evidence.
      </Notice>

      <section className="domain-grid" aria-label="Result domains">
        <article className="domain-card">
          <span className="domain-name">CYVRA QC</span>
          <span className="status-pill status-neutral">Grade pending</span>
          <h2>Device condition</h2>
          <p>An A–E grade requires the approved evidence threshold, deterministic rules and server-authoritative issuance.</p>
          <ul className="detail-list">
            <li>Evidence coverage: unavailable</li>
            <li>Dimensions assessed: none</li>
            <li>Applied caps: none</li>
          </ul>
        </article>

        <article className="domain-card">
          <span className="domain-name">CYVRA ERASE</span>
          <span className="status-pill status-neutral">Not assessed</span>
          <h2>Privacy exposure</h2>
          <p>The future Privacy Exposure Map will use approved metadata without presenting private customer content.</p>
          <ul className="detail-list">
            <li>Data-location coverage: unavailable</li>
            <li>Private content collected: no</li>
            <li>Data erased: no</li>
          </ul>
        </article>
      </section>

      <div className="action-row">
        <button className="button button-secondary" type="button" onClick={() => onNavigate("verification")}>
          Review verification scope
        </button>
      </div>
    </div>
  );
}

function ReportScreen({ onNavigate }: Pick<ShellScreenProps, "onNavigate">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="COMBINED EVIDENCE PACKAGE"
        title="Report"
        copy="The customer report will preserve specification, condition, privacy exposure and erasure status as separate sections."
      />

      <section className="report-empty" aria-labelledby="report-state-title">
        <span className="empty-mark" aria-hidden="true">
          P
        </span>
        <span className="card-label">REPORT STATE</span>
        <h2 id="report-state-title">No report has been generated</h2>
        <p>
          Report generation and authenticity verification are not connected in this foundation. The application will
          never label a report authenticated before verification succeeds.
        </p>
        <div className="report-state-grid">
          <div>
            <span>Evidence manifest</span>
            <strong>Not created</strong>
          </div>
          <div>
            <span>Authenticity</span>
            <strong>Not requested</strong>
          </div>
          <div>
            <span>Erasure status</span>
            <strong>No data was erased</strong>
          </div>
        </div>
        <button className="button button-secondary" type="button" onClick={() => onNavigate("results")}>
          Review result states
        </button>
      </section>
    </div>
  );
}

function HelpScreen({ bridge }: Pick<ShellScreenProps, "bridge">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="PRIVACY-SAFE SUPPORT"
        title="Help and recovery"
        copy="Recovery guidance explains what happened, what remains safe and which bounded action is available."
      />

      {bridge.status === "error" ? (
        <Notice kind="error" title="Trusted boundary unavailable">
          {bridge.safeMessage} No customer operation was started. Restart the internal shell before continuing.
        </Notice>
      ) : bridge.status === "loading" ? (
        <Notice kind="information" title="Checking trusted boundary">
          No customer operation can start while the internal application boundary is being checked.
        </Notice>
      ) : (
        <Notice kind="success" title="Foundation loaded safely">
          The shell exposes no live customer operation and retains no customer data.
        </Notice>
      )}

      <section className="support-grid" aria-label="Support topics">
        <article className="support-card">
          <span className="support-number">01</span>
          <h2>Activation and binding</h2>
          <p>Future recovery will use server-authoritative, audited support decisions.</p>
        </article>
        <article className="support-card">
          <span className="support-number">02</span>
          <h2>Evidence limitations</h2>
          <p>Permission denied, unsupported and collection error states remain distinct.</p>
        </article>
        <article className="support-card">
          <span className="support-number">03</span>
          <h2>Report authenticity</h2>
          <p>Local results remain visibly unverified until authentication succeeds.</p>
        </article>
        <article className="support-card">
          <span className="support-number">04</span>
          <h2>Privacy-safe diagnostics</h2>
          <p>Raw identifiers, activation keys and personal content never belong in support logs.</p>
        </article>
      </section>
    </div>
  );
}

export function ShellScreen({ current, bridge, onNavigate }: ShellScreenProps) {
  switch (current) {
    case "overview":
      return <OverviewScreen onNavigate={onNavigate} />;
    case "verification":
      return <VerificationScreen bridge={bridge} />;
    case "results":
      return <ResultsScreen onNavigate={onNavigate} />;
    case "report":
      return <ReportScreen onNavigate={onNavigate} />;
    case "help":
      return <HelpScreen bridge={bridge} />;
  }
}
