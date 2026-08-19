import { FormEvent, useEffect, useMemo, useState } from "react";
import { NavLink, Navigate, Route, Routes, useNavigate } from "react-router";
import {
  ACCOUNTS_EMAIL,
  API_BASE_URL,
  ApiError,
  beginAdminLogin,
  getSession,
  logout,
  activeAdminRole,
  type AdminRole,
  type SessionUser,
  verifyAdminCode,
} from "./authApi";

type GateState = "checking" | "anonymous" | "challenge" | "authorized" | "pending" | "forbidden" | "unavailable";

type ModuleCard = {
  title: string;
  description: string;
  status: string;
};

const navigation = [
  ["/", "Overview"],
  ["/customers", "Customers"],
  ["/orders", "Orders / Purchases"],
  ["/payments", "Payments"],
  ["/approvals", "Approvals"],
  ["/licences", "Licences"],
  ["/entitlements", "Download Entitlements"],
  ["/activations", "Activations / Devices"],
  ["/audit", "Audit Events"],
  ["/reports/management", "Management Report"],
  ["/reports/accounts", "Accounts Report"],
] as const;

const overviewCards: ModuleCard[] = [
  { title: "Identity authority", description: "Email OTP + server session + active admin role.", status: "Foundation ready" },
  { title: "Commercial approvals", description: "Payment and purchase authority stays server-controlled.", status: "Backend package pending" },
  { title: "Licence control", description: "Issue, revoke and entitlement state will be authoritative in Neon.", status: "Backend package pending" },
  { title: "Device activation", description: "One licence binds to one authorised device after first activation.", status: "C6 frozen" },
];

function roleLabel(role: AdminRole): string {
  return role === "super_admin" ? "Super Administrator" : "Accounts Administrator";
}

function LoginGate({
  state,
  onAuthorized,
}: {
  state: GateState;
  onAuthorized: (user: SessionUser, role: AdminRole) => void;
}) {
  const [email, setEmail] = useState("");
  const [challengeId, setChallengeId] = useState("");
  const [code, setCode] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const challengeOpen = challengeId.length > 0;

  async function sendCode(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const result = await beginAdminLogin(email);
      setEmail(email.trim().toLowerCase());
      setChallengeId(result.challengeId);
      setMessage("Verification code sent. The code is valid for 10 minutes.");
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "Admin verification could not be started.");
    } finally {
      setBusy(false);
    }
  }

  async function verifyCode(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    try {
      await verifyAdminCode(challengeId, code);
      const session = await getSession();
      if (!session.authenticated) {
        throw new ApiError("The server did not establish an authenticated admin session.", 401);
      }
      const role = activeAdminRole(session.user);
      if (role !== null) {
        onAuthorized(session.user, role);
        return;
      }
      if (session.user.email.toLowerCase() === ACCOUNTS_EMAIL) {
        setError("Email verified. The Accounts Administrator role is still awaiting Super Administrator approval.");
      } else {
        setError("This verified account does not have an active CYVRA administration role.");
      }
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "The verification code could not be confirmed.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="gate-page">
      <section className="gate-panel">
        <div className="gate-brand">
          <span className="eyebrow">CYVORIQ INTERNAL SYSTEM</span>
          <h1>CYVRA Admin Portal</h1>
          <p>Restricted operational control plane for authorized CYVORIQ administrators.</p>
          <div className="security-chain">
            <span>Cloudflare Access</span><b>→</b><span>Email OTP</span><b>→</b><span>Server Session</span><b>→</b><span>Neon RBAC</span>
          </div>
        </div>
        <div className="gate-card">
          <span className="status-chip">ADMIN AUTHORITY</span>
          <h2>{challengeOpen ? "Verify your identity" : "Internal sign in"}</h2>
          <p className="muted">Only approved CYVORIQ authority identities can continue.</p>

          {state === "unavailable" && (
            <div className="notice warning">The admin UI is deployed, but the production API domain is not reachable yet. No protected data is exposed.</div>
          )}

          {!challengeOpen ? (
            <form onSubmit={sendCode} className="stack-form">
              <label>
                <span>CYVORIQ email</span>
                <input type="email" value={email} onChange={(event) => setEmail(event.target.value)} placeholder="ceo@cyvra.co.in" autoComplete="email" required disabled={busy} />
              </label>
              {error && <div className="notice error">{error}</div>}
              <button className="primary-button" type="submit" disabled={busy}>{busy ? "Connecting..." : "Send verification code"}</button>
            </form>
          ) : (
            <form onSubmit={verifyCode} className="stack-form">
              <div className="identity-line">Code sent to <strong>{email}</strong></div>
              <label>
                <span>6-digit verification code</span>
                <input className="code-input" type="text" inputMode="numeric" pattern="[0-9]{6}" maxLength={6} value={code} onChange={(event) => setCode(event.target.value.replace(/\D/g, "").slice(0, 6))} autoComplete="one-time-code" required disabled={busy} autoFocus />
              </label>
              {message && <div className="notice success">{message}</div>}
              {error && <div className="notice error">{error}</div>}
              <button className="primary-button" type="submit" disabled={busy || code.length !== 6}>{busy ? "Verifying..." : "Verify & enter portal"}</button>
              <button className="text-button" type="button" onClick={() => { setChallengeId(""); setCode(""); setError(""); setMessage(""); }} disabled={busy}>Use another email</button>
            </form>
          )}

          <small className="control-plane-note">Control plane: {API_BASE_URL}</small>
        </div>
      </section>
    </main>
  );
}

function StatusPage({ title, description }: { title: string; description: string }) {
  return (
    <section className="content-page">
      <span className="eyebrow">SERVER-AUTHORITATIVE MODULE</span>
      <h1>{title}</h1>
      <p className="page-lede">{description}</p>
      <div className="empty-state">
        <strong>No live operational data is loaded in C4 Foundation.</strong>
        <p>The user interface is ready. Data will be connected only after the corresponding Worker authorization and audit endpoints pass verification.</p>
      </div>
    </section>
  );
}

function OverviewPage({ role }: { role: AdminRole }) {
  return (
    <section className="content-page">
      <span className="eyebrow">CONTROL PLANE OVERVIEW</span>
      <h1>Administration readiness</h1>
      <p className="page-lede">The portal shell is live; commercial and customer data remain disconnected until server-side admin APIs are explicitly authorized.</p>
      <div className="safety-banner">NO LIVE CUSTOMER, PAYMENT, LICENCE OR DEVICE DATA LOADED</div>
      <div className="module-grid">
        {overviewCards.map((card) => (
          <article className="module-card" key={card.title}>
            <span>{card.status}</span>
            <h2>{card.title}</h2>
            <p>{card.description}</p>
          </article>
        ))}
      </div>
      <div className="authority-card">
        <div><span>Current authority</span><strong>{roleLabel(role)}</strong></div>
        <div><span>Browser authority</span><strong>None</strong></div>
        <div><span>Database credentials</span><strong>Never exposed</strong></div>
        <div><span>Destructive authority</span><strong>None</strong></div>
      </div>
    </section>
  );
}

function ReportPage({ kind }: { kind: "Management" | "Accounts" }) {
  const items = kind === "Management"
    ? ["Customer/account status", "Orders and payment state", "Licence and entitlement state", "Activations and bound devices", "Operational exceptions"]
    : ["Payment verification queue", "Purchase approval queue", "Reconciliation references", "Licence/entitlement state", "Activation support status"];

  return (
    <section className="content-page">
      <span className="eyebrow">REPORTING FOUNDATION</span>
      <h1>{kind} Report</h1>
      <p className="page-lede">Report structure is frozen. Live metrics and export actions remain disabled until audited server endpoints are connected.</p>
      <div className="report-grid">
        {items.map((item) => <div key={item}><span>READY FOR API</span><strong>{item}</strong></div>)}
      </div>
    </section>
  );
}

function AdminShell({ user, role, onLogout }: { user: SessionUser; role: AdminRole; onLogout: () => Promise<void> }) {
  const [loggingOut, setLoggingOut] = useState(false);
  const navigate = useNavigate();
  const links = useMemo(() => role === "super_admin" ? [...navigation, ["/roles", "Internal Users / Roles"] as const] : navigation, [role]);

  async function signOut() {
    setLoggingOut(true);
    try { await onLogout(); } finally { navigate("/", { replace: true }); setLoggingOut(false); }
  }

  return (
    <div className="admin-shell">
      <aside className="sidebar">
        <div className="portal-brand"><span className="brand-mark">C</span><div>CYVRA<strong>ADMIN</strong></div></div>
        <div className="internal-badge">INTERNAL · RESTRICTED</div>
        <nav>
          {links.map(([to, label]) => <NavLink key={to} to={to} end={to === "/"}>{label}</NavLink>)}
        </nav>
        <button className="signout-button" type="button" onClick={signOut} disabled={loggingOut}>{loggingOut ? "Signing out..." : "Sign out"}</button>
      </aside>
      <div className="workspace">
        <header className="topbar">
          <div><span>Authenticated identity</span><strong>{user.email}</strong></div>
          <div className="role-pill">{roleLabel(role)}</div>
        </header>
        <main className="workspace-main">
          <Routes>
            <Route path="/" element={<OverviewPage role={role} />} />
            <Route path="/customers" element={<StatusPage title="Customers" description="Verified customer identities, account state and organization context." />} />
            <Route path="/orders" element={<StatusPage title="Orders / Purchases" description="Commercial orders and purchase lifecycle controls." />} />
            <Route path="/payments" element={<StatusPage title="Payments" description="Payment confirmation and reconciliation authority." />} />
            <Route path="/approvals" element={<StatusPage title="Approvals" description="Purchase approval and rejection workflow with audit evidence." />} />
            <Route path="/licences" element={<StatusPage title="Licences" description="Server-issued licence state, revocation and customer relationship." />} />
            <Route path="/entitlements" element={<StatusPage title="Download Entitlements" description="Protected package-release authority after all commercial gates pass." />} />
            <Route path="/activations" element={<StatusPage title="Activations / Bound Devices" description="One-device activation binding, revalidation and key-reuse exception handling." />} />
            <Route path="/audit" element={<StatusPage title="Audit Events" description="Immutable operational evidence for material authority actions." />} />
            <Route path="/reports/management" element={<ReportPage kind="Management" />} />
            <Route path="/reports/accounts" element={<ReportPage kind="Accounts" />} />
            {role === "super_admin" && <Route path="/roles" element={<StatusPage title="Internal Users / Roles" description="Super Administrator authority for internal role approval and revocation." />} />}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </div>
  );
}

export default function App() {
  const [gate, setGate] = useState<GateState>("checking");
  const [user, setUser] = useState<SessionUser | null>(null);
  const [role, setRole] = useState<AdminRole | null>(null);

  useEffect(() => {
    let active = true;
    getSession()
      .then((session) => {
        if (!active) return;
        if (!session.authenticated) { setGate("anonymous"); return; }
        const adminRole = activeAdminRole(session.user);
        if (adminRole !== null) { setUser(session.user); setRole(adminRole); setGate("authorized"); return; }
        if (session.user.email.toLowerCase() === ACCOUNTS_EMAIL) { setGate("pending"); return; }
        setGate("forbidden");
      })
      .catch(() => { if (active) setGate("unavailable"); });
    return () => { active = false; };
  }, []);

  async function handleLogout() {
    try { await logout(); } finally { setUser(null); setRole(null); setGate("anonymous"); }
  }

  if (gate === "checking") {
    return <main className="center-page"><div className="checking-card"><span className="status-chip">SECURE SESSION</span><h1>Verifying admin authority...</h1><p>No protected data is rendered until the server confirms an active administration role.</p></div></main>;
  }

  if (gate === "pending") {
    return <main className="center-page"><div className="checking-card"><span className="status-chip">ROLE APPROVAL REQUIRED</span><h1>Accounts authority is pending.</h1><p>Your email may be verified, but <strong>accounts_admin</strong> access requires approval by the active Super Administrator.</p><button className="secondary-button" type="button" onClick={handleLogout}>Sign out</button></div></main>;
  }

  if (gate === "forbidden") {
    return <main className="center-page"><div className="checking-card"><span className="status-chip">ACCESS DENIED</span><h1>No active admin role.</h1><p>This authenticated identity is not authorized for the CYVRA Admin Portal.</p><button className="secondary-button" type="button" onClick={handleLogout}>Sign out</button></div></main>;
  }

  if (gate === "authorized" && user !== null && role !== null) {
    return <AdminShell user={user} role={role} onLogout={handleLogout} />;
  }

  return <LoginGate state={gate} onAuthorized={(nextUser, nextRole) => { setUser(nextUser); setRole(nextRole); setGate("authorized"); }} />;
}
