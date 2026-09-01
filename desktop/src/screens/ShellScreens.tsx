import { Notice } from "../components/Notice";
import type {
  BridgeState,
  NavigationId,
  VerificationPhase,
  VerificationRecord,
} from "../types/shell";

interface ShellScreenProps {
  current: NavigationId;
  bridge: BridgeState;
  verificationPhase: VerificationPhase;
  verification: VerificationRecord | null;
  verificationError: string | null;
  onNavigate: (target: NavigationId) => void;
  onRunVerification: () => void;
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

function stageState(phase: VerificationPhase): string {
  if (phase === "running") return "Running";
  if (phase === "complete") return "Complete";
  if (phase === "error") return "Stopped";
  return "Pending";
}

function OverviewScreen({
  onNavigate,
  verification,
  verificationPhase,
}: Pick<ShellScreenProps, "onNavigate" | "verification" | "verificationPhase">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="LOCAL ASSESSMENT"
        title="Overview"
        copy="CYVRA Erase inspects this Windows PC for hardware identity and document locations, then builds a local assessment report. It does not erase files."
      />

      <Notice kind="information" title="What this application does">
        Run Device Verification to collect hardware inventory and a map of documents and application data
        (type and location only). Private file contents are not read. Purge stays off until a later approved phase.
      </Notice>

      <section className="primary-panel" aria-labelledby="current-state-title">
        <div>
          <span className="card-label">CURRENT STATE</span>
          <h2 id="current-state-title">
            {verificationPhase === "complete" ? "Assessment complete on this PC" : "Ready to verify this device"}
          </h2>
          <p>
            {verification
              ? `${verification.manufacturer} ${verification.model} · ${verification.personalLocationCount} document locations · ${verification.pdemObjectCount} PDEM objects.`
              : "No device has been verified in this session. Start from Verification."}
          </p>
        </div>
        <button className="button button-primary" type="button" onClick={() => onNavigate("verification")}>
          {verification ? "Review verification" : "Open verification"}
        </button>
      </section>

      <section className="status-grid" aria-label="Capability status">
        <StatusCard
          label="ACTIVATION"
          value="Device bind live"
          copy="Enter the emailed key on first run. One Windows PC per licence."
        />
        <StatusCard
          label="HARDWARE ENGINE"
          value={verification ? verification.hardwareResult : "Idle"}
          copy="Same hardware_inventory_v1 engine as the CYVRA hardware validator, in-process."
        />
        <StatusCard
          label="DOCUMENT MAP"
          value={verification ? `${verification.pdemObjectCount} objects` : "Idle"}
          copy="Same PDEM engine as the verification agent. Metadata only."
        />
        <StatusCard
          label="PURGE"
          value="Disabled"
          copy="No erase, overwrite or wipe is available in this version."
        />
      </section>

      <Notice kind="warning" title="Assessment boundary">
        CYVRA Erase V1 is assessment-only. No destructive operation is exposed by this shell.
      </Notice>
    </div>
  );
}

function VerificationScreen({
  bridge,
  verificationPhase,
  verification,
  verificationError,
  onRunVerification,
}: Pick<
  ShellScreenProps,
  "bridge" | "verificationPhase" | "verification" | "verificationError" | "onRunVerification"
>) {
  const nativeReady = bridge.status === "ready" && bridge.bootstrap.coreBoundary === "direct_typed_cyvra_core";
  const collectionOn = bridge.status === "ready" && bridge.bootstrap.liveCollectionEnabled;
  const canRun = collectionOn && verificationPhase !== "running";

  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="PASSIVE ASSESSMENT"
        title="Verification"
        copy="One journey runs the hardware validator and the document/PDEM engine inside this application. Both stay non-destructive."
      />

      <div className="two-column-grid">
        <section className="content-panel" aria-labelledby="scope-title">
          <span className="card-label">FROZEN SCOPE</span>
          <h2 id="scope-title">What this scan reviews</h2>
          <ul className="check-list">
            <li>Hardware identity, firmware, processor and memory</li>
            <li>Document locations: Office, PDF, images and other known types</li>
            <li>Application data paths without reading message bodies</li>
            <li>Local assessment report — not a wipe certificate</li>
          </ul>
        </section>

        <section className="content-panel" aria-labelledby="boundary-title">
          <span className="card-label">TRUSTED BOUNDARY</span>
          <h2 id="boundary-title">Typed Rust core</h2>
          <dl className="definition-list">
            <div>
              <dt>Native boundary</dt>
              <dd>{nativeReady ? "Linked" : "Available only in the installed application"}</dd>
            </div>
            <div>
              <dt>Hardware engine</dt>
              <dd>In-process CYVRA hardware validation</dd>
            </div>
            <div>
              <dt>Document engine</dt>
              <dd>{collectionOn ? "Enabled for this PC" : "Unavailable in browser preview"}</dd>
            </div>
          </dl>
        </section>
      </div>

      {verificationError ? (
        <Notice kind="error" title="Verification did not finish">
          {verificationError}
        </Notice>
      ) : null}

      <section className="content-panel" aria-labelledby="stages-title">
        <div className="panel-heading">
          <div>
            <span className="card-label">EIGHT TRUTHFUL STAGES</span>
            <h2 id="stages-title">
              {verificationPhase === "idle"
                ? "Verification has not started"
                : verificationPhase === "running"
                  ? "Verification is running on this PC"
                  : verificationPhase === "complete"
                    ? "Verification finished"
                    : "Verification stopped"}
            </h2>
          </div>
          <span
            className={`status-pill ${verificationPhase === "complete" ? "status-positive" : "status-neutral"}`}
          >
            {stageState(verificationPhase)}
          </span>
        </div>
        <ol className="stage-list">
          {verificationStages.map((stage, index) => (
            <li key={stage}>
              <span className="stage-number" aria-hidden="true">
                {index + 1}
              </span>
              <span>{stage}</span>
              <strong>{stageState(verificationPhase)}</strong>
            </li>
          ))}
        </ol>
      </section>

      <div className="action-row">
        <button className="button button-primary" type="button" disabled={!canRun} onClick={onRunVerification}>
          {verificationPhase === "running" ? "Running Device Verification" : "Run Device Verification"}
        </button>
        <p>
          {collectionOn
            ? "This can take several minutes. CYVRA will not erase files."
            : "Open the installed Windows application to run verification. Browser preview cannot scan a device."}
        </p>
      </div>

      {verification ? (
        <Notice kind="success" title="Local engines finished">
          Hardware result {verification.hardwareResult}. Document map {verification.personalLocationCount} locations,{" "}
          {verification.pdemObjectCount} PDEM objects. Content inspected: no. Data erased: no.
        </Notice>
      ) : null}
    </div>
  );
}

function ResultsScreen({
  onNavigate,
  verification,
}: Pick<ShellScreenProps, "onNavigate" | "verification">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="EVIDENCE-LED OUTCOMES"
        title="Results"
        copy="Hardware condition and privacy exposure stay separate. This is an assessment, not a sanitization certificate."
      />

      {verification ? (
        <Notice kind="success" title="Assessment recorded for this session">
          {verification.assessmentSummary}
        </Notice>
      ) : (
        <Notice kind="warning" title="No results available">
          No verification has completed. The shell does not create sample results that could be mistaken for device
          evidence.
        </Notice>
      )}

      <section className="domain-grid" aria-label="Result domains">
        <article className="domain-card">
          <span className="domain-name">HARDWARE</span>
          <span className={`status-pill ${verification?.hardwarePassed ? "status-positive" : "status-neutral"}`}>
            {verification ? verification.hardwareResult : "Pending"}
          </span>
          <h2>{verification ? `${verification.manufacturer} ${verification.model}` : "Device condition"}</h2>
          <p>
            {verification
              ? `${verification.osCaption} on ${verification.hostname}. Identifiers in the hardware text stay redacted.`
              : "Run Device Verification to collect the hardware inventory."}
          </p>
          <ul className="detail-list">
            <li>Hardware engine: {verification ? verification.hardwareResult : "unavailable"}</li>
            <li>Private content collected: no</li>
            <li>Data erased: no</li>
          </ul>
        </article>

        <article className="domain-card">
          <span className="domain-name">CYVRA ERASE</span>
          <span className={`status-pill ${verification ? "status-positive" : "status-neutral"}`}>
            {verification ? "Mapped" : "Not assessed"}
          </span>
          <h2>Privacy exposure</h2>
          <p>Document and application locations only. File contents are not opened.</p>
          <ul className="detail-list">
            <li>Data-location coverage: {verification ? `${verification.personalLocationCount} locations` : "unavailable"}</li>
            <li>PDEM objects: {verification ? String(verification.pdemObjectCount) : "none"}</li>
            <li>Private content collected: no</li>
          </ul>
        </article>
      </section>

      <div className="action-row">
        <button className="button button-secondary" type="button" onClick={() => onNavigate("report")}>
          Open assessment report
        </button>
        <button className="button button-secondary" type="button" onClick={() => onNavigate("verification")}>
          Review verification
        </button>
      </div>
    </div>
  );
}

function ReportScreen({ onNavigate, verification }: Pick<ShellScreenProps, "onNavigate" | "verification">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="PRE-SANITIZATION ASSESSMENT"
        title="Report"
        copy="This is Report A: hardware plus document map. The signed sanitization certificate is a later phase after an approved purge."
      />

      {verification ? (
        <section className="content-panel" aria-labelledby="report-state-title">
          <span className="card-label">REPORT STATE</span>
          <h2 id="report-state-title">Local assessment report</h2>
          <p>
            {verification.manufacturer} {verification.model} · hardware {verification.hardwareResult} ·{" "}
            {verification.pdemObjectCount} PDEM objects. Authenticity is local-only until a later cloud report slice.
          </p>
          <div className="report-state-grid">
            <div>
              <span>Evidence manifest</span>
              <strong>Created locally</strong>
            </div>
            <div>
              <span>Authenticity</span>
              <strong>Not authenticated by CYVRA cloud</strong>
            </div>
            <div>
              <span>Erasure status</span>
              <strong>No data was erased</strong>
            </div>
          </div>
          <h3 className="report-subhead">Hardware validation</h3>
          <pre className="report-json">{verification.hardwareValidation}</pre>
          <h3 className="report-subhead">Assessment JSON</h3>
          <pre className="report-json">{verification.reportJson}</pre>
          <button className="button button-secondary" type="button" onClick={() => onNavigate("results")}>
            Back to results
          </button>
        </section>
      ) : (
        <section className="report-empty" aria-labelledby="report-state-title">
          <span className="empty-mark" aria-hidden="true">
            P
          </span>
          <span className="card-label">REPORT STATE</span>
          <h2 id="report-state-title">No report has been generated</h2>
          <p>Run Device Verification first. The application will never label a report authenticated before verification succeeds.</p>
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
          <button className="button button-secondary" type="button" onClick={() => onNavigate("verification")}>
            Open verification
          </button>
        </section>
      )}
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
          {bridge.safeMessage} No customer operation was started. Restart the application before continuing.
        </Notice>
      ) : bridge.status === "loading" ? (
        <Notice kind="information" title="Checking trusted boundary">
          No customer operation can start while the internal application boundary is being checked.
        </Notice>
      ) : (
        <Notice kind="success" title="Application loaded safely">
          Verification is local and non-destructive. CYVRA does not erase files in this version.
        </Notice>
      )}

      <section className="support-grid" aria-label="Support topics">
        <article className="support-card">
          <span className="support-number">01</span>
          <h2>Activation and binding</h2>
          <p>Use the emailed licence on this Windows PC. One device per key.</p>
        </article>
        <article className="support-card">
          <span className="support-number">02</span>
          <h2>Evidence limitations</h2>
          <p>Permission denied, unsupported and collection error states remain distinct. Contents are not inspected.</p>
        </article>
        <article className="support-card">
          <span className="support-number">03</span>
          <h2>Report authenticity</h2>
          <p>Local results remain visibly unverified until cloud authentication is a later approved slice.</p>
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

export function ShellScreen({
  current,
  bridge,
  verificationPhase,
  verification,
  verificationError,
  onNavigate,
  onRunVerification,
}: ShellScreenProps) {
  switch (current) {
    case "overview":
      return (
        <OverviewScreen
          onNavigate={onNavigate}
          verification={verification}
          verificationPhase={verificationPhase}
        />
      );
    case "verification":
      return (
        <VerificationScreen
          bridge={bridge}
          verificationPhase={verificationPhase}
          verification={verification}
          verificationError={verificationError}
          onRunVerification={onRunVerification}
        />
      );
    case "results":
      return <ResultsScreen onNavigate={onNavigate} verification={verification} />;
    case "report":
      return <ReportScreen onNavigate={onNavigate} verification={verification} />;
    case "help":
      return <HelpScreen bridge={bridge} />;
  }
}
