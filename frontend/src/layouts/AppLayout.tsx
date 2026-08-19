import { useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router";
import { logout } from "../services/authApi";

const appLinks = [
  ["/app/dashboard", "Dashboard"],
  ["/app/devices", "Devices"],
  ["/app/assessments", "Assessments"],
  ["/app/evidence", "Evidence"],
  ["/app/verification", "Verification"],
  ["/app/reports", "Reports"],
  ["/app/certificates", "Certificates"],
  ["/app/settings", "Settings"],
] as const;

export default function AppLayout() {
  const navigate = useNavigate();
  const [loggingOut, setLoggingOut] = useState(false);

  async function handleLogout() {
    setLoggingOut(true);
    try {
      await logout();
    } finally {
      navigate("/account?mode=signin", { replace: true });
    }
  }

  return (
    <div className="app-shell">
      <aside className="app-sidebar">
        <NavLink to="/" className="brand brand-inverse">
          <span className="brand-mark">C</span>
          <span>CYVORIQ <strong>ERASE</strong></span>
        </NavLink>
        <nav className="app-nav" aria-label="Application navigation">
          {appLinks.map(([to, label]) => (
            <NavLink key={to} to={to}>{label}</NavLink>
          ))}
        </nav>
        <button
          className="app-logout"
          type="button"
          onClick={handleLogout}
          disabled={loggingOut}
        >
          {loggingOut ? "Signing out..." : "Sign Out"}
        </button>
      </aside>
      <main className="app-content">
        <Outlet />
      </main>
    </div>
  );
}
