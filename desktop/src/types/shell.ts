export type NavigationId = "overview" | "verification" | "results" | "report" | "help";

export interface NavigationItem {
  id: NavigationId;
  label: string;
  shortLabel: string;
}

export const NAVIGATION_ITEMS: readonly NavigationItem[] = [
  { id: "overview", label: "Overview", shortLabel: "O" },
  { id: "verification", label: "Verification", shortLabel: "V" },
  { id: "results", label: "Results", shortLabel: "R" },
  { id: "report", label: "Report", shortLabel: "P" },
  { id: "help", label: "Help", shortLabel: "?" },
];

export interface ShellBootstrap {
  appVersion: string;
  runtimeMode: string;
  coreBoundary: string;
  destructiveOperationsEnabled: boolean;
  liveActivationEnabled: boolean;
  liveCollectionEnabled: boolean;
  gradingIssuanceEnabled: boolean;
  reportAuthenticationEnabled: boolean;
}

export type VerificationPhase = "idle" | "running" | "complete" | "error";

export interface NamedValue {
  label: string;
  value: string;
}

export interface ScanTarget {
  letter: string;
  label: string;
  kind: string;
  sizeLabel: string;
  defaultSelected: boolean;
  hint: string;
}

export interface VerificationProgress {
  percent: number;
  stageIndex: number;
  stage: string;
  detail: string;
}

export interface VerificationRecord {
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
  hardwareFields: NamedValue[];
  locationGroups: NamedValue[];
  message: string;
}

export type BridgeState =
  | { status: "loading" }
  | { status: "ready"; bootstrap: ShellBootstrap }
  | { status: "error"; safeMessage: string };

export type AdvanceScanPhase = "idle" | "running" | "complete" | "error";

/** Both permissions default to off and are asked for before Advance scan runs. */
export interface AdvanceScanConsent {
  benchmarks: boolean;
  writeBenchmark: boolean;
}

export interface TelemetryGroup {
  title: string;
  note: string | null;
  rows: NamedValue[];
}

export interface DomainCoverage {
  domain: string;
  awarded: number;
  assessed: number;
  notAssessable: number;
  weight: number;
  state: string;
  confidence: string;
  note: string;
}

export interface AdvanceScanProgress {
  percent: number;
  stageIndex: number;
  stage: string;
  detail: string;
}

export type AttestationValue = "skip" | "pass" | "fail";
export type PortAttestationValue = "skip" | "all_passed" | "partial" | "any_failed";

/** Banded CG-1.0 grades are software-observed until a technician verifies the device. */
export const SOFTWARE_OBSERVED_LABEL = "software-observed";

/** Phase-one technician checks. Defaults are not attempted, never zero-scored. */
export interface AdvanceInteractive {
  colourWash: AttestationValue;
  keyboard: AttestationValue;
  trackpad: AttestationValue;
  speakers: AttestationValue;
  capture: AttestationValue;
  physicalPorts: PortAttestationValue;
  liveUsb: string;
  livePower: string;
  liveCamera: string;
}

export const DEFAULT_ADVANCE_INTERACTIVE: AdvanceInteractive = {
  colourWash: "skip",
  keyboard: "skip",
  trackpad: "skip",
  speakers: "skip",
  capture: "skip",
  physicalPorts: "skip",
  liveUsb: "",
  livePower: "",
  liveCamera: "",
};

export interface LiveRemovableVolume {
  letter: string;
  label: string;
  sizeLabel: string;
}

export interface LivePowerStatus {
  present: boolean;
  onMains: boolean;
  charging: boolean;
  statusCode: number | null;
  statusLabel: string;
  chargePercent: number | null;
  available: boolean;
  detail: string;
}

export interface LiveIntakeProbe {
  removable: LiveRemovableVolume[];
  power: LivePowerStatus;
}

export type DeviceFormHint = "portable" | "fixed" | "unknown";

/** Ordered Advance scan stages. Must stay aligned with agent-windows diagnostics::STAGES. */
export const ADVANCE_SCAN_STAGES = [
  "Preparing advance scan",
  "Reading battery and power",
  "Reading processor and cache",
  "Reading memory",
  "Reading storage identity and health",
  "Reading ports and connectivity",
  "Reading display panel",
  "Reading cameras and microphones",
  "Running permitted benchmarks",
  "Scoring coverage",
  "Preparing Report D",
] as const;

/** Chassis type from Report A, when the basic assessment has already run. */
export function inferDeviceForm(verification: VerificationRecord | null): DeviceFormHint {
  if (!verification) {
    return "unknown";
  }
  const form = verification.hardwareFields
    .find((row) => row.label.toLowerCase() === "form factor")
    ?.value.toLowerCase()
    .trim();
  if (form === "laptop" || form === "tablet") {
    return "portable";
  }
  if (form === "desktop") {
    return "fixed";
  }
  return "unknown";
}

export interface AdvanceScanRecord {
  ok: boolean;
  message: string;
  schemaVersion: string;
  elevationState: string;
  elevationLabel: string;
  benchmarksConsented: boolean;
  writeBenchmarkConsented: boolean;
  bytesWritten: number;
  destructiveOperationsEnabled: boolean;
  contentInspected: boolean;
  boundaryNote: string;
  temporaryFilesNote: string;
  telemetryGroups: TelemetryGroup[];
  coverageRows: NamedValue[];
  coverageDomains: DomainCoverage[];
  methodRows: NamedValue[];
  rubricRows: NamedValue[];
  notAssessable: string[];
  gradingEngine: string;
  gradingRubric: string;
  gradeLabel: string;
  gradeCondition: string;
  gradeObservation: string | null;
  gradeWithheld: boolean;
  gradeWithheldReason: string | null;
  coveragePercent: number;
  indexPercent: number | null;
  provisional: boolean;
  issuanceNotice: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function assertSafeShellBootstrap(value: unknown): ShellBootstrap {
  if (!isRecord(value)) {
    throw new Error("Invalid shell bootstrap shape");
  }

  const stringKeys = ["appVersion", "runtimeMode", "coreBoundary"] as const;
  const booleanKeys = [
    "destructiveOperationsEnabled",
    "liveActivationEnabled",
    "liveCollectionEnabled",
    "gradingIssuanceEnabled",
    "reportAuthenticationEnabled",
  ] as const;

  for (const key of stringKeys) {
    if (typeof value[key] !== "string" || value[key].length === 0) {
      throw new Error(`Invalid shell bootstrap field: ${key}`);
    }
  }

  for (const key of booleanKeys) {
    if (typeof value[key] !== "boolean") {
      throw new Error(`Invalid shell bootstrap field: ${key}`);
    }
  }

  const lockedOff = [
    "destructiveOperationsEnabled",
    "gradingIssuanceEnabled",
    "reportAuthenticationEnabled",
  ] as const;

  if (lockedOff.some((key) => value[key] !== false)) {
    throw new Error("Unsafe capability enabled in CYVRA Erase");
  }

  return value as unknown as ShellBootstrap;
}
