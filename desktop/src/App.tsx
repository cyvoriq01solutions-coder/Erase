import { useEffect, useState } from "react";
import { loadShellBootstrap } from "./adapters/desktopBridge";
import { AppFrame } from "./components/AppFrame";
import { InstallerSetup } from "./components/InstallerSetup";
import { ShellScreen } from "./screens/ShellScreens";
import type { BridgeState, NavigationId } from "./types/shell";

export default function App() {
  const [setupComplete, setSetupComplete] = useState(false);
  const [current, setCurrent] = useState<NavigationId>("overview");
  const [bridge, setBridge] = useState<BridgeState>({ status: "loading" });

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
    <AppFrame bridge={bridge} current={current} onNavigate={setCurrent}>
      <ShellScreen current={current} bridge={bridge} onNavigate={setCurrent} />
    </AppFrame>
  );
}
