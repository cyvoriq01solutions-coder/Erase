import { withSecurityHeaders } from "./middleware/securityHeaders";
import { handleHealth, type RuntimeEnv } from "./routes/health";
import { json } from "./services/http";

export interface Env extends RuntimeEnv {}

function route(request: Request, env: Env): Response {
  const url = new URL(request.url);

  if (request.method === "GET" && url.pathname === "/api/v1/health") {
    return handleHealth(env);
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

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "OPTIONS") {
      return withSecurityHeaders(new Response(null, { status: 204 }));
    }

    try {
      return withSecurityHeaders(route(request, env));
    } catch {
      return withSecurityHeaders(
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
