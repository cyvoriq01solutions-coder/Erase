import { useEffect, useState } from "react";
import { Navigate, Outlet, useLocation } from "react-router";
import { getSession } from "../services/authApi";

type AuthState = "checking" | "authenticated" | "anonymous" | "error";

export default function RequireAuth() {
  const location = useLocation();
  const [state, setState] = useState<AuthState>("checking");
  const [retryKey, setRetryKey] = useState(0);

  useEffect(() => {
    let active = true;
    setState("checking");

    getSession()
      .then((session) => {
        if (!active) return;
        setState(session.authenticated ? "authenticated" : "anonymous");
      })
      .catch(() => {
        if (!active) return;
        setState("error");
      });

    return () => {
      active = false;
    };
  }, [retryKey]);

  if (state === "checking") {
    return (
      <main className="auth-guard-page">
        <div className="auth-guard-card">
          <span className="status-pill">SECURE SESSION</span>
          <h1>Checking your CYVRA session...</h1>
          <p>Please wait while the server verifies your account.</p>
        </div>
      </main>
    );
  }

  if (state === "anonymous") {
    const returnTo = `${location.pathname}${location.search}${location.hash}`;
    return (
      <Navigate
        to={`/account?mode=signin&returnTo=${encodeURIComponent(returnTo)}`}
        replace
      />
    );
  }

  if (state === "error") {
    return (
      <main className="auth-guard-page">
        <div className="auth-guard-card">
          <span className="status-pill">SESSION CHECK UNAVAILABLE</span>
          <h1>We could not verify your session.</h1>
          <p>No protected information has been displayed. Retry the secure session check.</p>
          <button className="button button-orange" type="button" onClick={() => setRetryKey((value) => value + 1)}>
            Retry
          </button>
        </div>
      </main>
    );
  }

  return <Outlet />;
}
