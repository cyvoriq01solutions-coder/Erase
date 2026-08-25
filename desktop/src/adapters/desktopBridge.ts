import { invoke, isTauri } from "@tauri-apps/api/core";
import { assertSafeShellBootstrap, type ShellBootstrap } from "../types/shell";

const BROWSER_FOUNDATION_BOOTSTRAP: ShellBootstrap = Object.freeze({
  appVersion: "0.3.0-browser-preview",
  runtimeMode: "browser_design_adapter",
  coreBoundary: "native_bridge_not_loaded",
  destructiveOperationsEnabled: false,
  liveActivationEnabled: false,
  liveCollectionEnabled: false,
  gradingIssuanceEnabled: false,
  reportAuthenticationEnabled: false,
});

export async function loadShellBootstrap(): Promise<ShellBootstrap> {
  if (!isTauri()) {
    return BROWSER_FOUNDATION_BOOTSTRAP;
  }

  const response = await invoke<unknown>("get_shell_bootstrap");
  return assertSafeShellBootstrap(response);
}
