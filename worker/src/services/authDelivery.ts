export interface AuthDeliveryEnv {
  AUTH_EMAIL_ENDPOINT?: string;
  AUTH_EMAIL_TOKEN?: string;
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

export async function deliverLoginCode(
  env: AuthDeliveryEnv,
  delivery: LoginCodeDelivery,
): Promise<void> {
  const endpoint = env.AUTH_EMAIL_ENDPOINT?.trim();
  const token = env.AUTH_EMAIL_TOKEN?.trim();

  if (!endpoint || !token) {
    throw new AuthDeliveryError("Authentication email delivery is not configured");
  }

  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      type: "cyvoriq_login_code",
      to: delivery.email,
      code: delivery.code,
      challengeId: delivery.challengeId,
      expiresAt: delivery.expiresAt,
    }),
  });

  if (!response.ok) {
    throw new AuthDeliveryError();
  }
}
