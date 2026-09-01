import { useEffect, useState } from "react";
import { loadShellBootstrap, runDeviceVerification } from "./adapters/desktopBridge";
import { AppFrame } from "./components/AppFrame";
import { InstallerSetup } from "./components/InstallerSetup";
import { ShellScreen } from "./screens/ShellScreens";
import type { BridgeState, NavigationId, VerificationPhase, VerificationRecord } from "./types/shell";

export default function App() {
  const [setupComplete, setSetupComplete] = useState(false);
  const [current, setCurrent] = useState<NavigationId>("overview");
  const [bridge, setBridge] = useState<BridgeState>({ status: "loading" });
  const [verificationPhase, setVerificationPhase] = useState<VerificationPhase>("idle");
  const [verification, setVerification] = useState<VerificationRecord | null>(null);
  const [verificationError, setVerificationError] = useState<string | null>(null);

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

  async function handleRunVerification() {
    setVerificationError(null);
    setVerificationPhase("running");
    try {
      const record = await runDeviceVerification();
      setVerification(record);
      setVerificationPhase("complete");
      setCurrent("results");
    } catch (error) {
      setVerificationPhase("error");
      setVerificationError(
        error instanceof Error ? error.message : "CYVRA could not complete device verification.",
      );
    }
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
    >
      <ShellScreen
        current={current}
        bridge={bridge}
        verificationPhase={verificationPhase}
        verification={verification}
        verificationError={verificationError}
        onNavigate={setCurrent}
        onRunVerification={() => {
          void handleRunVerification();
        }}
      />
    </AppFrame>
  );
}
