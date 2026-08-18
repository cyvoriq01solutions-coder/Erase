import { hashSessionToken } from "./authCrypto";
import { queryDatabase, type HyperdriveBinding } from "./database";

export const CUSTOMER_SESSION_COOKIE = "cyvoriq_session";

export interface AuthenticatedSession {
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

export function readSessionToken(request: Request): string | null {
  const token = parseCookies(request.headers.get("Cookie")).get(
    CUSTOMER_SESSION_COOKIE,
  );

  if (token === undefined || !/^[0-9a-f]{64}$/i.test(token)) {
    return null;
  }

  return token.toLowerCase();
}

export function buildSessionCookie(token: string, expiresAt: string): string {
  const expires = new Date(expiresAt);
  const maxAgeSeconds = Math.max(
    0,
    Math.floor((expires.getTime() - Date.now()) / 1000),
  );

  return [
    `${CUSTOMER_SESSION_COOKIE}=${token}`,
    "Path=/",
    `Max-Age=${maxAgeSeconds}`,
    `Expires=${expires.toUTCString()}`,
    "HttpOnly",
    "Secure",
    "SameSite=Lax",
  ].join("; ");
}

export function buildExpiredSessionCookie(): string {
  return [
    `${CUSTOMER_SESSION_COOKIE}=`,
    "Path=/",
    "Max-Age=0",
    "Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    "HttpOnly",
    "Secure",
    "SameSite=Lax",
  ].join("; ");
}

export async function getAuthenticatedSession(
  hyperdrive: HyperdriveBinding,
  token: string,
): Promise<AuthenticatedSession | null> {
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
        COALESCE(
          ARRAY_AGG(ur.role ORDER BY ur.role)
            FILTER (WHERE ur.status = 'active'),
          ARRAY[]::text[]
        ) AS roles
      FROM customer_sessions s
      INNER JOIN users u
        ON u.id = s.user_id
       AND u.organization_id = s.organization_id
      INNER JOIN organizations o
        ON o.id = s.organization_id
      LEFT JOIN user_roles ur
        ON ur.user_id = s.user_id
       AND ur.organization_id = s.organization_id
      WHERE s.token_hash = $1
        AND s.revoked_at IS NULL
        AND s.expires_at > NOW()
        AND u.account_status = 'active'
      GROUP BY
        s.id,
        s.organization_id,
        s.user_id,
        s.expires_at,
        o.slug,
        u.email,
        u.display_name,
        u.account_status
      LIMIT 1
    `,
    [tokenHash],
  );

  if (rows.length !== 1) {
    return null;
  }

  const row = rows[0];
  const rawRoles = Array.isArray(row.roles) ? row.roles : [];

  await queryDatabase(
    hyperdrive,
    `
      UPDATE customer_sessions
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
    roles: rawRoles.map((role) => String(role)),
    expiresAt: new Date(String(row.expires_at)).toISOString(),
  };
}

export async function revokeSession(
  hyperdrive: HyperdriveBinding,
  token: string,
): Promise<void> {
  const tokenHash = await hashSessionToken(token);
  await queryDatabase(
    hyperdrive,
    `
      UPDATE customer_sessions
      SET revoked_at = COALESCE(revoked_at, NOW())
      WHERE token_hash = $1
    `,
    [tokenHash],
  );
}
