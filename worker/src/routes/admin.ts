import {
  buildExpiredAdminSessionCookie,
  buildExpiredLegacyAdminSessionCookie,
  getAuthenticatedAdminSession,
  readAdminSessionToken,
  type AuthenticatedAdminSession,
} from "../services/adminAuthSession";
import {
  inviteAdminIdentity,
  listAdminIdentities,
} from "../services/adminIdentity";
import {
  ACCOUNTS_APPROVER_EMAIL,
  canAccessControlPanel,
  canManageAdminRoles,
  type ActiveRole,
} from "../services/authorizationPolicy";
import {
  queryDatabase,
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "../services/database";
import { json } from "../services/http";

export interface AdminApiEnv {
  HYPERDRIVE: HyperdriveBinding;
}

const MAX_JSON_BODY_BYTES = 8 * 1024;

function activeRoles(session: AuthenticatedAdminSession): ActiveRole[] {
  return session.roles
    .filter(
      (role): role is ActiveRole["role"] =>
        role === "accounts_admin" || role === "super_admin",
    )
    .map((role) => ({ role, status: "active" as const }));
}

async function readJsonObject(request: Request): Promise<Record<string, unknown>> {
  const contentLength = Number(request.headers.get("Content-Length") ?? "0");
  if (Number.isFinite(contentLength) && contentLength > MAX_JSON_BODY_BYTES) {
    throw new Error("request_too_large");
  }

  const body = await request.text();
  if (body.length > MAX_JSON_BODY_BYTES) {
    throw new Error("request_too_large");
  }

  const parsed: unknown = JSON.parse(body);
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("invalid_json");
  }
  return parsed as Record<string, unknown>;
}

export async function requireAdminSession(
  request: Request,
  env: AdminApiEnv,
): Promise<{ session: AuthenticatedAdminSession; roles: ActiveRole[] } | Response> {
  const token = readAdminSessionToken(request);
  if (token === null) {
    const response = json(
      { error: "unauthorized", message: "Admin authentication is required." },
      { status: 401 },
    );
    response.headers.append(
      "Set-Cookie",
      buildExpiredLegacyAdminSessionCookie(),
    );
    return response;
  }

  const session = await getAuthenticatedAdminSession(env.HYPERDRIVE, token);
  if (session === null) {
    const response = json(
      { error: "unauthorized", message: "Admin session is invalid or expired." },
      { status: 401 },
    );
    response.headers.append("Set-Cookie", buildExpiredAdminSessionCookie());
    response.headers.append(
      "Set-Cookie",
      buildExpiredLegacyAdminSessionCookie(),
    );
    return response;
  }

  const roles = activeRoles(session);
  if (!canAccessControlPanel(roles)) {
    return json(
      { error: "forbidden", message: "An active administration role is required." },
      { status: 403 },
    );
  }

  return { session, roles };
}

async function requireSuperAdmin(
  request: Request,
  env: AdminApiEnv,
): Promise<{ session: AuthenticatedAdminSession; roles: ActiveRole[] } | Response> {
  const result = await requireAdminSession(request, env);
  if (result instanceof Response) {
    return result;
  }

  if (!canManageAdminRoles(result.roles)) {
    return json(
      { error: "forbidden", message: "Super Administrator authority is required for this action." },
      { status: 403 },
    );
  }

  return result;
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

export async function handleListAdminUsers(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const authority = await requireSuperAdmin(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  const users = await listAdminIdentities(env.HYPERDRIVE);
  return json(
    {
      users: users.map((user) => ({
        id: user.userId,
        email: user.email,
        displayName: user.displayName,
        accountStatus: user.accountStatus,
        emailVerifiedAt: user.emailVerifiedAt,
        role: user.role,
        roleStatus: user.roleStatus,
      })),
    },
    { status: 200 },
  );
}

export async function handleInviteAdminUser(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const authority = await requireSuperAdmin(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  let body: Record<string, unknown>;
  try {
    body = await readJsonObject(request);
  } catch {
    return json({ error: "invalid_request", message: "Invalid request body." }, { status: 400 });
  }

  if (typeof body.email !== "string") {
    return json(
      { error: "invalid_email", message: "A corporate administrator email is required." },
      { status: 400 },
    );
  }
  if (body.role !== undefined && body.role !== "accounts_admin") {
    return json(
      { error: "invalid_role", message: "Only Accounts Administrator invitations are supported in C4.1." },
      { status: 400 },
    );
  }
  if (
    body.displayName !== undefined &&
    body.displayName !== null &&
    typeof body.displayName !== "string"
  ) {
    return json({ error: "invalid_request", message: "Invalid display name." }, { status: 400 });
  }

  try {
    const invited = await inviteAdminIdentity(
      env.HYPERDRIVE,
      {
        email: body.email,
        displayName: body.displayName as string | null | undefined,
        role: "accounts_admin",
      },
      authority.session.userId,
    );

    return json(
      {
        user: {
          id: invited.userId,
          email: invited.email,
          displayName: invited.displayName,
          accountStatus: invited.accountStatus,
          emailVerifiedAt: invited.emailVerifiedAt,
          role: invited.role,
          roleStatus: invited.roleStatus,
        },
      },
      { status: 201 },
    );
  } catch (error) {
    return json(
      {
        error: "admin_invite_rejected",
        message: error instanceof Error ? error.message : "Administrator invitation could not be created.",
      },
      { status: 409 },
    );
  }
}

async function setAdminRoleStatus(
  env: AdminApiEnv,
  authority: AuthenticatedAdminSession,
  targetUserId: string,
  nextStatus: "active" | "revoked",
): Promise<
  | { status: "active"; userId: string; email: string }
  | { status: "revoked"; userId: string; email: string }
  | { status: "missing" }
  | { status: "unverified" }
  | { status: "self" }
> {
  return withDatabaseTransaction(env.HYPERDRIVE, async (client) => {
    if (targetUserId === authority.userId) {
      return { status: "self" as const };
    }

    const target = await client.query(
      `
        SELECT
          ur.id AS role_id,
          ur.status AS role_status,
          u.id AS user_id,
          u.email,
          u.account_status,
          u.email_verified_at
        FROM user_roles ur
        INNER JOIN users u
          ON u.id = ur.user_id
         AND u.organization_id = ur.organization_id
        INNER JOIN organizations o
          ON o.id = u.organization_id
        WHERE u.id = $1
          AND u.organization_id = $2
          AND o.account_type = 'internal'
          AND ur.role = 'accounts_admin'
        FOR UPDATE OF ur, u
      `,
      [targetUserId, authority.organizationId],
    );

    if (target.rowCount !== 1) {
      return { status: "missing" as const };
    }

    const row = target.rows[0];
    if (
      nextStatus === "active" &&
      (row.email_verified_at === null || String(row.account_status) !== "active")
    ) {
      return { status: "unverified" as const };
    }

    const currentStatus = String(row.role_status);
    if (currentStatus !== nextStatus) {
      if (nextStatus === "active") {
        await client.query(
          `
            UPDATE user_roles
            SET status = 'active',
                approved_by_user_id = $2,
                approved_at = NOW(),
                revoked_at = NULL
            WHERE id = $1
          `,
          [String(row.role_id), authority.userId],
        );
      } else {
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
            UPDATE admin_sessions
            SET revoked_at = COALESCE(revoked_at, NOW())
            WHERE user_id = $1
              AND organization_id = $2
              AND revoked_at IS NULL
          `,
          [String(row.user_id), authority.organizationId],
        );
      }

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
          ) VALUES ($1, $2, $3, $4, 'user_role', $5, $6::jsonb)
        `,
        [
          crypto.randomUUID(),
          authority.organizationId,
          authority.userId,
          nextStatus === "active" ? "ADMIN_ROLE_APPROVED" : "ADMIN_ROLE_REVOKED",
          String(row.role_id),
          JSON.stringify({
            role: "accounts_admin",
            targetEmail: String(row.email).toLowerCase(),
          }),
        ],
      );
    }

    return {
      status: nextStatus,
      userId: String(row.user_id),
      email: String(row.email).toLowerCase(),
    };
  });
}

async function roleActionResponse(
  result: Awaited<ReturnType<typeof setAdminRoleStatus>>,
): Promise<Response> {
  if (result.status === "missing") {
    return json(
      { error: "admin_identity_missing", message: "The administrator identity does not exist." },
      { status: 404 },
    );
  }
  if (result.status === "unverified") {
    return json(
      { error: "admin_identity_unverified", message: "The administrator must verify email ownership first." },
      { status: 409 },
    );
  }
  if (result.status === "self") {
    return json(
      { error: "self_role_change_denied", message: "You cannot change your own administration role here." },
      { status: 409 },
    );
  }

  return json(
    {
      status: result.status,
      role: "accounts_admin",
      email: result.email,
      userId: result.userId,
    },
    { status: 200 },
  );
}

export async function handleApproveAdminUser(
  request: Request,
  env: AdminApiEnv,
  targetUserId: string,
): Promise<Response> {
  const authority = await requireSuperAdmin(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  if (!/^[0-9a-f-]{36}$/i.test(targetUserId)) {
    return json({ error: "invalid_user", message: "Invalid administrator identity." }, { status: 400 });
  }

  return roleActionResponse(
    await setAdminRoleStatus(env, authority.session, targetUserId, "active"),
  );
}

export async function handleRevokeAdminUser(
  request: Request,
  env: AdminApiEnv,
  targetUserId: string,
): Promise<Response> {
  const authority = await requireSuperAdmin(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  if (!/^[0-9a-f-]{36}$/i.test(targetUserId)) {
    return json({ error: "invalid_user", message: "Invalid administrator identity." }, { status: 400 });
  }

  return roleActionResponse(
    await setAdminRoleStatus(env, authority.session, targetUserId, "revoked"),
  );
}

async function accountsUserId(env: AdminApiEnv): Promise<string | null> {
  const rows = await queryDatabase(
    env.HYPERDRIVE,
    `
      SELECT u.id
      FROM users u
      INNER JOIN user_roles ur
        ON ur.user_id = u.id
       AND ur.organization_id = u.organization_id
      WHERE LOWER(u.email) = $1
        AND ur.role = 'accounts_admin'
      LIMIT 1
    `,
    [ACCOUNTS_APPROVER_EMAIL],
  );
  return rows.length === 1 ? String(rows[0].id) : null;
}

// Compatibility wrappers retained during C4.1 frontend migration.
export async function handleApproveAccountsAdmin(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const userId = await accountsUserId(env);
  if (userId === null) {
    return json(
      { error: "accounts_identity_missing", message: "The Accounts Administrator identity does not exist." },
      { status: 404 },
    );
  }
  return handleApproveAdminUser(request, env, userId);
}

export async function handleRevokeAccountsAdmin(
  request: Request,
  env: AdminApiEnv,
): Promise<Response> {
  const userId = await accountsUserId(env);
  if (userId === null) {
    return json(
      { error: "accounts_identity_missing", message: "The Accounts Administrator identity does not exist." },
      { status: 404 },
    );
  }
  return handleRevokeAdminUser(request, env, userId);
}
