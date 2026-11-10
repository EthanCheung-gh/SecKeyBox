import { describe, expect, it } from 'vitest';
import { normalizeBase32, parseOtpauthUri } from '@/lib/otpauth';

const SECRET = 'JBSWY3DPEHPK3PXP';

describe('normalizeBase32', () => {
  it('strips spaces, dashes, padding and uppercases the secret', () => {
    expect(normalizeBase32('jbsw y3dp ehpk 3pxp')).toBe(SECRET);
    expect(normalizeBase32('JBSW-Y3DP-EHPK-3PXP')).toBe(SECRET);
    expect(normalizeBase32(`${SECRET}====`)).toBe(SECRET);
  });

  it('rejects secrets shorter than 16 characters', () => {
    expect(normalizeBase32('ABC')).toBeNull();
    expect(normalizeBase32('JBSWY3DPEHPK3PX')).toBeNull();
    expect(normalizeBase32('')).toBeNull();
  });

  it('rejects non-base32 characters', () => {
    expect(normalizeBase32('JBSWY3DP0HPK3PXP')).toBeNull();
    expect(normalizeBase32('hello world')).toBeNull();
  });
});

describe('parseOtpauthUri', () => {
  it('parses issuer, account, digits and period from a TOTP URI', () => {
    const result = parseOtpauthUri(
      `otpauth://totp/ACME%20Co:john@example.com?secret=${SECRET}&issuer=ACME%20Co&digits=6&period=30`
    );

    expect(result).toEqual({
      secret: SECRET,
      issuer: 'ACME Co',
      account: 'john@example.com',
      digits: 6,
      period: 30,
    });
  });

  it('falls back to the label issuer and treats a label without a colon as the account', () => {
    expect(parseOtpauthUri(`otpauth://totp/GitHub:alice?secret=${SECRET}`)).toEqual({
      secret: SECRET,
      issuer: 'GitHub',
      account: 'alice',
      digits: undefined,
      period: undefined,
    });

    expect(parseOtpauthUri(`otpauth://totp/just-an-account?secret=${SECRET}`)).toEqual({
      secret: SECRET,
      issuer: undefined,
      account: 'just-an-account',
      digits: undefined,
      period: undefined,
    });
  });

  it('prefers the issuer query parameter over the label', () => {
    const result = parseOtpauthUri(
      `otpauth://totp/LabelName:alice?secret=${SECRET}&issuer=RealIssuer`
    );

    expect(result.issuer).toBe('RealIssuer');
    expect(result.account).toBe('alice');
  });

  it('accepts a bare base32 secret', () => {
    expect(parseOtpauthUri('jbsw y3dp ehpk 3pxp')).toEqual({ secret: SECRET });
  });

  it('rejects HOTP URIs', () => {
    expect(() =>
      parseOtpauthUri(`otpauth://hotp/alice?secret=${SECRET}&counter=1`)
    ).toThrow('Only TOTP entries are supported (HOTP is not)');
  });

  it('rejects malformed URIs', () => {
    expect(() => parseOtpauthUri('otpauth://[::1')).toThrow('Malformed otpauth URI');
  });

  it('rejects a missing or too-short secret', () => {
    expect(() => parseOtpauthUri('otpauth://totp/alice?issuer=ACME')).toThrow(
      'The URI has no valid base32 secret'
    );
    expect(() => parseOtpauthUri('otpauth://totp/alice?secret=ABC')).toThrow(
      'The URI has no valid base32 secret'
    );
  });

  it('rejects empty input', () => {
    expect(() => parseOtpauthUri('   ')).toThrow('Enter an otpauth:// URI or a base32 secret');
  });

  it('reports digits=0 as undefined', () => {
    expect(parseOtpauthUri(`otpauth://totp/alice?secret=${SECRET}&digits=0`).digits).toBe(
      undefined
    );
  });
});
