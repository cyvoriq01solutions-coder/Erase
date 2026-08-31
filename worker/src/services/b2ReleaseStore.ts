import type { ReleaseStore } from "./packageRelease";

export interface B2StoreEnv {
  B2_BUCKET?: string;
  B2_ENDPOINT?: string;
  B2_REGION?: string;
  B2_KEY_ID?: string;
  B2_APPLICATION_KEY?: string;
}

function encoder(): TextEncoder {
  return new TextEncoder();
}

function toHex(buffer: ArrayBuffer): string {
  return [...new Uint8Array(buffer)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

async function sha256Hex(data: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", encoder().encode(data));
  return toHex(digest);
}

async function hmac(
  key: BufferSource,
  data: string,
): Promise<ArrayBuffer> {
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  return crypto.subtle.sign("HMAC", cryptoKey, encoder().encode(data));
}

function uriEncode(value: string, encodeSlash: boolean): string {
  let encoded = "";
  for (const char of value) {
    if (
      (char >= "A" && char <= "Z") ||
      (char >= "a" && char <= "z") ||
      (char >= "0" && char <= "9") ||
      char === "-" ||
      char === "_" ||
      char === "." ||
      char === "~"
    ) {
      encoded += char;
    } else if (char === "/" && !encodeSlash) {
      encoded += "/";
    } else {
      const bytes = encoder().encode(char);
      for (const byte of bytes) {
        encoded += `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
      }
    }
  }
  return encoded;
}

async function signingKey(
  secret: string,
  dateStamp: string,
  region: string,
): Promise<ArrayBuffer> {
  const kDate = await hmac(encoder().encode(`AWS4${secret}`), dateStamp);
  const kRegion = await hmac(kDate, region);
  const kService = await hmac(kRegion, "s3");
  return hmac(kService, "aws4_request");
}

function amzDate(now: Date): { amzDate: string; dateStamp: string } {
  const iso = now.toISOString().replace(/[:-]|\.\d{3}/g, "");
  return { amzDate: iso, dateStamp: iso.slice(0, 8) };
}

export function isB2StoreConfigured(env: B2StoreEnv): boolean {
  return Boolean(
    env.B2_BUCKET?.trim() &&
      env.B2_ENDPOINT?.trim() &&
      env.B2_REGION?.trim() &&
      env.B2_KEY_ID?.trim() &&
      env.B2_APPLICATION_KEY?.trim(),
  );
}

async function signedB2Request(
  env: B2StoreEnv,
  method: "GET" | "HEAD",
  key: string,
): Promise<Response> {
  const bucket = env.B2_BUCKET!.trim();
  const endpoint = env.B2_ENDPOINT!.trim().replace(/^https?:\/\//i, "");
  const region = env.B2_REGION!.trim();
  const accessKey = env.B2_KEY_ID!.trim();
  const secretKey = env.B2_APPLICATION_KEY!.trim();

  const host = `${bucket}.${endpoint}`;
  const canonicalUri = `/${uriEncode(key, false)}`;
  const { amzDate: xAmzDate, dateStamp } = amzDate(new Date());
  const payloadHash = "UNSIGNED-PAYLOAD";
  const signedHeaders = "host;x-amz-content-sha256;x-amz-date";
  const canonicalHeaders =
    `host:${host}\n` +
    `x-amz-content-sha256:${payloadHash}\n` +
    `x-amz-date:${xAmzDate}\n`;
  const canonicalRequest = [
    method,
    canonicalUri,
    "",
    canonicalHeaders,
    signedHeaders,
    payloadHash,
  ].join("\n");
  const credentialScope = `${dateStamp}/${region}/s3/aws4_request`;
  const stringToSign = [
    "AWS4-HMAC-SHA256",
    xAmzDate,
    credentialScope,
    await sha256Hex(canonicalRequest),
  ].join("\n");
  const signature = toHex(
    await hmac(await signingKey(secretKey, dateStamp, region), stringToSign),
  );
  const authorization =
    `AWS4-HMAC-SHA256 Credential=${accessKey}/${credentialScope}, ` +
    `SignedHeaders=${signedHeaders}, Signature=${signature}`;

  return fetch(`https://${host}${canonicalUri}`, {
    method,
    headers: {
      Authorization: authorization,
      host,
      "x-amz-content-sha256": payloadHash,
      "x-amz-date": xAmzDate,
    },
  });
}

function contentLength(response: Response): number {
  const header = response.headers.get("content-length");
  if (header === null) {
    return 0;
  }
  const size = Number(header);
  return Number.isFinite(size) ? size : 0;
}

export function createB2ReleaseStore(env: B2StoreEnv): ReleaseStore | null {
  if (!isB2StoreConfigured(env)) {
    return null;
  }

  return {
    async head(key: string) {
      const response = await signedB2Request(env, "HEAD", key);
      if (response.status === 404 || response.status === 403) {
        return null;
      }
      if (!response.ok) {
        return null;
      }
      const size = contentLength(response);
      if (size > 0) {
        return { size };
      }
      if (response.headers.get("etag")) {
        return { size: 1 };
      }
      return null;
    },
    async get(key: string) {
      const response = await signedB2Request(env, "GET", key);
      if (response.status === 404 || response.status === 403) {
        return null;
      }
      if (!response.ok || response.body === null) {
        return null;
      }
      const size = contentLength(response);
      if (size <= 0) {
        return null;
      }
      return {
        body: response.body,
        size,
        httpEtag: response.headers.get("etag") ?? undefined,
      };
    },
  };
}
