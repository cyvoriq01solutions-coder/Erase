export type AdminRole = "super_admin" | "accounts_admin";

export interface SessionUser {
  id: string;
  email: string;
  displayName: string | null;
  organizationId: string;
  organizationSlug: string;
  roles: string[];
}

export type AdminSessionResponse =
  | { authorized: false }
  | { authorized: true; user: SessionUser; expiresAt: string };

export interface ChallengeResponse {
  status: "accepted";
  challengeId: string;
  expiresAt: string;
  message: string;
}

export type AdminVerifyResponse =
  | {
      authenticated: true;
      status: "authenticated";
      role: AdminRole;
      expiresAt: string;
    }
  | {
      authenticated: false;
      status: "pending_approval";
      role: AdminRole;
      message: string;
    };

export interface AdminUserSummary {
  id: string;
  email: string;
  displayName: string | null;
  accountStatus: string;
  emailVerifiedAt: string | null;
  role: AdminRole;
  roleStatus: "pending" | "active" | "revoked";
}

export interface AdminUsersResponse {
  users: AdminUserSummary[];
}

export interface AdminRoleActionResponse {
  status: "active" | "revoked";
  role: "accounts_admin";
  email: string;
  userId: string;
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

export function beginAdminLogin(email: string): Promise<ChallengeResponse> {
  return requestJson<ChallengeResponse>("/api/v1/admin/auth/request-code", {
    method: "POST",
    body: JSON.stringify({ email: email.trim().toLowerCase() }),
  });
}

export function verifyAdminCode(
  challengeId: string,
  code: string,
): Promise<AdminVerifyResponse> {
  return requestJson<AdminVerifyResponse>("/api/v1/admin/auth/verify-code", {
    method: "POST",
    body: JSON.stringify({ challengeId, code }),
  });
}

export function getAdminSession(): Promise<AdminSessionResponse> {
  return requestJson<AdminSessionResponse>("/api/v1/admin/auth/session");
}

export function logout(): Promise<void> {
  return requestJson<void>("/api/v1/admin/auth/logout", { method: "POST" });
}

export function listAdminUsers(): Promise<AdminUsersResponse> {
  return requestJson<AdminUsersResponse>("/api/v1/admin/users");
}

export function inviteAdminUser(input: {
  email: string;
  displayName?: string | null;
  role?: "accounts_admin";
}): Promise<{ user: AdminUserSummary }> {
  return requestJson<{ user: AdminUserSummary }>("/api/v1/admin/users/invite", {
    method: "POST",
    body: JSON.stringify({
      email: input.email.trim().toLowerCase(),
      displayName: input.displayName ?? null,
      role: input.role ?? "accounts_admin",
    }),
  });
}

export function approveAdminUser(userId: string): Promise<AdminRoleActionResponse> {
  return requestJson<AdminRoleActionResponse>(
    `/api/v1/admin/users/${encodeURIComponent(userId)}/approve`,
    { method: "POST" },
  );
}

export function revokeAdminUser(userId: string): Promise<AdminRoleActionResponse> {
  return requestJson<AdminRoleActionResponse>(
    `/api/v1/admin/users/${encodeURIComponent(userId)}/revoke`,
    { method: "POST" },
  );
}

export type CustomerAccessStatus = "waiting" | "approved" | "rejected";

export interface CustomerAccessSummary {
  id: string;
  email: string;
  displayName: string | null;
  accountStatus: string;
  emailVerifiedAt: string | null;
  accessStatus: CustomerAccessStatus;
  rejectReason: string | null;
}

export function listCustomers(): Promise<{ customers: CustomerAccessSummary[] }> {
  return requestJson<{ customers: CustomerAccessSummary[] }>("/api/v1/admin/customers");
}

export function approveCustomer(userId: string): Promise<{ customer: CustomerAccessSummary }> {
  return requestJson<{ customer: CustomerAccessSummary }>(
    `/api/v1/admin/customers/${encodeURIComponent(userId)}/approve`,
    { method: "POST" },
  );
}

export function rejectCustomer(
  userId: string,
  reason: string,
): Promise<{ customer: CustomerAccessSummary }> {
  return requestJson<{ customer: CustomerAccessSummary }>(
    `/api/v1/admin/customers/${encodeURIComponent(userId)}/reject`,
    { method: "POST", body: JSON.stringify({ reason }) },
  );
}

export function activeAdminRole(user: SessionUser): AdminRole | null {
  if (user.roles.includes("super_admin")) return "super_admin";
  if (user.roles.includes("accounts_admin")) return "accounts_admin";
  return null;
}
