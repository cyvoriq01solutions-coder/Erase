import { json } from "../services/http";

export interface RuntimeEnv {
  APP_ENV: string;
  API_VERSION: string;
}

export function handleHealth(env: RuntimeEnv): Response {
  return json({
    service: "cyvoriq-erase-api",
    status: "ok",
    environment: env.APP_ENV,
    apiVersion: env.API_VERSION,
  });
}
