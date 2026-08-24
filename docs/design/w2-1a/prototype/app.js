"use strict";

const screens = [
  { id: "welcome", label: "01 · Welcome and Trust", title: "Welcome to CYVRA Erase", eyebrow: "TRUST AND PRODUCT IDENTITY" },
  { id: "activate", label: "02 · Sign In or Activate", title: "Sign in or activate", eyebrow: "AUTHORIZED ACCESS" },
  { id: "binding", label: "03 · Device-Binding Disclosure", title: "Bind this entitlement", eyebrow: "ONE DEVICE · ONE BINDING" },
  { id: "device", label: "04 · Device Detected", title: "Confirm this device", eyebrow: "DEVICE IDENTITY" },
  { id: "consent", label: "05 · Consent and Scope", title: "Review consent and scope", eyebrow: "PRIVACY BEFORE COLLECTION" },
  { id: "ready", label: "06 · Ready to Verify", title: "Ready to verify", eyebrow: "FINAL CHECK" },
  { id: "progress", label: "07 · Verification Progress", title: "Verification in progress", eyebrow: "PASSIVE ASSESSMENT" },
  { id: "overview", label: "08 · Overview", title: "Overview", eyebrow: "DEVICE VERIFICATION" },
  { id: "results", label: "09 · Results Overview", title: "Verification results", eyebrow: "COMPLETED WITH LIMITATIONS" },
  { id: "qc", label: "10 · CYVRA QC Results", title: "CYVRA QC results", eyebrow: "DEVICE CONDITION" },
  { id: "erase", label: "11 · CYVRA Erase Results", title: "CYVRA Erase results", eyebrow: "PRIVACY EXPOSURE MAP" },
  { id: "report", label: "12 · Combined Report Preview", title: "Combined report preview", eyebrow: "EVIDENCE PACKAGE" },
  { id: "completion", label: "13 · Completion", title: "Verification complete", eyebrow: "SAFE COMPLETION" },
  { id: "help", label: "14 · Help and Recovery", title: "Help and recovery", eyebrow: "PRIVACY-SAFE SUPPORT" },
];

const progressStages = [
  "Preparing verification",
  "Confirming device identity",
  "Collecting passive hardware information",
  "Assessing personal-data locations",
  "Building the Privacy Exposure Map",
  "Preparing evidence",
  "Verifying consistency",
  "Preparing results",
];

const state = {
  screen: "overview",
  scenario: "complete",
  progress: 3,
  consent: false,
  wireframe: false,
  highContrast: false,
};

const screenSelect = document.querySelector("#screen-select");
const scenarioSelect = document.querySelector("#scenario-select");
const fidelityToggle = document.querySelector("#fidelity-toggle");
const contrastToggle = document.querySelector("#contrast-toggle");
const screenRoot = document.querySelector("#prototype-screen");
const screenTitle = document.querySelector("#screen-title");
const screenEyebrow = document.querySelector("#screen-eyebrow");
const cancelModal = document.querySelector("#cancel-modal");
const toast = document.querySelector("#prototype-toast");

let toastTimer;
let returnFocus;

for (const screen of screens) {
  const option = document.createElement("option");
  option.value = screen.id;
  option.textContent = screen.label;
  screenSelect.append(option);
}

function resultData() {
  if (state.scenario === "excellent") {
    return {
      status: "issued",
      grade: "A",
      descriptor: "Excellent",
      coverage: 98,
      assessment: "Completed",
      limitation: "No material limitation identified by the approved evidence set.",
      ringClass: "grade-a",
      dimensions: [
        ["Core system operation", 98, "Confirmed good"],
        ["Display and input", 96, "Confirmed good"],
        ["Storage condition", 95, "Confirmed good"],
        ["Battery and power", 92, "Confirmed good"],
        ["Connectivity, ports and audio", 96, "Confirmed good"],
        ["Cosmetic and structural", 94, "Minor wear"],
      ],
    };
  }

  if (state.scenario === "insufficient") {
    return {
      status: "unable",
      grade: "?",
      descriptor: "Unable to grade",
      coverage: 71,
      assessment: "Insufficient evidence",
      limitation: "Required cosmetic views and battery-condition evidence are missing.",
      ringClass: "grade-empty",
      dimensions: [
        ["Core system operation", 92, "Confirmed good"],
        ["Display and input", 80, "Minor limitation"],
        ["Storage condition", 88, "Confirmed good"],
        ["Battery and power", 0, "Evidence missing"],
        ["Connectivity, ports and audio", 70, "Partial coverage"],
        ["Cosmetic and structural", 0, "Review required"],
      ],
    };
  }

  return {
    status: "issued",
    grade: "B",
    descriptor: "Good",
    coverage: 94,
    assessment: "Completed with limitations",
    limitation: "Battery capacity is below 80%. Secure Boot status was unavailable to the standard-user scan.",
    ringClass: "",
    dimensions: [
      ["Core system operation", 96, "Confirmed good"],
      ["Display and input", 88, "Minor limitation"],
      ["Storage condition", 90, "Confirmed good"],
      ["Battery and power", 72, "Degraded"],
      ["Connectivity, ports and audio", 86, "Minor limitation"],
      ["Cosmetic and structural", 82, "Minor wear"],
    ],
  };
}

function iconCheck() {
  return '<span class="check-icon" aria-hidden="true">✓</span>';
}

function notice(kind, symbol, content) {
  const className = kind ? `notice ${kind}` : "notice";
  return `<div class="${className}"><strong aria-hidden="true">${symbol}</strong><div>${content}</div></div>`;
}

function action(label, target, primary = true) {
  return `<button class="button ${primary ? "button-primary" : "button-secondary"}" type="button" data-screen="${target}">${label}</button>`;
}

function renderWelcome() {
  return `
    <div class="screen-grid">
      <section class="hero-panel">
        <span class="kicker">CYVRA ERASE · BY CYVORIQ SOLUTIONS</span>
        <h2>Know the device. Understand the exposure. Preserve the evidence.</h2>
        <p>
          CYVRA provides one guided Windows experience for CYVRA QC device grading and CYVRA Erase privacy-exposure
          assessment.
        </p>
        <ul class="trust-list">
          <li>${iconCheck()}<span>Signed publisher identity and protected entitlement are verified before assessment.</span></li>
          <li>${iconCheck()}<span>The V1 assessment is passive, read-only and designed to run as a standard user.</span></li>
          <li>${iconCheck()}<span>Private file contents, passwords, recovery keys, camera frames and microphone audio are excluded.</span></li>
        </ul>
        <div class="button-row">${action("Continue", "activate")}</div>
      </section>
      <aside class="panel">
        <span class="status-tag"><b aria-hidden="true">✓</b> Publisher verified</span>
        <h2>Trust before action</h2>
        <p>Product version, publisher, architecture and safety mode remain visible before the customer continues.</p>
        <div class="device-summary">
          <div class="fact"><span>Publisher</span><strong>CYVORIQ Solutions</strong></div>
          <div class="fact"><span>Application</span><strong>CYVRA Erase</strong></div>
          <div class="fact"><span>Architecture</span><strong>Windows x64</strong></div>
          <div class="fact"><span>Mode</span><strong>Assessment only</strong></div>
        </div>
        ${notice("warning", "!", "<strong>Assessment only</strong><br />This release does not erase data.")}
      </aside>
    </div>`;
}

function renderActivate() {
  return `
    <div class="screen-grid">
      <section class="hero-panel compact">
        <span class="kicker">AUTHORIZED ACCESS</span>
        <h2>Continue securely</h2>
        <p>Use the verified email route or a server-issued activation key. CYVRA never reveals whether an unrelated account or key exists.</p>
        <div class="segmented" aria-label="Activation method">
          <button type="button" class="active">Email sign in</button>
          <button type="button">Activation key</button>
        </div>
        <div class="form-stack">
          <div class="form-field">
            <label for="email-prototype">Email address</label>
            <input id="email-prototype" type="email" value="customer@example.com" autocomplete="off" />
            <span class="field-help">Prototype only. Nothing is transmitted or stored.</span>
          </div>
        </div>
        <div class="button-row">
          ${action("Continue securely", "binding")}
          ${action("Back", "welcome", false)}
        </div>
      </section>
      <aside class="panel">
        <h2>Protected entitlement</h2>
        <ul class="trust-list">
          <li>${iconCheck()}<span>Email ownership is verified using the approved one-time challenge.</span></li>
          <li>${iconCheck()}<span>Activation keys are masked and never written to customer logs.</span></li>
          <li>${iconCheck()}<span>The server—not this application—decides eligibility and entitlement state.</span></li>
        </ul>
        ${notice("", "i", "Network access is required to verify entitlement. Offline bypass is not available in V1.")}
      </aside>
    </div>`;
}

function renderBinding() {
  return `
    <div class="screen-grid">
      <section class="hero-panel compact">
        <span class="kicker">ONE ENTITLEMENT · ONE DEVICE</span>
        <h2>Bind this entitlement to the detected device</h2>
        <p>
          CYVRA creates a privacy-preserving device identity from approved hardware categories. Raw serials and UUIDs
          are not sent as the normal binding credential.
        </p>
        <div class="scope-grid">
          <div class="scope-card"><strong>Device categories</strong><span>Manufacturer, model, platform and approved stable inputs</span></div>
          <div class="scope-card"><strong>Server treatment</strong><span>Domain-separated pseudonymous binding value</span></div>
          <div class="scope-card"><strong>Same device</strong><span>Approved revalidation may continue</span></div>
          <div class="scope-card"><strong>Different device</strong><span>Rejected unless an audited support reset is approved</span></div>
        </div>
        <div class="consent-row">
          <input id="binding-agree" type="checkbox" checked />
          <label for="binding-agree">I understand that this entitlement will be bound to one privacy-preserving device identity.</label>
        </div>
        <div class="button-row">
          ${action("Agree and bind", "device")}
          ${action("Back", "activate", false)}
        </div>
      </section>
      <aside class="panel">
        <h2>Support and recovery</h2>
        <p>A hardware change does not silently create a new entitlement. A support-assisted rebind requires an authorized and auditable decision.</p>
        ${notice("warning", "!", "Do not continue if this is not the device you intend to verify.")}
      </aside>
    </div>`;
}

function renderDevice() {
  return `
    <div class="screen-grid">
      <section class="hero-panel compact">
        <span class="kicker">DETECTED DEVICE</span>
        <h2>Is this the device you intend to verify?</h2>
        <div class="device-summary">
          <div class="fact"><span>Manufacturer</span><strong>Lenovo</strong></div>
          <div class="fact"><span>Model</span><strong>ThinkPad T14 Gen 3</strong></div>
          <div class="fact"><span>Form factor</span><strong>Laptop · OEM reported</strong></div>
          <div class="fact"><span>Architecture</span><strong>x64</strong></div>
          <div class="fact"><span>Firmware mode</span><strong>UEFI</strong></div>
          <div class="fact"><span>Device identifier</span><strong>•••• •••• 7F2C</strong></div>
        </div>
        <div class="button-row">
          ${action("Confirm device", "consent")}
          <button class="button button-secondary" type="button" data-action="toast" data-message="Support-safe mismatch guidance opened.">This is not my device</button>
        </div>
      </section>
      <aside class="panel">
        <span class="status-tag"><b aria-hidden="true">✓</b> Identity consistent</span>
        <h2>Evidence sources</h2>
        <ul class="evidence-list">
          <li>${iconCheck()}<span>Windows system information · Reported</span></li>
          <li>${iconCheck()}<span>Firmware identity · Reported</span></li>
          <li>${iconCheck()}<span>Form factor · Derived with high confidence</span></li>
        </ul>
        <p>Exact sensitive identifiers remain masked in the normal interface.</p>
      </aside>
    </div>`;
}

function renderConsent() {
  return `
    <div class="screen-stack">
      ${notice("warning", "!", "<strong>Assessment only — this release does not erase data.</strong><br />No destructive command is included in this journey.")}
      <div class="screen-grid">
        <section class="hero-panel compact">
          <span class="kicker">WHAT CYVRA WILL ASSESS</span>
          <h2>Review the scan scope before giving consent</h2>
          <div class="scope-grid">
            <div class="scope-card"><strong>Passive hardware facts</strong><span>Device, firmware, processor, memory, storage, display, battery, ports and supported peripherals</span></div>
            <div class="scope-card"><strong>Privacy-exposure metadata</strong><span>Location categories and application-data presence without reading personal content</span></div>
            <div class="scope-card"><strong>Evidence and provenance</strong><span>Source, timestamp, status, confidence, permission and schema version</span></div>
            <div class="scope-card"><strong>CYVRA QC evidence</strong><span>Approved non-destructive condition and cosmetic evidence where supplied</span></div>
          </div>
        </section>
        <aside class="panel">
          <h2>CYVRA will not collect</h2>
          <ul class="scope-list">
            <li>${iconCheck()}<span>Private file contents or personal filenames</span></li>
            <li>${iconCheck()}<span>Passwords, tokens, keys or recovery material</span></li>
            <li>${iconCheck()}<span>Camera frames, microphone audio or sensor measurements</span></li>
            <li>${iconCheck()}<span>Browser-history, email or message content</span></li>
          </ul>
          <div class="consent-row">
            <input id="scan-consent" type="checkbox" ${state.consent ? "checked" : ""} />
            <label for="scan-consent">I have reviewed the assessment scope and give explicit consent to begin this non-destructive verification.</label>
          </div>
          <div class="button-row">
            <button class="button button-primary" id="consent-continue" type="button" data-screen="ready" ${state.consent ? "" : "disabled"}>Give consent</button>
          </div>
        </aside>
      </div>
    </div>`;
}

function renderReady() {
  return `
    <div class="screen-grid">
      <section class="hero-panel">
        <span class="kicker">FINAL CHECK</span>
        <h2>Ready to run one coordinated device verification</h2>
        <p>CYVRA will move through eight truthful stages. You may cancel safely without producing a completed grade or report.</p>
        <div class="summary-grid">
          <div class="summary-tile"><strong>Device confirmed</strong><span>Lenovo ThinkPad T14 Gen 3 · x64</span></div>
          <div class="summary-tile"><strong>Entitlement active</strong><span>Bound to this privacy-preserving device identity</span></div>
          <div class="summary-tile"><strong>Scope accepted</strong><span>Hardware, PDEM metadata and approved QC evidence</span></div>
          <div class="summary-tile"><strong>Runtime</strong><span>Standard user · Passive and read-only</span></div>
        </div>
        <div class="button-row">
          <button class="button button-primary" type="button" data-action="start-progress">Run Device Verification</button>
          ${action("Review consent", "consent", false)}
        </div>
      </section>
      <aside class="panel">
        <h2>Verification stages</h2>
        <ol class="plain-list">
          ${progressStages.map((stage, index) => `<li><strong>${String(index + 1).padStart(2, "0")}</strong> ${stage}</li>`).join("")}
        </ol>
        ${notice("", "i", "Progress is based on completed work. CYVRA does not display a fake time estimate.")}
      </aside>
    </div>`;
}

function renderProgress() {
  const completeCount = Math.min(state.progress, progressStages.length);
  const percentage = Math.round((completeCount / progressStages.length) * 100);
  const rows = progressStages.map((stage, index) => {
    const completed = index < completeCount;
    const running = index === completeCount && completeCount < progressStages.length;
    const limited = state.scenario === "complete" && index === 2 && completed;
    const classes = [completed ? "completed" : "", running ? "running" : "", limited ? "limited" : ""].filter(Boolean).join(" ");
    const detail = completed ? (limited ? "Completed with limitations" : "Completed") : running ? "Running" : "Pending";
    const marker = completed ? "✓" : String(index + 1).padStart(2, "0");
    return `<div class="progress-stage ${classes}"><span class="stage-number">${marker}</span><strong>${stage}</strong><small>${detail}</small></div>`;
  }).join("");

  return `
    <div class="screen-grid">
      <section class="panel">
        <span class="kicker">TRUTHFUL STAGE PROGRESS</span>
        <h2>${completeCount >= progressStages.length ? "All verification stages completed" : progressStages[completeCount]}</h2>
        <div class="stage-list">${rows}</div>
      </section>
      <aside class="panel progress-summary">
        <span class="status-tag"><b aria-hidden="true">●</b> ${completeCount >= progressStages.length ? "Completed" : "Running safely"}</span>
        <div class="progress-value"><strong>${percentage}%</strong><span>known work completed</span></div>
        <div class="progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${percentage}" aria-label="Verification progress"><span style="width:${percentage}%"></span></div>
        <p>Current collectors are bounded by timeout, output limit and cancellation controls.</p>
        ${state.scenario === "complete" && completeCount > 2 ? notice("warning", "!", "Secure Boot evidence is permission limited. Remaining safe collection continues.") : ""}
        <div class="button-row">
          <button class="button button-primary" type="button" data-action="advance-progress">${completeCount >= progressStages.length ? "View results" : "Advance prototype stage"}</button>
          <button class="button button-secondary" type="button" data-action="open-cancel">Cancel safely</button>
        </div>
      </aside>
    </div>`;
}

function renderOverview() {
  return `
    <div class="screen-stack">
      <section class="hero-panel compact">
        <div class="domain-header">
          <div>
            <span class="kicker">CURRENT DEVICE</span>
            <h2>Lenovo ThinkPad T14 Gen 3</h2>
            <p>Windows 11 Pro · x64 · Entitlement active · Device binding verified</p>
          </div>
          <span class="status-tag"><b aria-hidden="true">✓</b> Ready</span>
        </div>
        <div class="button-row">${action("Run Device Verification", "ready")}</div>
      </section>
      <div class="summary-grid">
        <article class="summary-card"><span class="kicker">LAST VERIFICATION</span><h2>24 Aug 2026</h2><p>Completed with two explicit limitations.</p></article>
        <article class="summary-card"><span class="kicker">CYVRA QC</span><h2>Grade B — Good</h2><p>Evidence coverage 94% · One battery limitation.</p></article>
        <article class="summary-card"><span class="kicker">CYVRA ERASE</span><h2>Assessment complete</h2><p>Exposure metadata identified · No data was erased.</p></article>
        <article class="summary-card"><span class="kicker">REPORT</span><h2>Preview available</h2><p>Authenticity verification remains pending in this prototype.</p></article>
      </div>
      ${notice("warning", "!", "Secure Boot state was unavailable to the standard-user scan. This is an evidence limitation, not failed hardware.")}
    </div>`;
}

function renderNetworkError() {
  return `
    <section class="error-panel">
      <div class="error-symbol" aria-hidden="true">!</div>
      <span class="kicker">NETWORK RECOVERY</span>
      <h2>CYVRA could not verify the latest server state</h2>
      <p>Your local read-only evidence remains safe. No entitlement, grade or authenticated report was created during this failed request.</p>
      <div class="support-code">Support code: NET-RETRY-204 · Contains no raw device identifier</div>
      <div class="button-row centered">
        <button class="button button-primary" type="button" data-action="toast" data-message="Bounded retry started in the prototype.">Retry securely</button>
        ${action("Open help", "help", false)}
      </div>
    </section>`;
}

function renderResults() {
  if (state.scenario === "network") return renderNetworkError();
  const data = resultData();
  const gradeHeading = data.status === "issued" ? `Grade ${data.grade} — ${data.descriptor}` : "Unable to grade";
  const gradeStatus = data.status === "issued" ? "Grade issued" : "Insufficient evidence";
  return `
    <div class="screen-stack">
      ${notice("success", "✓", "Verification completed safely. All limitations remain explicit and no data was erased.")}
      <div class="domain-grid">
        <article class="result-domain-card">
          <div class="domain-header">
            <div><span class="kicker">CYVRA QC</span><h2>${gradeHeading}</h2></div>
            <span class="status-tag ${data.status === "issued" ? "" : "warning"}">${gradeStatus}</span>
          </div>
          <p>${data.limitation}</p>
          <div class="summary-grid">
            <div class="summary-tile"><strong>${data.coverage}% coverage</strong><span>Approved grade-bearing evidence</span></div>
            <div class="summary-tile"><strong>${data.assessment}</strong><span>Rules profile cyvra_qc_condition_v1</span></div>
          </div>
          <div class="button-row">${action("Review CYVRA QC", "qc")}</div>
        </article>
        <article class="result-domain-card erase">
          <div class="domain-header">
            <div><span class="kicker">CYVRA ERASE</span><h2>Privacy assessment complete</h2></div>
            <span class="status-tag">Assessment complete</span>
          </div>
          <p>Potential personal-data locations were mapped using approved metadata. Private content was not opened or uploaded.</p>
          <div class="summary-grid">
            <div class="summary-tile"><strong>6 categories</strong><span>Exposure-location categories identified</span></div>
            <div class="summary-tile"><strong>2 limitations</strong><span>Permission and unsupported-source limitations</span></div>
          </div>
          <div class="button-row">${action("Review CYVRA Erase", "erase", false)}</div>
        </article>
      </div>
      ${notice("warning", "!", "<strong>No data was erased.</strong> This assessment identifies exposure and prepares evidence for an authorized next step.")}
      <div class="button-row">${action("Preview combined report", "report")}</div>
    </div>`;
}

function renderQc() {
  if (state.scenario === "network") return renderNetworkError();
  const data = resultData();
  const issued = data.status === "issued";
  const gradeTitle = issued ? `Grade ${data.grade} — ${data.descriptor}` : "Unable to grade — insufficient evidence";
  const rows = data.dimensions.map(([name, value, label]) => `
    <div class="dimension-row">
      <strong>${name}</strong>
      <div class="meter-track ${value < 75 ? "warning" : ""}"><span style="width:${Math.max(value, 3)}%"></span></div>
      <span>${label}</span>
    </div>`).join("");

  return `
    <div class="screen-stack">
      <section class="hero-panel grade-layout">
        <div class="grade-ring ${data.ringClass}" aria-label="${gradeTitle}; ${data.coverage} percent evidence coverage">
          <div class="grade-mark"><strong>${data.grade}</strong><span>${issued ? data.descriptor : "Not issued"}</span><small>${data.coverage}% evidence</small></div>
        </div>
        <div class="grade-copy">
          <span class="kicker">CYVRA QC · CONDITION ASSESSMENT</span>
          <h2>${gradeTitle}</h2>
          <p>${data.limitation}</p>
          <div class="button-row">
            ${issued ? action("Continue to privacy results", "erase") : '<button class="button button-primary" type="button" data-action="toast" data-message="Additional-evidence guidance opened.">Provide missing evidence</button>'}
            ${action("Evidence details", "qc", false)}
          </div>
        </div>
      </section>
      <div class="screen-grid">
        <section class="panel">
          <div class="domain-header"><h2>Condition dimensions</h2><span class="status-tag ${issued ? "" : "warning"}">${data.coverage}% coverage</span></div>
          <div class="dimension-list">${rows}</div>
        </section>
        <aside class="panel">
          <h2>Material limitations</h2>
          <ul class="evidence-list">
            <li><span class="check-icon" aria-hidden="true">!</span><span>${data.limitation}</span></li>
            <li>${iconCheck()}<span>Grade reflects condition—not device specification, warranty, resale value or auction price.</span></li>
            <li>${iconCheck()}<span>Assessment date: 24 Aug 2026 · Rules version 1.0.0</span></li>
          </ul>
        </aside>
      </div>
      <section class="panel">
        <h2>Evidence provenance</h2>
        <table class="evidence-table">
          <thead><tr><th>Category</th><th>Status</th><th>Source</th><th>Confidence</th></tr></thead>
          <tbody>
            <tr><td>System identity</td><td>Reported</td><td>Windows + firmware</td><td>High</td></tr>
            <tr><td>Battery capacity</td><td>${issued ? "Derived" : "Not reported"}</td><td>Firmware/driver capacity</td><td>${issued ? "Medium" : "Unavailable"}</td></tr>
            <tr><td>Cosmetic condition</td><td>${issued ? "Operator reviewed" : "Review required"}</td><td>Guided customer media</td><td>${issued ? "High" : "Pending"}</td></tr>
            <tr><td>Secure Boot</td><td>Permission denied</td><td>Windows security state</td><td>Explicit limitation</td></tr>
          </tbody>
        </table>
      </section>
      ${notice("", "i", "CYVRA QC Grade describes assessed device condition at the recorded assessment time. It is not a resale valuation or warranty.")}
    </div>`;
}

function renderErase() {
  return `
    <div class="screen-stack">
      ${notice("warning", "!", "<strong>Assessment only — this release does not erase data.</strong>")}
      <section class="hero-panel compact">
        <div class="domain-header">
          <div><span class="kicker">CYVRA ERASE · PRIVACY EXPOSURE</span><h2>Potential personal-data locations identified</h2><p>Metadata-only findings are grouped without showing personal filenames or content.</p></div>
          <span class="status-tag">Assessment complete</span>
        </div>
        <div class="privacy-map">
          <div class="privacy-node"><strong>User profiles</strong><span>3 profile locations observed · Contents not read</span></div>
          <div class="privacy-node"><strong>Documents and downloads</strong><span>Location metadata present · Contents not read</span></div>
          <div class="privacy-node"><strong>Browser and app data</strong><span>2 application categories observed · History not collected</span></div>
          <div class="privacy-node"><strong>Mail and messages</strong><span>Application-data location detected · Bodies excluded</span></div>
          <div class="privacy-node"><strong>Cloud sync locations</strong><span>1 configured location observed · Tokens excluded</span></div>
          <div class="privacy-node"><strong>Removable and other volumes</strong><span>One source permission limited</span></div>
        </div>
      </section>
      <div class="screen-grid equal">
        <section class="panel"><h2>Assessment coverage</h2><div class="summary-grid"><div class="summary-tile"><strong>92%</strong><span>Approved metadata scope assessed</span></div><div class="summary-tile"><strong>2</strong><span>Explicit limitations</span></div></div></section>
        <section class="panel"><h2>What remains private</h2><p>CYVRA did not collect file contents, email or message bodies, browser history, passwords, tokens, recovery keys, screenshots, camera frames or microphone audio.</p></section>
      </div>
      ${notice("", "i", "No data was erased. This assessment identifies exposure and prepares evidence for an authorized next step.")}
      <div class="button-row">${action("Preview combined report", "report")}${action("Back to QC results", "qc", false)}</div>
    </div>`;
}

function renderReport() {
  if (state.scenario === "network") return renderNetworkError();
  const data = resultData();
  const gradeDisplay = data.status === "issued" ? `${data.grade} — ${data.descriptor}` : "Unable to grade";
  return `
    <div class="screen-stack">
      ${notice("warning", "!", "This prototype report is <strong>not authenticated</strong>. Production authenticity requires the separately approved report-verification service.")}
      <article class="report-page">
        <header class="report-header">
          <div><span class="kicker">CYVORIQ SOLUTIONS</span><h2>CYVRA Device Verification Report</h2><p>Combined CYVRA QC and CYVRA Erase evidence summary</p></div>
          <div class="report-id">REPORT PREVIEW<br />CVR-2026-08-24-7F2C<br />Revision 1</div>
        </header>
        <section class="report-section">
          <h3>Device and assessment</h3>
          <div class="report-columns">
            <div class="fact"><span>Device</span><strong>Lenovo ThinkPad T14 Gen 3</strong></div>
            <div class="fact"><span>Assessment time</span><strong>24 Aug 2026 · 15:24 UTC</strong></div>
            <div class="fact"><span>Operating system</span><strong>Windows 11 Pro · x64</strong></div>
            <div class="fact"><span>Identifier</span><strong>Masked · •••• 7F2C</strong></div>
          </div>
        </section>
        <section class="report-section">
          <h3>CYVRA QC condition result</h3>
          <div class="report-grade"><strong>${data.grade}</strong><div><span>${gradeDisplay}</span><small>${data.coverage}% evidence coverage · Rules cyvra_qc_condition_v1</small></div></div>
          <p>${data.limitation}</p>
        </section>
        <section class="report-section">
          <h3>CYVRA Erase privacy-exposure result</h3>
          <p>Six metadata categories identified. Private content was not collected. No data was erased.</p>
        </section>
        <section class="report-section">
          <h3>Integrity and authenticity</h3>
          <div class="report-columns">
            <div class="fact"><span>Evidence manifest</span><strong>SHA-256 · 83b9…24e1</strong></div>
            <div class="fact"><span>Authenticity</span><strong>Verification pending</strong></div>
          </div>
        </section>
        <p>CYVRA QC Grade describes assessed device condition at the recorded assessment time. It is not a resale valuation or warranty.</p>
      </article>
      <div class="button-row centered">
        <button class="button button-primary" type="button" data-action="toast" data-message="Prototype only: report generation is not connected.">Generate report</button>
        ${action("Finish", "completion", false)}
      </div>
    </div>`;
}

function renderCompletion() {
  return `
    <div class="screen-grid">
      <section class="hero-panel">
        <span class="status-tag"><b aria-hidden="true">✓</b> Verification completed safely</span>
        <h2>Your device assessment is complete</h2>
        <p>CYVRA preserved the evidence summary and recorded all known limitations. No data was erased.</p>
        <div class="summary-grid">
          <div class="summary-tile"><strong>Grade B — Good</strong><span>CYVRA QC condition result</span></div>
          <div class="summary-tile"><strong>Assessment complete</strong><span>CYVRA Erase privacy-exposure result</span></div>
          <div class="summary-tile"><strong>Report preview</strong><span>Authentication not connected in this prototype</span></div>
          <div class="summary-tile"><strong>24 Aug 2026</strong><span>Recorded assessment date</span></div>
        </div>
        <div class="button-row">${action("Return to overview", "overview")}${action("Open report", "report", false)}</div>
      </section>
      <aside class="panel">
        <h2>Recommended next action</h2>
        <p>Review the evidence and limitations before the device changes hands. Any future destructive lifecycle requires separate authorization and a supported CYVRA release.</p>
        ${notice("warning", "!", "A completed assessment is not proof that personal data was erased.")}
      </aside>
    </div>`;
}

function renderHelp() {
  const network = state.scenario === "network";
  return `
    <div class="screen-grid">
      <section class="hero-panel compact">
        <span class="kicker">HELP AND RECOVERY</span>
        <h2>${network ? "Connection could not be verified" : "How can we help?"}</h2>
        <p>Recovery guidance explains what happened, what remains safe and what the customer can do next.</p>
        <div class="scope-grid">
          <button class="scope-card" type="button" data-action="toast" data-message="Activation recovery guidance opened."><strong>Activation and binding</strong><span>Entitlement, same-device validation and support reset</span></button>
          <button class="scope-card" type="button" data-action="toast" data-message="Evidence limitation guidance opened."><strong>Evidence limitations</strong><span>Permission denied, unsupported and collection error states</span></button>
          <button class="scope-card" type="button" data-action="toast" data-message="Report guidance opened."><strong>Report and authenticity</strong><span>Preview, verification and tamper-safe support</span></button>
          <button class="scope-card" type="button" data-action="toast" data-message="Privacy-safe diagnostic preview opened."><strong>Privacy-safe diagnostics</strong><span>Preview approved categories before export</span></button>
        </div>
      </section>
      <aside class="panel">
        <h2>Support bundle preview</h2>
        <ul class="evidence-list">
          <li>${iconCheck()}<span>Application and schema versions</span></li>
          <li>${iconCheck()}<span>Privacy-safe collector status and error codes</span></li>
          <li>${iconCheck()}<span>Masked assessment reference</span></li>
          <li>${iconCheck()}<span>No raw serial, UUID, MAC address, key, token or personal content</span></li>
        </ul>
        <div class="support-code">Support code: ${network ? "NET-RETRY-204" : "CVQ-HELP-7F2C"}</div>
        <div class="button-row"><button class="button button-secondary" type="button" data-action="toast" data-message="Nothing was exported; this is a design prototype.">Preview diagnostics</button></div>
      </aside>
    </div>`;
}

function renderScreen() {
  const definition = screens.find((screen) => screen.id === state.screen) || screens[0];
  screenSelect.value = definition.id;
  screenTitle.textContent = definition.title;
  screenEyebrow.textContent = definition.eyebrow;

  const renderers = {
    welcome: renderWelcome,
    activate: renderActivate,
    binding: renderBinding,
    device: renderDevice,
    consent: renderConsent,
    ready: renderReady,
    progress: renderProgress,
    overview: renderOverview,
    results: renderResults,
    qc: renderQc,
    erase: renderErase,
    report: renderReport,
    completion: renderCompletion,
    help: renderHelp,
  };

  screenRoot.innerHTML = renderers[definition.id]();
  screenRoot.focus({ preventScroll: true });

  document.querySelectorAll("[data-nav-screen]").forEach((button) => {
    const navScreen = button.dataset.navScreen;
    const active = navScreen === state.screen || (navScreen === "results" && ["results", "qc", "erase"].includes(state.screen));
    button.classList.toggle("active", active);
    if (active) button.setAttribute("aria-current", "page");
    else button.removeAttribute("aria-current");
  });
}

function setScreen(screenId) {
  if (!screens.some((screen) => screen.id === screenId)) return;
  state.screen = screenId;
  renderScreen();
}

function showToast(message) {
  window.clearTimeout(toastTimer);
  toast.textContent = message;
  toast.hidden = false;
  toastTimer = window.setTimeout(() => {
    toast.hidden = true;
  }, 3200);
}

function openCancelModal(trigger) {
  returnFocus = trigger;
  cancelModal.hidden = false;
  document.querySelector("#continue-scan")?.focus();
}

function closeCancelModal() {
  cancelModal.hidden = true;
  returnFocus?.focus();
}

screenSelect.addEventListener("change", () => setScreen(screenSelect.value));

scenarioSelect.addEventListener("change", () => {
  state.scenario = scenarioSelect.value;
  renderScreen();
});

fidelityToggle.addEventListener("click", () => {
  state.wireframe = !state.wireframe;
  document.body.classList.toggle("wireframe", state.wireframe);
  fidelityToggle.setAttribute("aria-pressed", String(state.wireframe));
  fidelityToggle.textContent = state.wireframe ? "Show high fidelity" : "Show wireframe";
});

contrastToggle.addEventListener("click", () => {
  state.highContrast = !state.highContrast;
  document.body.classList.toggle("high-contrast", state.highContrast);
  contrastToggle.setAttribute("aria-pressed", String(state.highContrast));
});

document.addEventListener("change", (event) => {
  if (event.target instanceof HTMLInputElement && event.target.id === "scan-consent") {
    state.consent = event.target.checked;
    const button = document.querySelector("#consent-continue");
    if (button) button.disabled = !state.consent;
  }
});

document.addEventListener("click", (event) => {
  const target = event.target instanceof Element ? event.target.closest("button") : null;
  if (!target) return;

  if (target.dataset.screen) {
    setScreen(target.dataset.screen);
    return;
  }

  if (target.dataset.navScreen) {
    setScreen(target.dataset.navScreen);
    return;
  }

  const actionName = target.dataset.action;
  if (actionName === "start-progress") {
    state.progress = 0;
    setScreen("progress");
  } else if (actionName === "advance-progress") {
    if (state.progress >= progressStages.length) setScreen("results");
    else {
      state.progress += 1;
      renderScreen();
    }
  } else if (actionName === "open-cancel") {
    openCancelModal(target);
  } else if (actionName === "toast") {
    showToast(target.dataset.message || "Prototype action completed.");
  }
});

document.querySelector("#continue-scan").addEventListener("click", closeCancelModal);

document.querySelector("#confirm-cancel").addEventListener("click", () => {
  closeCancelModal();
  state.progress = 0;
  setScreen("ready");
  showToast("Verification cancelled safely. No completed grade or report was issued.");
});

cancelModal.addEventListener("click", (event) => {
  if (event.target === cancelModal) closeCancelModal();
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !cancelModal.hidden) closeCancelModal();
});

scenarioSelect.value = state.scenario;
renderScreen();
