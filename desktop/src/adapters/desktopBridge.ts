import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  assertSafeShellBootstrap,
  type ScanTarget,
  type ShellBootstrap,
  type VerificationProgress,
  type VerificationRecord,
} from "../types/shell";

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

const PREVIEW_SCAN_TARGETS: ScanTarget[] = [
  {
    letter: "C",
    label: "Windows",
    kind: "Internal disk",
    sizeLabel: "System drive",
    defaultSelected: true,
    hint: "This is the Windows system drive. Recommended for every assessment.",
  },
  {
    letter: "D",
    label: "Backup disk",
    kind: "Removable or USB",
    sizeLabel: "External",
    defaultSelected: false,
    hint: "Left off by default. Select this only if you want it included.",
  },
];

export async function loadShellBootstrap(): Promise<ShellBootstrap> {
  if (!isTauri()) {
    return BROWSER_FOUNDATION_BOOTSTRAP;
  }

  const response = await invoke<unknown>("get_shell_bootstrap");
  return assertSafeShellBootstrap(response);
}

export async function listScanTargets(): Promise<ScanTarget[]> {
  if (!isTauri()) {
    return PREVIEW_SCAN_TARGETS;
  }

  const response = await invoke<ScanTarget[]>("list_scan_targets");
  return Array.isArray(response) ? response : [];
}

export async function runDeviceVerification(driveLetters: string[]): Promise<VerificationRecord> {
  if (!isTauri()) {
    throw new Error("Device verification runs only in the installed CYVRA Erase application.");
  }

  const response = await invoke<{
    ok: boolean;
    message: string;
    hardwareResult: string;
    hardwarePassed: boolean;
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
    scannedDrives: string;
    hardwareFields: { label: string; value: string }[];
    locationGroups: { label: string; value: string }[];
  }>("run_device_verification", { driveLetters });

  if (!response.ok || response.destructiveOperationsEnabled || response.contentInspected) {
    throw new Error(response.message || "CYVRA stopped the assessment.");
  }

  return {
    hardwareResult: response.hardwareResult,
    hardwarePassed: response.hardwarePassed,
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
    scannedDrives: response.scannedDrives,
    hardwareFields: response.hardwareFields ?? [],
    locationGroups: response.locationGroups ?? [],
    message: response.message,
  };
}

export async function subscribeVerificationProgress(
  onProgress: (progress: VerificationProgress) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return () => undefined;
  }

  const unlisten = await listen<VerificationProgress>("verification-progress", (event) => {
    onProgress(event.payload);
  });
  return unlisten;
}

export async function closeApplication(): Promise<void> {
  if (!isTauri()) {
    window.close();
    return;
  }

  await invoke("close_window");
}
