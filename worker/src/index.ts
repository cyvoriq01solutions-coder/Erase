import {
  handleCorsPreflight,
  isTrustedBrowserMutation,
  withCorsHeaders,
  type CorsEnv,
} from "./middleware/cors";
import { withSecurityHeaders } from "./middleware/securityHeaders";
import {
  handleAdminSession,
  handleApproveAccountsAdmin,
  handleApproveAdminUser,
  handleInviteAdminUser,
  handleListAdminUsers,
  handleRevokeAccountsAdmin,
  handleRevokeAdminUser,
  type AdminApiEnv,
} from "./routes/admin";
import {
  handleAdminAuthLogout,
  handleAdminAuthSession,
  handleAdminRequestCode,
  handleAdminVerifyCode,
  type AdminAuthApiEnv,
} from "./routes/adminAuth";
import {
  handleLogout,
  handleRegister,
  handleRequestCode,
  handleSession,
  handleVerifyCode,
  type AuthApiEnv,
} from "./routes/auth";
import {
  handleApproveCustomer,
  handleListCustomers,
  handleRejectCustomer,
  type CustomerAccessApiEnv,
} from "./routes/customerAccess";
import {
  handleDownloadStatus,
  type DownloadStatusEnv,
} from "./routes/downloadStatus";
import { handleHealth, type RuntimeEnv } from "./routes/health";
import { json } from "./services/http";

export interface Env
  extends RuntimeEnv,
    AuthApiEnv,
    AdminAuthApiEnv,
    AdminApiEnv,
    CustomerAccessApiEnv,
    DownloadStatusEnv,
    CorsEnv {}

async function route(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);

  if (request.method === "GET" && url.pathname === "/api/v1/health") {
    return handleHealth(env);
  }

  // Customer authentication realm.
  if (request.method === "POST" && url.pathname === "/api/v1/auth/register") {
    return handleRegister(request, env);
  }

  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/auth/request-code"
  ) {
    return handleRequestCode(request, env);
  }

  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/auth/verify-code"
  ) {
    return handleVerifyCode(request, env);
  }

  if (request.method === "GET" && url.pathname === "/api/v1/auth/session") {
    return handleSession(request, env);
  }

  if (
    request.method === "GET" &&
    url.pathname === "/api/v1/auth/download-status"
  ) {
    return handleDownloadStatus(request, env);
  }

  if (request.method === "POST" && url.pathname === "/api/v1/auth/logout") {
    return handleLogout(request, env);
  }

  // Dedicated Admin authentication realm.
  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/admin/auth/request-code"
  ) {
    return handleAdminRequestCode(request, env);
  }

  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/admin/auth/verify-code"
  ) {
    return handleAdminVerifyCode(request, env);
  }

  if (
    request.method === "GET" &&
    url.pathname === "/api/v1/admin/auth/session"
  ) {
    return handleAdminAuthSession(request, env);
  }

  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/admin/auth/logout"
  ) {
    return handleAdminAuthLogout(request, env);
  }

  // Protected Admin control-plane APIs. The compatibility session route also
  // reads only the dedicated Admin cookie during the C4.1 migration.
  if (request.method === "GET" && url.pathname === "/api/v1/admin/session") {
    return handleAdminSession(request, env);
  }

  if (request.method === "GET" && url.pathname === "/api/v1/admin/customers") {
    return handleListCustomers(request, env);
  }

  const customerAccessMatch = url.pathname.match(
    /^\/api\/v1\/admin\/customers\/([0-9a-f-]{36})\/(approve|reject)$/i,
  );
  if (request.method === "POST" && customerAccessMatch !== null) {
    const [, userId, action] = customerAccessMatch;
    return action === "approve"
      ? handleApproveCustomer(request, env, userId)
      : handleRejectCustomer(request, env, userId);
  }

  if (request.method === "GET" && url.pathname === "/api/v1/admin/users") {
    return handleListAdminUsers(request, env);
  }

  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/admin/users/invite"
  ) {
    return handleInviteAdminUser(request, env);
  }

  const adminRoleMatch = url.pathname.match(
    /^\/api\/v1\/admin\/users\/([0-9a-f-]{36})\/(approve|revoke)$/i,
  );
  if (request.method === "POST" && adminRoleMatch !== null) {
    const [, userId, action] = adminRoleMatch;
    return action === "approve"
      ? handleApproveAdminUser(request, env, userId)
      : handleRevokeAdminUser(request, env, userId);
  }

  // Compatibility endpoints retained only while the existing Admin frontend is
  // migrated to the generic Internal Users APIs.
  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/admin/roles/accounts/approve"
  ) {
    return handleApproveAccountsAdmin(request, env);
  }

  if (
    request.method === "POST" &&
    url.pathname === "/api/v1/admin/roles/accounts/revoke"
  ) {
    return handleRevokeAccountsAdmin(request, env);
  }

  if (url.pathname.startsWith("/api/")) {
    return json(
      {
        error: "not_found",
        message: "API route not found",
      },
      { status: 404 },
    );
  }

  return json(
    {
      service: "cyvoriq-erase-api",
      message: "CYVORIQ control-plane API",
    },
    { status: 200 },
  );
}

function finalizeResponse(
  request: Request,
  env: Env,
  response: Response,
): Response {
  return withSecurityHeaders(withCorsHeaders(request, env, response));
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const preflight = handleCorsPreflight(request, env);
    if (preflight !== null) {
      return withSecurityHeaders(preflight);
    }

    if (!isTrustedBrowserMutation(request, env)) {
      return finalizeResponse(
        request,
        env,
        json(
          {
            error: "forbidden_origin",
            message: "Request origin is not allowed",
          },
          { status: 403 },
        ),
      );
    }

    try {
      return finalizeResponse(request, env, await route(request, env));
    } catch {
      return finalizeResponse(
        request,
        env,
        json(
          {
            error: "internal_error",
            message: "Unexpected server error",
          },
          { status: 500 },
        ),
      );
    }
  },
} satisfies ExportedHandler<Env>;
