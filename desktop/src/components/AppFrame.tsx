import { useEffect, useRef, type ReactNode } from "react";
import logoUrl from "../assets/cyvoriq-logo.webp";
import {
  NAVIGATION_ITEMS,
  type BridgeState,
  type NavigationId,
  type VerificationPhase,
  type VerificationProgress,
} from "../types/shell";

interface AppFrameProps {
  bridge: BridgeState;
  current: NavigationId;
  onNavigate: (target: NavigationId) => void;
  verificationPhase: VerificationPhase;
  progress: VerificationProgress | null;
  onExit: () => void;
  children: ReactNode;
}

function bridgeLabel(bridge: BridgeState): string {
  if (bridge.status === "loading") return "Checking application safety";
  if (bridge.status === "error") return "Safety boundary unavailable";
  if (bridge.bootstrap.runtimeMode === "browser_design_adapter") return "Browser preview";
  return "Ready for this PC";
}

function footerState(phase: VerificationPhase, progress: VerificationProgress | null): string {
  if (phase === "complete") return "assessment complete";
  if (phase === "running") {
    const detail = progress?.detail ?? "verification running";
    return `${progress?.percent ?? 0}% · ${detail}`;
  }
  if (phase === "error") return "verification stopped";
  return "no verification started";
}

export function AppFrame({
  bridge,
  current,
  onNavigate,
  verificationPhase,
  progress,
  onExit,
  children,
}: AppFrameProps) {
  const workspaceRef = useRef<HTMLElement>(null);

  useEffect(() => {
    workspaceRef.current?.focus({ preventScroll: true });
  }, [current]);

  const version = bridge.status === "ready" ? bridge.bootstrap.appVersion : "checking";
  const running = verificationPhase === "running";

  return (
    <div className="application-shell">
      <a className="skip-link" href="#workspace">
        Skip to application content
      </a>

      <div className="foundation-banner" role="note">
        <strong>CYVRA ERASE · LOCAL ASSESSMENT</strong>
        <span>
          Hardware and document map on this PC. Purge, grading issuance and cloud report authentication stay off. Report D can carry a local integrity seal.
        </span>
      </div>

      <header className="titlebar">
        <div className="brand-lockup">
          <img src={logoUrl} alt="CYVORIQ Solutions" width="70" height="42" />
          <span className="brand-divider" aria-hidden="true" />
          <span className="product-name">
            <strong>CYVRA ERASE</strong>
            <small>by CYVORIQ Solutions</small>
          </span>
        </div>

        <div className="titlebar-status" aria-label="Application foundation status">
          <span className="foundation-chip">ASSESSMENT ONLY</span>
          <span>Version {version}</span>
          <button className="button button-quiet" type="button" onClick={onExit}>
            Exit
          </button>
        </div>
      </header>

      <div className="shell-layout">
        <aside className="sidebar">
          <nav aria-label="Primary application navigation">
            {NAVIGATION_ITEMS.map((item) => (
              <button
                key={item.id}
                type="button"
                className={current === item.id ? "nav-item nav-item-active" : "nav-item"}
                aria-current={current === item.id ? "page" : undefined}
                disabled={running && item.id !== "verification" && item.id !== "help"}
                onClick={() => onNavigate(item.id)}
              >
                <span className="nav-mark" aria-hidden="true">
                  {item.shortLabel}
                </span>
                <span>{item.label}</span>
              </button>
            ))}
          </nav>

          <div className="sidebar-status">
            <span className={`status-dot status-dot-${bridge.status}`} aria-hidden="true" />
            <div>
              <strong>Standard-user runtime</strong>
              <span>{bridgeLabel(bridge)}</span>
            </div>
          </div>

          <button className="privacy-link" type="button" onClick={() => onNavigate("help")}>
            Settings &amp; privacy
          </button>
          <button className="privacy-link" type="button" onClick={onExit}>
            Exit CYVRA Erase
          </button>
        </aside>

        <main id="workspace" className="workspace" ref={workspaceRef} tabIndex={-1}>
          {children}
        </main>
      </div>

      <footer className="safety-footer">
        <span>Status: {footerState(verificationPhase, progress)}</span>
        <span>{bridgeLabel(bridge)}</span>
        <strong>
          <span className="safe-dot" aria-hidden="true" /> Non-destructive foundation
        </strong>
      </footer>
    </div>
  );
}
