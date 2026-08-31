interface Hyperdrive {
  connectionString: string;
}

interface R2Object {
  readonly size: number;
  readonly httpEtag?: string;
}

interface R2ObjectBody extends R2Object {
  readonly body: ReadableStream;
}

interface R2Bucket {
  get(key: string): Promise<R2ObjectBody | null>;
  head(key: string): Promise<R2Object | null>;
}

interface Env {
  APP_ENV: string;
  API_VERSION: string;
  HYPERDRIVE: Hyperdrive;
  RELEASES: R2Bucket;
  AUTH_PEPPER: string;
  AUTH_EMAIL_ENDPOINT?: string;
  AUTH_EMAIL_TOKEN?: string;
  AUTH_EMAIL_FROM?: string;
  PORTAL_ORIGINS?: string;
  ADMIN_PORTAL_ORIGIN?: string;
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
