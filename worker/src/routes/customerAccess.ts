import { AuthDeliveryError, type AuthDeliveryEnv } from "../services/authDelivery";
import { deliverAccessRejectionEmail } from "../services/accessDecisionEmail";
import {
  canEnableDownloadEntitlement,
} from "../services/authorizationPolicy";
import {
  approveCustomerAccess,
  listVerifiedCustomers,
  rejectCustomerAccess,
} from "../services/customerAccess";
import { json } from "../services/http";
import {
  requireAdminSession,
  type AdminApiEnv,
} from "./admin";

export interface CustomerAccessApiEnv extends AdminApiEnv, AuthDeliveryEnv {}

const MAX_JSON_BODY_BYTES = 8 * 1024;

async function requireAccessApprover(
  request: Request,
  env: CustomerAccessApiEnv,
) {
  const authority = await requireAdminSession(request, env);
  if (authority instanceof Response) {
    return authority;
  }
  if (!canEnableDownloadEntitlement(authority.roles)) {
    return json(
      {
        error: "forbidden",
        message: "An accounts or Super Administrator role is required to decide customer access.",
      },
      { status: 403 },
    );
  }
  return authority;
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

function serializeCustomer(row: Awaited<ReturnType<typeof listVerifiedCustomers>>[number]) {
  return {
    id: row.userId,
    email: row.email,
    displayName: row.displayName,
    accountStatus: row.accountStatus,
    emailVerifiedAt: row.emailVerifiedAt,
    accessStatus: row.accessStatus,
    rejectReason: row.rejectReason,
  };
}

export async function handleListCustomers(
  request: Request,
  env: CustomerAccessApiEnv,
): Promise<Response> {
  const authority = await requireAccessApprover(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  const customers = await listVerifiedCustomers(env.HYPERDRIVE);
  return json({ customers: customers.map(serializeCustomer) });
}

export async function handleApproveCustomer(
  request: Request,
  env: CustomerAccessApiEnv,
  userId: string,
): Promise<Response> {
  const authority = await requireAccessApprover(request, env);
  if (authority instanceof Response) {
    return authority;
  }
  if (!/^[0-9a-f-]{36}$/i.test(userId)) {
    return json({ error: "invalid_user", message: "Invalid customer identity." }, { status: 400 });
  }

  const updated = await approveCustomerAccess(
    env.HYPERDRIVE,
    authority.session.userId,
    authority.session.organizationId,
    userId,
  );
  if (updated === null) {
    return json(
      { error: "customer_missing", message: "That verified customer was not found." },
      { status: 404 },
    );
  }
  return json({ customer: serializeCustomer(updated) });
}

export async function handleRejectCustomer(
  request: Request,
  env: CustomerAccessApiEnv,
  userId: string,
): Promise<Response> {
  const authority = await requireAccessApprover(request, env);
  if (authority instanceof Response) {
    return authority;
  }
  if (!/^[0-9a-f-]{36}$/i.test(userId)) {
    return json({ error: "invalid_user", message: "Invalid customer identity." }, { status: 400 });
  }

  let body: Record<string, unknown>;
  try {
    body = await readJsonObject(request);
  } catch {
    return json({ error: "invalid_request", message: "Invalid request body." }, { status: 400 });
  }
  const reason = body.reason;
  if (typeof reason !== "string") {
    return json(
      { error: "invalid_reason", message: "Enter a rejection reason (8 to 500 characters)." },
      { status: 400 },
    );
  }

  let updated;
  try {
    updated = await rejectCustomerAccess(
      env.HYPERDRIVE,
      authority.session.userId,
      authority.session.organizationId,
      userId,
      reason,
    );
  } catch (error) {
    return json(
      {
        error: "invalid_reason",
        message: error instanceof Error ? error.message : "Enter a rejection reason.",
      },
      { status: 400 },
    );
  }

  if (updated === null) {
    return json(
      { error: "customer_missing", message: "That verified customer was not found." },
      { status: 404 },
    );
  }

  try {
    await deliverAccessRejectionEmail(env, {
      email: updated.email,
      reason: updated.rejectReason ?? reason,
    });
  } catch (error) {
    if (!(error instanceof AuthDeliveryError)) {
      throw error;
    }
    return json(
      {
        error: "access_email_unavailable",
        message: "Access was recorded as rejected, but the customer email could not be sent. Retry shortly.",
        customer: serializeCustomer(updated),
      },
      { status: 503 },
    );
  }

  return json({ customer: serializeCustomer(updated) });
}
