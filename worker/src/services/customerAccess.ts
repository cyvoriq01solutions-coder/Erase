import type { Client } from "pg";

import { withDatabaseTransaction, type HyperdriveBinding } from "./database";

export type CustomerAccessStatus = "waiting" | "approved" | "rejected";

export interface CustomerAccessRow {
  userId: string;
  organizationId: string;
  email: string;
  displayName: string | null;
  accountStatus: string;
  emailVerifiedAt: string | null;
  accessStatus: CustomerAccessStatus;
  rejectReason: string | null;
}

function normalizeReason(raw: string): string {
  const reason = raw.trim().replace(/\s+/g, " ");
  if (reason.length < 8 || reason.length > 500) {
    throw new Error("Reject reason must be between 8 and 500 characters.");
  }
  return reason;
}

function mapRow(row: Record<string, unknown>): CustomerAccessRow {
  const accessStatus = String(row.access_status);
  if (
    accessStatus !== "waiting" &&
    accessStatus !== "approved" &&
    accessStatus !== "rejected"
  ) {
    throw new Error("Invalid access status");
  }

  return {
    userId: String(row.user_id),
    organizationId: String(row.organization_id),
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
    accessStatus,
    rejectReason:
      row.reject_reason === null || row.reject_reason === undefined
        ? null
        : String(row.reject_reason),
  };
}

export async function listVerifiedCustomers(
  hyperdrive: HyperdriveBinding,
): Promise<CustomerAccessRow[]> {
  return withDatabaseTransaction(hyperdrive, async (client) => {
    const result = await client.query(
      `
        SELECT
          u.id AS user_id,
          u.organization_id,
          u.email,
          u.display_name,
          u.account_status,
          u.email_verified_at,
          COALESCE(d.status, 'waiting') AS access_status,
          d.reject_reason
        FROM users u
        INNER JOIN organizations o
          ON o.id = u.organization_id
        INNER JOIN user_roles ur
          ON ur.user_id = u.id
         AND ur.organization_id = u.organization_id
        LEFT JOIN customer_access_decisions d
          ON d.user_id = u.id
        WHERE ur.role = 'customer'
          AND ur.status = 'active'
          AND o.account_type <> 'internal'
          AND u.email_verified_at IS NOT NULL
          AND u.account_status = 'active'
        ORDER BY
          CASE COALESCE(d.status, 'waiting')
            WHEN 'waiting' THEN 0
            WHEN 'rejected' THEN 1
            ELSE 2
          END,
          LOWER(u.email)
      `,
    );
    return result.rows.map((row) => mapRow(row as Record<string, unknown>));
  });
}

export async function getCustomerDownloadStatus(
  hyperdrive: HyperdriveBinding,
  userId: string,
): Promise<CustomerAccessRow | null> {
  return withDatabaseTransaction(hyperdrive, async (client) => {
    const result = await client.query(
      `
        SELECT
          u.id AS user_id,
          u.organization_id,
          u.email,
          u.display_name,
          u.account_status,
          u.email_verified_at,
          COALESCE(d.status, 'waiting') AS access_status,
          d.reject_reason
        FROM users u
        INNER JOIN organizations o
          ON o.id = u.organization_id
        INNER JOIN user_roles ur
          ON ur.user_id = u.id
         AND ur.organization_id = u.organization_id
        LEFT JOIN customer_access_decisions d
          ON d.user_id = u.id
        WHERE u.id = $1
          AND ur.role = 'customer'
        LIMIT 1
      `,
      [userId],
    );
    if (result.rowCount !== 1) {
      return null;
    }
    return mapRow(result.rows[0] as Record<string, unknown>);
  });
}

async function lockCustomer(
  client: Client,
  userId: string,
): Promise<{ organizationId: string; email: string } | null> {
  const result = await client.query(
    `
      SELECT u.id, u.organization_id, u.email, u.account_status, u.email_verified_at, o.account_type
      FROM users u
      INNER JOIN organizations o ON o.id = u.organization_id
      INNER JOIN user_roles ur
        ON ur.user_id = u.id
       AND ur.organization_id = u.organization_id
      WHERE u.id = $1
        AND ur.role = 'customer'
        AND ur.status = 'active'
      FOR UPDATE OF u
    `,
    [userId],
  );
  if (result.rowCount !== 1) {
    return null;
  }
  const row = result.rows[0];
  if (
    String(row.account_type) === "internal" ||
    row.email_verified_at === null ||
    String(row.account_status) !== "active"
  ) {
    return null;
  }
  return {
    organizationId: String(row.organization_id),
    email: String(row.email),
  };
}

export async function approveCustomerAccess(
  hyperdrive: HyperdriveBinding,
  actorUserId: string,
  actorOrganizationId: string,
  targetUserId: string,
): Promise<CustomerAccessRow | null> {
  return withDatabaseTransaction(hyperdrive, async (client) => {
    const target = await lockCustomer(client, targetUserId);
    if (target === null) {
      return null;
    }

    await client.query(
      `
        INSERT INTO customer_access_decisions (
          id,
          organization_id,
          user_id,
          status,
          reject_reason,
          decided_by_user_id,
          decided_at,
          updated_at
        ) VALUES ($1, $2, $3, 'approved', NULL, $4, NOW(), NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET
          status = 'approved',
          reject_reason = NULL,
          decided_by_user_id = EXCLUDED.decided_by_user_id,
          decided_at = NOW(),
          updated_at = NOW()
      `,
      [crypto.randomUUID(), target.organizationId, targetUserId, actorUserId],
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
        ) VALUES ($1, $2, $3, 'CUSTOMER_ACCESS_APPROVED', 'user', $4, $5::jsonb)
      `,
      [
        crypto.randomUUID(),
        actorOrganizationId,
        actorUserId,
        targetUserId,
        JSON.stringify({ email: target.email.toLowerCase() }),
      ],
    );

    const listed = await client.query(
      `
        SELECT
          u.id AS user_id,
          u.organization_id,
          u.email,
          u.display_name,
          u.account_status,
          u.email_verified_at,
          d.status AS access_status,
          d.reject_reason
        FROM users u
        INNER JOIN customer_access_decisions d ON d.user_id = u.id
        WHERE u.id = $1
      `,
      [targetUserId],
    );
    return mapRow(listed.rows[0] as Record<string, unknown>);
  });
}

export async function rejectCustomerAccess(
  hyperdrive: HyperdriveBinding,
  actorUserId: string,
  actorOrganizationId: string,
  targetUserId: string,
  rawReason: string,
): Promise<CustomerAccessRow | null> {
  const reason = normalizeReason(rawReason);

  return withDatabaseTransaction(hyperdrive, async (client) => {
    const target = await lockCustomer(client, targetUserId);
    if (target === null) {
      return null;
    }

    await client.query(
      `
        INSERT INTO customer_access_decisions (
          id,
          organization_id,
          user_id,
          status,
          reject_reason,
          decided_by_user_id,
          decided_at,
          updated_at
        ) VALUES ($1, $2, $3, 'rejected', $5, $4, NOW(), NOW())
        ON CONFLICT (user_id)
        DO UPDATE SET
          status = 'rejected',
          reject_reason = EXCLUDED.reject_reason,
          decided_by_user_id = EXCLUDED.decided_by_user_id,
          decided_at = NOW(),
          updated_at = NOW()
      `,
      [crypto.randomUUID(), target.organizationId, targetUserId, actorUserId, reason],
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
        ) VALUES ($1, $2, $3, 'CUSTOMER_ACCESS_REJECTED', 'user', $4, $5::jsonb)
      `,
      [
        crypto.randomUUID(),
        actorOrganizationId,
        actorUserId,
        targetUserId,
        JSON.stringify({ email: target.email.toLowerCase(), reason }),
      ],
    );

    const listed = await client.query(
      `
        SELECT
          u.id AS user_id,
          u.organization_id,
          u.email,
          u.display_name,
          u.account_status,
          u.email_verified_at,
          d.status AS access_status,
          d.reject_reason
        FROM users u
        INNER JOIN customer_access_decisions d ON d.user_id = u.id
        WHERE u.id = $1
      `,
      [targetUserId],
    );
    return mapRow(listed.rows[0] as Record<string, unknown>);
  });
}
