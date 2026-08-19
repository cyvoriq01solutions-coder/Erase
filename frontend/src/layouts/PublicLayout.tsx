import { NavLink, Outlet, useLocation } from "react-router";

const links = [
  ["/why-cyvra", "Why CYVRA"],
  ["/how-it-works", "How It Works"],
  ["/dpdp-readiness", "DPDP Readiness"],
  ["/individuals", "Individuals"],
  ["/enterprise", "Enterprise"],
  ["/resources", "Resources"],
] as const;

export default function PublicLayout() {
  const location = useLocation();
  const isHome = location.pathname === "/";

  return (
    <div className="site-shell">
      <header className="site-header">
        <NavLink to="/" className="brand brand-logo" aria-label="CYVORIQ Solutions home">
          <img src="/cyvoriq-logo.webp" alt="CYVORIQ Solutions" />
          <span className="product-lockup">
            <strong>CYVRA ERASE</strong>
            <small>by CYVORIQ Solutions</small>
          </span>
        </NavLink>

        <nav className="site-nav" aria-label="Primary navigation">
          {links.map(([to, label]) => (
            <NavLink key={to} to={to} className={({ isActive }) => (isActive ? "active" : undefined)}>
              {label}
            </NavLink>
          ))}
        </nav>

        <div className="header-actions">
          <NavLink className="header-signin" to="/download">Sign In</NavLink>
          <NavLink className="button button-small button-orange" to="/download">
            Download CYVRA Erase
          </NavLink>
        </div>
      </header>

      {!isHome && (
        <div className="return-home-row">
          <NavLink className="return-home-tab" to="/" aria-label="Return to CYVRA home page">
            <span aria-hidden="true">←</span> Return to Home
          </NavLink>
        </div>
      )}

      <Outlet />

      <footer className="site-footer">
        <div>
          <strong>CYVORIQ SOLUTIONS</strong>
          <span>Secure Lifecycle. Trusted Future.</span>
        </div>
        <div className="footer-meta">
          <span>CYVRA Erase · Evidence-backed device retirement</span>
          <span>Designed to support DPDP readiness</span>
        </div>
      </footer>
    </div>
  );
}
