import type { IntegritySeal } from "../types/shell";

export interface SealCheck {
  digestMatches: boolean;
  signatureValid: boolean | null;
  ok: boolean;
  detail: string;
}

function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Number.parseInt(hex.slice(index * 2, index * 2 + 2), 16);
  }
  return bytes;
}

function bytesToHex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** Group a hex digest so a technician can read it aloud. */
export function groupHex(hex: string): string {
  return hex.replace(/(.{4})/g, "$1 ").trim();
}

/**
 * Re-hash the stored canonical JSON and, when WebView supports Ed25519,
 * check the local signature. This is the in-app verify page. It does not
 * contact cyvra.co.in and does not claim CYVORIQ authenticated the report.
 */
export async function verifyIntegritySeal(seal: IntegritySeal): Promise<SealCheck> {
  const encoded = new TextEncoder().encode(seal.canonicalJson);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", encoded));
  const digestMatches = bytesToHex(digest) === seal.digestHex;

  let signatureValid: boolean | null = null;
  try {
    const key = await crypto.subtle.importKey(
      "raw",
      hexToBytes(seal.publicKeyHex) as BufferSource,
      "Ed25519",
      false,
      ["verify"],
    );
    signatureValid = await crypto.subtle.verify(
      "Ed25519",
      key,
      hexToBytes(seal.signatureHex) as BufferSource,
      encoded,
    );
  } catch {
    signatureValid = null;
  }

  if (digestMatches && signatureValid === true) {
    return {
      digestMatches,
      signatureValid,
      ok: true,
      detail:
        "This report’s JSON matches the SHA-256 digest and the Ed25519 signature. The copy was not altered after the scan. That is a local integrity check, not cloud authentication.",
    };
  }

  if (!digestMatches) {
    return {
      digestMatches,
      signatureValid,
      ok: false,
      detail: "The SHA-256 digest does not match the stored JSON. This copy was altered.",
    };
  }

  if (signatureValid === false) {
    return {
      digestMatches,
      signatureValid,
      ok: false,
      detail: "The Ed25519 signature does not match this report. This copy was altered.",
    };
  }

  return {
    digestMatches,
    signatureValid,
    ok: digestMatches,
    detail:
      "The SHA-256 digest matches. This PC could not check Ed25519 in the web view, so the signature was not re-verified here.",
  };
}
