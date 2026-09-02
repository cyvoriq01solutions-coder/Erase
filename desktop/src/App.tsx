import { useEffect, useState } from "react";
import {
  closeApplication,
  listScanTargets,
  loadShellBootstrap,
  runAdvanceScan,
  runDeviceVerification,
  subscribeVerificationProgress,
} from "./adapters/desktopBridge";
import { AppFrame } from "./components/AppFrame";
import { InstallerSetup } from "./components/InstallerSetup";
import { ShellScreen } from "./screens/ShellScreens";
import type {
  AdvanceScanConsent,
  AdvanceScanPhase,
  AdvanceScanRecord,
  BridgeState,
  NavigationId,
  ScanTarget,
  VerificationPhase,
  VerificationProgress,
  VerificationRecord,
} from "./types/shell";

export default function App() {
  const [setupComplete, setSetupComplete] = useState(false);
  const [current, setCurrent] = useState<NavigationId>("overview");
  const [bridge, setBridge] = useState<BridgeState>({ status: "loading" });
  const [verificationPhase, setVerificationPhase] = useState<VerificationPhase>("idle");
  const [verification, setVerification] = useState<VerificationRecord | null>(null);
  const [verificationError, setVerificationError] = useState<string | null>(null);
  const [progress, setProgress] = useState<VerificationProgress | null>(null);
  const [scanTargets, setScanTargets] = useState<ScanTarget[]>([]);
  const [selectedDrives, setSelectedDrives] = useState<string[]>([]);
  const [advanceScan, setAdvanceScan] = useState<AdvanceScanRecord | null>(null);
  const [advanceScanPhase, setAdvanceScanPhase] = useState<AdvanceScanPhase>("idle");
  const [advanceScanError, setAdvanceScanError] = useState<string | null>(null);
  const [advanceConsent, setAdvanceConsent] = useState<AdvanceScanConsent>({
    benchmarks: false,
    writeBenchmark: false,
  });

  useEffect(() => {
    let active = true;

    loadShellBootstrap()
      .then((bootstrap) => {
        if (active) setBridge({ status: "ready", bootstrap });
      })
      .catch(() => {
        if (active) {
          setBridge({
            status: "error",
            safeMessage: "CYVRA could not verify the internal application boundary.",
          });
        }
      });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!setupComplete) return;
    let active = true;

    listScanTargets()
      .then((targets) => {
        if (!active) return;
        setScanTargets(targets);
        setSelectedDrives((currentLetters) => {
          if (currentLetters.length > 0) return currentLetters;
          if (targets.length === 0) return ["C"];
          const defaults = targets.filter((target) => target.defaultSelected).map((target) => target.letter);
          return defaults.length > 0 ? defaults : [targets[0].letter];
        });
      })
      .catch(() => {
        if (active) setScanTargets([]);
      });

    return () => {
      active = false;
    };
  }, [setupComplete]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void subscribeVerificationProgress((next) => {
      setProgress(next);
    }).then((stop) => {
      unlisten = stop;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  async function handleRunVerification() {
    setVerificationError(null);
    setProgress({
      percent: 2,
      stageIndex: 0,
      stage: "Preparing verification",
      detail: "Starting a local assessment on the selected drives.",
    });
    setVerificationPhase("running");
    try {
      const record = await runDeviceVerification(selectedDrives);
      setVerification(record);
      setProgress({
        percent: 100,
        stageIndex: 7,
        stage: "Preparing results",
        detail: "The local assessment is ready to review.",
      });
      setVerificationPhase("complete");
      setCurrent("results");
    } catch (error) {
      setVerificationPhase("error");
      setVerificationError(
        error instanceof Error ? error.message : "CYVRA could not complete device verification.",
      );
    }
  }

  async function handleRunAdvanceScan() {
    setAdvanceScanError(null);
    setAdvanceScanPhase("running");
    try {
      const record = await runAdvanceScan(advanceConsent);
      setAdvanceScan(record);
      setAdvanceScanPhase("complete");
    } catch (error) {
      setAdvanceScanPhase("error");
      setAdvanceScanError(
        error instanceof Error ? error.message : "CYVRA could not complete Advance scan.",
      );
    }
  }

  function handleToggleAdvanceConsent(field: keyof AdvanceScanConsent) {
    if (advanceScanPhase === "running") return;
    setAdvanceConsent((current) => {
      const next = { ...current, [field]: !current[field] };
      // A write test is meaningless without benchmarks, so it never stays on alone.
      if (!next.benchmarks) {
        next.writeBenchmark = false;
      }
      return next;
    });
  }

  function handleToggleDrive(letter: string) {
    if (verificationPhase === "running") return;
    setSelectedDrives((currentLetters) =>
      currentLetters.includes(letter)
        ? currentLetters.filter((item) => item !== letter)
        : [...currentLetters, letter],
    );
  }

  if (bridge.status === "loading") {
    return (
      <main className="setup-shell">
        <div className="setup-card">
          <span className="eyebrow">CYVRA ERASE</span>
          <h1>Starting CYVRA Erase</h1>
          <p>Checking the application safety boundary.</p>
        </div>
      </main>
    );
  }

  if (!setupComplete) {
    return (
      <InstallerSetup
        liveActivationEnabled={
          bridge.status === "ready" && bridge.bootstrap.liveActivationEnabled
        }
        onFinished={() => setSetupComplete(true)}
      />
    );
  }

  return (
    <AppFrame
      bridge={bridge}
      current={current}
      onNavigate={setCurrent}
      verificationPhase={verificationPhase}
      progress={progress}
      onExit={() => {
        void closeApplication();
      }}
    >
      <ShellScreen
        current={current}
        bridge={bridge}
        verificationPhase={verificationPhase}
        verification={verification}
        verificationError={verificationError}
        progress={progress}
        scanTargets={scanTargets}
        selectedDrives={selectedDrives}
        onToggleDrive={handleToggleDrive}
        onNavigate={setCurrent}
        onRunVerification={() => {
          void handleRunVerification();
        }}
        advanceScan={advanceScan}
        advanceScanPhase={advanceScanPhase}
        advanceScanError={advanceScanError}
        advanceConsent={advanceConsent}
        onToggleAdvanceConsent={handleToggleAdvanceConsent}
        onRunAdvanceScan={() => {
          void handleRunAdvanceScan();
        }}
        onExit={() => {
          void closeApplication();
        }}
      />
    </AppFrame>
  );
}
