import { Client } from "pg";

export interface HyperdriveBinding {
  connectionString: string;
}

export async function queryDatabase(
  hyperdrive: HyperdriveBinding,
  text: string,
  values: readonly unknown[] = [],
): Promise<Record<string, unknown>[]> {
  const client = new Client({
    connectionString: hyperdrive.connectionString,
  });

  try {
    await client.connect();
    const result = await client.query(text, [...values]);
    return result.rows;
  } finally {
    await client.end();
  }
}

export async function withDatabaseTransaction<T>(
  hyperdrive: HyperdriveBinding,
  operation: (client: Client) => Promise<T>,
): Promise<T> {
  const client = new Client({
    connectionString: hyperdrive.connectionString,
  });

  await client.connect();

  try {
    await client.query("BEGIN");
    const result = await operation(client);
    await client.query("COMMIT");
    return result;
  } catch (error) {
    try {
      await client.query("ROLLBACK");
    } catch {
      // Preserve the original transaction error.
    }
    throw error;
  } finally {
    await client.end();
  }
}
