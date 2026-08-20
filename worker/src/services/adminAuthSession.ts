import { hashSessionToken } from "./authCrypto";
import { queryDatabase, type HyperdriveBinding } from "./database";
import { CYVORIQ_INTERNAL_ORG_ID } from "./adminIdentity";

export const ADMIN_SESSION_COOKIE = "__Host-cyvoriq_admin_session";
export const LEGACY_ADMIN_SESSION_COOKIE = "cyvoriq_admin_session";

export interface AuthenticatedAdminSession {
  sessionId: string;
  organizationId: string;
  organizationSlug: string;
  userId: string;
  email: string;
  displayName: string | null;
  accountStatus: string;
  roles: string[];
  expiresAt: string;
}

function parseCookies(header: string | null): Map<string, string> {
  const cookies = new Map<string, string>();
  if (header === null || header.length === 0) {
    return cookies;
  }

  for (const part of header.split(";")) {
    const separator = part.indexOf("=");
    if (separator <= 0) {
      continue;
    }
    const name = part.slice(0, separator).trim();
    const value = part.slice(separator + 1).trim();
    if (name.length > 0) {
      cookies.set(name, value);
    }
  }

  return cookies;
}

export function readAdminSessionToken(request: Request): string | null {
  const token = parseCookies(request.headers.get("Cookie")).get(
    ADMIN_SESSION_COOKIE,
  );

  if (token === undefined || !/^[0-9a-f]{64}$/i.test(token)) {
    return null;
  }
  return token.toLowerCase();
}

export function buildAdminSessionCookie(token: string, expiresAt: string): string {
  const expires = new Date(expiresAt);
  const maxAgeSeconds = Math.max(
    0,
    Math.floor((expires.getTime() - Date.now()) / 1000),
  );

  return [
    `${ADMIN_SESSION_COOKIE}=${token}`,
    "Path=/",
    `Max-Age=${maxAgeSeconds}`,
    `Expires=${expires.toUTCString()}`,
    "HttpOnly",
    "Secure",
    "SameSite=Strict",
  ].join("; ");
}

export function buildExpiredAdminSessionCookie(): string {
  return [
    `${ADMIN_SESSION_COOKIE}=`,
    "Path=/",
    "Max-Age=0",
    "Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    "HttpOnly",
    "Secure",
    "SameSite=Strict",
  ].join("; ");
}

export function buildExpiredLegacyAdminSessionCookie(): string {
  return [
    `${LEGACY_ADMIN_SESSION_COOKIE}=`,
    "Path=/",
    "Max-Age=0",
    "Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    "HttpOnly",
    "Secure",
    "SameSite=Lax",
  ].join("; ");
}

export async function getAuthenticatedAdminSession(
  hyperdrive: HyperdriveBinding,
  token: string,
): Promise<AuthenticatedAdminSession | null> {
  const tokenHash = await hashSessionToken(token);
  const rows = await queryDatabase(
    hyperdrive,
    `
      SELECT
        s.id AS session_id,
        s.organization_id,
        s.user_id,
        s.expires_at,
        o.slug AS organization_slug,
        u.email,
        u.display_name,
        u.account_status,
        u.email_verified_at,
        COALESCE(
          ARRAY_AGG(ur.role ORDER BY ur.role)
            FILTER (
              WHERE ur.status = 'active'
                AND ur.role IN ('super_admin', 'accounts_admin')
            ),
          ARRAY[]::text[]
        ) AS roles
      FROM admin_sessions s
      INNER JOIN users u
        ON u.id = s.user_id
       AND u.organization_id = s.organization_id
      INNER JOIN organizations o
        ON o.id = s.organization_id
      LEFT JOIN user_roles ur
        ON ur.user_id = s.user_id
       AND ur.organization_id = s.organization_id
      WHERE s.token_hash = $1
        AND s.organization_id = $2
        AND s.revoked_at IS NULL
        AND s.expires_at > NOW()
        AND u.account_status = 'active'
        AND u.email_verified_at IS NOT NULL
        AND o.account_type = 'internal'
      GROUP BY
        s.id,
        s.organization_id,
        s.user_id,
        s.expires_at,
        o.slug,
        u.email,
        u.display_name,
        u.account_status,
        u.email_verified_at
      LIMIT 1
    `,
    [tokenHash, CYVORIQ_INTERNAL_ORG_ID],
  );

  if (rows.length !== 1) {
    return null;
  }

  const row = rows[0];
  const rawRoles = Array.isArray(row.roles) ? row.roles : [];
  const roles = rawRoles.map((role) => String(role));
  if (!roles.some((role) => role === "super_admin" || role === "accounts_admin")) {
    return null;
  }

  await queryDatabase(
    hyperdrive,
    `
      UPDATE admin_sessions
      SET last_seen_at = NOW()
      WHERE id = $1
        AND (last_seen_at IS NULL OR last_seen_at < NOW() - INTERVAL '5 minutes')
    `,
    [String(row.session_id)],
  );

  return {
    sessionId: String(row.session_id),
    organizationId: String(row.organization_id),
    organizationSlug: String(row.organization_slug),
    userId: String(row.user_id),
    email: String(row.email),
    displayName:
      row.display_name === null || row.display_name === undefined
        ? null
        : String(row.display_name),
    accountStatus: String(row.account_status),
    roles,
    expiresAt: new Date(String(row.expires_at)).toISOString(),
  };
}

export async function revokeAdminSession(
  hyperdrive: HyperdriveBinding,
  token: string,
): Promise<void> {
  const tokenHash = await hashSessionToken(token);
  await queryDatabase(
    hyperdrive,
    `
      UPDATE admin_sessions
      SET revoked_at = COALESCE(revoked_at, NOW())
      WHERE token_hash = $1
    `,
    [tokenHash],
  );
}
