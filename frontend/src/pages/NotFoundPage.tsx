import { NavLink } from "react-router";

export default function NotFoundPage() {
  return (
    <main className="content-page">
      <span className="eyebrow">404</span>
      <h1>Page not found</h1>
      <NavLink className="button" to="/">Return home</NavLink>
    </main>
  );
}
