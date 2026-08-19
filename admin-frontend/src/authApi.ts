export const CEO_EMAIL = "ceo@cyvra.co.in";
export const ACCOUNTS_EMAIL = "accounts@cyvra.co.in";

export type AdminRole = "super_admin" | "accounts_admin";

export interface SessionUser {
  id: string;
  email: string;
  displayName: string | null;
  organizationId: string;
  organizationSlug: string;
  roles: string[];
}

export type SessionResponse =
  | { authenticated: false }
  | { authenticated: true; user: SessionUser; expiresAt: string };

export interface ChallengeResponse {
  status: "accepted";
  challengeId: string;
  message: string;
}

const configuredBase = import.meta.env.VITE_API_BASE_URL?.trim();
export const API_BASE_URL = (configuredBase || "https://api.cyvra.co.in").replace(/\/+$/, "");

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly code?: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function requestJson<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body !== undefined && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  let response: Response;
  try {
    response = await fetch(`${API_BASE_URL}${path}`, {
      ...init,
      headers,
      credentials: "include",
    });
  } catch {
    throw new ApiError("The CYVRA control plane is not reachable yet.", 0, "network_unavailable");
  }

  if (response.status === 204) {
    return undefined as T;
  }

  let payload: unknown = null;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok) {
    const body = payload as { message?: unknown; error?: unknown } | null;
    throw new ApiError(
      typeof body?.message === "string" ? body.message : "The request could not be completed.",
      response.status,
      typeof body?.error === "string" ? body.error : undefined,
    );
  }

  return payload as T;
}

export function isAllowedAdminEmail(email: string): boolean {
  const normalized = email.trim().toLowerCase();
  return normalized === CEO_EMAIL || normalized === ACCOUNTS_EMAIL;
}

export function beginAdminLogin(email: string): Promise<ChallengeResponse> {
  const normalized = email.trim().toLowerCase();
  if (!isAllowedAdminEmail(normalized)) {
    throw new ApiError("This identity is not permitted to use the CYVRA Admin Portal.", 403, "admin_identity_denied");
  }

  return requestJson<ChallengeResponse>("/api/v1/auth/register", {
    method: "POST",
    body: JSON.stringify({
      email: normalized,
      accountType: "individual",
    }),
  });
}

export function verifyAdminCode(challengeId: string, code: string): Promise<{ authenticated: true; expiresAt: string }> {
  return requestJson("/api/v1/auth/verify-code", {
    method: "POST",
    body: JSON.stringify({ challengeId, code }),
  });
}

export function getSession(): Promise<SessionResponse> {
  return requestJson<SessionResponse>("/api/v1/auth/session");
}

export function logout(): Promise<void> {
  return requestJson<void>("/api/v1/auth/logout", { method: "POST" });
}

export function activeAdminRole(user: SessionUser): AdminRole | null {
  if (user.roles.includes("super_admin")) return "super_admin";
  if (user.roles.includes("accounts_admin")) return "accounts_admin";
  return null;
}
