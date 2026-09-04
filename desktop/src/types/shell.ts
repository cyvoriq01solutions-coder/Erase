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

export type WorkstreamId = "assessment" | "advance" | "purge";

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
  exposureMap: ExposureEntry[];
  message: string;
}

export interface ExposureEntry {
  folder: string;
  category: string;
  files: number;
  bytes: number;
  sizeLabel: string;
  classification: string;
  confidence: string;
  contentInspected: boolean;
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
export type UsbPortMark = "skip" | "pass" | "fail" | "absent";

export const USB_PORT_LABELS = ["USB 1", "USB 2", "USB 3", "USB 4"] as const;

export interface UsbPortState {
  id: 1 | 2 | 3 | 4;
  mark: UsbPortMark;
  volumeLetter: string;
  speedLabel: string;
}

export function defaultUsbPorts(): UsbPortState[] {
  return [
    { id: 1, mark: "skip", volumeLetter: "", speedLabel: "" },
    { id: 2, mark: "skip", volumeLetter: "", speedLabel: "" },
    { id: 3, mark: "skip", volumeLetter: "", speedLabel: "" },
    { id: 4, mark: "skip", volumeLetter: "", speedLabel: "" },
  ];
}

export function usbPortMarkLabel(mark: UsbPortMark): string {
  if (mark === "pass") {
    return "Pass";
  }
  if (mark === "fail") {
    return "Fail";
  }
  if (mark === "absent") {
    return "Not on this PC";
  }
  return "Not attempted";
}

/** Four chassis ticks can still derive the CG-1.0 physical-port band. F2 does not run live USB ticks; absent sockets are not failures. */
export function derivePhysicalPorts(ports: UsbPortState[]): PortAttestationValue {
  const onChassis = ports.filter((port) => port.mark !== "absent");
  const scored = onChassis.filter((port) => port.mark === "pass" || port.mark === "fail");
  if (scored.some((port) => port.mark === "fail")) {
    return "any_failed";
  }
  if (scored.length === 0) {
    return "skip";
  }
  if (onChassis.length > 0 && onChassis.every((port) => port.mark === "pass")) {
    return "all_passed";
  }
  return "partial";
}

export function summariseUsbPort(port: UsbPortState): string {
  const parts = [`USB ${port.id}: ${usbPortMarkLabel(port.mark)}`];
  if (port.volumeLetter.trim()) {
    const letter = port.volumeLetter.trim().replace(/:$/, "");
    parts.push(`${letter}:`);
  }
  if (port.speedLabel.trim()) {
    parts.push(port.speedLabel.trim());
  }
  return parts.join(" · ");
}

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
  usbPorts: UsbPortState[];
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
  usbPorts: defaultUsbPorts(),
  liveUsb: "",
  livePower: "",
  liveCamera: "",
};

export interface LiveRemovableVolume {
  letter: string;
  label: string;
  sizeLabel: string;
  speedLabel: string;
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
  integritySeal: IntegritySeal | null;
}

export interface IntegritySeal {
  scheme: string;
  digestHex: string;
  publicKeyHex: string;
  signatureHex: string;
  qrPayload: string;
  qrSvg: string;
  canonicalJson: string;
  notice: string;
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
