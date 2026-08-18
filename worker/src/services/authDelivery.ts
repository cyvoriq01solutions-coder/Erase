export interface AuthDeliveryEnv {
  RESEND_API_KEY?: string;
  AUTH_EMAIL_FROM?: string;
}

export interface LoginCodeDelivery {
  email: string;
  code: string;
  challengeId: string;
  expiresAt: string;
}

export class AuthDeliveryError extends Error {
  constructor(message = "Authentication email delivery failed") {
    super(message);
    this.name = "AuthDeliveryError";
  }
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export async function deliverLoginCode(
  env: AuthDeliveryEnv,
  delivery: LoginCodeDelivery,
): Promise<void> {
  const apiKey = env.RESEND_API_KEY?.trim();
  const from = env.AUTH_EMAIL_FROM?.trim();

  if (!apiKey || !from) {
    throw new AuthDeliveryError("Authentication email delivery is not configured");
  }

  const expiry = new Date(delivery.expiresAt);
  const expiryText = Number.isNaN(expiry.getTime())
    ? "10 minutes"
    : expiry.toISOString();
  const safeCode = escapeHtml(delivery.code);

  const response = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
      "Idempotency-Key": `cyvoriq-auth-${delivery.challengeId}`,
    },
    body: JSON.stringify({
      from,
      to: [delivery.email],
      subject: "Your CYVORIQ verification code",
      text: [
        "CYVORIQ account verification",
        "",
        `Your verification code is: ${delivery.code}`,
        "",
        `This code expires at ${expiryText}.`,
        "If you did not request this code, you can ignore this email.",
      ].join("\n"),
      html: `
        <div style="font-family:Arial,sans-serif;max-width:560px;margin:0 auto;padding:24px;">
          <h2>CYVORIQ account verification</h2>
          <p>Your verification code is:</p>
          <p style="font-size:32px;font-weight:700;letter-spacing:6px;">${safeCode}</p>
          <p>This code expires at ${escapeHtml(expiryText)}.</p>
          <p>If you did not request this code, you can ignore this email.</p>
        </div>
      `,
    }),
  });

  if (!response.ok) {
    throw new AuthDeliveryError();
  }
}
