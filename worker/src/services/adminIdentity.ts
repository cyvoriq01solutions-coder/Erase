import type { Client } from "pg";

import {
  ACCOUNTS_APPROVER_EMAIL,
  CEO_SUPER_USER_EMAIL,
  isAccountsAuthorityIdentity,
  isBootstrapSuperUser,
  type ControlPanelRole,
} from "./authorizationPolicy";
import { normalizeEmail } from "./authIdentity";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "./database";

export const CYVORIQ_INTERNAL_ORG_ID = "00000000-0000-4000-8000-000000000001";
export const CYVORIQ_INTERNAL_ORG_SLUG = "cyvoriq-internal";

export type AdminRoleStatus = "pending" | "active" | "revoked";

export interface AdminIdentity {
  organizationId: string;
  organizationSlug: string;
  userId: string;
  email: string;
  displayName: string | null;
  accountStatus: string;
  emailVerifiedAt: string | null;
  role: ControlPanelRole;
  roleStatus: AdminRoleStatus;
}

function normalizeOptionalName(value: string | null | undefined): string | null {
  if (value === null || value === undefined) {
    return null;
  }

  const normalized = value.trim().replace(/\s+/g, " ");
  if (normalized.length === 0) {
    return null;
  }
  if (normalized.length > 160) {
    throw new Error("Name is too long");
  }
  return normalized;
}

export function isCorporateAdminEmail(email: string): boolean {
  const normalized = normalizeEmail(email);
  return normalized !== null && normalized.endsWith("@cyvra.co.in");
}

async function ensureInternalOrganization(client: Client): Promise<void> {
  await client.query(
    `
      INSERT INTO organizations (id, name, slug, account_type)
      VALUES ($1, 'CYVORIQ', $2, 'internal')
      ON CONFLICT (id) DO NOTHING
    `,
    [CYVORIQ_INTERNAL_ORG_ID, CYVORIQ_INTERNAL_ORG_SLUG],
  );

  const result = await client.query(
    `
      SELECT id, slug, account_type
      FROM organizations
      WHERE id = $1
      FOR UPDATE
    `,
    [CYVORIQ_INTERNAL_ORG_ID],
  );

  if (result.rowCount !== 1) {
    throw new Error("CYVORIQ internal organization is unavailable");
  }

  const row = result.rows[0];
  if (
    String(row.slug) !== CYVORIQ_INTERNAL_ORG_SLUG ||
    String(row.account_type) !== "internal"
  ) {
    throw new Error("CYVORIQ internal organization identity mismatch");
  }
}

async function findAdminIdentityByEmail(
  client: Client,
  email: string,
): Promise<AdminIdentity | null> {
  const result = await client.query(
    `
      SELECT
        o.id AS organization_id,
        o.slug AS organization_slug,
        o.account_type,
        u.id AS user_id,
        u.email,
        u.display_name,
        u.account_status,
        u.email_verified_at,
        ur.role,
        ur.status AS role_status
      FROM users u
      INNER JOIN organizations o
        ON o.id = u.organization_id
      INNER JOIN user_roles ur
        ON ur.user_id = u.id
       AND ur.organization_id = u.organization_id
      WHERE LOWER(u.email) = $1
        AND ur.role IN ('super_admin', 'accounts_admin')
      ORDER BY CASE ur.role WHEN 'super_admin' THEN 0 ELSE 1 END
      LIMIT 2
    `,
    [email],
  );

  if (result.rowCount === 0) {
    return null;
  }

  const row = result.rows[0];
  if (String(row.account_type) !== "internal") {
    return null;
  }

  const role = String(row.role);
  const roleStatus = String(row.role_status);
  if (
    (role !== "super_admin" && role !== "accounts_admin") ||
    (roleStatus !== "pending" && roleStatus !== "active" && roleStatus !== "revoked")
  ) {
    return null;
  }

  return {
    organizationId: String(row.organization_id),
    organizationSlug: String(row.organization_slug),
    userId: String(row.user_id),
    email: String(row.email),
    displayName:
      row.display_name === null || row.display_name === undefined
        ? null
        : String(row.display_name),
    accountStatus: String(row.account_status),
    emailVerifiedAt:
      row.email_verified_at === null || row.email_verified_at === undefined
        ? null
        : new Date(String(row.email_verified_at)).toISOString(),
    role: role as ControlPanelRole,
    roleStatus: roleStatus as AdminRoleStatus,
  };
}

async function ensureBootstrapIdentity(
  client: Client,
  email: string,
): Promise<AdminIdentity | null> {
  const bootstrapRole: ControlPanelRole | null = isBootstrapSuperUser(email)
    ? "super_admin"
    : isAccountsAuthorityIdentity(email)
      ? "accounts_admin"
      : null;

  if (bootstrapRole === null) {
    return null;
  }

  await ensureInternalOrganization(client);

  const existingUser = await client.query(
    `
      SELECT id, organization_id
      FROM users
      WHERE LOWER(email) = $1
      FOR UPDATE
    `,
    [email],
  );

  let userId: string;
  if (existingUser.rowCount === 0) {
    userId = crypto.randomUUID();
    await client.query(
      `
        INSERT INTO users (
          id,
          organization_id,
          email,
          account_status
        ) VALUES ($1, $2, $3, 'pending_email_verification')
      `,
      [userId, CYVORIQ_INTERNAL_ORG_ID, email],
    );
  } else if (existingUser.rowCount === 1) {
    const row = existingUser.rows[0];
    if (String(row.organization_id) !== CYVORIQ_INTERNAL_ORG_ID) {
      return null;
    }
    userId = String(row.id);
  } else {
    return null;
  }

  await client.query(
    `
      INSERT INTO user_roles (
        id,
        organization_id,
        user_id,
        role,
        status
      ) VALUES ($1, $2, $3, $4, 'pending')
      ON CONFLICT (user_id, role) DO NOTHING
    `,
    [crypto.randomUUID(), CYVORIQ_INTERNAL_ORG_ID, userId, bootstrapRole],
  );

  return findAdminIdentityByEmail(client, email);
}

export async function resolveAdminIdentityForLogin(
  hyperdrive: HyperdriveBinding,
  rawEmail: string,
): Promise<AdminIdentity | null> {
  const email = normalizeEmail(rawEmail);
  if (email === null) {
    return null;
  }

  return withDatabaseTransaction(hyperdrive, async (client) => {
    const existing = await findAdminIdentityByEmail(client, email);
    if (existing !== null) {
      return existing;
    }

    return ensureBootstrapIdentity(client, email);
  });
}

export interface InviteAdminInput {
  email: string;
  displayName?: string | null;
  role: "accounts_admin";
}

export async function inviteAdminIdentity(
  hyperdrive: HyperdriveBinding,
  input: InviteAdminInput,
  actorUserId: string,
): Promise<AdminIdentity> {
  const email = normalizeEmail(input.email);
  if (email === null || !isCorporateAdminEmail(email)) {
    throw new Error("A valid @cyvra.co.in corporate email is required");
  }
  if (isBootstrapSuperUser(email)) {
    throw new Error("The bootstrap Super Administrator cannot be created through invitations");
  }
  if (input.role !== "accounts_admin") {
    throw new Error("Only accounts_admin invitations are supported in C4.1");
  }

  const displayName = normalizeOptionalName(input.displayName);

  return withDatabaseTransaction(hyperdrive, async (client) => {
    await ensureInternalOrganization(client);

    const existing = await findAdminIdentityByEmail(client, email);
    if (existing !== null) {
      return existing;
    }

    const conflictingUser = await client.query(
      `
        SELECT id, organization_id
        FROM users
        WHERE LOWER(email) = $1
        FOR UPDATE
      `,
      [email],
    );
    if (conflictingUser.rowCount !== 0) {
      throw new Error("This email already belongs to another account");
    }

    const userId = crypto.randomUUID();
    const roleId = crypto.randomUUID();
    await client.query(
      `
        INSERT INTO users (
          id,
          organization_id,
          email,
          display_name,
          account_status
        ) VALUES ($1, $2, $3, $4, 'pending_email_verification')
      `,
      [userId, CYVORIQ_INTERNAL_ORG_ID, email, displayName],
    );

    await client.query(
      `
        INSERT INTO user_roles (
          id,
          organization_id,
          user_id,
          role,
          status
        ) VALUES ($1, $2, $3, 'accounts_admin', 'pending')
      `,
      [roleId, CYVORIQ_INTERNAL_ORG_ID, userId],
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
        ) VALUES ($1, $2, $3, 'ADMIN_USER_CREATED', 'user', $4, $5::jsonb)
      `,
      [
        crypto.randomUUID(),
        CYVORIQ_INTERNAL_ORG_ID,
        actorUserId,
        userId,
        JSON.stringify({ email, role: "accounts_admin", invitation: true }),
      ],
    );

    const invited = await findAdminIdentityByEmail(client, email);
    if (invited === null) {
      throw new Error("Unable to resolve invited administrator");
    }
    return invited;
  });
}

export async function listAdminIdentities(
  hyperdrive: HyperdriveBinding,
): Promise<AdminIdentity[]> {
  const clientResult = await withDatabaseTransaction(hyperdrive, async (client) => {
    const result = await client.query(
      `
        SELECT
          o.id AS organization_id,
          o.slug AS organization_slug,
          u.id AS user_id,
          u.email,
          u.display_name,
          u.account_status,
          u.email_verified_at,
          ur.role,
          ur.status AS role_status
        FROM users u
        INNER JOIN organizations o
          ON o.id = u.organization_id
        INNER JOIN user_roles ur
          ON ur.user_id = u.id
         AND ur.organization_id = u.organization_id
        WHERE u.organization_id = $1
          AND ur.role IN ('super_admin', 'accounts_admin')
        ORDER BY LOWER(u.email), ur.role
      `,
      [CYVORIQ_INTERNAL_ORG_ID],
    );
    return result.rows;
  });

  return clientResult.map((row) => ({
    organizationId: String(row.organization_id),
    organizationSlug: String(row.organization_slug),
    userId: String(row.user_id),
    email: String(row.email),
    displayName:
      row.display_name === null || row.display_name === undefined
        ? null
        : String(row.display_name),
    accountStatus: String(row.account_status),
    emailVerifiedAt:
      row.email_verified_at === null || row.email_verified_at === undefined
        ? null
        : new Date(String(row.email_verified_at)).toISOString(),
    role: String(row.role) as ControlPanelRole,
    roleStatus: String(row.role_status) as AdminRoleStatus,
  }));
}

export const bootstrapAdminEmails = Object.freeze([
  CEO_SUPER_USER_EMAIL,
  ACCOUNTS_APPROVER_EMAIL,
] as const);
