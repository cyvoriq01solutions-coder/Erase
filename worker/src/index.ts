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
  handleRevokeAccountsAdmin,
  type AdminApiEnv,
} from "./routes/admin";
import {
  handleLogout,
  handleRegister,
  handleRequestCode,
  handleSession,
  handleVerifyCode,
  type AuthApiEnv,
} from "./routes/auth";
import {
  handleDatabaseHealth,
  type DatabaseHealthEnv,
} from "./routes/databaseHealth";
import {
  handleDatabaseTables,
  type DatabaseTablesEnv,
} from "./routes/databaseTables";
import { handleHealth, type RuntimeEnv } from "./routes/health";
import { json } from "./services/http";

export interface Env
  extends RuntimeEnv,
    DatabaseHealthEnv,
    DatabaseTablesEnv,
    AuthApiEnv,
    AdminApiEnv,
    CorsEnv {}

async function route(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);

  if (request.method === "GET" && url.pathname === "/api/v1/health") {
    return handleHealth(env);
  }

  if (request.method === "GET" && url.pathname === "/api/v1/db/health") {
    return handleDatabaseHealth(env);
  }

  if (request.method === "GET" && url.pathname === "/api/v1/db/tables") {
    return handleDatabaseTables(env);
  }

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

  if (request.method === "POST" && url.pathname === "/api/v1/auth/logout") {
    return handleLogout(request, env);
  }

  if (request.method === "GET" && url.pathname === "/api/v1/admin/session") {
    return handleAdminSession(request, env);
  }

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
