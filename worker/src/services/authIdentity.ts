import { queryDatabase, type HyperdriveBinding } from "./database";

export interface CustomerIdentity {
  organizationId: string;
  organizationSlug: string;
  userId: string;
  email: string;
  displayName: string | null;
}

export function normalizeEmail(value: string): string | null {
  const normalized = value.trim().toLowerCase();

  if (
    normalized.length < 3 ||
    normalized.length > 254 ||
    !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalized)
  ) {
    return null;
  }

  return normalized;
}

export function normalizeOrganizationSlug(value: string): string | null {
  const normalized = value.trim().toLowerCase();

  if (
    normalized.length < 2 ||
    normalized.length > 80 ||
    !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(normalized)
  ) {
    return null;
  }

  return normalized;
}

export async function findCustomerIdentity(
  hyperdrive: HyperdriveBinding,
  organizationSlug: string,
  email: string,
): Promise<CustomerIdentity | null> {
  const rows = await queryDatabase(
    hyperdrive,
    `
      SELECT
        o.id AS organization_id,
        o.slug AS organization_slug,
        u.id AS user_id,
        u.email,
        u.display_name
      FROM organizations o
      INNER JOIN users u
        ON u.organization_id = o.id
      WHERE LOWER(o.slug) = $1
        AND LOWER(u.email) = $2
      LIMIT 2
    `,
    [organizationSlug, email],
  );

  if (rows.length !== 1) {
    return null;
  }

  const row = rows[0];
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
