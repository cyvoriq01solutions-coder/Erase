import { queryDatabase, type HyperdriveBinding } from "../services/database";
import { json } from "../services/http";

export interface DatabaseHealthEnv {
  HYPERDRIVE: HyperdriveBinding;
  APP_ENV: string;
  API_VERSION: string;
}

export async function handleDatabaseHealth(
  env: DatabaseHealthEnv,
): Promise<Response> {
  const rows = await queryDatabase(
    env.HYPERDRIVE,
    "SELECT current_database() AS database, current_user AS user, NOW() AS timestamp",
  );

  return json({
    service: "cyvoriq-erase-api",
    status: "ok",
    database: rows[0] ?? null,
    environment: env.APP_ENV,
    apiVersion: env.API_VERSION,
  });
}
