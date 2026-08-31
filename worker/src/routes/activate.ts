import {
  bindLicenseToDevice,
  DeviceBindError,
} from "../services/deviceActivation";
import { json } from "../services/http";
import type { HyperdriveBinding } from "../services/database";

export interface ActivateApiEnv {
  HYPERDRIVE: HyperdriveBinding;
  AUTH_PEPPER: string;
}

const MAX_JSON_BODY_BYTES = 8 * 1024;

function optionalString(
  object: Record<string, unknown>,
  key: string,
): string | null {
  const value = object[key];
  if (value === undefined || value === null) {
    return null;
  }
  if (typeof value !== "string") {
    throw new Error("invalid_json");
  }
  return value;
}

function requiredString(
  object: Record<string, unknown>,
  key: string,
): string {
  const value = optionalString(object, key);
  if (value === null || value.trim().length === 0) {
    throw new Error("invalid_json");
  }
  return value;
}

async function readJsonObject(request: Request): Promise<Record<string, unknown>> {
  const contentLength = Number(request.headers.get("Content-Length") ?? "0");
  if (Number.isFinite(contentLength) && contentLength > MAX_JSON_BODY_BYTES) {
    throw new Error("request_too_large");
  }

  const body = await request.text();
  if (body.length > MAX_JSON_BODY_BYTES) {
    throw new Error("request_too_large");
  }

  const parsed: unknown = JSON.parse(body);
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("invalid_json");
  }

  return parsed as Record<string, unknown>;
}

export async function handleActivateLicense(
  request: Request,
  env: ActivateApiEnv,
): Promise<Response> {
  let payload: Record<string, unknown>;
  try {
    payload = await readJsonObject(request);
  } catch {
    return json(
      { error: "invalid_json", message: "Activation request was not valid JSON." },
      { status: 400 },
    );
  }

  let activationKey: string;
  let machineGuid: string;
  let hostname: string | null;
  try {
    activationKey = requiredString(payload, "activationKey");
    machineGuid = requiredString(payload, "machineGuid");
    hostname = optionalString(payload, "hostname");
  } catch {
    return json(
      {
        error: "invalid_json",
        message: "activationKey and machineGuid are required.",
      },
      { status: 400 },
    );
  }

  try {
    const result = await bindLicenseToDevice(env.HYPERDRIVE, env.AUTH_PEPPER, {
      activationKey,
      machineGuid,
      hostname,
    });
    return json({
      status: result.status,
      keyPrefix: result.keyPrefix,
      message:
        result.status === "already_bound"
          ? "This PC is already bound to the licence."
          : "This PC is now bound to the licence.",
    });
  } catch (error) {
    if (error instanceof DeviceBindError) {
      const status =
        error.code === "device_mismatch"
          ? 409
          : error.code === "unknown_key" || error.code === "invalid_key"
            ? 400
            : 403;
      return json({ error: error.code, message: error.message }, { status });
    }
    throw error;
  }
}
