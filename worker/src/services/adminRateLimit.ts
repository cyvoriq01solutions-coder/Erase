import type { Client } from "pg";

import {
  hashAdminRateLimitKey,
  type AdminRateLimitScope,
} from "./authCrypto";
import { normalizeEmail } from "./authIdentity";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "./database";

const ADMIN_RATE_LIMIT_WINDOW_MINUTES = 15;
const ADMIN_RATE_LIMIT_RETENTION_HOURS = 24;
const ADMIN_RATE_LIMIT_CLEANUP_BATCH_SIZE = 100;

const ADMIN_RATE_LIMITS: Record<AdminRateLimitScope, number> = {
  source: 5,
  identity: 3,
};

export interface AdminRateLimitResult {
  allowed: boolean;
  scope: AdminRateLimitScope;
}

function normalizeIpv4Address(value: string): string | null {
  const parts = value.split(".");
  if (parts.length !== 4) {
    return null;
  }

  const normalized: string[] = [];
  for (const part of parts) {
    if (!/^(?:0|[1-9][0-9]{0,2})$/.test(part)) {
      return null;
    }

    const octet = Number(part);
    if (octet > 255) {
      return null;
    }
    normalized.push(String(octet));
  }

  return normalized.join(".");
}

function normalizeIpv6Address(value: string): string | null {
  if (!value.includes(":") || !/^[0-9a-f:.]+$/i.test(value)) {
    return null;
  }

  try {
    const hostname = new URL(`http://[${value}]/`).hostname;
    if (!hostname.startsWith("[") || !hostname.endsWith("]")) {
      return null;
    }
    return hostname.slice(1, -1).toLowerCase();
  } catch {
    return null;
  }
}

export function normalizeAdminRateLimitSource(
  value: string,
): string | null {
  if (
    value.length < 3 ||
    value.length > 45 ||
    value !== value.trim() ||
    value.includes(",")
  ) {
    return null;
  }

  return value.includes(":")
    ? normalizeIpv6Address(value)
    : normalizeIpv4Address(value);
}

export function readAdminRateLimitSource(
  request: Request,
): string | null {
  const value = request.headers.get("CF-Connecting-IP");
  return value === null ? null : normalizeAdminRateLimitSource(value);
}

export function normalizeAdminRateLimitValue(
  scope: AdminRateLimitScope,
  value: string | null,
): string | null {
  if (value === null) {
    return null;
  }

  return scope === "identity"
    ? normalizeEmail(value)
    : normalizeAdminRateLimitSource(value);
}

async function pruneExpiredAdminRateLimits(client: Client): Promise<void> {
  await client.query(
    `
      WITH expired AS (
        SELECT scope, key_hash
        FROM admin_auth_rate_limits
        WHERE updated_at < clock_timestamp()
          - ($1::integer * INTERVAL '1 hour')
        ORDER BY updated_at
        FOR UPDATE SKIP LOCKED
        LIMIT $2
      )
      DELETE FROM admin_auth_rate_limits AS rate_limit
      USING expired
      WHERE rate_limit.scope = expired.scope
        AND rate_limit.key_hash = expired.key_hash
    `,
    [
      ADMIN_RATE_LIMIT_RETENTION_HOURS,
      ADMIN_RATE_LIMIT_CLEANUP_BATCH_SIZE,
    ],
  );
}

export async function consumeAdminRateLimit(
  hyperdrive: HyperdriveBinding,
  pepper: string,
  scope: AdminRateLimitScope,
  value: string | null,
): Promise<AdminRateLimitResult> {
  const normalizedValue = normalizeAdminRateLimitValue(scope, value);
  if (normalizedValue === null) {
    return { allowed: false, scope };
  }

  const keyHash = await hashAdminRateLimitKey(
    pepper,
    scope,
    normalizedValue,
  );
  const limit = ADMIN_RATE_LIMITS[scope];

  return withDatabaseTransaction(hyperdrive, async (client) => {
    await pruneExpiredAdminRateLimits(client);

    await client.query(
      `
        INSERT INTO admin_auth_rate_limits (
          scope,
          key_hash,
          request_timestamps
        )
        VALUES ($1, $2, ARRAY[]::TIMESTAMPTZ[])
        ON CONFLICT (scope, key_hash) DO NOTHING
      `,
      [scope, keyHash],
    );

    const result = await client.query(
      `
        WITH locked AS MATERIALIZED (
          SELECT request_timestamps
          FROM admin_auth_rate_limits
          WHERE scope = $1
            AND key_hash = $2
          FOR UPDATE
        ),
        evaluated AS MATERIALIZED (
          SELECT
            request_timestamps,
            clock_timestamp() AS evaluated_at
          FROM locked
        ),
        cleaned AS MATERIALIZED (
          SELECT
            ARRAY(
              SELECT ts
              FROM unnest(evaluated.request_timestamps) AS ts
              WHERE ts > evaluated.evaluated_at
                - ($4::integer * INTERVAL '1 minute')
              ORDER BY ts
            ) AS recent,
            evaluated.evaluated_at
          FROM evaluated
        )
        UPDATE admin_auth_rate_limits AS rate_limit
        SET
          request_timestamps = CASE
            WHEN cardinality(cleaned.recent) < $3::integer
              THEN cleaned.recent || ARRAY[cleaned.evaluated_at]
            ELSE cleaned.recent
          END,
          updated_at = cleaned.evaluated_at
        FROM cleaned
        WHERE rate_limit.scope = $1
          AND rate_limit.key_hash = $2
        RETURNING
          cardinality(cleaned.recent) < $3::integer AS allowed
      `,
      [
        scope,
        keyHash,
        limit,
        ADMIN_RATE_LIMIT_WINDOW_MINUTES,
      ],
    );

    if (result.rowCount !== 1) {
      throw new Error("Unable to evaluate admin authentication rate limit");
    }

    return {
      allowed: result.rows[0].allowed === true,
      scope,
    };
  });
}
