use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use zeroize::Zeroize;

use crate::error::{Result, VaultError};

const MEMORY_COST: u32 = 64 * 1024; // 64MB in KiB (Argon2 m_cost unit)
const TIME_COST: u32 = 3;
const PARALLELISM: u32 = 4;

pub struct DerivedKey {
    pub key: [u8; 32],
    pub salt: String,
    pub hash: String,
}

impl Drop for DerivedKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

pub fn derive_key(password: &str, salt: Option<&str>) -> Result<DerivedKey> {
    let salt = match salt {
        Some(s) => SaltString::from_b64(s).map_err(|e| VaultError::CryptoError(e.to_string()))?,
        None => SaltString::generate(&mut OsRng),
    };

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST, TIME_COST, PARALLELISM, None)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    );

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let key_bytes = hash
        .hash
        .ok_or_else(|| VaultError::CryptoError("Failed to generate hash".to_string()))?
        .as_bytes();

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes[..32]);

    Ok(DerivedKey {
        key,
        salt: salt.to_string(),
        hash: hash.to_string(),
    })
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST, TIME_COST, PARALLELISM, None)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    );

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}