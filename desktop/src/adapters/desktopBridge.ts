import { invoke, isTauri } from "@tauri-apps/api/core";
import { assertSafeShellBootstrap, type ShellBootstrap, type VerificationRecord } from "../types/shell";

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

export async function runDeviceVerification(): Promise<VerificationRecord> {
  if (!isTauri()) {
    throw new Error("Device verification runs only in the installed CYVRA Erase application.");
  }

  const response = await invoke<{
    ok: boolean;
    message: string;
    hardwareResult: string;
    hardwarePassed: boolean;
    hardwareValidation: string;
    reportJson: string;
    manufacturer: string;
    model: string;
    hostname: string;
    osCaption: string;
    personalLocationCount: number;
    pdemObjectCount: number;
    contentInspected: boolean;
    destructiveOperationsEnabled: boolean;
    assessmentStatus: string;
    assessmentSummary: string;
  }>("run_device_verification");

  if (!response.ok || response.destructiveOperationsEnabled || response.contentInspected) {
    throw new Error(response.message || "CYVRA stopped the assessment.");
  }

  return {
    hardwareResult: response.hardwareResult,
    hardwarePassed: response.hardwarePassed,
    hardwareValidation: response.hardwareValidation,
    reportJson: response.reportJson,
    manufacturer: response.manufacturer,
    model: response.model,
    hostname: response.hostname,
    osCaption: response.osCaption,
    personalLocationCount: response.personalLocationCount,
    pdemObjectCount: response.pdemObjectCount,
    contentInspected: response.contentInspected,
    destructiveOperationsEnabled: response.destructiveOperationsEnabled,
    assessmentStatus: response.assessmentStatus,
    assessmentSummary: response.assessmentSummary,
    message: response.message,
  };
}
