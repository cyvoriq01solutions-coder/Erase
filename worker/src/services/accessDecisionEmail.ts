import {
  AuthDeliveryError,
  type AuthDeliveryEnv,
} from "./authDelivery";

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export async function deliverAccessRejectionEmail(
  env: AuthDeliveryEnv,
  input: { email: string; reason: string },
): Promise<void> {
  const endpoint = env.AUTH_EMAIL_ENDPOINT?.trim();
  const token = env.AUTH_EMAIL_TOKEN?.trim();
  const from = env.AUTH_EMAIL_FROM?.trim();

  if (!endpoint || !token || !from) {
    throw new AuthDeliveryError("Access decision email delivery is not configured");
  }

  const safeReason = escapeHtml(input.reason.trim());
  const text = [
    "CYVRA Erase access decision",
    "",
    "Your request to download CYVRA Erase was not approved.",
    "",
    "Issue: commercial access was rejected by CYVORIQ administration.",
    `Reason: ${input.reason.trim()}`,
    "",
    "If you believe this is a mistake, reply to this message or contact CYVORIQ support.",
  ].join("\n");

  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "Idempotency-Key": `cyvoriq-access-reject-${input.email}-${Date.now()}`,
    },
    body: JSON.stringify({
      from,
      to: [input.email],
      subject: "CYVRA Erase access was not approved",
      text,
      html: `
        <div style="font-family:Arial,sans-serif;max-width:560px;margin:0 auto;padding:24px;">
          <h2>CYVRA Erase access decision</h2>
          <p>Your request to download CYVRA Erase was not approved.</p>
          <p><strong>Issue:</strong> commercial access was rejected by CYVORIQ administration.</p>
          <p><strong>Reason:</strong> ${safeReason}</p>
          <p>If you believe this is a mistake, contact CYVORIQ support.</p>
        </div>
      `,
    }),
  });

  if (!response.ok) {
    throw new AuthDeliveryError();
  }
}

export async function deliverLicenseIssuedEmail(
  env: AuthDeliveryEnv,
  input: { email: string; activationKey: string; keyPrefix: string },
): Promise<void> {
  const endpoint = env.AUTH_EMAIL_ENDPOINT?.trim();
  const token = env.AUTH_EMAIL_TOKEN?.trim();
  const from = env.AUTH_EMAIL_FROM?.trim();

  if (!endpoint || !token || !from) {
    throw new AuthDeliveryError("Licence email delivery is not configured");
  }

  const safeKey = escapeHtml(input.activationKey);
  const text = [
    "CYVRA Erase activation key",
    "",
    "An administrator issued your CYVRA Erase licence.",
    `Prefix: ${input.keyPrefix}`,
    `Activation key: ${input.activationKey}`,
    "",
    "Store this key. CYVORIQ does not keep the full key after issuance.",
    "Windows device binding is not live yet. Do not share the key.",
    "",
    "Sign in at https://www.cyvra.co.in/download",
  ].join("\n");

  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "Idempotency-Key": `cyvoriq-license-${input.email}-${Date.now()}`,
    },
    body: JSON.stringify({
      from,
      to: [input.email],
      subject: "Your CYVRA Erase activation key",
      text,
      html: `
        <div style="font-family:Arial,sans-serif;max-width:560px;margin:0 auto;padding:24px;">
          <h2>CYVRA Erase activation key</h2>
          <p>An administrator issued your CYVRA Erase licence.</p>
          <p><strong>Prefix:</strong> ${escapeHtml(input.keyPrefix)}</p>
          <p><strong>Activation key:</strong> <code style="font-size:16px;">${safeKey}</code></p>
          <p>Store this key. CYVORIQ does not keep the full key after issuance. Windows device binding is not live yet.</p>
          <p>Sign in at <a href="https://www.cyvra.co.in/download">www.cyvra.co.in/download</a>.</p>
        </div>
      `,
    }),
  });

  if (!response.ok) {
    throw new AuthDeliveryError();
  }
}
