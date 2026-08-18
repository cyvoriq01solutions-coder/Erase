import type { Client } from "pg";

import {
  ACCOUNTS_APPROVER_EMAIL,
  CEO_SUPER_USER_EMAIL,
  isAccountsAuthorityIdentity,
  isBootstrapSuperUser,
} from "./authorizationPolicy";
import {
  normalizeEmail,
  type CustomerIdentity,
} from "./authIdentity";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "./database";

const CYVORIQ_INTERNAL_ORG_ID = "00000000-0000-4000-8000-000000000001";
const CYVORIQ_INTERNAL_ORG_SLUG = "cyvoriq-internal";

type CustomerAccountType = "individual" | "enterprise";

export interface RegisterCustomerInput {
  email: string;
  displayName?: string | null;
  accountType?: CustomerAccountType;
  organizationName?: string | null;
}

export interface RegistrationResult {
  status: "created" | "existing";
  identity: CustomerIdentity;
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

function customerSlug(organizationId: string): string {
  return `acct-${organizationId.replace(/-/g, "").slice(0, 20)}`;
}

async function findIdentityByEmail(
  client: Client,
  email: string,
): Promise<CustomerIdentity | null> {
  const result = await client.query(
    `
      SELECT
        o.id AS organization_id,
        o.slug AS organization_slug,
        u.id AS user_id,
        u.email,
        u.display_name
      FROM users u
      INNER JOIN organizations o
        ON o.id = u.organization_id
      WHERE LOWER(u.email) = $1
      LIMIT 1
    `,
    [email],
  );

  if (result.rowCount !== 1) {
    return null;
  }

  const row = result.rows[0];
  return {
    organizationId: String(row.organization_id),
    organizationSlug: String(row.organization_slug),
    userId: String(row.user_id),
    email: String(row.email),
    displayName:
      row.display_name === null || row.display_name === undefined
        ? null
        : String(row.display_name),
  };
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

export async function registerCustomerIdentity(
  hyperdrive: HyperdriveBinding,
  input: RegisterCustomerInput,
): Promise<RegistrationResult> {
  const email = normalizeEmail(input.email);
  if (email === null) {
    throw new Error("Invalid email address");
  }

  const displayName = normalizeOptionalName(input.displayName);
  const requestedAccountType = input.accountType ?? "individual";
  const organizationName = normalizeOptionalName(input.organizationName);

  if (
    requestedAccountType !== "individual" &&
    requestedAccountType !== "enterprise"
  ) {
    throw new Error("Invalid account type");
  }

  if (requestedAccountType === "enterprise" && organizationName === null) {
    throw new Error("Enterprise organization name is required");
  }

  return withDatabaseTransaction(hyperdrive, async (client) => {
    const existing = await findIdentityByEmail(client, email);
    if (existing !== null) {
      return { status: "existing", identity: existing };
    }

    const isInternalAuthority =
      isBootstrapSuperUser(email) || isAccountsAuthorityIdentity(email);

    let organizationId: string;
    let organizationSlug: string;
    let createdCustomerOrganization = false;

    if (isInternalAuthority) {
      await ensureInternalOrganization(client);
      organizationId = CYVORIQ_INTERNAL_ORG_ID;
      organizationSlug = CYVORIQ_INTERNAL_ORG_SLUG;
    } else {
      organizationId = crypto.randomUUID();
      organizationSlug = customerSlug(organizationId);
      const name =
        requestedAccountType === "enterprise"
          ? organizationName!
          : displayName ?? "Customer Account";

      await client.query(
        `
          INSERT INTO organizations (id, name, slug, account_type)
          VALUES ($1, $2, $3, $4)
        `,
        [organizationId, name, organizationSlug, requestedAccountType],
      );
      createdCustomerOrganization = true;
    }

    const userId = crypto.randomUUID();
    const insertUser = await client.query(
      `
        INSERT INTO users (
          id,
          organization_id,
          email,
          display_name,
          account_status
        ) VALUES ($1, $2, $3, $4, 'pending_email_verification')
        ON CONFLICT DO NOTHING
        RETURNING id
      `,
      [userId, organizationId, email, displayName],
    );

    if (insertUser.rowCount !== 1) {
      if (createdCustomerOrganization) {
        await client.query("DELETE FROM organizations WHERE id = $1", [
          organizationId,
        ]);
      }

      const racedExisting = await findIdentityByEmail(client, email);
      if (racedExisting === null) {
        throw new Error("Unable to resolve registered identity");
      }

      return { status: "existing", identity: racedExisting };
    }

    const role = isBootstrapSuperUser(email)
      ? "super_admin"
      : isAccountsAuthorityIdentity(email)
        ? "accounts_admin"
        : "customer";

    await client.query(
      `
        INSERT INTO user_roles (
          id,
          organization_id,
          user_id,
          role,
          status
        ) VALUES ($1, $2, $3, $4, 'pending')
      `,
      [crypto.randomUUID(), organizationId, userId, role],
    );

    return {
      status: "created",
      identity: {
        organizationId,
        organizationSlug,
        userId,
        email,
        displayName,
      },
    };
  });
}

export const internalAuthorityEmails = Object.freeze([
  CEO_SUPER_USER_EMAIL,
  ACCOUNTS_APPROVER_EMAIL,
] as const);
