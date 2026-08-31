export interface CorsEnv {
  PORTAL_ORIGINS?: string;
  ADMIN_PORTAL_ORIGIN?: string;
}

const ALLOWED_METHODS = "GET, POST, OPTIONS";
const ALLOWED_HEADERS = "Content-Type";
const PREFLIGHT_MAX_AGE_SECONDS = 600;
const ADMIN_API_PREFIX = "/api/v1/admin/";

function configuredCustomerOrigins(env: CorsEnv): Set<string> {
  return new Set(
    (env.PORTAL_ORIGINS ?? "")
      .split(",")
      .map((origin) => origin.trim())
      .filter((origin) => origin.length > 0),
  );
}

function configuredAdminOrigins(env: CorsEnv): Set<string> {
  const origin = env.ADMIN_PORTAL_ORIGIN?.trim();

  return new Set(origin === undefined || origin.length === 0 ? [] : [origin]);
}

function isAdminApiRequest(request: Request): boolean {
  const pathname = new URL(request.url).pathname;
  return (
    pathname === "/api/v1/admin" ||
    pathname.startsWith(ADMIN_API_PREFIX)
  );
}

function configuredOriginsForRequest(
  request: Request,
  env: CorsEnv,
): Set<string> {
  return isAdminApiRequest(request)
    ? configuredAdminOrigins(env)
    : configuredCustomerOrigins(env);
}

function allowedOrigin(request: Request, env: CorsEnv): string | null {
  const origin = request.headers.get("Origin");
  if (origin === null) {
    return null;
  }

  return configuredOriginsForRequest(request, env).has(origin) ? origin : null;
}

function appendVary(headers: Headers, value: string): void {
  const existing = headers.get("Vary");
  if (existing === null || existing.length === 0) {
    headers.set("Vary", value);
    return;
  }

  const values = new Set(
    existing
      .split(",")
      .map((item) => item.trim())
      .filter((item) => item.length > 0),
  );
  values.add(value);
  headers.set("Vary", Array.from(values).join(", "));
}

export function withCorsHeaders(
  request: Request,
  env: CorsEnv,
  response: Response,
): Response {
  const origin = allowedOrigin(request, env);
  if (origin === null) {
    return response;
  }

  const headers = new Headers(response.headers);
  headers.set("Access-Control-Allow-Origin", origin);
  headers.set("Access-Control-Allow-Credentials", "true");
  headers.set("Access-Control-Expose-Headers", "Content-Disposition, Content-Length, ETag");
  appendVary(headers, "Origin");

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

export function handleCorsPreflight(
  request: Request,
  env: CorsEnv,
): Response | null {
  if (request.method !== "OPTIONS") {
    return null;
  }

  const origin = request.headers.get("Origin");
  if (
    origin === null ||
    !configuredOriginsForRequest(request, env).has(origin)
  ) {
    return new Response(null, { status: 403 });
  }

  const headers = new Headers();
  headers.set("Access-Control-Allow-Origin", origin);
  headers.set("Access-Control-Allow-Credentials", "true");
  headers.set("Access-Control-Allow-Methods", ALLOWED_METHODS);
  headers.set("Access-Control-Allow-Headers", ALLOWED_HEADERS);
  headers.set("Access-Control-Max-Age", String(PREFLIGHT_MAX_AGE_SECONDS));
  headers.set("Vary", "Origin");

  return new Response(null, { status: 204, headers });
}

export function isTrustedBrowserMutation(
  request: Request,
  env: CorsEnv,
): boolean {
  if (!["POST", "PUT", "PATCH", "DELETE"].includes(request.method)) {
    return true;
  }

  const origin = request.headers.get("Origin");
  if (origin !== null) {
    return configuredOriginsForRequest(request, env).has(origin);
  }

  const fetchSite = request.headers.get("Sec-Fetch-Site")?.toLowerCase();
  if (fetchSite === "cross-site") {
    return false;
  }

  // Trusted non-browser clients such as the Windows agent, CI smoke tests,
  // and curl do not necessarily include browser Origin headers.
  return true;
}
