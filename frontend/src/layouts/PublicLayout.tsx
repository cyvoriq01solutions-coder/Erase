import { NavLink, Outlet } from "react-router";

const links = [
  ["/platform", "Platform"],
  ["/how-it-works", "How It Works"],
  ["/assurance", "Assurance"],
  ["/security", "Security"],
  ["/resources", "Resources"],
  ["/contact", "Contact"],
] as const;

export default function PublicLayout() {
  return (
    <div className="site-shell">
      <header className="site-header">
        <NavLink to="/" className="brand" aria-label="CYVORIQ Erase home">
          <span className="brand-mark">C</span>
          <span>CYVORIQ <strong>ERASE</strong></span>
        </NavLink>
        <nav className="site-nav" aria-label="Primary navigation">
          {links.map(([to, label]) => (
            <NavLink key={to} to={to} className={({ isActive }) => isActive ? "active" : undefined}>
              {label}
            </NavLink>
          ))}
        </nav>
        <NavLink className="button button-small" to="/app/dashboard">Open Platform</NavLink>
      </header>
      <Outlet />
      <footer className="site-footer">
        <span>CYVORIQ SOLUTIONS</span>
        <span>Security · Trust · Evidence · Engineering</span>
      </footer>
    </div>
  );
}
