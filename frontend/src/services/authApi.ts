export type AccountType = "individual" | "enterprise";

export interface RegisterInput {
  email: string;
  displayName?: string;
  organizationName?: string;
  accountType: AccountType;
}

export interface ChallengeResponse {
  status: "accepted";
  challengeId: string;
  message: string;
}

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

function resolveApiBaseUrl(): string {
  const configuredBase = import.meta.env.VITE_API_BASE_URL?.trim();
  if (configuredBase) {
    return configuredBase.replace(/\/+$/, "");
  }

  if (
    typeof window !== "undefined" &&
    window.location.hostname === "portal-auth-ui-v1.erase-e93.pages.dev"
  ) {
    return "https://portal-auth-ui-v1-cyvoriq-erase-api.mswaroop707.workers.dev";
  }

  return "https://api.cyvra.co.in";
}

const API_BASE_URL = resolveApiBaseUrl();

class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly code?: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

async function requestJson<T>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const headers = new Headers(init.headers);
  if (init.body !== undefined && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    headers,
    credentials: "include",
  });

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
    const message =
      typeof body?.message === "string"
        ? body.message
        : "The CYVORIQ service could not complete this request.";
    const code = typeof body?.error === "string" ? body.error : undefined;
    throw new ApiError(message, response.status, code);
  }

  return payload as T;
}

export function registerCustomer(input: RegisterInput): Promise<ChallengeResponse> {
  return requestJson<ChallengeResponse>("/api/v1/auth/register", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function requestLoginCode(email: string): Promise<ChallengeResponse> {
  return requestJson<ChallengeResponse>("/api/v1/auth/request-code", {
    method: "POST",
    body: JSON.stringify({ email }),
  });
}

export function verifyLoginCode(
  challengeId: string,
  code: string,
): Promise<{ authenticated: true; expiresAt: string }> {
  return requestJson("/api/v1/auth/verify-code", {
    method: "POST",
    body: JSON.stringify({ challengeId, code }),
  });
}

export function getSession(): Promise<SessionResponse> {
  return requestJson<SessionResponse>("/api/v1/auth/session");
}

export type DownloadAccessStatus = "waiting" | "approved" | "rejected";

export interface DownloadStatusResponse {
  authenticated: boolean;
  entitled: boolean;
  accessStatus?: DownloadAccessStatus;
  rejectReason?: string | null;
  packageAvailable?: boolean;
  message?: string;
}

export function getDownloadStatus(): Promise<DownloadStatusResponse> {
  return requestJson<DownloadStatusResponse>("/api/v1/auth/download-status");
}

export function logout(): Promise<void> {
  return requestJson<void>("/api/v1/auth/logout", { method: "POST" });
}

export { ApiError };
