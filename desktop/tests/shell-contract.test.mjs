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
  assert.match(combined, /"run_device_verification"/);
  assert.match(combined, /"list_scan_targets"/);
  assert.match(combined, /"run_advance_scan"/);
  assert.match(combined, /"close_window"/);
  assert.equal(occurrences(combined, /invoke<|invoke\(/g), 6);

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
  assert.equal(occurrences(rust, /#\[tauri::command\]/g), 6);
  assert.match(rust, /activate_license/);
  assert.match(rust, /run_device_verification/);
  assert.match(rust, /list_scan_targets/);
  assert.match(rust, /run_advance_scan/);
  assert.match(rust, /close_window/);
  assert.match(rust, /TypeId::of::<cyvra_core::CollectorError>/);

  for (const flag of [
    "destructive_operations_enabled",
    "grading_issuance_enabled",
    "report_authentication_enabled",
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
  assert.match(combined, /advance-progress/);
  assert.match(combined, /AdvanceProgressRing|advance-ring/);
  assert.match(combined, /Save Report D as PDF/);
  assert.match(combined, /Coverage by diagnostic area/);
  assert.match(combined, /Grading rubric/);
  assert.match(combined, /Method and limitations/);
  assert.match(combined, /inferDeviceForm/);
  assert.match(combined, /advance-scan-progress/);

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
  assert.match(licence, /not a customer\s+release/i);
  assert.doesNotMatch(licence, /erase customer files/i);
});

test("root workspace exposes bounded desktop checks", () => {
  const packageJson = JSON.parse(readFileSync(join(repositoryRoot, "package.json"), "utf8"));

  assert.equal(packageJson.scripts["desktop:check"], "npm --prefix desktop run check");
  assert.equal(packageJson.scripts["desktop:test"], "npm --prefix desktop test");
  assert.equal(packageJson.scripts["desktop:build"], "npm --prefix desktop run build");
});
