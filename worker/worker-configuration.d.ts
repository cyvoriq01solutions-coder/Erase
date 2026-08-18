interface Hyperdrive {
  connectionString: string;
}

interface Env {
  APP_ENV: string;
  API_VERSION: string;
  HYPERDRIVE: Hyperdrive;
  AUTH_PEPPER: string;
  AUTH_EMAIL_ENDPOINT?: string;
  AUTH_EMAIL_TOKEN?: string;
}

interface ExportedHandler<Environment = Env> {
  fetch?(
    request: Request,
    env: Environment,
    ctx: ExecutionContext,
  ): Response | Promise<Response>;
}

interface ExecutionContext {
  waitUntil(promise: Promise<unknown>): void;
  passThroughOnException(): void;
}
