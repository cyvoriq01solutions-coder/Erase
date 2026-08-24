import { useEffect, useState } from "react";
import { loadShellBootstrap } from "./adapters/desktopBridge";
import { AppFrame } from "./components/AppFrame";
import { ShellScreen } from "./screens/ShellScreens";
import type { BridgeState, NavigationId } from "./types/shell";

export default function App() {
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

  return (
    <AppFrame bridge={bridge} current={current} onNavigate={setCurrent}>
      <ShellScreen current={current} bridge={bridge} onNavigate={setCurrent} />
    </AppFrame>
  );
}
