import { queryDatabase, type HyperdriveBinding } from "../services/database";
import { json } from "../services/http";

export interface DatabaseTablesEnv {
  HYPERDRIVE: HyperdriveBinding;
}

const EXPECTED_TABLES = [
  "assessments",
  "assets",
  "audit_events",
  "devices",
  "evidence",
  "organizations",
  "users",
  "verification_results",
];

export async function handleDatabaseTables(
  env: DatabaseTablesEnv,
): Promise<Response> {
  const rows = await queryDatabase(
    env.HYPERDRIVE,
    `
      SELECT table_name
      FROM information_schema.tables
      WHERE table_schema = 'public'
        AND table_type = 'BASE TABLE'
      ORDER BY table_name
    `,
  );

  const tables = rows.map((row) => String(row.table_name));
  const missing = EXPECTED_TABLES.filter((table) => !tables.includes(table));

  return json({
    status: missing.length === 0 ? "ok" : "incomplete",
    expectedCount: EXPECTED_TABLES.length,
    actualCount: tables.length,
    tables,
    missing,
  });
}
