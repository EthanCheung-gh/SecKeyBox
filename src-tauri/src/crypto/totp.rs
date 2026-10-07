//! TOTP (RFC 6238) two-factor code generation.
//!
//! Secrets are stored as base32 (the format almost every service shows in
//! its "manual setup key" field). Verification is fully offline; codes are
//! generated from the stored secret plus the current time.

use hmac::{Hmac, Mac};
use sha1::Sha1;

use crate::error::{Result, VaultError};

/// Standard TOTP period in seconds.
pub const TIME_STEP: u64 = 30;
/// Standard code length.
pub const DIGITS: u32 = 6;

fn hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
    let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    let out = mac.finalize().into_bytes();
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&out);
    arr
}

/// Decode an RFC 4648 base32 secret (no padding required, case-insensitive).
/// Whitespace and dashes (used for display) are ignored.
pub fn decode_base32(input: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    let mut bits: u32 = 0;
    let mut bit_count = 0u8;
    let mut out = Vec::new();

    for c in input.chars() {
        if c.is_whitespace() || c == '-' {
            continue;
        }
        let c = c.to_ascii_uppercase();
        let val = ALPHABET
            .iter()
            .position(|&a| a == c as u8)
            .ok_or_else(|| VaultError::CryptoError(format!("invalid base32 character: {c}")))?;
        bits = (bits << 5) | val as u32;
        bit_count += 5;
        if bit_count >= 8 {
            bit_count -= 8;
            out.push((bits >> bit_count) as u8);
        }
    }

    Ok(out)
}

/// Generate a `digits`-long TOTP code for `secret_b32` at `unix_timestamp`.
pub fn generate(secret_b32: &str, unix_timestamp: i64) -> Result<String> {
    let secret = decode_base32(secret_b32)?;
    if secret.is_empty() {
        return Err(VaultError::CryptoError("empty TOTP secret".to_string()));
    }

    let counter = (unix_timestamp.max(0) as u64) / TIME_STEP;
    let mac = hmac_sha1(&secret, &counter.to_be_bytes());

    // RFC 4226 dynamic truncation → 31-bit offset
    let offset = (mac[mac.len() - 1] & 0x0f) as usize;
    let bin_code = ((mac[offset] as u32 & 0x7f) << 24)
        | ((mac[offset + 1] as u32 & 0xff) << 16)
        | ((mac[offset + 2] as u32 & 0xff) << 8)
        | (mac[offset + 3] as u32 & 0xff);

    let modulus = 10u32.pow(DIGITS);
    let code = bin_code % modulus;
    Ok(format!("{code:0width$}", width = DIGITS as usize))
}

/// Seconds remaining in the current time step (for the countdown ring).
pub fn seconds_remaining(unix_timestamp: i64) -> u64 {
    TIME_STEP - ((unix_timestamp.max(0) as u64) % TIME_STEP)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 appendix B test vectors (secret "12345678901234567890",
    // SHA-1, 8-digit codes); we compare against the last 6 digits, since
    // bin_code % 10^6 == (bin_code % 10^8) % 10^6.
    #[test]
    fn rfc_6238_test_vectors() {
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let expected_8_digits = [
            (59i64, "94287082"),
            (1111111109, "07081804"),
            (1111111111, "14050471"),
            (1234567890, "89005924"),
            (2000000000, "69279037"),
        ];
        for (ts, expected) in expected_8_digits {
            let code = generate(secret, ts).unwrap();
            assert_eq!(code.len(), 6);
            assert_eq!(code, expected[expected.len() - 6..], "timestamp {ts}");
        }
    }

    #[test]
    fn base32_decoding_variants() {
        // lowercase + spaces + dashes decode to the same bytes
        let canonical = decode_base32("GEZDGNBVGY3TQOJQ").unwrap();
        assert_eq!(canonical, b"1234567890");
        let spaced = decode_base32("gezd gnbv-gy3t qojq").unwrap();
        assert_eq!(spaced, b"1234567890");
        assert!(decode_base32("GEZD1").is_err());
    }

    #[test]
    fn seconds_remaining_within_step() {
        assert_eq!(seconds_remaining(0), 30);
        assert_eq!(seconds_remaining(29), 1);
        assert_eq!(seconds_remaining(30), 30);
    }
}
