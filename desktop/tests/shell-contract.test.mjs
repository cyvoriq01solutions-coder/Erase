import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";
import test from "node:test";

const desktopRoot = process.cwd();
const repositoryRoot = join(desktopRoot, "..");

function read(path) {
  return readFileSync(join(desktopRoot, path), "utf8");
}

function sourceFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? sourceFiles(path) : [path];
  });
}

function occurrences(source, pattern) {
  return source.match(pattern)?.length ?? 0;
}

test("desktop dependencies are exact and reproducible", () => {
  const packageJson = JSON.parse(read("package.json"));

  assert.deepEqual(packageJson.engines, {
    node: ">=24.0.0 <25.0.0",
    npm: ">=11.0.0 <12.0.0",
  });
  assert.equal(packageJson.dependencies["@tauri-apps/api"], "2.11.1");
  assert.equal(packageJson.dependencies.react, "19.2.8");
  assert.equal(packageJson.dependencies["react-dom"], "19.2.8");
  assert.equal(packageJson.devDependencies["@tauri-apps/cli"], "2.11.4");
  assert.equal(packageJson.devDependencies.typescript, "5.9.3");
  assert.equal(packageJson.devDependencies.vite, "8.2.0");
});

test("customer shell exposes exactly the five frozen destinations", () => {
  const navigation = read("src/types/shell.ts");
  const ids = [...navigation.matchAll(/\{ id: "([a-z]+)"/g)].map((match) => match[1]);

  assert.deepEqual(ids, ["overview", "verification", "results", "report", "help"]);
});

test("desktop uses the exact approved CYVORIQ logo asset", () => {
  const logo = readFileSync(join(desktopRoot, "src/assets/cyvoriq-logo.webp"));
  const blobHeader = Buffer.from(`blob ${logo.byteLength}\0`);
  const gitBlobSha = createHash("sha1").update(blobHeader).update(logo).digest("hex");

  assert.equal(gitBlobSha, "ed67aaa21c9af39f38371bcec5a6bca62365be26");
});

test("frontend bridge is narrow and has no network or persistence client", () => {
  const files = sourceFiles(join(desktopRoot, "src"));
  const sources = files.map((file) => ({
    path: relative(desktopRoot, file),
    source: readFileSync(file, "utf8"),
  }));
  const combined = sources.map(({ source }) => source).join("\n");

  assert.equal(occurrences(combined, /invoke<[^>]+>\("get_shell_bootstrap"\)/g), 1);
  assert.match(combined, /"activate_license"/);
  assert.match(combined, /"activate_purge_license"/);
  assert.match(combined, /"run_device_verification"/);
  assert.match(combined, /"list_scan_targets"/);
  assert.match(combined, /"probe_live_intake"/);
  assert.match(combined, /"run_advance_scan"/);
  assert.match(combined, /"run_mode_s_purge"/);
  assert.match(combined, /"close_window"/);
  assert.equal(occurrences(combined, /invoke<|invoke\(/g), 9);

  for (const forbidden of [
    /\bfetch\s*\(/,
    /\bXMLHttpRequest\b/,
    /\bWebSocket\b/,
    /\blocalStorage\b/,
    /\bsessionStorage\b/,
  ]) {
    assert.doesNotMatch(combined, forbidden);
  }
});

test("Rust command boundary links the reusable core and fails closed", () => {
  const cargo = read("src-tauri/Cargo.toml");
  const rust = read("src-tauri/src/lib.rs");

  assert.match(cargo, /cyvra-core\s*=\s*\{\s*package\s*=\s*"cyvoriq-erase-agent",\s*path\s*=\s*"\.\.\/\.\.\/agent-windows"\s*\}/);
  assert.equal(occurrences(rust, /#\[tauri::command\]/g), 9);
  assert.match(rust, /activate_license/);
  assert.match(rust, /activate_purge_license/);
  assert.match(rust, /run_device_verification/);
  assert.match(rust, /list_scan_targets/);
  assert.match(rust, /probe_live_intake/);
  assert.match(rust, /run_advance_scan/);
  assert.match(rust, /run_mode_s_purge/);
  assert.match(rust, /#\[allow\(clippy::too_many_arguments\)\]/);
  assert.match(rust, /close_window/);
  assert.match(rust, /TypeId::of::<cyvra_core::CollectorError>/);

  for (const flag of [
    "destructive_operations_enabled",
    "grading_issuance_enabled",
    "report_authentication_enabled",
    "purge_licence_bound",
  ]) {
    assert.match(rust, new RegExp(`${flag}: false`));
  }
  assert.match(rust, /live_activation_enabled: true/);
  assert.match(rust, /live_collection_enabled: true/);

  assert.doesNotMatch(rust, /std::process|std::fs|Command::new|remove_file|remove_dir/);
});

test("Tauri capability and webview policy remain least privilege", () => {
  const capability = JSON.parse(read("src-tauri/capabilities/main-window.json"));
  const configuration = JSON.parse(read("src-tauri/tauri.conf.json"));

  assert.deepEqual(capability.permissions, ["core:default"]);
  assert.equal(configuration.app.withGlobalTauri, false);
  assert.equal(configuration.app.security.freezePrototype, true);
  assert.equal(configuration.app.security.assetProtocol.enable, false);
  assert.equal(configuration.bundle.active, true);
  assert.deepEqual(configuration.bundle.targets, ["nsis"]);
  assert.equal(configuration.bundle.windows.nsis.installMode, "perMachine");
  assert.equal(configuration.bundle.windows.allowDowngrades, false);
  assert.equal(configuration.bundle.windows.webviewInstallMode.type, "downloadBootstrapper");
  assert.equal(configuration.bundle.publisher, "CYVORIQ Solutions");
  assert.equal(configuration.app.security.csp["default-src"], "'self'");
  assert.match(configuration.app.security.csp["connect-src"], /ipc:/);
  assert.equal(configuration.app.security.csp["object-src"], "'none'");
  assert.match(configuration.app.security.csp["img-src"], /blob:/);
  assert.match(configuration.app.security.csp["media-src"], /mediastream:/);
});

test("foundation does not ship destructive or obsolete customer wording", () => {
  const files = sourceFiles(join(desktopRoot, "src"));
  const combined = files.map((file) => readFileSync(file, "utf8")).join("\n");

  assert.doesNotMatch(combined, /XCQC/);
  assert.doesNotMatch(combined, />\s*(Erase now|Wipe device|Delete data|Bypass password)\s*</i);
  assert.doesNotMatch(combined, /Assessment JSON/);
  assert.doesNotMatch(combined, /Typed Rust/);
  assert.match(combined, /No data was erased/);
  assert.match(combined, /Generate report/);
  assert.match(combined, /Purge stays off|PURGE/);
  assert.match(combined, /Exit CYVRA Erase/);
  assert.match(combined, /Data purge/);
  assert.match(combined, /willingly and knowingly/);
  assert.match(combined, /button-danger/);
  assert.match(combined, /Save as PDF/);
  assert.match(combined, /pre-sanitization assessment/);
  assert.match(combined, /Battery health/);
  assert.match(combined, /not collected in this scan/i);
  assert.match(combined, /HDMI ports/);
  assert.doesNotMatch(combined, /Battery health: 0%/);
  assert.doesNotMatch(combined, /HDMI ports:\s*0/);
  assert.match(combined, /BIOS \/ OEM serial/);
  assert.match(combined, /Chassis serial/);
  assert.match(combined, /Motherboard serial/);
  assert.match(combined, /CYVORIQ Solutions Pvt/);
  assert.match(combined, /computer-generated/);
  assert.match(combined, /physical verification/);
  assert.doesNotMatch(combined, /Windows product key/i);
  assert.doesNotMatch(combined, /NIST certified/i);
  assert.doesNotMatch(combined, /This memory is verified/i);
  assert.doesNotMatch(combined, /no thermal throttling/i);
});

test("advance scan is opt-in, honest about gaps, and never claims an AI grade", () => {
  const files = sourceFiles(join(desktopRoot, "src"));
  const combined = files.map((file) => readFileSync(file, "utf8")).join("\n");
  const app = read("src/App.tsx");
  const bridge = read("src/adapters/desktopBridge.ts");
  const rust = read("src-tauri/src/lib.rs");

  assert.match(combined, /Advance scan/);
  assert.match(combined, /REPORT D/);
  assert.match(combined, /Graded by CYVRA Grading Engine/);
  assert.match(combined, /Assessed Health Index/);
  assert.match(combined, /not assessable/i);
  assert.match(combined, /physical verification/i);
  assert.match(combined, /This is not an issued CYVORIQ grading certificate/);
  assert.match(combined, /software-observed/);
  assert.match(combined, /issuanceNotice|issuance_notice/);
  assert.match(combined, /Coverage statement/);
  assert.match(combined, /advance-progress/);
  assert.match(combined, /AdvanceProgressRing|advance-ring/);
  assert.match(combined, /Save Report D as PDF/);
  assert.match(combined, /Coverage by diagnostic area/);
  assert.match(combined, /Grading rubric/);
  assert.match(combined, /Method and limitations/);
  assert.match(combined, /inferDeviceForm/);
  assert.match(combined, /advance-scan-progress/);
  assert.match(combined, /USB video service/i);
  assert.match(combined, /physically verified/i);
  assert.match(combined, /frames are not stored|not stored on Report D|was not stored/i);
  assert.match(combined, /live camera preview/i);
  assert.match(combined, /USB insertion/i);
  assert.match(combined, /charger/i);
  assert.match(combined, /BatteryStatus/);
  assert.match(combined, /Local integrity seal/);
  assert.match(combined, /Ed25519/);
  assert.match(combined, /SHA-256/);
  assert.match(combined, /Verify this report/);
  assert.match(combined, /cyvra-erd-ed25519-v1/);
  assert.doesNotMatch(combined, /Authenticated by CYVORIQ/);
  assert.match(combined, /storage SMART/i);
  assert.match(combined, /reliability counters/i);
  assert.match(combined, /EDID/);
  assert.match(combined, /Wi-Fi/);
  assert.match(combined, /MAC addresses are never printed/);
  assert.match(combined, /memory verified/i);
  assert.match(combined, /Allow benchmarks/);
  assert.match(combined, /Package temperature is not collected/);
  assert.match(combined, /processor identity/i);
  assert.match(combined, /Technician checks/);
  assert.match(combined, /colour wash/i);
  assert.match(combined, /Fn combinations/);
  assert.match(combined, /Keystrokes are not stored/);
  assert.match(combined, /live camera preview/i);
  assert.doesNotMatch(combined, /MacAddress/);

  // Both permissions must start off, and a write test cannot stand alone.
  assert.match(app, /benchmarks:\s*false/);
  assert.match(app, /writeBenchmark:\s*false/);
  assert.match(app, /next\.writeBenchmark = false/);

  // The bridge refuses a result that reports a write the operator never allowed.
  assert.match(bridge, /bytesWritten > 0/);
  assert.match(rust, /grade_withheld/);
  assert.match(rust, /provisional/);

  for (const forbidden of [
    /CYVRA AI/i,
    /machine learning/i,
    /neural/i,
    /Certified by CYVORIQ/i,
    /Certificate of Grading/i,
  ]) {
    assert.doesNotMatch(combined, forbidden);
  }
});

test("NSIS installer licence exists and is assessment-only", () => {
  const licence = read("src-tauri/LICENSE.installer.txt");

  assert.match(licence, /assessment/i);
  assert.match(licence, /SOFTWARE LICENSE TERMS/);
  assert.match(licence, /unsigned engineering/i);
  assert.match(licence, /auth@cyvra.co.in/);
  assert.doesNotMatch(licence, /not a customer\s+release/i);
  assert.doesNotMatch(licence, /erase customer files/i);
  assert.doesNotMatch(licence, /\bNSIS\b/);
  assert.doesNotMatch(licence, /Backblaze/i);
});

test("F2 removes live USB and charger overlays and keeps three workstreams", () => {
  const interactive = read("src/screens/InteractiveChecks.tsx");
  const installer = read("src/components/InstallerSetup.tsx");
  const screens = read("src/screens/ShellScreens.tsx");
  const app = read("src/App.tsx");
  const diagnostic = read("src/report/diagnosticPdf.ts");
  const assessment = read("src/report/assessmentPdf.ts");
  const types = read("src/types/shell.ts");
  const collector = readFileSync(join(repositoryRoot, "agent-windows/src/collector_runtime.rs"), "utf8");
  const customer = [interactive, installer, screens, diagnostic, assessment].join("\n");

  assert.doesNotMatch(interactive, /probeLiveIntake/);
  assert.doesNotMatch(interactive, /setInterval/);
  assert.doesNotMatch(interactive, /button-usb/);
  assert.doesNotMatch(interactive, /Check USB ports/);
  assert.doesNotMatch(interactive, /Start charger check/);
  assert.match(interactive, /read once during Advance scan/);
  assert.match(types, /derivePhysicalPorts/);
  assert.match(types, /WorkstreamId/);
  assert.match(screens, /workstream-card-assessment/);
  assert.match(screens, /workstream-card-advance/);
  assert.match(screens, /workstream-card-purge/);
  assert.match(screens, /Back to main/);
  assert.match(screens, /01 · STANDARD ASSESSMENT/);
  assert.match(screens, /02 · ADVANCED DIAGNOSTIC|02 · ADVANCE DIAGNOSTIC/);
  assert.match(screens, /03 · DATA PURGE/);
  assert.match(assessment, /Intake & Pre-Sanitization Assessment Record/);
  assert.match(diagnostic, /Technical Diagnostic & Condition Evidence Record/);
  assert.match(app, /chooseWorkstream/);
  assert.match(installer, /setup-progress/);
  assert.match(installer, /CYVORIQ SOLUTIONS/);
  assert.match(screens, /Physical verification/);
  assert.match(diagnostic, /Physical verification/);
  assert.match(diagnostic, /Advance scan pass/);
  assert.match(collector, /-WindowStyle/);
  assert.match(collector, /0x0800_0000/);
  assert.doesNotMatch(customer, /_{8,}/);
  assert.doesNotMatch(customer, /webview/i);
  assert.doesNotMatch(customer, /Win32_Battery/);
  assert.doesNotMatch(customer, /powercfg/i);
  assert.doesNotMatch(customer, /\bNSIS\b/);
  assert.doesNotMatch(customer, /\bVite\b/);
  assert.doesNotMatch(customer, /Backblaze/i);
});

test("F4 surfaces the privacy exposure map without opening contents", () => {
  const screens = read("src/screens/ShellScreens.tsx");
  const assessment = read("src/report/assessmentPdf.ts");
  const types = read("src/types/shell.ts");
  const bridge = read("src/adapters/desktopBridge.ts");
  const rust = read("src-tauri/src/lib.rs");
  const core = readFileSync(join(repositoryRoot, "agent-windows/src/lib.rs"), "utf8");

  assert.match(types, /exposureMap/);
  assert.match(bridge, /exposureMap/);
  assert.match(rust, /exposure_map/);
  assert.match(core, /exposure_map/);
  assert.match(screens, /Privacy exposure map/i);
  assert.match(screens, /Where files are, what they are/);
  assert.match(screens, /File names and contents are not recorded/);
  assert.match(core, /content_inspected: false/);
  assert.equal(occurrences(rust, /#\[tauri::command\]/g), 9);
});

test("F5 shows all fourteen Report A sections on screen", () => {
  const screens = read("src/screens/ShellScreens.tsx");
  const assessment = read("src/report/assessmentPdf.ts");

  assert.match(assessment, /buildAssessmentSections/);
  assert.match(screens, /buildAssessmentSections/);
  assert.match(screens, /END OF REPORT A/);
  assert.match(assessment, /1\. Document Control/);
  assert.match(assessment, /2\. Purpose/);
  assert.match(assessment, /3\. Scope/);
  assert.match(assessment, /4\. Device Identity/);
  assert.match(assessment, /5\. Hardware Inventory/);
  assert.match(assessment, /6\. Deferred/);
  assert.match(assessment, /7\. Metadata-Based Data-Exposure/);
  assert.match(assessment, /8\. Privacy/);
  assert.match(assessment, /9\. Authorization/);
  assert.match(assessment, /10\. Assessment Findings/);
  assert.match(assessment, /11\. Evidence Limitations/);
  assert.match(assessment, /12\. Production Audit-Readiness/);
  assert.match(assessment, /13\. Issuing Organisation/);
  assert.match(assessment, /14\. Verification/);
  assert.match(assessment, /Not recorded in this version/);
  assert.match(assessment, /END OF REPORT A/);
});

test("F6 shows numbered Report D sections on screen", () => {
  const screens = read("src/screens/ShellScreens.tsx");
  const diagnostic = read("src/report/diagnosticPdf.ts");

  assert.match(diagnostic, /buildDiagnosticSections/);
  assert.match(screens, /buildDiagnosticSections/);
  assert.match(screens, /END OF REPORT D/);
  assert.match(diagnostic, /1\. Document Control/);
  assert.match(diagnostic, /2\. Evidence Status/);
  assert.match(diagnostic, /3\. Device Identity/);
  assert.match(diagnostic, /4\. Coverage statement/);
  assert.match(diagnostic, /5\. Coverage by diagnostic area/);
  assert.match(diagnostic, /6\. Key Findings/);
  assert.match(diagnostic, /15\. Method and limitations/);
  assert.match(diagnostic, /16\. Grading rubric/);
  assert.match(diagnostic, /17\. Local Integrity Evidence/);
  assert.match(diagnostic, /18\. Audit-Ready/);
  assert.match(diagnostic, /19\. Final Recommended Next Action/);
  assert.match(diagnostic, /20\. Controlled Issuance Statement/);
  assert.match(diagnostic, /END OF REPORT D/);
});

test("S-setup shows commercial Software License Terms", () => {
  const installer = read("src/components/InstallerSetup.tsx");
  const licence = read("src-tauri/LICENSE.installer.txt");
  const styles = read("src/styles.css");

  assert.match(installer, /SOFTWARE LICENSE TERMS/);
  assert.match(installer, /GRANT OF LICENCE/);
  assert.match(installer, /LIMITATION OF LIABILITY/);
  assert.match(installer, /I accept these Software License Terms/);
  assert.match(installer, /Signing in on the website is not a licence|Website sign-in, OTP, or account approval is not a licence/);
  assert.match(installer, /one authorised Windows device/);
  assert.match(installer, /auth@cyvra.co.in/);
  assert.match(installer, /pre-sanitization assessment/);
  assert.match(installer, /This build is unsigned until Authenticode/);
  assert.match(licence, /SOFTWARE LICENSE TERMS/);
  assert.match(licence, /GRANT OF LICENCE/);
  assert.match(licence, /LIMITATION OF LIABILITY/);
  assert.match(licence, /auth@cyvra.co.in/);
  assert.match(licence, /pre-sanitization assessment/);
  assert.doesNotMatch(installer, /not a customer\s+release/i);
  assert.doesNotMatch(licence, /not a customer\s+release/i);
  assert.match(styles, /max-height:\s*320px/);
});

test("P-SECONDARY runs Mode S on extra disks and issues Report S only after verify PASS", () => {
  const screens = read("src/screens/ShellScreens.tsx");
  const rust = read("src-tauri/src/lib.rs");
  const bridge = read("src/adapters/desktopBridge.ts");
  const types = read("src/types/shell.ts");
  const pdf = read("src/report/sanitizationPdf.ts");
  const styles = read("src/styles.css");
  const tauriConf = read("src-tauri/tauri.conf.json");
  const buildRs = read("src-tauri/build.rs");
  const core = readFileSync(join(repositoryRoot, "agent-windows/src/purge/mod.rs"), "utf8");
  const helper = readFileSync(join(repositoryRoot, "agent-windows/src/bin/cyvra-purge-helper.rs"), "utf8");

  assert.match(screens, /Activate Purge/);
  assert.match(screens, /CYVRA-PRG-/);
  assert.match(screens, /Type ERASE in capital letters/);
  assert.match(screens, /Save Report S as PDF/);
  assert.match(screens, /USB stays off until you opt in/);
  assert.match(screens, /system disk — Mode S refused/);
  assert.match(screens, /never CERTIFIED SECURE/);
  assert.match(screens, /No data was erased/);
  assert.match(bridge, /activate_purge_license/);
  assert.match(bridge, /run_mode_s_purge/);
  assert.match(rust, /auth\/activate-purge/);
  assert.match(rust, /run_mode_s_purge/);
  assert.match(rust, /purge_licence_bound: false/);
  assert.match(rust, /destructive_operations_enabled: false/);
  assert.doesNotMatch(rust, /std::process|std::fs|Command::new|remove_file|remove_dir/);
  assert.match(types, /purgeLicenceBound/);
  assert.match(pdf, /Report S — Sanitization/);
  assert.match(pdf, /locally verified, not a laboratory certification/);
  assert.doesNotMatch(pdf, /Authenticated by CYVORIQ/);
  assert.doesNotMatch(pdf, /CERTIFIED SECURE/);
  assert.match(styles, /--purge: #b45309/);
  assert.match(styles, /--purge-pass: #0f766e/);
  assert.match(core, /Type ERASE in capital letters/);
  assert.match(helper, /--plan/);
  assert.match(tauriConf, /cyvra-purge-helper/);
  assert.match(tauriConf, /stage:purge-helper/);
  assert.match(buildRs, /cyvra-purge-helper\.exe/);
  assert.match(buildRs, /if !helper.exists/);
  assert.doesNotMatch(screens, /Authenticated by CYVORIQ/);
});

test("root workspace exposes bounded desktop checks", () => {
  const packageJson = JSON.parse(readFileSync(join(repositoryRoot, "package.json"), "utf8"));

  assert.equal(packageJson.scripts["desktop:check"], "npm --prefix desktop run check");
  assert.equal(packageJson.scripts["desktop:test"], "npm --prefix desktop test");
  assert.equal(packageJson.scripts["desktop:build"], "npm --prefix desktop run build");
});
