import {
  buildExpiredSessionCookie,
  getAuthenticatedSession,
  readSessionToken,
  type AuthenticatedSession,
} from "../services/authSession";
import {
  ACCOUNTS_APPROVER_EMAIL,
  canAccessControlPanel,
  canExerciseSuperUserAuthority,
  type ActiveRole,
} from "../services/authorizationPolicy";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "../services/database";
import { json } from "../services/http";

export interface AdminApiEnv {
  HYPERDRIVE: HyperdriveBinding;
}

function activeRoles(session: AuthenticatedSession): ActiveRole[] {
  return session.roles
    .filter(
      (role): role is ActiveRole["role"] =>
        role === "customer" || role === "accounts_admin" || role === "super_admin",
    )
    .map((role) => ({ role, status: "active" as const }));
}

async function requireAdminSession(
  request: Request,
  env: AdminApiEnv,
): Promise<{ session: AuthenticatedSession; roles: ActiveRole[] } | Response> {
  const token = readSessionToken(request);
  if (token === null) {
    return json({ error: "unauthorized", message: "Admin authentication is required." }, { status: 401 });
  }

  const session = await getAuthenticatedSession(env.HYPERDRIVE, token);
  if (session === null) {
    const response = json({ error: "unauthorized", message: "Admin session is invalid or expired." }, { status: 401 });
    response.headers.append("Set-Cookie", buildExpiredSessionCookie());
    return response;
  }

  const roles = activeRoles(session);
  if (!canAccessControlPanel(roles)) {
    return json({ error: "forbidden", message: "An active administration role is required." }, { status: 403 });
  }

  return { session, roles };
}

export async function handleAdminSession(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const result = await requireAdminSession(request, env);
  if (result instanceof Response) {
    return result;
  }

  return json(
    {
      authorized: true,
      user: {
        id: result.session.userId,
        email: result.session.email,
        displayName: result.session.displayName,
        organizationId: result.session.organizationId,
        organizationSlug: result.session.organizationSlug,
        roles: result.session.roles,
      },
      expiresAt: result.session.expiresAt,
    },
    { status: 200 },
  );
}

async function requireBootstrapSuperAdmin(
  request: Request,
  env: AdminApiEnv,
): Promise<{ session: AuthenticatedSession; roles: ActiveRole[] } | Response> {
  const result = await requireAdminSession(request, env);
  if (result instanceof Response) {
    return result;
  }

  if (!canExerciseSuperUserAuthority(result.session.email, result.roles)) {
    return json(
      { error: "forbidden", message: "Super Administrator authority is required for this action." },
      { status: 403 },
    );
  }

  return result;
}

export async function handleApproveAccountsAdmin(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const authority = await requireBootstrapSuperAdmin(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  const result = await withDatabaseTransaction(env.HYPERDRIVE, async (client) => {
    const target = await client.query(
      `
        SELECT
          ur.id AS role_id,
          ur.status AS role_status,
          u.id AS user_id,
          u.account_status,
          u.email_verified_at
        FROM user_roles ur
        INNER JOIN users u
          ON u.id = ur.user_id
         AND u.organization_id = ur.organization_id
        WHERE LOWER(u.email) = $1
          AND ur.organization_id = $2
          AND ur.role = 'accounts_admin'
        FOR UPDATE OF ur
      `,
      [ACCOUNTS_APPROVER_EMAIL, authority.session.organizationId],
    );

    if (target.rowCount !== 1) {
      return { status: "missing" as const };
    }

    const row = target.rows[0];
    if (row.email_verified_at === null || String(row.account_status) !== "active") {
      return { status: "unverified" as const };
    }

    if (String(row.role_status) !== "active") {
      await client.query(
        `
          UPDATE user_roles
          SET status = 'active',
              approved_by_user_id = $2,
              approved_at = NOW(),
              revoked_at = NULL
          WHERE id = $1
        `,
        [String(row.role_id), authority.session.userId],
      );

      await client.query(
        `
          INSERT INTO audit_events (
            id,
            organization_id,
            actor_id,
            event_type,
            entity_type,
            entity_id,
            details
          ) VALUES ($1, $2, $3, 'ADMIN_ROLE_APPROVED', 'user_role', $4, $5::jsonb)
        `,
        [
          crypto.randomUUID(),
          authority.session.organizationId,
          authority.session.userId,
          String(row.role_id),
          JSON.stringify({ role: "accounts_admin", targetEmail: ACCOUNTS_APPROVER_EMAIL }),
        ],
      );
    }

    return { status: "active" as const, userId: String(row.user_id) };
  });

  if (result.status === "missing") {
    return json(
      { error: "accounts_identity_missing", message: "The Accounts identity must register and verify its email first." },
      { status: 409 },
    );
  }

  if (result.status === "unverified") {
    return json(
      { error: "accounts_identity_unverified", message: "The Accounts identity must complete email verification first." },
      { status: 409 },
    );
  }

  return json(
    {
      status: "active",
      role: "accounts_admin",
      email: ACCOUNTS_APPROVER_EMAIL,
      userId: result.userId,
    },
    { status: 200 },
  );
}

export async function handleRevokeAccountsAdmin(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const authority = await requireBootstrapSuperAdmin(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  const result = await withDatabaseTransaction(env.HYPERDRIVE, async (client) => {
    const target = await client.query(
      `
        SELECT ur.id AS role_id, ur.status AS role_status, u.id AS user_id
        FROM user_roles ur
        INNER JOIN users u
          ON u.id = ur.user_id
         AND u.organization_id = ur.organization_id
        WHERE LOWER(u.email) = $1
          AND ur.organization_id = $2
          AND ur.role = 'accounts_admin'
        FOR UPDATE OF ur
      `,
      [ACCOUNTS_APPROVER_EMAIL, authority.session.organizationId],
    );

    if (target.rowCount !== 1) {
      return { status: "missing" as const };
    }

    const row = target.rows[0];
    if (String(row.role_status) !== "revoked") {
      await client.query(
        `
          UPDATE user_roles
          SET status = 'revoked',
              revoked_at = NOW()
          WHERE id = $1
        `,
        [String(row.role_id)],
      );

      await client.query(
        `
          INSERT INTO audit_events (
            id,
            organization_id,
            actor_id,
            event_type,
            entity_type,
            entity_id,
            details
          ) VALUES ($1, $2, $3, 'ADMIN_ROLE_REVOKED', 'user_role', $4, $5::jsonb)
        `,
        [
          crypto.randomUUID(),
          authority.session.organizationId,
          authority.session.userId,
          String(row.role_id),
          JSON.stringify({ role: "accounts_admin", targetEmail: ACCOUNTS_APPROVER_EMAIL }),
        ],
      );
    }

    return { status: "revoked" as const, userId: String(row.user_id) };
  });

  if (result.status === "missing") {
    return json(
      { error: "accounts_identity_missing", message: "The Accounts Administrator identity does not exist." },
      { status: 404 },
    );
  }

  return json(
    {
      status: "revoked",
      role: "accounts_admin",
      email: ACCOUNTS_APPROVER_EMAIL,
      userId: result.userId,
    },
    { status: 200 },
  );
}
