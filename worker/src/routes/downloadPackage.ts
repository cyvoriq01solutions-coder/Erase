import { getAuthenticatedSession, readSessionToken } from "../services/authSession";
import { getCustomerDownloadStatus } from "../services/customerAccess";
import {
  withDatabaseTransaction,
  type HyperdriveBinding,
} from "../services/database";
import { json } from "../services/http";
import {
  isSetupPackagePresent,
  resolveReleaseArtifact,
  type ReleaseStore,
} from "../services/packageRelease";

export interface DownloadPackageEnv {
  HYPERDRIVE: HyperdriveBinding;
  RELEASES: ReleaseStore;
}

async function requireEntitledCustomer(
  request: Request,
  env: DownloadPackageEnv,
): Promise<
  | {
      userId: string;
      organizationId: string;
    }
  | Response
> {
  const token = readSessionToken(request);
  if (token === null) {
    return json(
      { error: "unauthorized", message: "Customer authentication is required." },
      { status: 401 },
    );
  }

  const session = await getAuthenticatedSession(env.HYPERDRIVE, token);
  if (session === null) {
    return json(
      { error: "unauthorized", message: "Customer session is invalid or expired." },
      { status: 401 },
    );
  }

  const status = await getCustomerDownloadStatus(env.HYPERDRIVE, session.userId);
  if (status?.accessStatus !== "approved") {
    return json(
      {
        error: "forbidden",
        message: "Download is not authorised for this account.",
      },
      { status: 403 },
    );
  }

  return {
    userId: session.userId,
    organizationId: session.organizationId,
  };
}

export async function handleDownloadPackage(
  request: Request,
  env: DownloadPackageEnv,
  artifactName: string,
): Promise<Response> {
  const artifact = resolveReleaseArtifact(artifactName);
  if (artifact === null) {
    return json(
      { error: "not_found", message: "Unknown download artifact." },
      { status: 404 },
    );
  }

  const authority = await requireEntitledCustomer(request, env);
  if (authority instanceof Response) {
    return authority;
  }

  const object = await env.RELEASES.get(artifact.key);
  if (object === null || object.size <= 0) {
    return json(
      {
        error: "package_not_released",
        message: "The Windows package is not in the private release store yet.",
      },
      { status: 404 },
    );
  }

  await withDatabaseTransaction(env.HYPERDRIVE, async (client) => {
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
        ) VALUES ($1, $2, $3, 'DOWNLOAD_GRANTED', 'user', $4, $5::jsonb)
      `,
      [
        crypto.randomUUID(),
        authority.organizationId,
        authority.userId,
        authority.userId,
        JSON.stringify({
          artifact: artifactName,
          objectKey: artifact.key,
          filename: artifact.filename,
          bytes: object.size,
        }),
      ],
    );
  });

  const headers = new Headers();
  headers.set("Content-Type", artifact.contentType);
  headers.set(
    "Content-Disposition",
    `attachment; filename="${artifact.filename}"`,
  );
  headers.set("Content-Length", String(object.size));
  if (object.httpEtag) {
    headers.set("ETag", object.httpEtag);
  }

  return new Response(object.body, { status: 200, headers });
}

export async function handleDownloadStatus(
  request: Request,
  env: DownloadPackageEnv,
): Promise<Response> {
  const token = readSessionToken(request);
  if (token === null) {
    return json({ authenticated: false, entitled: false, packageAvailable: false }, { status: 200 });
  }

  const session = await getAuthenticatedSession(env.HYPERDRIVE, token);
  if (session === null) {
    return json({ authenticated: false, entitled: false, packageAvailable: false }, { status: 200 });
  }

  const status = await getCustomerDownloadStatus(env.HYPERDRIVE, session.userId);
  const accessStatus = status?.accessStatus ?? "waiting";
  const entitled = accessStatus === "approved";
  const packageAvailable = await isSetupPackagePresent(env.RELEASES);

  let message = "Waiting for CYVRA administration to approve download access.";
  if (accessStatus === "rejected") {
    message = "Access was not approved.";
  } else if (entitled && packageAvailable) {
    message = "Access is approved. Download is authorised from this page only.";
  } else if (entitled) {
    message =
      "Access is approved. The Windows package is not in the private release store yet.";
  }

  return json({
    authenticated: true,
    entitled,
    accessStatus,
    rejectReason: accessStatus === "rejected" ? status?.rejectReason ?? null : null,
    packageAvailable,
    message,
  });
}
