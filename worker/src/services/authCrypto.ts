const textEncoder = new TextEncoder();

const OTP_SPACE = 1_000_000;
const UINT32_RANGE = 0x1_0000_0000;
const OTP_REJECTION_LIMIT = UINT32_RANGE - (UINT32_RANGE % OTP_SPACE);

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
}

function hexToBytes(hex: string): Uint8Array<ArrayBuffer> {
  if (hex.length % 2 !== 0 || !/^[0-9a-f]+$/i.test(hex)) {
    throw new Error("Invalid hexadecimal digest");
  }

  const bytes = new Uint8Array(hex.length / 2);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number.parseInt(hex.slice(index * 2, index * 2 + 2), 16);
  }
  return bytes;
}

async function importHmacKey(secret: string): Promise<CryptoKey> {
  if (secret.length < 32) {
    throw new Error("AUTH_PEPPER must be at least 32 characters");
  }

  return crypto.subtle.importKey(
    "raw",
    textEncoder.encode(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"],
  );
}

function otpMessage(
  challengeId: string,
  organizationId: string,
  userId: string,
  code: string,
): Uint8Array<ArrayBuffer> {
  return textEncoder.encode(
    `cyvoriq-erase:otp:v1:${challengeId}:${organizationId}:${userId}:${code}`,
  );
}

export function generateOneTimeCode(): string {
  const random = new Uint32Array(1);
  let value: number;

  do {
    crypto.getRandomValues(random);
    value = random[0];
  } while (value >= OTP_REJECTION_LIMIT);

  return String(value % OTP_SPACE).padStart(6, "0");
}

export function generateSessionToken(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return bytesToHex(bytes);
}

export async function hashSessionToken(token: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", textEncoder.encode(token));
  return bytesToHex(new Uint8Array(digest));
}

export async function hashOneTimeCode(
  pepper: string,
  challengeId: string,
  organizationId: string,
  userId: string,
  code: string,
): Promise<string> {
  const key = await importHmacKey(pepper);
  const signature = await crypto.subtle.sign(
    "HMAC",
    key,
    otpMessage(challengeId, organizationId, userId, code),
  );
  return bytesToHex(new Uint8Array(signature));
}

export async function verifyOneTimeCode(
  pepper: string,
  challengeId: string,
  organizationId: string,
  userId: string,
  code: string,
  expectedHash: string,
): Promise<boolean> {
  const key = await importHmacKey(pepper);
  return crypto.subtle.verify(
    "HMAC",
    key,
    hexToBytes(expectedHash),
    otpMessage(challengeId, organizationId, userId, code),
  );
}

export type AdminRateLimitScope = "source" | "identity";

export async function hashAdminRateLimitKey(
  pepper: string,
  scope: AdminRateLimitScope,
  value: string,
): Promise<string> {
  const key = await importHmacKey(pepper);
  const message = textEncoder.encode(
    `cyvoriq-erase:admin-rate-limit:v1:${scope}:${value}`,
  );
  const signature = await crypto.subtle.sign(
    "HMAC",
    key,
    message,
  );
  return bytesToHex(new Uint8Array(signature));
}

const LICENSE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

export function generateActivationKey(): string {
  const groups: string[] = [];
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  for (let group = 0; group < 4; group += 1) {
    let chunk = "";
    for (let index = 0; index < 4; index += 1) {
      chunk += LICENSE_ALPHABET[bytes[group * 4 + index]! % LICENSE_ALPHABET.length];
    }
    groups.push(chunk);
  }
  return `CYVRA-${groups.join("-")}`;
}

export function activationKeyPrefix(key: string): string {
  const parts = key.split("-");
  return `${parts[0]}-${parts[1]}`;
}

export async function hashActivationKey(
  pepper: string,
  key: string,
): Promise<string> {
  const hmacKey = await importHmacKey(pepper);
  const message = textEncoder.encode(`cyvoriq-erase:license:v1:${key}`);
  const signature = await crypto.subtle.sign("HMAC", hmacKey, message);
  return bytesToHex(new Uint8Array(signature));
}
