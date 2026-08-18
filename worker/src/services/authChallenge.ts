import type { Client } from "pg";

import {
  generateOneTimeCode,
  generateSessionToken,
  hashOneTimeCode,
  hashSessionToken,
  verifyOneTimeCode,
} from "./authCrypto";
import type { CustomerIdentity } from "./authIdentity";
import {
  isAccountsAuthorityIdentity,
  isBootstrapSuperUser,
} from "./authorizationPolicy";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "./database";

const CHALLENGE_TTL_MS = 10 * 60 * 1000;
const SESSION_TTL_MS = 12 * 60 * 60 * 1000;

export interface IssuedLoginChallenge {
  challengeId: string;
  code: string;
  expiresAt: string;
}

export type VerifyLoginChallengeResult =
  | { status: "invalid" }
  | {
      status: "authenticated";
      sessionId: string;
      sessionToken: string;
      organizationId: string;
      userId: string;
      expiresAt: string;
    };

async function lockCustomerIdentity(
  client: Client,
  identity: CustomerIdentity,
): Promise<boolean> {
  const result = await client.query(
    `
      SELECT id, account_status
      FROM users
      WHERE id = $1
        AND organization_id = $2
      FOR UPDATE
    `,
    [identity.userId, identity.organizationId],
  );

  if (result.rowCount !== 1) {
    return false;
  }

  const status = String(result.rows[0].account_status);
  return status !== "suspended" && status !== "closed";
}

async function activateVerifiedIdentity(
  client: Client,
  organizationId: string,
  userId: string,
): Promise<boolean> {
  const result = await client.query(
    `
      SELECT email, account_status, email_verified_at
      FROM users
      WHERE id = $1
        AND organization_id = $2
      FOR UPDATE
    `,
    [userId, organizationId],
  );

  if (result.rowCount !== 1) {
    return false;
  }

  const user = result.rows[0];
  const email = String(user.email).trim().toLowerCase();
  const accountStatus = String(user.account_status);

  if (accountStatus === "suspended" || accountStatus === "closed") {
    return false;
  }

  const firstVerification = user.email_verified_at === null;

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

  if (isBootstrapSuperUser(email)) {
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
  } else if (!isAccountsAuthorityIdentity(email)) {
    await client.query(
      `
        UPDATE user_roles
        SET status = 'active',
            approved_at = COALESCE(approved_at, NOW()),
            revoked_at = NULL
        WHERE user_id = $1
          AND organization_id = $2
          AND role = 'customer'
      `,
      [userId, organizationId],
    );
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
        ) VALUES ($1, $2, $3, 'EMAIL_VERIFIED', 'user', $3, $4::jsonb)
      `,
      [
        crypto.randomUUID(),
        organizationId,
        userId,
        JSON.stringify({ method: "email_otp" }),
      ],
    );
  }

  return true;
}

export async function issueLoginChallenge(
  hyperdrive: HyperdriveBinding,
  pepper: string,
  identity: CustomerIdentity,
): Promise<IssuedLoginChallenge> {
  const challengeId = crypto.randomUUID();
  const code = generateOneTimeCode();
  const expiresAt = new Date(Date.now() + CHALLENGE_TTL_MS);
  const challengeHash = await hashOneTimeCode(
    pepper,
    challengeId,
    identity.organizationId,
    identity.userId,
    code,
  );

  return withDatabaseTransaction(hyperdrive, async (client) => {
    if (!(await lockCustomerIdentity(client, identity))) {
      throw new Error("Customer identity is unavailable");
    }

    await client.query(
      `
        UPDATE login_challenges
        SET consumed_at = NOW()
        WHERE organization_id = $1
          AND user_id = $2
          AND consumed_at IS NULL
      `,
      [identity.organizationId, identity.userId],
    );

    await client.query(
      `
        INSERT INTO login_challenges (
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

export async function verifyLoginChallenge(
  hyperdrive: HyperdriveBinding,
  pepper: string,
  challengeId: string,
  code: string,
): Promise<VerifyLoginChallengeResult> {
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
        FROM login_challenges
        WHERE id = $1
        FOR UPDATE
      `,
      [challengeId],
    );

    if (challengeResult.rowCount !== 1) {
      return { status: "invalid" };
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
          "UPDATE login_challenges SET consumed_at = NOW() WHERE id = $1",
          [challengeId],
        );
      }
      return { status: "invalid" };
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
          UPDATE login_challenges
          SET attempts = $2,
              consumed_at = CASE WHEN $2 >= max_attempts THEN NOW() ELSE consumed_at END
          WHERE id = $1
        `,
        [challengeId, nextAttempts],
      );
      return { status: "invalid" };
    }

    if (!(await activateVerifiedIdentity(client, organizationId, userId))) {
      await client.query(
        "UPDATE login_challenges SET consumed_at = NOW() WHERE id = $1",
        [challengeId],
      );
      return { status: "invalid" };
    }

    const sessionId = crypto.randomUUID();
    const sessionToken = generateSessionToken();
    const sessionHash = await hashSessionToken(sessionToken);
    const sessionExpiresAt = new Date(Date.now() + SESSION_TTL_MS);

    await client.query(
      `
        UPDATE login_challenges
        SET consumed_at = NOW()
        WHERE id = $1
      `,
      [challengeId],
    );

    await client.query(
      `
        INSERT INTO customer_sessions (
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

    return {
      status: "authenticated",
      sessionId,
      sessionToken,
      organizationId,
      userId,
      expiresAt: sessionExpiresAt.toISOString(),
    };
  });
}
