import type { Client } from "pg";

import { hashActivationKey, hashDeviceFingerprint } from "./authCrypto";
import { withDatabaseTransaction, type HyperdriveBinding } from "./database";

const ERASE_KEY_PATTERN =
  /^CYVRA-[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{4}(?:-[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{4}){3}$/;
const PURGE_KEY_PATTERN =
  /^CYVRA-PRG-[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{4}(?:-[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{4}){3}$/;

export type LicenseProduct = "CYVORIQ_ERASE" | "CYVORIQ_PURGE";

export class DeviceBindError extends Error {
  constructor(
    readonly code:
      | "invalid_key"
      | "unknown_key"
      | "wrong_product"
      | "license_inactive"
      | "device_mismatch",
    message: string,
  ) {
    super(message);
    this.name = "DeviceBindError";
  }
}

export interface DeviceBindResult {
  status: "bound" | "already_bound";
  keyPrefix: string;
  hostname: string | null;
}

function normalizeHostname(raw: string | null): string | null {
  if (raw === null) {
    return null;
  }
  const hostname = raw.trim().replace(/\s+/g, " ");
  if (hostname.length === 0) {
    return null;
  }
  if (hostname.length > 128) {
    throw new DeviceBindError("invalid_key", "Hostname is too long.");
  }
  return hostname;
}

function normalizeMachineGuid(raw: string): string {
  const guid = raw.trim().toLowerCase();
  if (guid.length < 8 || guid.length > 128 || !/^[0-9a-f-]+$/i.test(guid)) {
    throw new DeviceBindError("invalid_key", "Device identity is not valid.");
  }
  return guid;
}

function normalizeActivationKey(raw: string, expectedProduct: LicenseProduct): string {
  const key = raw.trim().toUpperCase();
  const looksPurge = key.startsWith("CYVRA-PRG-");
  if (expectedProduct === "CYVORIQ_ERASE" && looksPurge) {
    throw new DeviceBindError(
      "wrong_product",
      "That key is a CYVRA Purge licence. Open Data purge and use Activate Purge.",
    );
  }
  if (expectedProduct === "CYVORIQ_PURGE" && !looksPurge) {
    throw new DeviceBindError(
      "wrong_product",
      "That key is a CYVRA Erase assessment licence. Use Activate on first run.",
    );
  }
  const pattern = expectedProduct === "CYVORIQ_PURGE" ? PURGE_KEY_PATTERN : ERASE_KEY_PATTERN;
  if (!pattern.test(key)) {
    throw new DeviceBindError("invalid_key", "Activation key format is not valid.");
  }
  return key;
}

function hostnameFromMetadata(raw: unknown): string | null {
  if (raw === null || raw === undefined || typeof raw !== "object" || Array.isArray(raw)) {
    return null;
  }
  const hostname = (raw as Record<string, unknown>).hostname;
  return typeof hostname === "string" && hostname.length > 0 ? hostname : null;
}

export async function bindLicenseToDevice(
  hyperdrive: HyperdriveBinding,
  pepper: string,
  input: {
    activationKey: string;
    machineGuid: string;
    hostname: string | null;
    expectedProduct: LicenseProduct;
  },
): Promise<DeviceBindResult> {
  const activationKey = normalizeActivationKey(input.activationKey, input.expectedProduct);
  const machineGuid = normalizeMachineGuid(input.machineGuid);
  const hostname = normalizeHostname(input.hostname);
  const keyHash = await hashActivationKey(pepper, activationKey);
  const fingerprintHash = await hashDeviceFingerprint(pepper, machineGuid);

  return withDatabaseTransaction(hyperdrive, async (client) => {
    const license = await client.query(
      `
        SELECT id, organization_id, issued_to_user_id, key_prefix, status, max_devices, product
        FROM licenses
        WHERE key_hash = $1
        FOR UPDATE
      `,
      [keyHash],
    );
    if (license.rowCount !== 1) {
      throw new DeviceBindError(
        "unknown_key",
        "That activation key was not recognised.",
      );
    }

    const row = license.rows[0] as Record<string, unknown>;
    const licenseId = String(row.id);
    const organizationId = String(row.organization_id);
    const issuedTo =
      row.issued_to_user_id === null || row.issued_to_user_id === undefined
        ? null
        : String(row.issued_to_user_id);
    const keyPrefix = String(row.key_prefix);
    const status = String(row.status);
    const maxDevices = Number(row.max_devices);
    const product = String(row.product);

    if (product !== input.expectedProduct) {
      throw new DeviceBindError(
        "wrong_product",
        input.expectedProduct === "CYVORIQ_PURGE"
          ? "That key is not a CYVRA Purge licence."
          : "That key is a CYVRA Purge licence. Open Data purge and use Activate Purge.",
      );
    }

    if (status !== "active") {
      throw new DeviceBindError(
        "license_inactive",
        "This licence is not active.",
      );
    }

    const existingSame = await client.query(
      `
        SELECT id, status, metadata
        FROM device_activations
        WHERE license_id = $1
          AND fingerprint_hash = $2
        FOR UPDATE
      `,
      [licenseId, fingerprintHash],
    );

    if ((existingSame.rowCount ?? 0) > 0) {
      const current = existingSame.rows[0] as Record<string, unknown>;
      if (String(current.status) !== "active") {
        throw new DeviceBindError(
          "license_inactive",
          "This device activation is no longer active.",
        );
      }
      await client.query(
        `
          UPDATE device_activations
          SET last_seen_at = NOW()
          WHERE id = $1
        `,
        [String(current.id)],
      );
      await writeAudit(client, {
        organizationId,
        actorId: issuedTo,
        eventType: "LICENSE_REVALIDATED",
        entityId: licenseId,
        details: { keyPrefix, hostname },
      });
      const storedHostname = hostnameFromMetadata(current.metadata);
      return {
        status: "already_bound",
        keyPrefix,
        hostname: storedHostname ?? hostname,
      };
    }

    const others = await client.query(
      `
        SELECT id
        FROM device_activations
        WHERE license_id = $1
          AND status = 'active'
        FOR UPDATE
      `,
      [licenseId],
    );
    if ((others.rowCount ?? 0) >= maxDevices) {
      await writeAudit(client, {
        organizationId,
        actorId: issuedTo,
        eventType: "LICENSE_ACTIVATION_REJECTED",
        entityId: licenseId,
        details: { keyPrefix, reason: "device_mismatch" },
      });
      throw new DeviceBindError(
        "device_mismatch",
        "This licence is already bound to a different Windows device.",
      );
    }

    const deviceId = crypto.randomUUID();
    const activationId = crypto.randomUUID();

    await client.query(
      `
        INSERT INTO devices (
          id,
          organization_id,
          platform,
          hostname
        ) VALUES ($1, $2, 'windows', $3)
      `,
      [deviceId, organizationId, hostname],
    );

    await client.query(
      `
        INSERT INTO device_activations (
          id,
          organization_id,
          license_id,
          device_id,
          activated_by_user_id,
          fingerprint_hash,
          status,
          last_seen_at,
          metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', NOW(), $7::jsonb)
      `,
      [
        activationId,
        organizationId,
        licenseId,
        deviceId,
        issuedTo,
        fingerprintHash,
        JSON.stringify({ hostname }),
      ],
    );

    await writeAudit(client, {
      organizationId,
      actorId: issuedTo,
      eventType: "LICENSE_ACTIVATED",
      entityId: licenseId,
      details: { keyPrefix, hostname, activationId },
    });

    return {
      status: "bound",
      keyPrefix,
      hostname,
    };
  });
}

async function writeAudit(
  client: Client,
  input: {
    organizationId: string;
    actorId: string | null;
    eventType: string;
    entityId: string;
    details: Record<string, unknown>;
  },
): Promise<void> {
  await client.query(
    `
      INSERT INTO audit_events (
        id,
        organization_id,
        actor_id,
        event_type,
        entity_type,
        entity_id,
        details
      ) VALUES ($1, $2, $3, $4, 'license', $5, $6::jsonb)
    `,
    [
      crypto.randomUUID(),
      input.organizationId,
      input.actorId,
      input.eventType,
      input.entityId,
      JSON.stringify(input.details),
    ],
  );
}
