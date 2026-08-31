import { FormEvent, useEffect, useMemo, useState } from "react";
import { NavLink, Navigate, Route, Routes, useNavigate } from "react-router";
import {
  API_BASE_URL,
  ApiError,
  activeAdminRole,
  approveAdminUser,
  approveCustomer,
  beginAdminLogin,
  getAdminSession,
  inviteAdminUser,
  issueCustomerLicense,
  listAdminUsers,
  listCustomers,
  logout,
  rejectCustomer,
  revokeAdminUser,
  type AdminRole,
  type AdminUserSummary,
  type CustomerAccessSummary,
  type SessionUser,
  verifyAdminCode,
} from "./authApi";

type GateState = "checking" | "anonymous" | "authorized" | "unavailable";

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
  ["/releases", "Software Releases"],
  ["/activations", "Activations / Devices"],
  ["/audit", "Audit Events"],
  ["/reports/management", "Management Report"],
  ["/reports/accounts", "Accounts Report"],
] as const;

const overviewCards: ModuleCard[] = [
  {
    title: "Dedicated admin identity",
    description: "Admin OTP + dedicated admin session + Neon RBAC. Customer sessions are not accepted.",
    status: "C4.1",
  },
  {
    title: "Internal user authority",
    description: "Super Administrator invitations and role approvals remain server-controlled and audit-recorded.",
    status: "C4.1",
  },
  {
    title: "Commercial approvals",
    description: "Verified customers wait on Customers. Approve or reject with a reason email. Package release is still later.",
    status: "C5",
  },
  {
    title: "Licence issuance",
    description: "Server-generated activation key, hashed in Neon, emailed once. Windows device bind stays off.",
    status: "C licence",
  },
  {
    title: "Device activation",
    description: "One licence binds to one authorised device after first activation.",
    status: "C6 frozen",
  },
];

function roleLabel(role: AdminRole): string {
  return role === "super_admin" ? "Super Administrator" : "Accounts Administrator";
}

async function resolveAuthorizedAdmin(): Promise<
  { user: SessionUser; role: AdminRole } | "anonymous"
> {
  const session = await getAdminSession();
  if (!session.authorized) {
    return "anonymous";
  }

  const role = activeAdminRole(session.user);
  if (role === null) {
    return "anonymous";
  }

  return { user: session.user, role };
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
  const [pendingApproval, setPendingApproval] = useState(false);
  const challengeOpen = challengeId.length > 0;

  async function sendCode(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    setMessage("");
    setPendingApproval(false);

    try {
      const result = await beginAdminLogin(email);
      setEmail(email.trim().toLowerCase());
      setChallengeId(result.challengeId);
      setMessage(result.message);
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "Admin verification could not be started.",
      );
    } finally {
      setBusy(false);
    }
  }

  async function verifyCode(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError("");
    setMessage("");

    try {
      const result = await verifyAdminCode(challengeId, code);
      if (!result.authenticated) {
        setPendingApproval(true);
        setChallengeId("");
        setCode("");
        setMessage(
          "Email ownership is verified. Your administration role is still awaiting Super Administrator approval. Request a new code after approval.",
        );
        return;
      }

      const resolved = await resolveAuthorizedAdmin();
      if (resolved === "anonymous") {
        setError("A dedicated Admin session could not be established.");
        return;
      }

      onAuthorized(resolved.user, resolved.role);
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "The verification code could not be confirmed.",
      );
    } finally {
      setBusy(false);
    }
  }

  function resetIdentity() {
    setChallengeId("");
    setCode("");
    setError("");
    setMessage("");
    setPendingApproval(false);
  }

  return (
    <main className="gate-page">
      <section className="gate-panel">
        <div className="gate-brand">
          <span className="eyebrow">CYVORIQ INTERNAL CONTROL</span>
          <h1>CYVRA Admin</h1>
          <p>
            Restricted operational control plane. Administration authority is
            validated independently from all customer accounts and sessions.
          </p>
          <div className="security-chain">
            <span>Cloudflare Access</span>
            <b>→</b>
            <span>Admin Email OTP</span>
            <b>→</b>
            <span>Dedicated Admin Session</span>
            <b>→</b>
            <span>Neon RBAC</span>
          </div>
        </div>

        <div className="gate-card">
          <span className="status-chip">INTERNAL AUTHORITY</span>
          <h2>{challengeOpen ? "Verify administrator" : "Administrator sign in"}</h2>
          <p className="muted">
            No public Admin registration exists. Eligibility is decided by the
            control plane, not by this browser.
          </p>

          {state === "unavailable" && (
            <div className="notice warning">
              The Admin UI is available, but the control-plane API could not be
              reached. No protected data is exposed.
            </div>
          )}

          {!challengeOpen ? (
            <form onSubmit={sendCode} className="stack-form">
              <label>
                <span>CYVORIQ corporate email</span>
                <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder="name@cyvoriq.com"
                  autoComplete="email"
                  required
                  disabled={busy}
                />
              </label>
              {message && (
                <div className={pendingApproval ? "notice warning" : "notice success"}>
                  {message}
                </div>
              )}
              {error && <div className="notice error">{error}</div>}
              <button className="primary-button" type="submit" disabled={busy}>
                {busy ? "Checking authority..." : "Send secure code"}
              </button>
            </form>
          ) : (
            <form onSubmit={verifyCode} className="stack-form">
              <div className="identity-line">
                Code sent to <strong>{email}</strong>
              </div>
              <label>
                <span>6-digit verification code</span>
                <input
                  className="code-input"
                  type="text"
                  inputMode="numeric"
                  pattern="[0-9]{6}"
                  maxLength={6}
                  value={code}
                  onChange={(event) =>
                    setCode(event.target.value.replace(/\D/g, "").slice(0, 6))
                  }
                  autoComplete="one-time-code"
                  required
                  disabled={busy}
                  autoFocus
                />
              </label>
              {message && <div className="notice success">{message}</div>}
              {error && <div className="notice error">{error}</div>}
              <button
                className="primary-button"
                type="submit"
                disabled={busy || code.length !== 6}
              >
                {busy ? "Verifying authority..." : "Verify & enter control plane"}
              </button>
              <button
                className="text-button"
                type="button"
                onClick={resetIdentity}
                disabled={busy}
              >
                Use another email
              </button>
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
        <strong>Control surface reserved.</strong>
        <p>
          Live operational data will be connected only after the corresponding
          Worker authorization, audit and data-boundary tests pass.
        </p>
      </div>
    </section>
  );
}

function CustomersPage() {
  const [customers, setCustomers] = useState<CustomerAccessSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<Record<string, boolean>>({});
  const [rejectingId, setRejectingId] = useState<string | null>(null);
  const [rejectReason, setRejectReason] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [issuedKey, setIssuedKey] = useState<{ email: string; activationKey: string } | null>(null);

  async function refresh() {
    setLoading(true);
    try {
      const result = await listCustomers();
      setCustomers(result.customers);
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "Verified customers could not be loaded.",
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  const waitingIds = customers
    .filter((customer) => customer.accessStatus === "waiting")
    .map((customer) => customer.id);
  const selectedWaiting = waitingIds.filter((id) => selected[id]);

  async function approveSelected() {
    if (selectedWaiting.length === 0) {
      setError("Select at least one waiting customer.");
      return;
    }
    setBusy(true);
    setMessage("");
    setError("");
    try {
      for (const id of selectedWaiting) {
        await approveCustomer(id);
      }
      setSelected({});
      setMessage(`Approved ${selectedWaiting.length} customer(s). Download remains locked until the package is released.`);
      await refresh();
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "Approval failed.");
    } finally {
      setBusy(false);
    }
  }

  async function submitReject(userId: string) {
    setBusy(true);
    setMessage("");
    setError("");
    try {
      await rejectCustomer(userId, rejectReason);
      setRejectingId(null);
      setRejectReason("");
      setMessage("Customer access was rejected and an email with the reason was sent.");
      await refresh();
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "Rejection failed.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="content-page">
      <span className="eyebrow">C5 ACCESS DECISIONS</span>
      <h1>Customers</h1>
      <p className="page-lede">
        Verified customer identities wait here until a Super Administrator or Accounts
        Administrator approves download access. Rejection sends the reason to the customer email.
        This does not publish a Windows installer. Issuing a licence emails the activation key once;
        the full key is not stored.
      </p>
      <div className="safety-banner">NO BROWSER-SIDE DOWNLOAD AUTHORITY · PACKAGE NOT RELEASED · KEY SHOWN ONCE</div>

      <div className="admin-users-card" style={{ marginTop: 28 }}>
        <div className="admin-users-header">
          <div>
            <span className="eyebrow">NEON RBAC</span>
            <h2>Waiting and decided customers</h2>
          </div>
          <div className="admin-user-actions">
            <button className="secondary-button compact-button" type="button" onClick={() => void refresh()} disabled={loading || busy}>
              Refresh
            </button>
            <button
              className="primary-button compact-button"
              type="button"
              onClick={() => void approveSelected()}
              disabled={busy || selectedWaiting.length === 0}
            >
              Approve selected
            </button>
          </div>
        </div>

        {loading ? (
          <p className="muted">Loading verified customers...</p>
        ) : customers.length === 0 ? (
          <p className="muted">No verified customers yet.</p>
        ) : (
          <div className="admin-user-list">
            {customers.map((customer) => (
              <article key={customer.id} className="admin-user-row">
                <label className="admin-user-identity">
                  {customer.accessStatus === "waiting" && (
                    <input
                      type="checkbox"
                      checked={Boolean(selected[customer.id])}
                      onChange={(event) =>
                        setSelected((current) => ({
                          ...current,
                          [customer.id]: event.target.checked,
                        }))
                      }
                      disabled={busy}
                    />
                  )}
                  <strong>{customer.displayName || customer.email}</strong>
                  <span>{customer.email}</span>
                </label>
                <div className="admin-user-state">
                  <span>Access</span>
                  <strong>{customer.accessStatus}</strong>
                  {customer.rejectReason && <small>{customer.rejectReason}</small>}
                </div>
                <div className="admin-user-state">
                  <span>Licence</span>
                  <strong>
                    {customer.licensePrefix
                      ? `${customer.licensePrefix}…`
                      : customer.accessStatus === "approved"
                        ? "Not issued"
                        : "—"}
                  </strong>
                </div>
                <div className="admin-user-actions">
                  {customer.accessStatus !== "approved" && (
                    <button
                      className="primary-button compact-button"
                      type="button"
                      disabled={busy}
                      onClick={() => {
                        setBusy(true);
                        setError("");
                        void approveCustomer(customer.id)
                          .then(() => refresh())
                          .catch((caught) => {
                            setError(
                              caught instanceof ApiError
                                ? caught.message
                                : "Approval failed.",
                            );
                          })
                          .finally(() => setBusy(false));
                      }}
                    >
                      Approve
                    </button>
                  )}
                  {customer.accessStatus === "approved" && !customer.licensePrefix && (
                    <button
                      className="primary-button compact-button"
                      type="button"
                      disabled={busy}
                      onClick={() => {
                        setBusy(true);
                        setError("");
                        setMessage("");
                        void issueCustomerLicense(customer.id)
                          .then((result) => {
                            setIssuedKey({
                              email: result.customer.email,
                              activationKey: result.activationKey,
                            });
                            setMessage(
                              `Licence issued for ${result.customer.email}. Copy the key now; it will not be shown again.`,
                            );
                            return refresh();
                          })
                          .catch((caught) => {
                            setError(
                              caught instanceof ApiError
                                ? caught.message
                                : "Licence issuance failed.",
                            );
                          })
                          .finally(() => setBusy(false));
                      }}
                    >
                      Issue licence
                    </button>
                  )}
                  <button
                    className="danger-button compact-button"
                    type="button"
                    disabled={busy}
                    onClick={() => {
                      setRejectingId(customer.id);
                      setRejectReason("");
                    }}
                  >
                    Reject
                  </button>
                </div>
                {rejectingId === customer.id && (
                  <form
                    className="stack-form"
                    style={{ gridColumn: "1 / -1" }}
                    onSubmit={(event) => {
                      event.preventDefault();
                      void submitReject(customer.id);
                    }}
                  >
                    <label>
                      <span>Reason sent to the customer (8–500 characters)</span>
                      <input
                        value={rejectReason}
                        onChange={(event) => setRejectReason(event.target.value)}
                        required
                        minLength={8}
                        maxLength={500}
                        disabled={busy}
                      />
                    </label>
                    <button className="danger-button" type="submit" disabled={busy}>
                      Send rejection email
                    </button>
                  </form>
                )}
              </article>
            ))}
          </div>
        )}
      </div>
      {message && <p className="notice success role-notice">{message}</p>}
      {error && <p className="notice error role-notice">{error}</p>}
      {issuedKey && (
        <div className="notice success role-notice" role="status">
          <p>
            Full activation key for <strong>{issuedKey.email}</strong> (shown once):
          </p>
          <p>
            <code style={{ fontSize: 18, letterSpacing: "0.04em" }}>{issuedKey.activationKey}</code>
          </p>
          <button
            className="secondary-button compact-button"
            type="button"
            onClick={() => void navigator.clipboard.writeText(issuedKey.activationKey)}
          >
            Copy key
          </button>
        </div>
      )}
    </section>
  );
}

function OverviewPage({ role }: { role: AdminRole }) {
  return (
    <section className="content-page">
      <span className="eyebrow">CONTROL PLANE OVERVIEW</span>
      <h1>Administration readiness</h1>
      <p className="page-lede">
        Internal authority is isolated from customer authentication. Commercial,
        release and activation modules remain locked until their server packages
        are verified.
      </p>
      <div className="safety-banner">
        NO BROWSER-SIDE ROLE, PAYMENT, LICENCE OR RELEASE AUTHORITY
      </div>
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
        <div>
          <span>Current authority</span>
          <strong>{roleLabel(role)}</strong>
        </div>
        <div>
          <span>Customer session reuse</span>
          <strong>Denied</strong>
        </div>
        <div>
          <span>Database credentials</span>
          <strong>Never exposed</strong>
        </div>
        <div>
          <span>Destructive authority</span>
          <strong>None</strong>
        </div>
      </div>
    </section>
  );
}

function ReportPage({ kind }: { kind: "Management" | "Accounts" }) {
  const items =
    kind === "Management"
      ? [
          "Customer/account status",
          "Orders and payment state",
          "Licence and entitlement state",
          "Software release state",
          "Activations and bound devices",
          "Operational exceptions",
        ]
      : [
          "Payment verification queue",
          "Purchase approval queue",
          "Reconciliation references",
          "Licence/entitlement state",
          "Activation support status",
        ];

  return (
    <section className="content-page">
      <span className="eyebrow">REPORTING FOUNDATION</span>
      <h1>{kind} Report</h1>
      <p className="page-lede">
        Report structure is frozen. Live metrics and export actions remain disabled
        until audited server endpoints are connected.
      </p>
      <div className="report-grid">
        {items.map((item) => (
          <div key={item}>
            <span>READY FOR API</span>
            <strong>{item}</strong>
          </div>
        ))}
      </div>
    </section>
  );
}

function InternalRolesPage() {
  const [users, setUsers] = useState<AdminUserSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [busyUserId, setBusyUserId] = useState<string | null>(null);
  const [inviteBusy, setInviteBusy] = useState(false);
  const [displayName, setDisplayName] = useState("");
  const [email, setEmail] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  async function refresh() {
    setLoading(true);
    try {
      const result = await listAdminUsers();
      setUsers(result.users);
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "Internal administrators could not be loaded.",
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function invite(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setInviteBusy(true);
    setMessage("");
    setError("");
    try {
      const result = await inviteAdminUser({
        email,
        displayName: displayName || null,
        role: "accounts_admin",
      });
      setMessage(
        `${result.user.email} was created as a pending Accounts Administrator. The user must verify email ownership before role approval.`,
      );
      setEmail("");
      setDisplayName("");
      await refresh();
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "The administrator invitation could not be created.",
      );
    } finally {
      setInviteBusy(false);
    }
  }

  async function changeRole(user: AdminUserSummary, action: "approve" | "revoke") {
    setBusyUserId(user.id);
    setMessage("");
    setError("");
    try {
      const result =
        action === "approve"
          ? await approveAdminUser(user.id)
          : await revokeAdminUser(user.id);
      setMessage(
        `${result.email} Accounts Administrator role is now ${result.status}. The action is audit-recorded server-side.`,
      );
      await refresh();
    } catch (caught) {
      setError(
        caught instanceof ApiError
          ? caught.message
          : "The role action could not be completed.",
      );
    } finally {
      setBusyUserId(null);
    }
  }

  return (
    <section className="content-page">
      <span className="eyebrow">SUPER ADMINISTRATOR ONLY</span>
      <h1>Internal Users / Roles</h1>
      <p className="page-lede">
        There is no public Admin registration. Create an internal identity here,
        verify its corporate email, then approve the role server-side.
      </p>

      <div className="internal-users-layout">
        <form className="invite-card stack-form" onSubmit={invite}>
          <span className="status-chip">CREATE / INVITE</span>
          <h2>Invite administrator</h2>
          <label>
            <span>Full name</span>
            <input
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              placeholder="Administrator name"
              disabled={inviteBusy}
            />
          </label>
          <label>
            <span>Corporate email</span>
            <input
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="name@cyvoriq.com"
              required
              disabled={inviteBusy}
            />
          </label>
          <label>
            <span>Role</span>
            <input value="Accounts Administrator" readOnly />
          </label>
          <button className="primary-button" type="submit" disabled={inviteBusy}>
            {inviteBusy ? "Creating identity..." : "Create pending administrator"}
          </button>
        </form>

        <div className="admin-users-card">
          <div className="admin-users-header">
            <div>
              <span className="status-chip">NEON RBAC</span>
              <h2>Internal administrators</h2>
            </div>
            <button className="text-button" type="button" onClick={() => void refresh()} disabled={loading}>
              Refresh
            </button>
          </div>

          {loading ? (
            <p className="muted">Loading server-authoritative identities...</p>
          ) : users.length === 0 ? (
            <p className="muted">No internal administrators found.</p>
          ) : (
            <div className="admin-user-list">
              {users.map((user) => (
                <article className="admin-user-row" key={`${user.id}-${user.role}`}>
                  <div className="admin-user-identity">
                    <strong>{user.displayName || user.email}</strong>
                    <span>{user.email}</span>
                  </div>
                  <div className="admin-user-state">
                    <span>{roleLabel(user.role)}</span>
                    <strong>{user.roleStatus}</strong>
                    <small>{user.emailVerifiedAt ? "Email verified" : "Email verification pending"}</small>
                  </div>
                  {user.role === "accounts_admin" && (
                    <div className="admin-user-actions">
                      <button
                        className="primary-button compact-button"
                        type="button"
                        disabled={busyUserId !== null || user.roleStatus === "active"}
                        onClick={() => void changeRole(user, "approve")}
                      >
                        Approve
                      </button>
                      <button
                        className="danger-button compact-button"
                        type="button"
                        disabled={busyUserId !== null || user.roleStatus === "revoked"}
                        onClick={() => void changeRole(user, "revoke")}
                      >
                        Revoke
                      </button>
                    </div>
                  )}
                </article>
              ))}
            </div>
          )}
        </div>
      </div>

      {message && <div className="notice success role-notice">{message}</div>}
      {error && <div className="notice error role-notice">{error}</div>}
    </section>
  );
}

function AdminShell({
  user,
  role,
  onLogout,
}: {
  user: SessionUser;
  role: AdminRole;
  onLogout: () => Promise<void>;
}) {
  const [loggingOut, setLoggingOut] = useState(false);
  const navigate = useNavigate();
  const links = useMemo(
    () =>
      role === "super_admin"
        ? [...navigation, ["/roles", "Internal Users / Roles"] as const]
        : navigation,
    [role],
  );

  async function signOut() {
    setLoggingOut(true);
    try {
      await onLogout();
    } finally {
      navigate("/", { replace: true });
      setLoggingOut(false);
    }
  }

  return (
    <div className="admin-shell">
      <aside className="sidebar">
        <div className="portal-brand">
          <span className="brand-mark">C</span>
          <div>
            CYVRA<strong>ADMIN CONTROL</strong>
          </div>
        </div>
        <div className="internal-badge">INTERNAL · RESTRICTED</div>
        <nav>
          {links.map(([to, label]) => (
            <NavLink key={to} to={to} end={to === "/"}>
              {label}
            </NavLink>
          ))}
        </nav>
        <button
          className="signout-button"
          type="button"
          onClick={signOut}
          disabled={loggingOut}
        >
          {loggingOut ? "Signing out..." : "Sign out"}
        </button>
      </aside>
      <div className="workspace">
        <header className="topbar">
          <div>
            <span>Authenticated administrator</span>
            <strong>{user.email}</strong>
          </div>
          <div className="role-pill">{roleLabel(role)}</div>
        </header>
        <main className="workspace-main">
          <Routes>
            <Route path="/" element={<OverviewPage role={role} />} />
            <Route path="/customers" element={<CustomersPage />} />
            <Route
              path="/orders"
              element={
                <StatusPage
                  title="Orders / Purchases"
                  description="Commercial orders and purchase lifecycle controls."
                />
              }
            />
            <Route
              path="/payments"
              element={
                <StatusPage
                  title="Payments"
                  description="Payment confirmation and reconciliation authority."
                />
              }
            />
            <Route
              path="/approvals"
              element={
                <StatusPage
                  title="Approvals"
                  description="Purchase approval and rejection workflow with audit evidence."
                />
              }
            />
            <Route
              path="/licences"
              element={
                <StatusPage
                  title="Licences"
                  description="Server-issued licence state, revocation and customer relationship."
                />
              }
            />
            <Route
              path="/entitlements"
              element={
                <StatusPage
                  title="Download Entitlements"
                  description="Protected package-release authority after all commercial gates pass."
                />
              }
            />
            <Route
              path="/releases"
              element={
                <StatusPage
                  title="Software Releases"
                  description="Signed installer release metadata and private Backblaze B2 publication authority."
                />
              }
            />
            <Route
              path="/activations"
              element={
                <StatusPage
                  title="Activations / Bound Devices"
                  description="One-device activation binding, revalidation and key-reuse exception handling."
                />
              }
            />
            <Route
              path="/audit"
              element={
                <StatusPage
                  title="Audit Events"
                  description="Operational evidence for material internal authority actions."
                />
              }
            />
            <Route path="/reports/management" element={<ReportPage kind="Management" />} />
            <Route path="/reports/accounts" element={<ReportPage kind="Accounts" />} />
            {role === "super_admin" && (
              <Route path="/roles" element={<InternalRolesPage />} />
            )}
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

    resolveAuthorizedAdmin()
      .then((resolved) => {
        if (!active) return;
        if (resolved !== "anonymous") {
          setUser(resolved.user);
          setRole(resolved.role);
          setGate("authorized");
        } else {
          setGate("anonymous");
        }
      })
      .catch((caught) => {
        if (!active) return;
        setGate(caught instanceof ApiError && caught.status === 0 ? "unavailable" : "anonymous");
      });

    return () => {
      active = false;
    };
  }, []);

  if (gate === "checking") {
    return (
      <main className="center-page">
        <div className="checking-card">
          <span className="status-chip">ADMIN AUTHORITY</span>
          <h1>Checking dedicated Admin session</h1>
          <p>Customer sessions are not accepted by this control plane.</p>
        </div>
      </main>
    );
  }

  if (gate !== "authorized" || user === null || role === null) {
    return (
      <LoginGate
        state={gate}
        onAuthorized={(nextUser, nextRole) => {
          setUser(nextUser);
          setRole(nextRole);
          setGate("authorized");
        }}
      />
    );
  }

  return (
    <AdminShell
      user={user}
      role={role}
      onLogout={async () => {
        try {
          await logout();
        } finally {
          setUser(null);
          setRole(null);
          setGate("anonymous");
        }
      }}
    />
  );
}
