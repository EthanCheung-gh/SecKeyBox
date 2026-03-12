use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Item not found")]
    ItemNotFound,

    #[error("Group not empty")]
    GroupNotEmpty,

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Vault not initialized")]
    VaultNotInitialized,

    #[error("Vault already initialized")]
    VaultAlreadyInitialized,
}

pub type Result<T> = std::result::Result<T, VaultError>;