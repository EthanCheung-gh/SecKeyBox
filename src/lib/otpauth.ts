/**
 * Parser for `otpauth://totp/...` URIs (what Google Authenticator, 1Password,
 * Bitwarden and most QR codes encode) plus bare base32 secrets.
 *
 * The backend TOTP implementation is fixed at 6 digits / 30 seconds, so
 * `digits` and `period` are surfaced to callers for validation instead of
 * being silently ignored.
 */
export interface OtpauthResult {
  secret: string;
  issuer?: string;
  account?: string;
  digits?: number;
  period?: number;
}

const BASE32_RE = /^[A-Z2-7]+=*$/;

/** Uppercase, strip spaces/dashes/padding; null when not valid base32. */
export function normalizeBase32(raw: string): string | null {
  const cleaned = raw.replace(/[\s-]/g, '').toUpperCase().replace(/=+$/, '');
  // Minimum 16 chars (10 bytes) so ordinary words like "hello world" are
  // not mistaken for a secret; real TOTP secrets are 16 or 32 chars.
  if (cleaned.length < 16) return null;
  return BASE32_RE.test(cleaned) ? cleaned : null;
}

export function parseOtpauthUri(input: string): OtpauthResult {
  const trimmed = input.trim();
  if (trimmed === '') {
    throw new Error('Enter an otpauth:// URI or a base32 secret');
  }

  // Bare secret: accept it directly so pasted secrets keep working.
  if (!trimmed.toLowerCase().startsWith('otpauth://')) {
    const secret = normalizeBase32(trimmed);
    if (secret) return { secret };
    throw new Error('Not an otpauth:// URI or a valid base32 secret');
  }

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    throw new Error('Malformed otpauth URI');
  }

  if (url.host.toLowerCase() !== 'totp') {
    throw new Error('Only TOTP entries are supported (HOTP is not)');
  }

  const secret = normalizeBase32(url.searchParams.get('secret') ?? '');
  if (!secret) {
    throw new Error('The URI has no valid base32 secret');
  }

  const label = decodeURIComponent(url.pathname.replace(/^\//, ''));
  const separator = label.indexOf(':');
  const labelIssuer = separator >= 0 ? label.slice(0, separator) : undefined;
  const account = separator >= 0 ? label.slice(separator + 1) : label;

  const digitsRaw = Number(url.searchParams.get('digits') ?? '');
  const periodRaw = Number(url.searchParams.get('period') ?? '');

  return {
    secret,
    issuer: url.searchParams.get('issuer') ?? labelIssuer ?? undefined,
    account: account || undefined,
    digits: Number.isFinite(digitsRaw) && digitsRaw > 0 ? digitsRaw : undefined,
    period: Number.isFinite(periodRaw) && periodRaw > 0 ? periodRaw : undefined,
  };
}

/** Decode a QR code from an image file using the browser canvas. */
export async function decodeQrFromImageFile(file: File): Promise<string> {
  const { default: jsQR } = await import('jsqr');
  const bitmap = await createImageBitmap(file);
  const canvas = document.createElement('canvas');
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('Canvas is not available');
  context.drawImage(bitmap, 0, 0);
  const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
  const result = jsQR(imageData.data, imageData.width, imageData.height);
  if (!result) throw new Error('No QR code found in this image');
  return result.data;
}
