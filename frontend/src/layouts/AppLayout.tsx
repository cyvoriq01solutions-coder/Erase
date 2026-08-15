import { NavLink, Outlet } from "react-router";

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
      </aside>
      <main className="app-content">
        <Outlet />
      </main>
    </div>
  );
}
