import {
  issueAdminLoginChallenge,
  verifyAdminLoginChallenge,
} from "../services/adminChallenge";
import {
  resolveAdminIdentityForLogin,
} from "../services/adminIdentity";
import {
  buildAdminSessionCookie,
  buildExpiredAdminSessionCookie,
  buildExpiredLegacyAdminSessionCookie,
  getAuthenticatedAdminSession,
  readAdminSessionToken,
  revokeAdminSession,
} from "../services/adminAuthSession";
import {
  deliverLoginCode,
  type AuthDeliveryEnv,
} from "../services/authDelivery";
import {
  queryDatabase,
  type HyperdriveBinding,
} from "../services/database";
import { json } from "../services/http";

export interface AdminAuthApiEnv extends AuthDeliveryEnv {
  HYPERDRIVE: HyperdriveBinding;
  AUTH_PEPPER: string;
}

const MAX_JSON_BODY_BYTES = 8 * 1024;
const PUBLIC_ADMIN_CHALLENGE_TTL_MS = 10 * 60 * 1000;
const PUBLIC_ADMIN_CHALLENGE_MESSAGE =
  "If this identity is eligible for CYVRA administration, a verification code will be sent.";

function buildAcceptedAdminChallengeResponse(
  challengeId: string = crypto.randomUUID(),
  expiresAt: string = new Date(
    Date.now() + PUBLIC_ADMIN_CHALLENGE_TTL_MS,
  ).toISOString(),
): Response {
  return json(
    {
      status: "accepted",
      challengeId,
      expiresAt,
      message: PUBLIC_ADMIN_CHALLENGE_MESSAGE,
    },
    { status: 202 },
  );
}

function isDeliveryConfigured(env: AdminAuthApiEnv): boolean {
  return Boolean(
    env.AUTH_EMAIL_ENDPOINT?.trim() &&
    env.AUTH_EMAIL_TOKEN?.trim() &&
    env.AUTH_EMAIL_FROM?.trim(),
  );
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

async function consumeChallenge(
  env: AdminAuthApiEnv,
  challengeId: string,
): Promise<void> {
  await queryDatabase(
    env.HYPERDRIVE,
    `
      UPDATE admin_login_challenges
      SET consumed_at = COALESCE(consumed_at, NOW())
      WHERE id = $1
    `,
    [challengeId],
  );
}

export async function handleAdminRequestCode(
  request: Request,
  env: AdminAuthApiEnv,
): Promise<Response> {
  if (!isDeliveryConfigured(env)) {
    return json(
      { error: "admin_email_unavailable", message: "Admin email verification is temporarily unavailable." },
      { status: 503 },
    );
  }

  let body: Record<string, unknown>;
  try {
    body = await readJsonObject(request);
  } catch {
    return json({ error: "invalid_request", message: "Invalid request body." }, { status: 400 });
  }

  const rawEmail = body.email;
  if (typeof rawEmail !== "string") {
    return json({ error: "invalid_email", message: "Enter a valid CYVORIQ email address." }, { status: 400 });
  }

  const identity = await resolveAdminIdentityForLogin(env.HYPERDRIVE, rawEmail);
  if (
    identity === null ||
    identity.accountStatus === "suspended" ||
    identity.accountStatus === "closed" ||
    identity.roleStatus === "revoked"
  ) {
    return buildAcceptedAdminChallengeResponse();
  }

  const challenge = await issueAdminLoginChallenge(
    env.HYPERDRIVE,
    env.AUTH_PEPPER,
    identity,
  );

  try {
    await deliverLoginCode(env, {
      email: identity.email,
      code: challenge.code,
      challengeId: challenge.challengeId,
      expiresAt: challenge.expiresAt,
    });
  } catch {
    await consumeChallenge(env, challenge.challengeId);
    return buildAcceptedAdminChallengeResponse();
  }

  return buildAcceptedAdminChallengeResponse(
    challenge.challengeId,
    challenge.expiresAt,
  );
}

export async function handleAdminVerifyCode(
  request: Request,
  env: AdminAuthApiEnv,
): Promise<Response> {
  let body: Record<string, unknown>;
  try {
    body = await readJsonObject(request);
  } catch {
    return json({ error: "invalid_request", message: "Invalid request body." }, { status: 400 });
  }

  const challengeId = body.challengeId;
  const code = body.code;
  if (
    typeof challengeId !== "string" ||
    typeof code !== "string" ||
    !/^[0-9a-f-]{36}$/i.test(challengeId) ||
    !/^[0-9]{6}$/.test(code)
  ) {
    return json(
      { error: "invalid_code", message: "The verification code is invalid or expired." },
      { status: 401 },
    );
  }

  const result = await verifyAdminLoginChallenge(
    env.HYPERDRIVE,
    env.AUTH_PEPPER,
    challengeId,
    code,
  );

  if (result.status === "invalid") {
    return json(
      { error: "invalid_code", message: "The verification code is invalid or expired." },
      { status: 401 },
    );
  }

  if (result.status === "pending_approval") {
    return json(
      {
        authenticated: false,
        status: "pending_approval",
        role: result.role,
        message: "Email verified. Administrator role approval is still required.",
      },
      { status: 200 },
    );
  }

  if (result.status === "revoked") {
    return json(
      {
        error: "admin_role_revoked",
        message: "This administration role has been revoked.",
      },
      { status: 403 },
    );
  }

  const response = json(
    {
      authenticated: true,
      status: "authenticated",
      role: result.role,
      expiresAt: result.expiresAt,
    },
    { status: 200 },
  );
  response.headers.append(
    "Set-Cookie",
    buildAdminSessionCookie(result.sessionToken, result.expiresAt),
  );
  response.headers.append(
    "Set-Cookie",
    buildExpiredLegacyAdminSessionCookie(),
  );
  return response;
}

export async function handleAdminAuthSession(
  request: Request,
  env: AdminAuthApiEnv,
): Promise<Response> {
  const token = readAdminSessionToken(request);
  if (token === null) {
    const response = json({ authorized: false }, { status: 200 });
    response.headers.append(
      "Set-Cookie",
      buildExpiredLegacyAdminSessionCookie(),
    );
    return response;
  }

  const session = await getAuthenticatedAdminSession(env.HYPERDRIVE, token);
  if (session === null) {
    const response = json({ authorized: false }, { status: 200 });
    response.headers.append("Set-Cookie", buildExpiredAdminSessionCookie());
    response.headers.append(
      "Set-Cookie",
      buildExpiredLegacyAdminSessionCookie(),
    );
    return response;
  }

  const response = json(
    {
      authorized: true,
      user: {
        id: session.userId,
        email: session.email,
        displayName: session.displayName,
        organizationId: session.organizationId,
        organizationSlug: session.organizationSlug,
        roles: session.roles,
      },
      expiresAt: session.expiresAt,
    },
    { status: 200 },
  );
  response.headers.append(
    "Set-Cookie",
    buildExpiredLegacyAdminSessionCookie(),
  );
  return response;
}

export async function handleAdminAuthLogout(
  request: Request,
  env: AdminAuthApiEnv,
): Promise<Response> {
  const token = readAdminSessionToken(request);
  if (token !== null) {
    const session = await getAuthenticatedAdminSession(env.HYPERDRIVE, token);
    await revokeAdminSession(env.HYPERDRIVE, token);

    if (session !== null) {
      await queryDatabase(
        env.HYPERDRIVE,
        `
          INSERT INTO audit_events (
            id,
            organization_id,
            actor_id,
            event_type,
            entity_type,
            entity_id,
            details
          ) VALUES ($1, $2, $3, 'ADMIN_LOGOUT', 'admin_session', $4, '{}'::jsonb)
        `,
        [crypto.randomUUID(), session.organizationId, session.userId, session.sessionId],
      );
    }
  }

  const response = new Response(null, { status: 204 });
  response.headers.append("Set-Cookie", buildExpiredAdminSessionCookie());
  response.headers.append(
    "Set-Cookie",
    buildExpiredLegacyAdminSessionCookie(),
  );
  return response;
}
