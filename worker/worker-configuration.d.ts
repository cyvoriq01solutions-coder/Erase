interface Hyperdrive {
  connectionString: string;
}

interface Env {
  APP_ENV: string;
  API_VERSION: string;
  HYPERDRIVE: Hyperdrive;
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
