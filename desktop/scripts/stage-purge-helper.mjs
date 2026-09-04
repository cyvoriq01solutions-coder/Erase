import { mkdirSync, copyFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const desktopRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const agentManifest = join(desktopRoot, "..", "agent-windows", "Cargo.toml");
const binariesDir = join(desktopRoot, "src-tauri", "binaries");
const isWindows = process.platform === "win32";
const builtName = isWindows ? "cyvra-purge-helper.exe" : "cyvra-purge-helper";
const destName = "cyvra-purge-helper.exe";

const build = spawnSync(
  "cargo",
  ["build", "--manifest-path", agentManifest, "--bin", "cyvra-purge-helper", "--release"],
  { stdio: "inherit", shell: isWindows },
);

if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

const source = join(desktopRoot, "..", "agent-windows", "target", "release", builtName);
if (!existsSync(source)) {
  console.error(`Purge helper was not built at ${source}`);
  process.exit(1);
}

mkdirSync(binariesDir, { recursive: true });
copyFileSync(source, join(binariesDir, destName));
console.log(`Staged ${destName} for the CYVRA installer.`);
