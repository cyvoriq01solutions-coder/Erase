import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ADVANCE_SCAN_STAGES,
  assertSafeShellBootstrap,
  type AdvanceScanConsent,
  type AdvanceScanProgress,
  type AdvanceScanRecord,
  type DeviceFormHint,
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

const advancePreviewListeners = new Set<(progress: AdvanceScanProgress) => void>();

function emitAdvancePreview(progress: AdvanceScanProgress): void {
  for (const listener of advancePreviewListeners) {
    listener(progress);
  }
}

function previewAdvanceRecord(consent: AdvanceScanConsent): AdvanceScanRecord {
  const notCollected =
    "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.";
  const needsKernel =
    "Not collected in this scan. This value requires a kernel-mode sensor driver, which CYVRA deliberately does not ship.";
  const declined = "Declined by the operator. No benchmark was run and nothing was written.";
  const notAttempted = "Not attempted in this scan. A technician records this at physical verification.";
  const benchmarkValue = consent.benchmarks ? notCollected : declined;
  const group = (title: string, rows: Array<[string, string]>, note: string | null = null) => ({
    title,
    note,
    rows: rows.map(([label, value]) => ({ label, value })),
  });

  return {
    ok: true,
    message:
      "Advance scan finished. Coverage 0%. No grade was issued because too little of this device could be assessed.",
    schemaVersion: "hardware_diagnostics_v1",
    elevationState: "not_requested",
    elevationLabel: "Administrator approval was not requested",
    benchmarksConsented: consent.benchmarks,
    writeBenchmarkConsented: consent.writeBenchmark,
    bytesWritten: 0,
    destructiveOperationsEnabled: false,
    contentInspected: false,
    boundaryNote:
      "Advance scan collection is read-only. Benchmarks were not permitted, so none were run. Nothing was written to any assessed drive. File contents were not opened. No data was erased. Purge stays off.",
    temporaryFilesNote: "No temporary file was created by this scan.",
    telemetryGroups: [
      group("Battery and power", [
        ["Battery probe", "Battery collection is only available on Windows."],
      ]),
      group(
        "Battery sources consulted",
        [
          ["Windows battery class", "Not queried on this PC"],
          ["Firmware static data", "Not queried on this PC"],
          ["Firmware full-charge capacity", "Not queried on this PC"],
          ["Firmware cycle count", "Not queried on this PC"],
          ["Windows battery report", "Not queried on this PC"],
        ],
        "Advance scan asks every source Windows offers and records the answer, so a missing value can be explained rather than guessed.",
      ),
      group("Processor and thermal", [
        ["Base clock", notCollected],
        ["Maximum clock", notCollected],
        ["Cache hierarchy", notCollected],
        ["Instruction sets", notCollected],
        ["Package temperature", needsKernel],
        ["Fan speed", needsKernel],
      ]),
      group("Memory", [
        ["Installed total", notCollected],
        ["Available", notCollected],
        ["Channel mode", notCollected],
      ]),
      group("Storage health and SMART", [
        ["Bus type", notCollected],
        ["Power-on hours", notCollected],
        ["Power cycles", notCollected],
        ["Total bytes written", notCollected],
        ["Percentage used", notCollected],
        ["Available spare", notCollected],
        ["Media errors", notCollected],
        ["Sectors pending reallocation", notCollected],
        ["Predicted failure", notCollected],
      ]),
      group(
        "Ports and connectivity",
        [
          ["USB controllers", "USB topology collection is only available on Windows."],
          ["USB hubs", "USB topology collection is only available on Windows."],
          ["USB controller ports", "USB topology collection is only available on Windows."],
          ["Negotiated port speeds", notCollected],
          ["Physically verified ports", notAttempted],
          ["Wi-Fi signal quality", "Display and radio collection is only available on Windows."],
          ["Wi-Fi link speed", "Display and radio collection is only available on Windows."],
          ["Bluetooth radio", "Display and radio collection is only available on Windows."],
          ["Ethernet link", "Display and radio collection is only available on Windows."],
        ],
        "Controller topology is not a count of plastic connectors. A port is confirmed only when a technician inserts a device. MAC addresses are never printed.",
      ),
      group(
        "USB sources consulted",
        [
          ["USB controllers", "Not queried on this PC"],
          ["USB hubs", "Not queried on this PC"],
          ["Attached USB devices", "Not queried on this PC"],
        ],
        "Advance scan walks USB controllers, hubs and attached devices. It does not guess empty sockets from SMBIOS labels.",
      ),
      group(
        "Radio sources consulted",
        [
          ["Wi-Fi adapter", "Not queried on this PC"],
          ["Bluetooth radio", "Not queried on this PC"],
          ["Ethernet adapter", "Not queried on this PC"],
        ],
        "Advance scan asks Windows for Wi-Fi, Bluetooth and Ethernet adapters. MAC addresses are dropped before they can be printed.",
      ),
      group(
        "Display panel",
        [
          ["Display probe", "Display and radio collection is only available on Windows."],
          ["Panel manufacturer", notCollected],
          ["Panel model", notCollected],
          ["Native resolution", notCollected],
          ["Refresh rate", notCollected],
          ["HDR capability", notCollected],
          ["Panel manufacture year", notCollected],
        ],
      ),
      group(
        "Display sources consulted",
        [
          ["Monitor identity", "Not queried on this PC"],
          ["EDID block", "Not queried on this PC"],
          ["Video controller", "Not queried on this PC"],
        ],
        "Advance scan reads monitor identity and the first 128 bytes of EDID. Native resolution is the preferred timing, not the current desktop mode.",
      ),
      group(
        "Cameras and microphones",
        [
          ["Capture probe", "Camera and microphone collection is only available on Windows."],
          ["Frames captured", "No"],
          ["Audio recorded", "No"],
          ["Camera image", notAttempted],
        ],
        "Enumeration only. No frame captured and no microphone audio is recorded in this adapter.",
      ),
      group(
        "Capture sources consulted",
        [
          ["PnP Camera class", "Not queried on this PC"],
          ["PnP Image class", "Not queried on this PC"],
          ["USB video service", "Not queried on this PC"],
          ["PnP Media class", "Not queried on this PC"],
          ["Audio endpoint class", "Not queried on this PC"],
          ["Windows sound device", "Not queried on this PC"],
        ],
        "The Camera ClassGuid alone misses some UVC webcams, so Advance scan also asks the USB video service and Image class.",
      ),
      group("Benchmarks", [
        ["Processor sustained clock", benchmarkValue],
        ["Memory pattern check", benchmarkValue],
        ["Sequential read", benchmarkValue],
        ["Random read", benchmarkValue],
        ["Write benchmark", consent.writeBenchmark ? notCollected : declined],
      ]),
      group("Technician checks", [
        ["Keyboard", notAttempted],
        ["Display inspection", notAttempted],
        ["Trackpad", notAttempted],
        ["Speakers", notAttempted],
        ["Camera image", notAttempted],
      ]),
    ],
    coverageRows: [
      { label: "Points in scope", value: "100" },
      { label: "Points assessed", value: "0" },
      { label: "Points awarded", value: "0" },
      { label: "Points not assessable", value: "100" },
      { label: "Coverage", value: "0%" },
      { label: "Assessed Health Index", value: "Not assessable in this scan" },
      { label: "Grading engine", value: "CYVRA Grading Engine" },
      { label: "Rubric", value: "CG-1.0" },
    ],
    coverageDomains: [
      {
        domain: "Battery and power",
        awarded: 0,
        assessed: 0,
        notAssessable: 20,
        weight: 20,
        state: "Not assessable",
        note: "The battery probe could not run on this PC",
      },
      {
        domain: "Processor and thermal stability",
        awarded: 0,
        assessed: 0,
        notAssessable: 20,
        weight: 20,
        state: "Not assessable",
        note: consent.benchmarks
          ? "Processor benchmark is not implemented in this collector version"
          : "Processor benchmark was declined by the operator",
      },
      {
        domain: "Memory integrity and speed",
        awarded: 0,
        assessed: 0,
        notAssessable: 15,
        weight: 15,
        state: "Not assessable",
        note: consent.benchmarks
          ? "Memory pattern check is not implemented in this collector version"
          : "Memory pattern check was declined by the operator",
      },
      {
        domain: "Storage health and SMART",
        awarded: 0,
        assessed: 0,
        notAssessable: 20,
        weight: 20,
        state: "Not assessable",
        note: "Storage SMART telemetry is not collected in this scan",
      },
      {
        domain: "Ports and connectivity",
        awarded: 0,
        assessed: 0,
        notAssessable: 10,
        weight: 10,
        state: "Not assessable",
        note: "Port topology and radio telemetry are not collected in this scan",
      },
      {
        domain: "Screen, keyboard and peripherals",
        awarded: 0,
        assessed: 0,
        notAssessable: 15,
        weight: 15,
        state: "Not assessable",
        note: "Interactive technician checks were not attempted in this scan",
      },
    ],
    methodRows: [
      {
        label: "Collection mode",
        value:
          "Read-only. Windows management classes, firmware tables, Windows' own battery report, storage reliability counters, EDID, and network adapters. MAC addresses are never collected.",
      },
      {
        label: "Battery capacity",
        value:
          "Design capacity and full-charge capacity as reported by firmware. Wear is the difference between them, never inferred from a charge level.",
      },
      {
        label: "Temperatures and fan speed",
        value:
          "Not collected. Reading CPU package temperature or fan RPM requires a kernel-mode sensor driver. CYVRA does not ship one, because the drivers commonly used for this are on Microsoft's vulnerable-driver blocklist.",
      },
      {
        label: "Memory testing",
        value:
          "A user-mode pattern check can never cover memory the kernel occupies, so full-coverage memory testing belongs to a pre-boot environment.",
      },
      {
        label: "Physical ports",
        value:
          "Windows exposes USB controller topology and attached devices, not the plastic connectors. A port is only confirmed when a technician inserts a device.",
      },
      {
        label: "Display panel",
        value:
          "Native width and height come from the EDID preferred timing, never from the current desktop mode. HDR is not guessed. Screen-domain points stay unawarded until a technician attests a colour wash.",
      },
      {
        label: "Radios",
        value:
          "Wi-Fi, Bluetooth and Ethernet adapters are enumerated without printing a MAC address. Signal quality is printed only when Windows returns it.",
      },
      {
        label: "Cameras and microphones",
        value:
          "Advance scan enumerates capture devices across several PnP classes, including the USB video service that the Camera ClassGuid misses. No frame is captured and no audio is recorded.",
      },
      {
        label: "Unknown values",
        value:
          "A value that was not read is printed as not collected. It is never replaced with zero and never estimated.",
      },
    ],
    rubricRows: [
      { label: "Battery and power", value: "20 points of 100" },
      { label: "Processor and thermal stability", value: "20 points of 100" },
      { label: "Memory integrity and speed", value: "15 points of 100" },
      { label: "Storage health and SMART", value: "20 points of 100" },
      { label: "Ports and connectivity", value: "10 points of 100" },
      { label: "Screen, keyboard and peripherals", value: "15 points of 100" },
      {
        label: "Grade bands",
        value: "A+ 90-100, A 80-89, B 65-79, C 50-64, F below 50, on the assessed index",
      },
      {
        label: "Coverage floor",
        value:
          "Below 70% coverage, or with a required area unassessed, the grade is withheld rather than banded",
      },
      {
        label: "Confirmed fault",
        value:
          "A measured critical fault forces F, because that is evidence held rather than evidence missing",
      },
    ],
    notAssessable: [
      "Battery and power — The battery probe could not run on this PC (20 of 20 points)",
      "Processor and thermal stability — Processor benchmark was declined by the operator (20 of 20 points)",
      "Memory integrity and speed — Memory pattern check was declined by the operator (15 of 15 points)",
      "Storage health and SMART — Storage SMART telemetry is not collected in this scan (20 of 20 points)",
      "Ports and connectivity — Port topology and radio telemetry are not collected in this scan (10 of 10 points)",
      "Screen, keyboard and peripherals — Interactive technician checks were not attempted in this scan (15 of 15 points)",
    ],
    gradingEngine: "CYVRA Grading Engine",
    gradingRubric: "CG-1.0",
    gradeLabel: "Grade withheld",
    gradeCondition: "Not enough of this device could be assessed",
    gradeWithheld: true,
    gradeWithheldReason:
      "Grade withheld. A required area could not be assessed in this scan: Storage health and SMART.",
    coveragePercent: 0,
    indexPercent: null,
    provisional: true,
  };
}

async function runAdvanceScanPreview(consent: AdvanceScanConsent): Promise<AdvanceScanRecord> {
  const details = [
    "Checking the Advance scan boundary and permissions.",
    "Asking Windows and the battery firmware for capacity.",
    "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.",
    "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.",
    "Not collected in this scan. Advance scan collection for this subsystem arrives in a later collector version.",
    "Walking USB controllers, hubs and attached devices. USB topology collection is only available on Windows.",
    "Reading panel identity from EDID. Display and radio collection is only available on Windows.",
    "Enumerating cameras and microphones. Camera and microphone collection is only available on Windows. No frame captured.",
    consent.benchmarks
      ? "Benchmarks were permitted, but none are implemented in this collector version."
      : "Benchmarks were not permitted, so none were run.",
    "Scoring only the areas that were actually assessed.",
    "Report D is ready. No grade was issued.",
  ];

  for (const [index, stage] of ADVANCE_SCAN_STAGES.entries()) {
    const percent = Math.round((index / (ADVANCE_SCAN_STAGES.length - 1)) * 100);
    emitAdvancePreview({
      percent,
      stageIndex: index,
      stage,
      detail: details[index] ?? stage,
    });
    await new Promise((resolve) => window.setTimeout(resolve, 160));
  }

  return previewAdvanceRecord(consent);
}

export async function runAdvanceScan(
  consent: AdvanceScanConsent,
  deviceForm: DeviceFormHint,
): Promise<AdvanceScanRecord> {
  if (!isTauri()) {
    return runAdvanceScanPreview(consent);
  }

  const response = await invoke<AdvanceScanRecord>("run_advance_scan", {
    benchmarksConsented: consent.benchmarks,
    writeBenchmarkConsented: consent.writeBenchmark,
    deviceForm,
  });

  if (!response.ok || response.destructiveOperationsEnabled || response.contentInspected) {
    throw new Error(response.message || "CYVRA stopped Advance scan.");
  }

  if (!consent.writeBenchmark && response.bytesWritten > 0) {
    throw new Error("CYVRA stopped Advance scan because a write was recorded without consent.");
  }

  return response;
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

export async function subscribeAdvanceScanProgress(
  onProgress: (progress: AdvanceScanProgress) => void,
): Promise<() => void> {
  if (!isTauri()) {
    advancePreviewListeners.add(onProgress);
    return () => {
      advancePreviewListeners.delete(onProgress);
    };
  }

  const unlisten = await listen<AdvanceScanProgress>("advance-scan-progress", (event) => {
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
