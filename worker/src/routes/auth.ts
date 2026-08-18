import {
  issueLoginChallenge,
  verifyLoginChallenge,
} from "../services/authChallenge";
import {
  deliverLoginCode,
  type AuthDeliveryEnv,
} from "../services/authDelivery";
import {
  findCustomerIdentityByEmail,
  normalizeEmail,
} from "../services/authIdentity";
import {
  registerCustomerIdentity,
  type RegisterCustomerInput,
} from "../services/authRegistration";
import {
  buildExpiredSessionCookie,
  buildSessionCookie,
  getAuthenticatedSession,
  readSessionToken,
  revokeSession,
} from "../services/authSession";
import {
  queryDatabase,
  type HyperdriveBinding,
} from "../services/database";
import { json } from "../services/http";

export interface AuthApiEnv extends AuthDeliveryEnv {
  HYPERDRIVE: HyperdriveBinding;
  AUTH_PEPPER: string;
}

const MAX_JSON_BODY_BYTES = 8 * 1024;

function acceptedChallenge(challengeId: string): Response {
  return json(
    {
      status: "accepted",
      challengeId,
      message: "If the email is eligible, a verification code will be sent.",
    },
    { status: 202 },
  );
}

function isDeliveryConfigured(env: AuthApiEnv): boolean {
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

function optionalString(
  object: Record<string, unknown>,
  key: string,
): string | null | undefined {
  const value = object[key];
  if (value === undefined) {
    return undefined;
  }
  if (value === null) {
    return null;
  }
  if (typeof value !== "string") {
    throw new Error("invalid_input");
  }
  return value;
}

async function consumeChallenge(
  env: AuthApiEnv,
  challengeId: string,
): Promise<void> {
  await queryDatabase(
    env.HYPERDRIVE,
    `
      UPDATE login_challenges
      SET consumed_at = COALESCE(consumed_at, NOW())
      WHERE id = $1
    `,
    [challengeId],
  );
}

async function issueAndDeliver(
  env: AuthApiEnv,
  identity: Awaited<ReturnType<typeof findCustomerIdentityByEmail>>,
): Promise<string | null> {
  if (identity === null) {
    return null;
  }

  const challenge = await issueLoginChallenge(
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
  } catch (error) {
    await consumeChallenge(env, challenge.challengeId);
    throw error;
  }

  return challenge.challengeId;
}

export async function handleRegister(
  request: Request,
  env: AuthApiEnv,
): Promise<Response> {
  if (!isDeliveryConfigured(env)) {
    return json(
      { error: "auth_email_unavailable", message: "Email verification is temporarily unavailable." },
      { status: 503 },
    );
  }

  let body: Record<string, unknown>;
  try {
    body = await readJsonObject(request);
  } catch {
    return json({ error: "invalid_request", message: "Invalid request body." }, { status: 400 });
  }

  const emailValue = body.email;
  if (typeof emailValue !== "string" || normalizeEmail(emailValue) === null) {
    return json({ error: "invalid_email", message: "Enter a valid email address." }, { status: 400 });
  }

  let input: RegisterCustomerInput;
  try {
    const accountTypeValue = body.accountType;
    if (
      accountTypeValue !== undefined &&
      accountTypeValue !== "individual" &&
      accountTypeValue !== "enterprise"
    ) {
      throw new Error("invalid_input");
    }

    input = {
      email: emailValue,
      displayName: optionalString(body, "displayName"),
      organizationName: optionalString(body, "organizationName"),
      accountType: accountTypeValue as RegisterCustomerInput["accountType"],
    };
  } catch {
    return json({ error: "invalid_request", message: "Invalid registration details." }, { status: 400 });
  }

  let registration;
  try {
    registration = await registerCustomerIdentity(env.HYPERDRIVE, input);
  } catch (error) {
    if (error instanceof Error && error.message.includes("required")) {
      return json({ error: "invalid_request", message: error.message }, { status: 400 });
    }
    throw error;
  }

  try {
    const challengeId = await issueAndDeliver(env, registration.identity);
    return acceptedChallenge(challengeId ?? crypto.randomUUID());
  } catch {
    return acceptedChallenge(crypto.randomUUID());
  }
}

export async function handleRequestCode(
  request: Request,
  env: AuthApiEnv,
): Promise<Response> {
  if (!isDeliveryConfigured(env)) {
    return json(
      { error: "auth_email_unavailable", message: "Email verification is temporarily unavailable." },
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
    return acceptedChallenge(crypto.randomUUID());
  }

  const email = normalizeEmail(rawEmail);
  if (email === null) {
    return acceptedChallenge(crypto.randomUUID());
  }

  const identity = await findCustomerIdentityByEmail(env.HYPERDRIVE, email);

  try {
    const challengeId = await issueAndDeliver(env, identity);
    return acceptedChallenge(challengeId ?? crypto.randomUUID());
  } catch {
    return acceptedChallenge(crypto.randomUUID());
  }
}

export async function handleVerifyCode(
  request: Request,
  env: AuthApiEnv,
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
    return json({ error: "invalid_code", message: "The verification code is invalid or expired." }, { status: 401 });
  }

  const result = await verifyLoginChallenge(
    env.HYPERDRIVE,
    env.AUTH_PEPPER,
    challengeId,
    code,
  );

  if (result.status !== "authenticated") {
    return json({ error: "invalid_code", message: "The verification code is invalid or expired." }, { status: 401 });
  }

  const response = json({ authenticated: true, expiresAt: result.expiresAt }, { status: 200 });
  response.headers.append(
    "Set-Cookie",
    buildSessionCookie(result.sessionToken, result.expiresAt),
  );
  return response;
}

export async function handleSession(
  request: Request,
  env: AuthApiEnv,
): Promise<Response> {
  const token = readSessionToken(request);
  if (token === null) {
    return json({ authenticated: false }, { status: 200 });
  }

  const session = await getAuthenticatedSession(env.HYPERDRIVE, token);
  if (session === null) {
    const response = json({ authenticated: false }, { status: 200 });
    response.headers.append("Set-Cookie", buildExpiredSessionCookie());
    return response;
  }

  return json(
    {
      authenticated: true,
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
}

export async function handleLogout(
  request: Request,
  env: AuthApiEnv,
): Promise<Response> {
  const token = readSessionToken(request);
  if (token !== null) {
    await revokeSession(env.HYPERDRIVE, token);
  }

  const response = new Response(null, { status: 204 });
  response.headers.append("Set-Cookie", buildExpiredSessionCookie());
  return response;
}
