import type { Client } from "pg";

import {
  generateOneTimeCode,
  generateSessionToken,
  hashOneTimeCode,
  hashSessionToken,
  verifyOneTimeCode,
} from "./authCrypto";
import type { AdminIdentity, AdminRoleStatus } from "./adminIdentity";
import { isBootstrapSuperUser, type ControlPanelRole } from "./authorizationPolicy";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "./database";

const ADMIN_CHALLENGE_TTL_MS = 10 * 60 * 1000;
const ADMIN_SESSION_TTL_MS = 4 * 60 * 60 * 1000;

export interface IssuedAdminLoginChallenge {
  challengeId: string;
  code: string;
  expiresAt: string;
}

export type VerifyAdminLoginChallengeResult =
  | { status: "invalid" }
  | {
      status: "pending_approval";
      organizationId: string;
      userId: string;
      email: string;
      role: ControlPanelRole;
    }
  | {
      status: "revoked";
      organizationId: string;
      userId: string;
      email: string;
      role: ControlPanelRole;
    }
  | {
      status: "authenticated";
      sessionId: string;
      sessionToken: string;
      organizationId: string;
      userId: string;
      email: string;
      role: ControlPanelRole;
      expiresAt: string;
    };

async function lockAdminIdentity(
  client: Client,
  identity: AdminIdentity,
): Promise<boolean> {
  const result = await client.query(
    `
      SELECT u.id, u.account_status, o.account_type
      FROM users u
      INNER JOIN organizations o ON o.id = u.organization_id
      WHERE u.id = $1
        AND u.organization_id = $2
      FOR UPDATE OF u
    `,
    [identity.userId, identity.organizationId],
  );

  if (result.rowCount !== 1) {
    return false;
  }

  const row = result.rows[0];
  const status = String(row.account_status);
  return (
    String(row.account_type) === "internal" &&
    status !== "suspended" &&
    status !== "closed"
  );
}

async function verifyAdminIdentityState(
  client: Client,
  organizationId: string,
  userId: string,
): Promise<{
  email: string;
  role: ControlPanelRole;
  roleStatus: AdminRoleStatus;
  firstVerification: boolean;
} | null> {
  const result = await client.query(
    `
      SELECT
        u.email,
        u.account_status,
        u.email_verified_at,
        o.account_type,
        ur.role,
        ur.status AS role_status
      FROM users u
      INNER JOIN organizations o
        ON o.id = u.organization_id
      INNER JOIN user_roles ur
        ON ur.user_id = u.id
       AND ur.organization_id = u.organization_id
      WHERE u.id = $1
        AND u.organization_id = $2
        AND ur.role IN ('super_admin', 'accounts_admin')
      ORDER BY CASE ur.role WHEN 'super_admin' THEN 0 ELSE 1 END
      LIMIT 1
      FOR UPDATE OF u, ur
    `,
    [userId, organizationId],
  );

  if (result.rowCount !== 1) {
    return null;
  }

  const row = result.rows[0];
  if (
    String(row.account_type) !== "internal" ||
    String(row.account_status) === "suspended" ||
    String(row.account_status) === "closed"
  ) {
    return null;
  }

  const email = String(row.email).trim().toLowerCase();
  const role = String(row.role) as ControlPanelRole;
  let roleStatus = String(row.role_status) as AdminRoleStatus;
  const firstVerification = row.email_verified_at === null;

  await client.query(
    `
      UPDATE users
      SET email_verified_at = COALESCE(email_verified_at, NOW()),
          account_status = CASE
            WHEN account_status = 'pending_email_verification' THEN 'active'
            ELSE account_status
          END
      WHERE id = $1
        AND organization_id = $2
    `,
    [userId, organizationId],
  );

  if (isBootstrapSuperUser(email) && role === "super_admin") {
    await client.query(
      `
        UPDATE user_roles
        SET status = 'active',
            approved_at = COALESCE(approved_at, NOW()),
            revoked_at = NULL
        WHERE user_id = $1
          AND organization_id = $2
          AND role = 'super_admin'
      `,
      [userId, organizationId],
    );
    roleStatus = "active";
  }

  if (firstVerification) {
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
        ) VALUES ($1, $2, $3, 'ADMIN_EMAIL_VERIFIED', 'user', $3, $4::jsonb)
      `,
      [
        crypto.randomUUID(),
        organizationId,
        userId,
        JSON.stringify({ method: "email_otp", role }),
      ],
    );
  }

  return { email, role, roleStatus, firstVerification };
}

export async function issueAdminLoginChallenge(
  hyperdrive: HyperdriveBinding,
  pepper: string,
  identity: AdminIdentity,
): Promise<IssuedAdminLoginChallenge> {
  const challengeId = crypto.randomUUID();
  const code = generateOneTimeCode();
  const expiresAt = new Date(Date.now() + ADMIN_CHALLENGE_TTL_MS);
  const challengeHash = await hashOneTimeCode(
    pepper,
    challengeId,
    identity.organizationId,
    identity.userId,
    code,
  );

  return withDatabaseTransaction(hyperdrive, async (client) => {
    if (!(await lockAdminIdentity(client, identity))) {
      throw new Error("Admin identity is unavailable");
    }

    await client.query(
      `
        UPDATE admin_login_challenges
        SET consumed_at = NOW()
        WHERE organization_id = $1
          AND user_id = $2
          AND consumed_at IS NULL
      `,
      [identity.organizationId, identity.userId],
    );

    await client.query(
      `
        INSERT INTO admin_login_challenges (
          id,
          organization_id,
          user_id,
          challenge_hash,
          delivery_channel,
          attempts,
          max_attempts,
          expires_at
        ) VALUES ($1, $2, $3, $4, 'email', 0, 5, $5)
      `,
      [
        challengeId,
        identity.organizationId,
        identity.userId,
        challengeHash,
        expiresAt.toISOString(),
      ],
    );

    return {
      challengeId,
      code,
      expiresAt: expiresAt.toISOString(),
    };
  });
}

export async function verifyAdminLoginChallenge(
  hyperdrive: HyperdriveBinding,
  pepper: string,
  challengeId: string,
  code: string,
): Promise<VerifyAdminLoginChallengeResult> {
  if (!/^[0-9]{6}$/.test(code)) {
    return { status: "invalid" };
  }

  return withDatabaseTransaction(hyperdrive, async (client) => {
    const challengeResult = await client.query(
      `
        SELECT
          id,
          organization_id,
          user_id,
          challenge_hash,
          attempts,
          max_attempts,
          expires_at,
          consumed_at
        FROM admin_login_challenges
        WHERE id = $1
        FOR UPDATE
      `,
      [challengeId],
    );

    if (challengeResult.rowCount !== 1) {
      return { status: "invalid" } as const;
    }

    const challenge = challengeResult.rows[0];
    const organizationId = String(challenge.organization_id);
    const userId = String(challenge.user_id);
    const attempts = Number(challenge.attempts);
    const maxAttempts = Number(challenge.max_attempts);
    const expiresAt = new Date(String(challenge.expires_at));

    if (
      challenge.consumed_at !== null ||
      attempts >= maxAttempts ||
      expiresAt.getTime() <= Date.now()
    ) {
      if (challenge.consumed_at === null) {
        await client.query(
          "UPDATE admin_login_challenges SET consumed_at = NOW() WHERE id = $1",
          [challengeId],
        );
      }
      return { status: "invalid" } as const;
    }

    const valid = await verifyOneTimeCode(
      pepper,
      challengeId,
      organizationId,
      userId,
      code,
      String(challenge.challenge_hash),
    );

    if (!valid) {
      const nextAttempts = attempts + 1;
      await client.query(
        `
          UPDATE admin_login_challenges
          SET attempts = $2,
              consumed_at = CASE WHEN $2 >= max_attempts THEN NOW() ELSE consumed_at END
          WHERE id = $1
        `,
        [challengeId, nextAttempts],
      );
      return { status: "invalid" } as const;
    }

    const identity = await verifyAdminIdentityState(client, organizationId, userId);
    await client.query(
      "UPDATE admin_login_challenges SET consumed_at = NOW() WHERE id = $1",
      [challengeId],
    );

    if (identity === null) {
      return { status: "invalid" } as const;
    }

    if (identity.roleStatus !== "active") {
      return {
        status: identity.roleStatus === "revoked" ? "revoked" : "pending_approval",
        organizationId,
        userId,
        email: identity.email,
        role: identity.role,
      } as const;
    }

    const sessionId = crypto.randomUUID();
    const sessionToken = generateSessionToken();
    const sessionHash = await hashSessionToken(sessionToken);
    const sessionExpiresAt = new Date(Date.now() + ADMIN_SESSION_TTL_MS);

    await client.query(
      `
        INSERT INTO admin_sessions (
          id,
          organization_id,
          user_id,
          token_hash,
          expires_at,
          last_seen_at
        ) VALUES ($1, $2, $3, $4, $5, NOW())
      `,
      [
        sessionId,
        organizationId,
        userId,
        sessionHash,
        sessionExpiresAt.toISOString(),
      ],
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
        ) VALUES ($1, $2, $3, 'ADMIN_LOGIN', 'admin_session', $4, $5::jsonb)
      `,
      [
        crypto.randomUUID(),
        organizationId,
        userId,
        sessionId,
        JSON.stringify({ role: identity.role, method: "email_otp" }),
      ],
    );

    return {
      status: "authenticated",
      sessionId,
      sessionToken,
      organizationId,
      userId,
      email: identity.email,
      role: identity.role,
      expiresAt: sessionExpiresAt.toISOString(),
    } as const;
  });
}
