use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use zeroize::Zeroize;

use crate::error::{Result, VaultError};

const NONCE_SIZE: usize = 12;

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<EncryptedData> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    Ok(plaintext)
}

pub struct SecureKey {
    key: [u8; 32],
}

impl SecureKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}