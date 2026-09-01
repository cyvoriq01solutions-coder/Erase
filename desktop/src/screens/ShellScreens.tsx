import { useMemo, useState } from "react";
import { Notice } from "../components/Notice";
import {
  lookupField,
  makeReportId,
  peripheralHealthRows,
  saveAssessmentPdf,
} from "../report/assessmentPdf";
import type {
  BridgeState,
  NamedValue,
  NavigationId,
  ScanTarget,
  VerificationPhase,
  VerificationProgress,
  VerificationRecord,
} from "../types/shell";

interface ShellScreenProps {
  current: NavigationId;
  bridge: BridgeState;
  verificationPhase: VerificationPhase;
  verification: VerificationRecord | null;
  verificationError: string | null;
  progress: VerificationProgress | null;
  scanTargets: ScanTarget[];
  selectedDrives: string[];
  onToggleDrive: (letter: string) => void;
  onNavigate: (target: NavigationId) => void;
  onRunVerification: () => void;
  onExit: () => void;
}

const verificationStages = [
  "Preparing verification",
  "Confirming device identity",
  "Collecting hardware information",
  "Assessing personal-data locations",
  "Building the Privacy Exposure Map",
  "Preparing evidence",
  "Checking consistency",
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

function stageStatus(
  index: number,
  phase: VerificationPhase,
  progress: VerificationProgress | null,
): "Pending" | "Running" | "Complete" | "Stopped" {
  if (phase === "error") return "Stopped";
  if (phase === "complete") return "Complete";
  if (phase !== "running") return "Pending";
  const current = progress?.stageIndex ?? 0;
  if (index < current) return "Complete";
  if (index === current) return "Running";
  return "Pending";
}

function hardwareResultLabel(result: string): string {
  if (result === "pass") return "Passed";
  if (result === "fail") return "Needs review";
  return "Not available on this PC";
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
        copy="CYVRA Erase reviews this Windows PC, then prepares a local assessment report. It maps hardware identity and where documents appear to live. It does not erase files."
      />

      <Notice kind="information" title="What this application does">
        Choose the drives to include, run verification, then generate a report you can keep or email.
        File contents are not opened. Purge stays off in this version.
      </Notice>

      <section className="primary-panel" aria-labelledby="current-state-title">
        <div>
          <span className="card-label">CURRENT STATE</span>
          <h2 id="current-state-title">
            {verificationPhase === "complete"
              ? "Assessment complete on this PC"
              : verificationPhase === "running"
                ? "Verification is in progress"
                : "Ready to verify this device"}
          </h2>
          <p>
            {verification
              ? `${verification.manufacturer} ${verification.model} · scanned ${verification.scannedDrives} · ${verification.personalLocationCount} document locations.`
              : "No device has been verified in this session. Start from Verification and choose which drives to include."}
          </p>
        </div>
        <button className="button button-primary" type="button" onClick={() => onNavigate("verification")}>
          {verification ? "Review verification" : "Start verification"}
        </button>
      </section>

      <section className="status-grid" aria-label="Capability status">
        <StatusCard
          label="ACTIVATION"
          value="Device bound"
          copy="Use the emailed licence key on this Windows PC. One PC per licence."
        />
        <StatusCard
          label="HARDWARE"
          value={verification ? hardwareResultLabel(verification.hardwareResult) : "Not yet run"}
          copy="Identity, firmware, processor and memory for this PC."
        />
        <StatusCard
          label="DOCUMENT MAP"
          value={verification ? `${verification.personalLocationCount} locations` : "Not yet run"}
          copy="Known document types and application folders. Metadata only."
        />
        <StatusCard
          label="PURGE"
          value="Disabled"
          copy="No erase, overwrite or wipe is available in this version."
        />
      </section>

      <Notice kind="warning" title="Assessment boundary">
        This report is a local assessment, not a sanitization certificate. CYVRA Erase will not erase files.
      </Notice>
    </div>
  );
}

function VerificationScreen({
  bridge,
  verificationPhase,
  verification,
  verificationError,
  progress,
  scanTargets,
  selectedDrives,
  onToggleDrive,
  onRunVerification,
}: Pick<
  ShellScreenProps,
  | "bridge"
  | "verificationPhase"
  | "verification"
  | "verificationError"
  | "progress"
  | "scanTargets"
  | "selectedDrives"
  | "onToggleDrive"
  | "onRunVerification"
>) {
  const collectionOn = bridge.status === "ready" && bridge.bootstrap.liveCollectionEnabled;
  const canRun =
    collectionOn && verificationPhase !== "running" && selectedDrives.length > 0;
  const percent = verificationPhase === "complete" ? 100 : (progress?.percent ?? 0);
  const currentDetail =
    verificationPhase === "running"
      ? (progress?.detail ?? "Verification is running. This can take several minutes.")
      : verificationPhase === "complete"
        ? "Verification finished. No data was erased."
        : "Choose the drives to include, then start verification.";

  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="DEVICE VERIFICATION"
        title="Verification"
        copy="Select the drives to include, then start the assessment. Leave USB and backup disks unchecked unless you want them in the report."
      />

      <section className="content-panel" aria-labelledby="drive-title">
        <div className="panel-heading">
          <div>
            <span className="card-label">STEP 1</span>
            <h2 id="drive-title">Choose drives to verify</h2>
          </div>
        </div>
        <p className="panel-lead">
          The Windows system drive is selected by default. Extra letters can be USB or backup disks.
          Uncheck anything you do not want scanned. CYVRA will not erase files on any drive.
        </p>
        {scanTargets.length === 0 ? (
          <Notice kind="information" title="Drive list">
            {collectionOn
              ? "CYVRA could not list drives yet. You can still start verification on the Windows system drive."
              : "Open the installed Windows application to choose drives on this PC. Browser preview cannot scan a device."}
          </Notice>
        ) : (
          <ul className="drive-list">
            {scanTargets.map((target) => {
              const checked = selectedDrives.includes(target.letter);
              const disabled = verificationPhase === "running";
              return (
                <li key={target.letter}>
                  <label className={checked ? "drive-card drive-card-selected" : "drive-card"}>
                    <input
                      type="checkbox"
                      checked={checked}
                      disabled={disabled}
                      onChange={() => onToggleDrive(target.letter)}
                    />
                    <span className="drive-letter" aria-hidden="true">
                      {target.letter}:
                    </span>
                    <span>
                      <strong>
                        {target.label} · {target.sizeLabel}
                      </strong>
                      <span className="drive-kind">{target.kind}</span>
                      <span className="drive-hint">{target.hint}</span>
                    </span>
                  </label>
                </li>
              );
            })}
          </ul>
        )}
      </section>

      {verificationError ? (
        <Notice kind="error" title="Verification did not finish">
          {verificationError}
        </Notice>
      ) : null}

      <section className="content-panel" aria-labelledby="progress-title">
        <div className="panel-heading">
          <div>
            <span className="card-label">STEP 2</span>
            <h2 id="progress-title">
              {verificationPhase === "idle"
                ? "Start verification"
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
            {verificationPhase === "running"
              ? `${percent}%`
              : verificationPhase === "complete"
                ? "Complete"
                : verificationPhase === "error"
                  ? "Stopped"
                  : "Ready"}
          </span>
        </div>

        <div
          className="progress-status"
          role="status"
          aria-live="polite"
          aria-label="Verification progress"
        >
          <div className="progress-track" aria-hidden="true">
            <div className="progress-fill" style={{ width: `${percent}%` }} />
          </div>
          <p className="progress-detail">{currentDetail}</p>
        </div>

        <ol className="stage-list">
          {verificationStages.map((stage, index) => {
            const status = stageStatus(index, verificationPhase, progress);
            return (
              <li key={stage} className={status === "Running" ? "stage-current" : undefined}>
                <span className="stage-number" aria-hidden="true">
                  {index + 1}
                </span>
                <span>{stage}</span>
                <strong>{status}</strong>
              </li>
            );
          })}
        </ol>
      </section>

      <div className="action-row">
        <button className="button button-primary" type="button" disabled={!canRun} onClick={onRunVerification}>
          {verificationPhase === "running" ? "Verification running…" : "Start verification"}
        </button>
        <p>
          {collectionOn
            ? selectedDrives.length === 0
              ? "Select at least one drive to continue."
              : "This can take several minutes on large disks. CYVRA will not erase files."
            : "Open the installed Windows application to run verification."}
        </p>
      </div>

      {verification ? (
        <Notice kind="success" title="Local assessment finished">
          Hardware {hardwareResultLabel(verification.hardwareResult).toLowerCase()}. Document map{" "}
          {verification.personalLocationCount} locations on {verification.scannedDrives}. File contents
          were not opened. No data was erased.
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
        eyebrow="ASSESSMENT SUMMARY"
        title="Results"
        copy="Hardware condition and privacy exposure stay separate. This is an assessment, not a sanitization certificate."
      />

      {verification ? (
        <Notice kind="success" title="Assessment recorded for this session">
          {verification.assessmentSummary}
        </Notice>
      ) : (
        <Notice kind="warning" title="No results available">
          No verification has completed. Run verification first so this screen shows evidence from this PC.
        </Notice>
      )}

      <section className="domain-grid" aria-label="Result domains">
        <article className="domain-card">
          <span className="domain-name">HARDWARE</span>
          <span className={`status-pill ${verification?.hardwarePassed ? "status-positive" : "status-neutral"}`}>
            {verification ? hardwareResultLabel(verification.hardwareResult) : "Pending"}
          </span>
          <h2>{verification ? `${verification.manufacturer} ${verification.model}` : "Device condition"}</h2>
          <p>
            {verification
              ? `${verification.osCaption} on ${verification.hostname}.`
              : "Run verification to collect hardware identity for this PC."}
          </p>
          <ul className="detail-list">
            <li>Private content collected: no</li>
            <li>Data erased: no</li>
            <li>Drives included: {verification ? verification.scannedDrives : "none yet"}</li>
          </ul>
        </article>

        <article className="domain-card">
          <span className="domain-name">PRIVACY EXPOSURE</span>
          <span className={`status-pill ${verification ? "status-positive" : "status-neutral"}`}>
            {verification ? "Mapped" : "Not assessed"}
          </span>
          <h2>Document locations</h2>
          <p>Known document types and application folders. File contents are not opened.</p>
          <ul className="detail-list">
            <li>
              Data-location coverage:{" "}
              {verification ? `${verification.personalLocationCount} locations` : "unavailable"}
            </li>
            <li>Mapped objects: {verification ? String(verification.pdemObjectCount) : "none"}</li>
            <li>Private content collected: no</li>
          </ul>
        </article>
      </section>

      <div className="action-row">
        <button
          className="button button-primary"
          type="button"
          disabled={!verification}
          onClick={() => onNavigate("report")}
        >
          Generate report
        </button>
        <button className="button button-secondary" type="button" onClick={() => onNavigate("verification")}>
          Review verification
        </button>
      </div>
    </div>
  );
}

function ReportTable({ title, rows, empty }: { title: string; rows: NamedValue[]; empty: string }) {
  return (
    <section className="report-table-block" aria-labelledby={`${title}-heading`}>
      <h3 id={`${title}-heading`}>{title}</h3>
      {rows.length === 0 ? (
        <p className="report-empty-copy">{empty}</p>
      ) : (
        <table className="report-table">
          <tbody>
            {rows.map((row) => (
              <tr key={`${row.label}-${row.value}`}>
                <th scope="row">{row.label}</th>
                <td>{row.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function ReportScreen({
  verification,
  onNavigate,
}: Pick<ShellScreenProps, "verification" | "onNavigate">) {
  const [email, setEmail] = useState("");
  const [emailNote, setEmailNote] = useState<string | null>(null);
  const [consentChecked, setConsentChecked] = useState(false);
  const [reportEmailed, setReportEmailed] = useState(false);
  const [pdfSaved, setPdfSaved] = useState(false);
  const [savingPdf, setSavingPdf] = useState(false);
  const [purgeNote, setPurgeNote] = useState<string | null>(null);
  const generatedAt = useMemo(() => new Date(), [verification]);
  const reportExported = reportEmailed || pdfSaved;

  const summaryRows = useMemo<NamedValue[]>(() => {
    if (!verification) return [];
    return [
      { label: "Computer name", value: verification.hostname },
      { label: "Manufacturer", value: verification.manufacturer },
      { label: "Model", value: verification.model },
      {
        label: "BIOS / OEM serial",
        value:
          lookupField(verification.hardwareFields, ["bios / oem serial"]) ??
          "Not reported by firmware",
      },
      {
        label: "Chassis serial",
        value:
          lookupField(verification.hardwareFields, ["chassis serial"]) ??
          "Not reported by firmware",
      },
      {
        label: "Motherboard serial",
        value:
          lookupField(verification.hardwareFields, ["motherboard serial"]) ??
          "Not reported by firmware",
      },
      { label: "Operating system", value: verification.osCaption },
      { label: "Drives included", value: verification.scannedDrives },
      { label: "Hardware result", value: hardwareResultLabel(verification.hardwareResult) },
      { label: "Document locations", value: String(verification.personalLocationCount) },
      { label: "Mapped objects", value: String(verification.pdemObjectCount) },
      { label: "File contents opened", value: "No" },
      { label: "Data erased", value: "No" },
    ];
  }, [verification]);

  const healthRows = useMemo(
    () => (verification ? peripheralHealthRows(verification) : []),
    [verification],
  );
  const reportId = verification ? makeReportId(verification, generatedAt) : "";

  function handleEmail() {
    if (!verification) return;
    const address = email.trim();
    if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(address)) {
      setEmailNote("Enter a valid email address.");
      return;
    }

    const body = [
      "CYVRA Erase — local pre-sanitization assessment (Report A)",
      `Report identifier: ${reportId}`,
      "",
      `Computer: ${verification.hostname}`,
      `Device: ${verification.manufacturer} ${verification.model}`,
      `BIOS / OEM serial: ${lookupField(verification.hardwareFields, ["bios / oem serial"]) ?? "Not reported by firmware"}`,
      `Operating system: ${verification.osCaption}`,
      `Drives included: ${verification.scannedDrives}`,
      `Hardware result: ${hardwareResultLabel(verification.hardwareResult)}`,
      `Document locations: ${verification.personalLocationCount}`,
      "File contents opened: No",
      "Data erased: No",
      "",
      verification.assessmentSummary,
      "",
      "This is a local assessment, not a sanitization certificate.",
      "Attach the PDF saved from this PC if the mail application did not include it.",
    ].join("\r\n");

    const href = `mailto:${encodeURIComponent(address)}?subject=${encodeURIComponent(
      `CYVRA Erase assessment ${reportId}`,
    )}&body=${encodeURIComponent(body)}`;
    window.location.assign(href);
    setReportEmailed(true);
    setEmailNote(
      "Your email application should open with this report. If nothing opens, use Save as PDF — that copy is enough.",
    );
  }

  function handleSavePdf() {
    if (!verification) return;
    setSavingPdf(true);
    setEmailNote(null);
    setPurgeNote(null);
    try {
      const saved = saveAssessmentPdf(verification);
      setPdfSaved(true);
      setEmailNote(
        `Saved ${saved.filename} to this PC’s Downloads folder (or the folder the save dialog chose). Keep a copy off the disks you may later erase.`,
      );
    } catch (caught) {
      setEmailNote(
        caught instanceof Error
          ? caught.message
          : "Could not write a PDF here. Use Print and choose Microsoft Print to PDF.",
      );
    } finally {
      setSavingPdf(false);
    }
  }

  function handlePrint() {
    window.print();
  }

  function handlePurgeIntent() {
    if (!verification) return;
    if (!reportExported) {
      setPurgeNote(
        "Save the assessment as a PDF (or email it) first. After a full-PC purge this application cannot create the report.",
      );
      return;
    }
    if (!consentChecked) {
      setPurgeNote("Tick the consent box before Data purge.");
      return;
    }
    setPurgeNote(
      "No data was erased. Data purge is not enabled in this installer. Your saved PDF and this consent do not start a wipe. A CYVRA Purge licence and a later signed build are required.",
    );
  }

  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="LOCAL ASSESSMENT REPORT"
        title="Report"
        copy="A professional operator copy of this PC’s assessment. Save it as a PDF. This is not a wipe certificate and is not authenticated by CYVRA cloud."
      />

      {verification ? (
        <section className="content-panel report-panel" aria-labelledby="report-state-title">
          <div id="assessment-print" className="assessment-print">
            <header className="report-letterhead">
              <p className="report-org">CYVORIQ Solutions · CYVRA Erase</p>
              <span className="card-label">REPORT A</span>
              <h2 id="report-state-title">Local pre-sanitization assessment</h2>
              <p className="report-meta">
                Report identifier <strong>{reportId}</strong>
                <span aria-hidden="true"> · </span>
                Generated on this PC <strong>{generatedAt.toLocaleString()}</strong>
              </p>
              <p className="local-assessment-notice">
                This is a local hardware and document-location assessment. It is not a sanitization
                certificate, not NIST SP 800-88 Purge proof, and not a DPDP compliance certificate. File
                contents were not opened. No drive was erased. Battery health and connector counts are
                listed only when this scan collected them — never guessed.
              </p>
            </header>
            <div className="report-state-grid">
              <div>
                <span>Created</span>
                <strong>On this PC</strong>
              </div>
              <div>
                <span>Cloud authentication</span>
                <strong>Not requested</strong>
              </div>
              <div>
                <span>Erasure status</span>
                <strong>No data was erased</strong>
              </div>
            </div>

            <ReportTable title="Assessment summary" rows={summaryRows} empty="No summary available." />
            <ReportTable
              title="Hardware recorded in this scan"
              rows={verification.hardwareFields}
              empty="Hardware details were not available on this PC."
            />
            <ReportTable
              title="Battery, cameras, microphones, and connectors"
              rows={healthRows}
              empty="Not collected in this scan."
            />
            <ReportTable
              title="Privacy exposure"
              rows={verification.locationGroups}
              empty="No document categories were recorded on the selected drives."
            />
          </div>

          <div className="email-row no-print">
            <label htmlFor="report-email">Keep a copy off this PC</label>
            <p className="panel-lead">
              Email did not open on some Windows PCs. Save as PDF first. Print and choose Microsoft Print
              to PDF if the download does not appear.
            </p>
            <div className="email-actions">
              <button
                className="button button-primary"
                type="button"
                disabled={savingPdf}
                onClick={handleSavePdf}
              >
                {savingPdf ? "Writing PDF…" : pdfSaved ? "Save as PDF again" : "Save as PDF"}
              </button>
              <button className="button button-secondary" type="button" onClick={handlePrint}>
                Print…
              </button>
            </div>
            <div className="email-actions email-actions-follow">
              <input
                id="report-email"
                type="email"
                value={email}
                placeholder="name@company.com"
                autoComplete="email"
                onChange={(event) => {
                  setEmail(event.target.value);
                  setEmailNote(null);
                }}
              />
              <button className="button button-secondary" type="button" onClick={handleEmail}>
                Email report
              </button>
            </div>
            {emailNote ? <p className="setup-note">{emailNote}</p> : null}
            {pdfSaved ? (
              <p className="setup-note">PDF saved on this PC. You may tick consent if you also accept the warning below.</p>
            ) : null}
          </div>

          <section className="purge-consent no-print" aria-labelledby="purge-consent-title">
            <h3 id="purge-consent-title">Data purge (not enabled)</h3>
            <p>
              Data purge permanently destroys data on the drives you select. Treat it as formatting those
              drives. It cannot be undone. After a full-PC purge, Windows and CYVRA Erase will not run on
              this computer, so save the PDF (or email it) first.
            </p>
            <label className="purge-consent-check" htmlFor="purge-consent-box">
              <input
                id="purge-consent-box"
                type="checkbox"
                checked={consentChecked}
                onChange={(event) => {
                  setConsentChecked(event.target.checked);
                  setPurgeNote(null);
                }}
              />
              <span>
                I understand I am opting in willingly and knowingly. Data purge is as irreversible as
                formatting the selected drives. I have saved this report as a PDF or emailed it, because
                CYVRA Erase cannot create it after a full-PC purge.
              </span>
            </label>
            <div className="action-row">
              <button
                className="button button-danger"
                type="button"
                disabled={!consentChecked || !reportExported}
                onClick={handlePurgeIntent}
              >
                Data purge
              </button>
            </div>
            {purgeNote ? <p className="setup-note">{purgeNote}</p> : null}
            {!reportExported ? (
              <p className="setup-note">
                Save as PDF (or email the report) before Data purge can be offered. The assessment must
                exist off this PC first.
              </p>
            ) : null}
          </section>

          <div className="action-row">
            <button className="button button-secondary" type="button" onClick={() => onNavigate("results")}>
              Back to results
            </button>
          </div>
        </section>
      ) : (
        <section className="report-empty" aria-labelledby="report-state-title">
          <span className="empty-mark" aria-hidden="true">
            P
          </span>
          <span className="card-label">REPORT STATE</span>
          <h2 id="report-state-title">No report has been generated</h2>
          <p>Run verification first, then choose Generate report. The application will not invent a report.</p>
          <div className="report-state-grid">
            <div>
              <span>Evidence</span>
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

function HelpScreen({
  bridge,
  onExit,
}: Pick<ShellScreenProps, "bridge" | "onExit">) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="HELP AND SETTINGS"
        title="Help and recovery"
        copy="Activation, privacy and how to close CYVRA Erase safely."
      />

      {bridge.status === "error" ? (
        <Notice kind="error" title="Application boundary unavailable">
          {bridge.safeMessage} No customer operation was started. Restart the application before continuing.
        </Notice>
      ) : bridge.status === "loading" ? (
        <Notice kind="information" title="Starting CYVRA Erase">
          No customer operation can start while the application is checking its safety boundary.
        </Notice>
      ) : (
        <Notice kind="success" title="Application loaded safely">
          Verification is local and non-destructive. CYVRA does not erase files in this version.
        </Notice>
      )}

      <section className="support-grid" aria-label="Support topics">
        <article className="support-card">
          <span className="support-number">01</span>
          <h2>Activation key</h2>
          <p>
            CYVRA emails the key from auth@cyvra.co.in after an administrator issues a licence. Enter it once on
            this Windows PC.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">02</span>
          <h2>Drive selection</h2>
          <p>
            Leave USB and backup disks unchecked unless you want them in the report. Large extra disks make
            verification take longer.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">03</span>
          <h2>What the report is</h2>
          <p>
            The report is a local pre-sanitization assessment. Save it as a PDF. It is not a wipe certificate.
            Battery health and port counts appear only when this scan collected them.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">04</span>
          <h2>Privacy</h2>
          <p>Passwords, recovery keys and file contents are not collected. Raw identifiers stay off support logs.</p>
        </article>
      </section>

      <div className="action-row">
        <button className="button button-secondary" type="button" onClick={onExit}>
          Exit CYVRA Erase
        </button>
      </div>
    </div>
  );
}

export function ShellScreen({
  current,
  bridge,
  verificationPhase,
  verification,
  verificationError,
  progress,
  scanTargets,
  selectedDrives,
  onToggleDrive,
  onNavigate,
  onRunVerification,
  onExit,
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
          progress={progress}
          scanTargets={scanTargets}
          selectedDrives={selectedDrives}
          onToggleDrive={onToggleDrive}
          onRunVerification={onRunVerification}
        />
      );
    case "results":
      return <ResultsScreen onNavigate={onNavigate} verification={verification} />;
    case "report":
      return <ReportScreen onNavigate={onNavigate} verification={verification} />;
    case "help":
      return <HelpScreen bridge={bridge} onExit={onExit} />;
  }
}
