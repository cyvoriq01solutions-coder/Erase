import { getAuthenticatedSession, readSessionToken } from "../services/authSession";
import { getCustomerDownloadStatus } from "../services/customerAccess";
import type { HyperdriveBinding } from "../services/database";
import { json } from "../services/http";

export interface DownloadStatusEnv {
  HYPERDRIVE: HyperdriveBinding;
}

export async function handleDownloadStatus(
  request: Request,
  env: DownloadStatusEnv,
): Promise<Response> {
  const token = readSessionToken(request);
  if (token === null) {
    return json({ authenticated: false, entitled: false }, { status: 200 });
  }

  const session = await getAuthenticatedSession(env.HYPERDRIVE, token);
  if (session === null) {
    return json({ authenticated: false, entitled: false }, { status: 200 });
  }

  const status = await getCustomerDownloadStatus(env.HYPERDRIVE, session.userId);
  const accessStatus = status?.accessStatus ?? "waiting";
  const entitled = accessStatus === "approved";

  return json({
    authenticated: true,
    entitled,
    accessStatus,
    rejectReason: accessStatus === "rejected" ? status?.rejectReason ?? null : null,
    packageAvailable: false,
    message: entitled
      ? "Access is approved. The Windows package is not released on this channel yet."
      : accessStatus === "rejected"
        ? "Access was not approved."
        : "Waiting for CYVRA administration to approve download access.",
  });
}
