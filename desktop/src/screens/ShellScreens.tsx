import { useMemo, useState } from "react";
import { Notice } from "../components/Notice";
import {
  lookupField,
  makeReportId,
  peripheralHealthRows,
  saveAssessmentPdf,
} from "../report/assessmentPdf";
import { makeDiagnosticId, saveDiagnosticPdf } from "../report/diagnosticPdf";
import { groupHex, verifyIntegritySeal, type SealCheck } from "../report/verifySeal";
import type {
  AdvanceInteractive,
  AdvanceScanConsent,
  AdvanceScanPhase,
  AdvanceScanProgress,
  AdvanceScanRecord,
  BridgeState,
  DomainCoverage,
  IntegritySeal,
  NamedValue,
  NavigationId,
  ScanTarget,
  VerificationPhase,
  VerificationProgress,
  VerificationRecord,
  WorkstreamId,
} from "../types/shell";
import { ADVANCE_SCAN_STAGES, SOFTWARE_OBSERVED_LABEL } from "../types/shell";
import { InteractiveChecks } from "./InteractiveChecks";

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
  advanceScan: AdvanceScanRecord | null;
  advanceScanPhase: AdvanceScanPhase;
  advanceScanError: string | null;
  advanceScanProgress: AdvanceScanProgress | null;
  advanceConsent: AdvanceScanConsent;
  onToggleAdvanceConsent: (field: keyof AdvanceScanConsent) => void;
  advanceInteractive: AdvanceInteractive;
  onChangeAdvanceInteractive: (next: AdvanceInteractive) => void;
  onRunAdvanceScan: () => void;
  workstream: WorkstreamId;
  onChooseWorkstream: (id: WorkstreamId) => void;
  onExit: () => void;
}

const verificationStages = [
  "Preparing Verification",
  "Confirming Device Identity",
  "Collecting Hardware Information",
  "Assessing Data Locations",
  "Building the Privacy Exposure Map",
  "Preparing Evidence",
  "Checking Consistency",
  "Preparing Results",
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
  onChooseWorkstream,
  verification,
  verificationPhase,
  advanceScan,
  advanceScanPhase,
}: Pick<
  ShellScreenProps,
  | "onChooseWorkstream"
  | "verification"
  | "verificationPhase"
  | "advanceScan"
  | "advanceScanPhase"
>) {
  return (
    <div className="screen-stack">
      <ScreenHeader
        eyebrow="WORKSTATION HOME"
        title="Choose an Assessment"
        copy="Select an assessment for this PC. Each assessment is non-destructive and returns you here when complete."
      />

      <section className="workstream-grid" aria-label="Workstreams">
        <article className="workstream-card workstream-card-assessment">
          <span className="workstream-kicker">01 · STANDARD ASSESSMENT</span>
          <h2>Report A</h2>
          <p>
            Identify this PC, review selected hardware and map approved data locations without
            opening personal files.
          </p>
          <ol>
            <li>Select the drives to assess.</li>
            <li>Run the verification.</li>
            <li>Generate Report A.</li>
          </ol>
          <p className="workstream-status">
            {verificationPhase === "complete"
              ? "Assessment complete."
              : verificationPhase === "running"
                ? "Assessment in progress."
                : "Not yet run on this PC."}
          </p>
          <button className="button button-primary" type="button" onClick={() => onChooseWorkstream("assessment")}>
            {verification ? "Open Standard Assessment" : "Start Standard Assessment"}
          </button>
        </article>

        <article className="workstream-card workstream-card-advance">
          <span className="workstream-kicker">02 · ADVANCED DIAGNOSTIC</span>
          <h2>Report D</h2>
          <p>
            Run an in-depth hardware diagnostic with optional benchmarks and technician checks. USB
            topology and battery/charger state are collected once during this scan.
          </p>
          <ol>
            <li>Run the advanced diagnostic.</li>
            <li>Complete optional technician checks.</li>
            <li>Generate Report D.</li>
          </ol>
          <p className="workstream-status">
            {advanceScanPhase === "complete"
              ? "Assessment complete. Report D is ready."
              : advanceScanPhase === "running"
                ? "Assessment in progress."
                : "Not yet run on this PC."}
          </p>
          <button className="button button-advance" type="button" onClick={() => onChooseWorkstream("advance")}>
            {advanceScan ? "Open Advanced Diagnostic" : "Start Advanced Diagnostic"}
          </button>
        </article>

        <article className="workstream-card workstream-card-purge">
          <span className="workstream-kicker">03 · DATA PURGE</span>
          <h2>Data Purge — Not Available</h2>
          <p>Secure data purge is not enabled in this version of CYVRA Erase.</p>
          <ol>
            <li>Complete the standard assessment first.</li>
            <li>Save Report A.</li>
            <li>Data purge stays off in this version.</li>
          </ol>
          <p className="workstream-status">No data will be erased, overwritten or destroyed.</p>
          <button className="button button-danger" type="button" onClick={() => onChooseWorkstream("purge")}>
            Data Purge Not Available
          </button>
        </article>
      </section>

      <Notice kind="warning" title="Assessment boundary">
        Reports A and D are local assessments, not sanitization certificates. CYVRA Erase will not
        erase files.
      </Notice>
    </div>
  );
}

function advanceStageStatus(
  index: number,
  phase: AdvanceScanPhase,
  progress: AdvanceScanProgress | null,
): "Pending" | "Running" | "Complete" | "Stopped" {
  if (phase === "error") return "Stopped";
  if (phase === "complete") return "Complete";
  if (phase !== "running") return "Pending";
  const current = progress?.stageIndex ?? 0;
  if (index < current) return "Complete";
  if (index === current) return "Running";
  return "Pending";
}

function AdvanceProgressRing({
  percent,
  phase,
  stage,
  detail,
}: {
  percent: number;
  phase: AdvanceScanPhase;
  stage: string;
  detail: string;
}) {
  const size = 156;
  const stroke = 11;
  const radius = (size - stroke) / 2;
  const circumference = 2 * Math.PI * radius;
  const clamped = Math.max(0, Math.min(100, percent));
  const offset = circumference - (clamped / 100) * circumference;
  const tone =
    phase === "error"
      ? "advance-ring-error"
      : phase === "complete"
        ? "advance-ring-done"
        : phase === "running"
          ? "advance-ring-live"
          : "advance-ring-idle";
  const shown = phase === "idle" ? 0 : clamped;

  return (
    <div
      className={`advance-progress ${tone}`}
      role="status"
      aria-live="polite"
      aria-label="Advance scan progress"
    >
      <div className="advance-ring-wrap">
        <svg
          className="advance-ring"
          width={size}
          height={size}
          viewBox={`0 0 ${size} ${size}`}
          aria-hidden="true"
        >
          <circle
            className="advance-ring-track"
            cx={size / 2}
            cy={size / 2}
            r={radius}
            strokeWidth={stroke}
          />
          <circle
            className="advance-ring-value"
            cx={size / 2}
            cy={size / 2}
            r={radius}
            strokeWidth={stroke}
            strokeDasharray={circumference}
            strokeDashoffset={offset}
            transform={`rotate(-90 ${size / 2} ${size / 2})`}
          />
        </svg>
        <div className="advance-ring-label">
          <strong>{phase === "idle" ? "0" : shown}</strong>
          <span>{phase === "idle" ? "ready" : "%"}</span>
        </div>
      </div>
      <div className="advance-progress-copy">
        <p className="advance-progress-kicker">
          {phase === "running"
            ? "Now reading"
            : phase === "complete"
              ? "Finished"
              : phase === "error"
                ? "Stopped"
                : "Waiting"}
        </p>
        <p className="advance-progress-stage">{stage}</p>
        <p className="advance-progress-detail">{detail}</p>
      </div>
    </div>
  );
}

function AdvanceScanPanel({
  collectionOn,
  previewMode,
  verificationPhase,
  advanceScan,
  advanceScanPhase,
  advanceScanError,
  advanceScanProgress,
  advanceConsent,
  onToggleAdvanceConsent,
  advanceInteractive,
  onChangeAdvanceInteractive,
  onRunAdvanceScan,
  onNavigate,
}: {
  collectionOn: boolean;
  previewMode: boolean;
} & Pick<
  ShellScreenProps,
  | "verificationPhase"
  | "advanceScan"
  | "advanceScanPhase"
  | "advanceScanError"
  | "advanceScanProgress"
  | "advanceConsent"
  | "onToggleAdvanceConsent"
  | "advanceInteractive"
  | "onChangeAdvanceInteractive"
  | "onRunAdvanceScan"
  | "onNavigate"
>) {
  const busy = advanceScanPhase === "running" || verificationPhase === "running";
  const canRun = (collectionOn || previewMode) && !busy;
  const percent =
    advanceScanPhase === "complete" ? 100 : (advanceScanProgress?.percent ?? 0);
  const stage =
    advanceScanProgress?.stage ??
    (advanceScanPhase === "complete" ? "Preparing Report D" : "Preparing advance scan");
  const detail =
    advanceScanPhase === "running"
      ? (advanceScanProgress?.detail ?? "Advance scan is reading this PC.")
      : advanceScanPhase === "complete"
        ? (advanceScanProgress?.detail ?? "Report D is ready.")
        : advanceScanPhase === "error"
          ? (advanceScanError ?? "Advance scan stopped.")
          : "CYVRA is preparing the diagnostic environment and checking available system capabilities.";

  return (
    <section className="content-panel advance-panel" aria-labelledby="advance-title">
      <div className="panel-heading">
        <div>
          <span className="card-label">02 · ADVANCE DIAGNOSTIC</span>
          <h2 id="advance-title">Advanced Diagnostic · Report D</h2>
        </div>
        <span
          className={`status-pill ${advanceScanPhase === "complete" ? "status-positive" : advanceScanPhase === "running" ? "status-advance" : "status-neutral"}`}
        >
          {advanceScanPhase === "running"
            ? `${percent}%`
            : advanceScanPhase === "complete"
              ? "Complete"
              : advanceScanPhase === "error"
                ? "Stopped"
                : "Optional"}
        </span>
      </div>
      <p className="panel-lead">
        CYVRA is reading approved system information without changing your files or settings. USB
        sockets and charger state are collected once in this pass and printed on Report D. Optional
        technician checks verify selected hardware functions. This version does not run a live USB
        insertion check or a live charger overlay. Snapshots are not stored. MAC addresses are never
        printed. Storage SMART is read, never erased. Package temperature is not collected. CPU,
        memory and storage workloads run only if you allow benchmarks. Report D then records a
        provisional grade. It still does not erase anything and it does not open file contents.
      </p>

      <ul className="advance-scope">
        <li>Deep collection is read-only unless you allow the optional write test. Storage SMART is read, never erased. MAC addresses are never printed.</li>
        <li>Windows may ask for administrator approval. Declining still produces Report D.</li>
        <li>Anything this build cannot read is printed as not collected in this scan. Package temperature is never invented.</li>
      </ul>

      <AdvanceProgressRing
        percent={percent}
        phase={advanceScanPhase}
        stage={stage}
        detail={detail}
      />

      <ol className="advance-stage-list">
        {ADVANCE_SCAN_STAGES.map((name, index) => {
          const status = advanceStageStatus(index, advanceScanPhase, advanceScanProgress);
          return (
            <li key={name} className={status === "Running" ? "advance-stage-current" : undefined}>
              <span className="advance-stage-number" aria-hidden="true">
                {String(index + 1).padStart(2, "0")}
              </span>
              <span>{name}</span>
              <strong>{status}</strong>
            </li>
          );
        })}
      </ol>

      <fieldset className="advance-consent">
        <legend>Permissions for This Run</legend>
        <label>
          <input
            type="checkbox"
            checked={advanceConsent.benchmarks}
            disabled={busy}
            onChange={() => onToggleAdvanceConsent("benchmarks")}
          />
          <span>
            <strong>Allow benchmarks</strong>
            <small>
              Run short processor, memory and storage performance checks. No personal data is
              accessed. Off by default. Without them, clock and memory-integrity points stay not
              assessable. Package temperature is not collected. This is never printed as memory
              verified.
            </small>
          </span>
        </label>
        <label>
          <input
            type="checkbox"
            checked={advanceConsent.writeBenchmark}
            disabled={busy || !advanceConsent.benchmarks}
            onChange={() => onToggleAdvanceConsent("writeBenchmark")}
          />
          <span>
            <strong>Allow Temporary Storage Write Test</strong>
            <small>
              Write and automatically delete a small temporary test file to measure storage
              performance. The temporary test file is removed automatically when the check is
              complete. Leave this off to keep the scan strictly read-only. Report D always records
              how many bytes were written.
            </small>
          </span>
        </label>
      </fieldset>

      <InteractiveChecks
        value={advanceInteractive}
        disabled={busy}
        onChange={onChangeAdvanceInteractive}
      />

      {advanceScanError ? (
        <Notice kind="error" title="Advance scan did not finish">
          {advanceScanError}
        </Notice>
      ) : null}

      <div className="action-row">
        <button
          className="button button-advance"
          type="button"
          disabled={!canRun}
          onClick={onRunAdvanceScan}
        >
          {advanceScanPhase === "running" ? "Advanced diagnostic running…" : "Run Advanced Diagnostic"}
        </button>
        <p>
          {collectionOn
              ? "The bright ring shows live percent and the exact subsystem being read. Battery, processor identity, storage SMART, panel EDID and radios are collected in this version. Workloads run only with consent."
              : previewMode
                ? "Browser preview walks the same stages. Processor identity, panel EDID, radios, battery firmware, SMART and consented workloads are only read on the installed Windows application."
              : "Open the installed Windows application to run Advance scan."}
        </p>
      </div>

      {advanceScan ? (
        <Notice
          kind={advanceScan.gradeWithheld ? "information" : "success"}
          title={
            advanceScan.gradeWithheld
              ? "Advance scan finished without a grade"
              : `Advance scan finished · provisional grade ${advanceScan.gradeLabel}`
          }
        >
          {advanceScan.gradeWithheld
            ? advanceScan.gradeWithheldReason ??
              "Too little of this device could be assessed to support a grade."
            : `Coverage ${advanceScan.coveragePercent}%. Report D is ready to review.`}{" "}
          <button className="link-button" type="button" onClick={() => onNavigate("report")}>
            Open Report D
          </button>
        </Notice>
      ) : null}
    </section>
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
  advanceScan,
  advanceScanPhase,
  advanceScanError,
  advanceScanProgress,
  advanceConsent,
  onToggleAdvanceConsent,
  advanceInteractive,
  onChangeAdvanceInteractive,
  onRunAdvanceScan,
  onNavigate,
  onChooseWorkstream,
  workstream,
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
  | "advanceScan"
  | "advanceScanPhase"
  | "advanceScanError"
  | "advanceScanProgress"
  | "advanceConsent"
  | "onToggleAdvanceConsent"
  | "advanceInteractive"
  | "onChangeAdvanceInteractive"
  | "onRunAdvanceScan"
  | "onNavigate"
  | "onChooseWorkstream"
  | "workstream"
>) {
  const collectionOn = bridge.status === "ready" && bridge.bootstrap.liveCollectionEnabled;
  const previewMode = bridge.status === "ready" && bridge.bootstrap.runtimeMode === "browser_design_adapter";
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
        copy="Confirm your selected drives and start the non-destructive assessment. Return home when each report is saved."
      />

      <div className="workstream-toolbar">
        <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
          Back to main
        </button>
      </div>

      <section
        className={`workstream-panel workstream-panel-assessment${workstream === "assessment" ? " workstream-panel-active" : ""}`}
        aria-labelledby="drive-title"
      >
        <div className="workstream-panel-head">
          <span className="workstream-kicker">01 · STANDARD ASSESSMENT</span>
          <h2 id="drive-title">Select Drives</h2>
        </div>
        <p className="panel-lead">
          Choose the drives you want to include in this assessment. This is the Windows system drive
          and is recommended for every assessment. Select additional internal or attached drives only
          when they need to be assessed. CYVRA Erase will not open or modify personal file contents.
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
                ? "Ready to Verify"
                : verificationPhase === "running"
                  ? "Verification in Progress"
                  : verificationPhase === "complete"
                    ? "Assessment Complete"
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
          {verificationPhase === "running" ? "Assessment in progress…" : "Start Verification"}
        </button>
        <p>
          {collectionOn
            ? selectedDrives.length === 0
              ? "Select at least one drive to continue."
              : "This assessment may take several minutes. CYVRA Erase will not erase files."
            : "Open the installed Windows application to run verification."}
        </p>
      </div>

      {verification ? (
        <Notice kind="success" title="Assessment Complete">
          Hardware {hardwareResultLabel(verification.hardwareResult).toLowerCase()}. Document map{" "}
          {verification.personalLocationCount} locations on {verification.scannedDrives}. File contents
          were not opened. No data was erased.
        </Notice>
      ) : null}
      </section>

      <div
        className={`workstream-panel workstream-panel-advance${workstream === "advance" ? " workstream-panel-active" : ""}`}
      >
      <AdvanceScanPanel
        collectionOn={collectionOn}
        previewMode={previewMode}
        verificationPhase={verificationPhase}
        advanceScan={advanceScan}
        advanceScanPhase={advanceScanPhase}
        advanceScanError={advanceScanError}
        advanceScanProgress={advanceScanProgress}
        advanceConsent={advanceConsent}
        onToggleAdvanceConsent={onToggleAdvanceConsent}
        advanceInteractive={advanceInteractive}
        onChangeAdvanceInteractive={onChangeAdvanceInteractive}
        onRunAdvanceScan={onRunAdvanceScan}
        onNavigate={onNavigate}
      />
      </div>

      <div
        className={`workstream-panel workstream-panel-purge${workstream === "purge" ? " workstream-panel-active" : ""}`}
      >
        <section className="content-panel" aria-labelledby="purge-teaser-title">
          <div className="panel-heading">
            <div>
              <span className="card-label">03 WIPE</span>
              <h2 id="purge-teaser-title">Wipe is listed, not enabled</h2>
            </div>
            <span className="status-pill status-caution">Not enabled</span>
          </div>
          <p>
            After a wipe you would receive a sanitization report and return here. On this build the
            engine stays fail-closed. No disk is erased from this screen.
          </p>
          <div className="action-row">
            <button className="button button-danger" type="button" onClick={() => onChooseWorkstream("purge")}>
              Open wipe report
            </button>
            <p>The wipe report records that sanitization did not run.</p>
          </div>
        </section>
      </div>
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
        copy="The assessment has finished successfully and your results are ready to review. This is an assessment, not a sanitization certificate."
      />

      <div className="workstream-toolbar">
        <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
          Back to main
        </button>
      </div>

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
          Review Results
        </button>
        <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
          Back to main
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

const GRADING_ENGINE_LABEL = "Graded by CYVRA Grading Engine";

function CoverageDomainTable({ domains }: { domains: DomainCoverage[] }) {
  if (domains.length === 0) {
    return (
      <section className="report-table-block" aria-labelledby="coverage-by-area-heading">
        <h3 id="coverage-by-area-heading">Coverage by diagnostic area</h3>
        <p className="report-empty-copy">No diagnostic areas were evaluated.</p>
      </section>
    );
  }

  return (
    <section className="report-table-block" aria-labelledby="coverage-by-area-heading">
      <h3 id="coverage-by-area-heading">Coverage by diagnostic area</h3>
      <table className="report-table coverage-table">
        <thead>
          <tr>
            <th scope="col">Area</th>
            <th scope="col">State</th>
            <th scope="col">Confidence</th>
            <th scope="col">Awarded</th>
            <th scope="col">Assessed</th>
            <th scope="col">Not assessable</th>
            <th scope="col">Weight</th>
          </tr>
        </thead>
        <tbody>
          {domains.map((domain) => (
            <tr key={domain.domain}>
              <th scope="row">
                {domain.domain}
                <small className="coverage-note">{domain.note}</small>
              </th>
              <td>{domain.state}</td>
              <td>{domain.confidence}</td>
              <td>{domain.awarded}</td>
              <td>{domain.assessed}</td>
              <td>{domain.notAssessable}</td>
              <td>{domain.weight}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}

function IntegritySealCard({ seal }: { seal: IntegritySeal }) {
  const [check, setCheck] = useState<SealCheck | null>(null);
  const [busy, setBusy] = useState(false);
  const qrSrc = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(seal.qrSvg)}`;

  async function handleVerify() {
    setBusy(true);
    try {
      setCheck(await verifyIntegritySeal(seal));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="seal-card" aria-labelledby="seal-title">
      <div className="seal-qr">
        <img src={qrSrc} width={168} height={168} alt="QR code of the Report D SHA-256 digest" />
        <span className="card-label">LOCAL SEAL</span>
      </div>
      <div className="seal-body">
        <h3 id="seal-title">Local integrity seal</h3>
        <p>
          This seal helps verify that the report has not been altered after the assessment was
          completed. {seal.notice}
        </p>
        <p className="seal-digest">
          <span>SHA-256 Digest</span> {groupHex(seal.digestHex)}
        </p>
        <p className="setup-note">
          The report is protected with a local integrity signature for verification. This is a local
          integrity check and does not confirm cloud authentication. Scheme {seal.scheme}.
        </p>
        <div className="email-actions no-print">
          <button
            className="button button-secondary"
            type="button"
            disabled={busy}
            onClick={() => {
              void handleVerify();
            }}
          >
            {busy ? "Checking…" : "Verify this report"}
          </button>
        </div>
        {check ? (
          <p className={check.ok ? "seal-pass" : "seal-fail"} role="status">
            {check.detail}
          </p>
        ) : null}
      </div>
    </section>
  );
}

function AdvanceReportBlock({
  advanceScan,
  verification,
  onNavigate,
}: { advanceScan: AdvanceScanRecord | null; verification: VerificationRecord | null } & Pick<
  ShellScreenProps,
  "onNavigate"
>) {
  const [pdfSaved, setPdfSaved] = useState(false);
  const [savingPdf, setSavingPdf] = useState(false);
  const [pdfNote, setPdfNote] = useState<string | null>(null);
  const generatedAt = useMemo(() => new Date(), [advanceScan]);
  const documentId = advanceScan ? makeDiagnosticId(verification, generatedAt) : "";

  function handleSaveDiagnostic() {
    if (!advanceScan) return;
    setSavingPdf(true);
    setPdfNote(null);
    try {
      const saved = saveDiagnosticPdf(advanceScan, verification);
      setPdfSaved(true);
      setPdfNote(
        `Saved ${saved.filename} to this PC’s Downloads folder (or the folder the save dialog chose). Keep a copy off the disks you may later erase.`,
      );
    } catch (caught) {
      setPdfNote(
        caught instanceof Error
          ? caught.message
          : "Could not write Report D here. Use Print and choose Microsoft Print to PDF.",
      );
    } finally {
      setSavingPdf(false);
    }
  }

  if (!advanceScan) {
    return (
      <section className="report-shell" aria-labelledby="advance-report-title">
        <header className="report-letterhead">
          <p className="report-org">CYVORIQ Solutions Pvt. Ltd.</p>
          <p className="report-issuer">
            Issued by the publisher of CYVRA Erase · computer-generated on this PC
          </p>
          <span className="card-label">REPORT D</span>
          <h2 id="advance-report-title">Technical Diagnostic &amp; Condition Evidence Record</h2>
          <p className="local-assessment-notice">
            Report D has not been prepared on this PC. Run the advanced diagnostic first. CYVRA will
            not estimate a grade from diagnostics it did not perform.
          </p>
        </header>
        <div className="action-row">
          <button
            className="button button-secondary"
            type="button"
            onClick={() => onNavigate("verification")}
          >
            Open Advance scan
          </button>
        </div>
      </section>
    );
  }

  const notAssessablePoints = advanceScan.coverageRows.find(
    (row) => row.label === "Points not assessable",
  )?.value;

  return (
    <section className="report-shell" aria-labelledby="advance-report-title">
      <header className="report-letterhead">
        <p className="report-org">CYVORIQ Solutions Pvt. Ltd.</p>
        <p className="report-issuer">
          Issued by the publisher of CYVRA Erase · computer-generated on this PC
        </p>
        <span className="card-label">REPORT D</span>
          <h2 id="advance-report-title">Technical Diagnostic &amp; Condition Evidence Record</h2>
          <p className="report-meta">
            Document no. <strong>{documentId}</strong>
            <span aria-hidden="true"> · </span>
            Generated <strong>{generatedAt.toLocaleString()}</strong>
          </p>
          <p className="local-assessment-notice">
            This report contains assessment information collected from this PC during the completed
            diagnostic session. It is a computer-generated diagnostic evaluation. It is not a
            sanitization certificate, not NIST SP 800-88 Purge proof, and not a DPDP compliance
            certificate. The final device grade is shown independently from the Assessed Health
            Index. Physical verification is required before a device is finally graded. Cloud
            authentication is not enabled in this version.
          </p>
      </header>

      <div className="report-state-grid">
        <div>
          <span>Scan scope</span>
          <strong>Advance scan</strong>
        </div>
        <div>
          <span>Administrator approval</span>
          <strong>{advanceScan.elevationLabel}</strong>
        </div>
        <div>
          <span>Bytes written to assessed drives</span>
          <strong>{advanceScan.bytesWritten}</strong>
        </div>
      </div>

      <ReportTable
        title="Coverage statement"
        rows={advanceScan.coverageRows}
        empty="No coverage statement was produced."
      />

      <CoverageDomainTable domains={advanceScan.coverageDomains} />

      <section className="grade-card" aria-labelledby="grade-title">
        <div className={advanceScan.gradeWithheld ? "grade-mark grade-mark-withheld" : "grade-mark"}>
          <span className="card-label">
            {advanceScan.provisional ? "PROVISIONAL GRADE" : "GRADE"}
          </span>
          <strong>{advanceScan.gradeLabel}</strong>
          <span>
            {advanceScan.gradeObservation
              ? `${advanceScan.gradeCondition} — ${SOFTWARE_OBSERVED_LABEL}`
              : advanceScan.gradeCondition}
          </span>
        </div>
        <div className="grade-body">
          <h3 id="grade-title">
            {advanceScan.indexPercent === null
              ? "Assessed Health Index not assessable in this scan"
              : `Assessed Health Index ${advanceScan.indexPercent} / 100`}
          </h3>
          <p>
            Coverage {advanceScan.coveragePercent}%
            {notAssessablePoints ? ` — ${notAssessablePoints} points were not assessable` : ""}.
          </p>
          <p className="grade-engine">
            {GRADING_ENGINE_LABEL} · rubric {advanceScan.gradingRubric}
          </p>
          {advanceScan.gradeWithheldReason ? (
            <p className="grade-withheld-reason">{advanceScan.gradeWithheldReason}</p>
          ) : null}
          <p className="grade-issuance">{advanceScan.issuanceNotice}</p>
          <p className="setup-note">
            A grade is never awarded for an area that could not be measured, and never deducted for
            one either. Physical verification by a technician is required for a final grade.
          </p>
        </div>
      </section>

      {advanceScan.integritySeal ? <IntegritySealCard seal={advanceScan.integritySeal} /> : null}

      <div className="email-row no-print">
        <label>Keep a Copy of Your Report</label>
        <p className="panel-lead">
          Save a PDF copy of this assessment for your records or review. If the PDF download does
          not appear, select Print and choose Microsoft Print to PDF.
        </p>
        <div className="email-actions">
          <button
            className="button button-advance"
            type="button"
            disabled={savingPdf}
            onClick={handleSaveDiagnostic}
          >
            {savingPdf ? "Writing PDF…" : pdfSaved ? "Save Report D again" : "Save Report D as PDF"}
          </button>
          <button className="button button-secondary" type="button" onClick={() => window.print()}>
            Print…
          </button>
        </div>
        {pdfNote ? <p className="setup-note">{pdfNote}</p> : null}
      </div>

      {advanceScan.telemetryGroups.map((group) => (
        <div key={group.title}>
          <ReportTable title={group.title} rows={group.rows} empty="Not collected in this scan." />
          {group.note ? <p className="telemetry-note">{group.note}</p> : null}
        </div>
      ))}

      <section className="report-table-block" aria-labelledby="not-assessable-heading">
        <h3 id="not-assessable-heading">Not assessable in this scan</h3>
        {advanceScan.notAssessable.length === 0 ? (
          <p className="report-empty-copy">Every diagnostic area was assessed.</p>
        ) : (
          <ul className="not-assessable-list">
            {advanceScan.notAssessable.map((entry) => (
              <li key={entry}>{entry}</li>
            ))}
          </ul>
        )}
      </section>

      <ReportTable
        title="Method and limitations"
        rows={advanceScan.methodRows}
        empty="No method statement was produced."
      />

      <ReportTable
        title="Grading rubric"
        rows={advanceScan.rubricRows}
        empty="No rubric was recorded."
      />

      <section className="report-table-block" aria-labelledby="advance-boundary-heading">
        <h3 id="advance-boundary-heading">Method and boundary</h3>
        <p className="report-empty-copy">{advanceScan.boundaryNote}</p>
        <p className="report-empty-copy">{advanceScan.temporaryFilesNote}</p>
        <p className="report-empty-copy">
          Issued by CYVORIQ Solutions Pvt. Ltd. as publisher of CYVRA Erase. This document is
          computer-generated on the assessed PC and is not cloud-authenticated in this version. A
          local integrity seal, when present, proves the JSON was not altered after the scan. It is
          not Authenticode and not a CYVORIQ certificate.
        </p>
        <div className="verification-block">
          <h3>Physical verification</h3>
          <p>
            Complete this block after inspecting the PC. USB topology and battery/charger state
            come from the Advance scan pass printed above. Live USB and live charger overlays are
            not used in this version. This is not a handwritten signature line.
          </p>
          <dl className="verification-fields">
            <div>
              <dt>Technician name</dt>
              <dd>Recorded at sign-off</dd>
            </div>
            <div>
              <dt>Date of inspection</dt>
              <dd>Recorded at sign-off</dd>
            </div>
            <div>
              <dt>Physical verification</dt>
              <dd>See Technician checks</dd>
            </div>
          </dl>
        </div>
      </section>
    </section>
  );
}

function ReportScreen({
  verification,
  advanceScan,
  onNavigate,
}: Pick<ShellScreenProps, "verification" | "advanceScan" | "onNavigate">) {
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
        copy="Review the assessment records collected on this PC. Report A is the intake record. Report D is the diagnostic evidence record. Data purge stays off."
      />

      <div className="workstream-toolbar">
        <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
          Back to main
        </button>
      </div>

      {verification ? (
        <section className="workstream-panel workstream-panel-assessment content-panel report-panel" aria-labelledby="report-state-title">
          <div id="assessment-print" className="assessment-print">
            <header className="report-letterhead">
              <p className="report-org">CYVORIQ Solutions Pvt. Ltd.</p>
              <p className="report-issuer">Issued by the publisher of CYVRA Erase · computer-generated on this PC</p>
              <span className="card-label">REPORT A</span>
              <h2 id="report-state-title">Intake &amp; Pre-Sanitization Assessment Record</h2>
              <p className="report-meta">
                Document no. <strong>{reportId}</strong>
                <span aria-hidden="true"> · </span>
                Generated <strong>{generatedAt.toLocaleString()}</strong>
              </p>
              <p className="local-assessment-notice">
                DOCUMENT STATUS: PRE-SANITIZATION ASSESSMENT — NO DATA ERASURE PERFORMED. This is a
                computer-generated local assessment. It is not a sanitization certificate, not NIST
                SP 800-88 Purge proof, and not a DPDP compliance certificate. File contents were not
                opened. No drive was erased. Device condition rating is possible only after physical
                verification. Cloud authentication is not enabled in this version.
              </p>
            </header>
            <div className="report-state-grid">
              <div>
                <span>Created</span>
                <strong>On this PC</strong>
              </div>
              <div>
                <span>Issuing organisation</span>
                <strong>CYVORIQ Solutions Pvt. Ltd.</strong>
              </div>
              <div>
                <span>Erasure status</span>
                <strong>No data was erased</strong>
              </div>
            </div>

            <ReportTable title="Executive Assessment Snapshot" rows={summaryRows} empty="No summary available." />
            <ReportTable
              title="5. Hardware Inventory Recorded During This Assessment"
              rows={verification.hardwareFields.filter((row) => {
                const label = row.label.toLowerCase();
                if (["computer name", "operating system", "manufacturer", "model", "bios / oem serial", "chassis serial", "motherboard serial", "smbios uuid"].includes(label)) {
                  return false;
                }
                return !/serial/i.test(row.label) || !/^0+$/.test(row.value.replace(/[^A-Za-z0-9]/g, ""));
              })}
              empty="Hardware details were not available on this PC."
            />
            <ReportTable
              title="6. Deferred, Unknown and Physical-Verification Items"
              rows={healthRows}
              empty="Not collected in this scan."
            />
            <ReportTable
              title="7. Metadata-Based Data-Exposure Indicators"
              rows={verification.locationGroups}
              empty="No document categories were recorded on the selected drives."
            />
          </div>

          <div className="email-row no-print">
            <label htmlFor="report-email">Keep a Copy of Your Report</label>
            <p className="panel-lead">
              Save a PDF copy of this assessment for your records or review. If the PDF download does
              not appear, select Print and choose Microsoft Print to PDF.
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
              <p className="setup-note">PDF saved on this PC. Keep a copy off disks you may later erase.</p>
            ) : null}
          </div>

          <div className="action-row">
            <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
              Back to main
            </button>
          </div>
        </section>
      ) : (
        <section className="workstream-panel workstream-panel-assessment report-empty" aria-labelledby="report-state-title">
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
          <div className="action-row">
            <button className="button button-secondary" type="button" onClick={() => onNavigate("verification")}>
              Open standard assessment
            </button>
            <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
              Back to main
            </button>
          </div>
        </section>
      )}

      <div className="workstream-panel workstream-panel-advance">
        <AdvanceReportBlock
          advanceScan={advanceScan}
          verification={verification}
          onNavigate={onNavigate}
        />
        <div className="action-row">
          <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
            Back to main
          </button>
        </div>
      </div>

      <section className="workstream-panel workstream-panel-purge purge-consent no-print" aria-labelledby="purge-consent-title">
        <div className="workstream-panel-head">
          <span className="workstream-kicker">03 · DATA PURGE</span>
          <h2 id="purge-consent-title">Wipe report (not enabled)</h2>
        </div>
        <p>
          After a wipe you would receive a sanitization report and return to the home screen. Data
          purge permanently destroys data on the drives you select. Treat it as formatting those
          drives. It cannot be undone. After a full-PC purge, Windows and CYVRA Erase will not run on
          this computer, so save Report A as a PDF (or email it) first.
        </p>
        {verification ? (
          <>
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
              <p>This installer will not erase files. The wipe report records that sanitization did not run.</p>
            </div>
            {purgeNote ? <p className="setup-note">{purgeNote}</p> : null}
            {!reportExported ? (
              <p className="setup-note">
                Save Report A as PDF (or email it) before Data purge can be offered. The assessment must
                exist off this PC first.
              </p>
            ) : null}
          </>
        ) : (
          <p className="setup-note">
            Run the standard assessment and save Report A first. Wipe stays off in this version. No
            data was erased.
          </p>
        )}
        <div className="action-row">
          <button className="button button-secondary" type="button" onClick={() => onNavigate("overview")}>
            Back to main
          </button>
        </div>
      </section>
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
        copy="From first login to saving Report D: what should happen, and what it means if it does not."
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
          <h2>Register and receive a key</h2>
          <p>
            Expected: an administrator approves the account, then auth@cyvra.co.in sends “Your CYVRA
            Erase activation key”. If no mail arrives, the account is not licensed yet — do not guess
            a key. If Activate later says invalid_key, the key is mistyped or already bound.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">02</span>
          <h2>Install and activate</h2>
          <p>
            Expected: Welcome, Terms, then Activate binds this Windows PC. If Windows SmartScreen
            warns, that is unsigned assessment software, not a failed install. One licence is one PC.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">03</span>
          <h2>Standard assessment and Report A</h2>
          <p>
            Expected: from home, open 01 Standard assessment. The system drive is selected; USB
            sticks stay off unless you want them in the report. Save Report A, then Back to main.
            Battery health and port counts appear only when this scan collected them. If a serial
            prints as not reported, Windows did not give one.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">04</span>
          <h2>USB sockets and charger on Report D</h2>
          <p>
            Expected: USB controllers, hubs, attached devices and battery/charger state are read
            once during Advance scan and printed on Report D. This version does not run a live USB
            insertion check or a live charger overlay. Those buttons froze some PCs by opening
            repeating command windows.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">05</span>
          <h2>Technician live checks</h2>
          <p>
            After colour wash and keyboard, open the camera if you need a live preview. Images are
            discarded. Nothing is written to a USB stick. Charging is telemetry from the Advance
            scan pass, not a grading point.
          </p>
        </article>
        <article className="support-card">
          <span className="support-number">06</span>
          <h2>Advance scan, wipe record, and verify</h2>
          <p>
            Expected: from home, open 02 Advance diagnostic, run the scan, Save Report D as PDF,
            then Back to main. A local integrity seal (SHA-256 and Ed25519) proves the JSON was not
            altered after the scan. Verify this report re-checks it on this PC. It is not a wipe
            certificate. Workstream 03 Wipe stays fail-closed. If the grade is withheld, coverage
            was too low — inspect Not assessable.
          </p>
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
  advanceScan,
  advanceScanPhase,
  advanceScanError,
  advanceScanProgress,
  advanceConsent,
  onToggleAdvanceConsent,
  advanceInteractive,
  onChangeAdvanceInteractive,
  onRunAdvanceScan,
  workstream,
  onChooseWorkstream,
  onExit,
}: ShellScreenProps) {
  switch (current) {
    case "overview":
      return (
        <OverviewScreen
          onChooseWorkstream={onChooseWorkstream}
          verification={verification}
          verificationPhase={verificationPhase}
          advanceScan={advanceScan}
          advanceScanPhase={advanceScanPhase}
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
          advanceScan={advanceScan}
          advanceScanPhase={advanceScanPhase}
          advanceScanError={advanceScanError}
          advanceScanProgress={advanceScanProgress}
          advanceConsent={advanceConsent}
          onToggleAdvanceConsent={onToggleAdvanceConsent}
          advanceInteractive={advanceInteractive}
          onChangeAdvanceInteractive={onChangeAdvanceInteractive}
          onRunAdvanceScan={onRunAdvanceScan}
          onNavigate={onNavigate}
          onChooseWorkstream={onChooseWorkstream}
          workstream={workstream}
        />
      );
    case "results":
      return <ResultsScreen onNavigate={onNavigate} verification={verification} />;
    case "report":
      return (
        <ReportScreen
          onNavigate={onNavigate}
          verification={verification}
          advanceScan={advanceScan}
        />
      );
    case "help":
      return <HelpScreen bridge={bridge} onExit={onExit} />;
  }
}
