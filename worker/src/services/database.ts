import { Client } from "pg";

export interface HyperdriveBinding {
  connectionString: string;
}

export async function queryDatabase(
  hyperdrive: HyperdriveBinding,
  text: string,
): Promise<Record<string, unknown>[]> {
  const client = new Client({
    connectionString: hyperdrive.connectionString,
  });

  try {
    await client.connect();
    const result = await client.query(text);
    return result.rows;
  } finally {
    await client.end();
  }
}
