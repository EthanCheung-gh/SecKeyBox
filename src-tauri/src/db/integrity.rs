//! Vault integrity self-check.
//!
//! Tries to decrypt every encrypted column with the in-memory master key and
//! runs SQLite's own structural checks. The report only carries item titles,
//! types, field names and error classes — never plaintext or ciphertext — so
//! it is safe to show in the UI and to log.

use std::time::Instant;

use rusqlite::Connection;
use serde::Serialize;

use crate::error::{Result, VaultError};

use super::util::chrono_timestamp;

/// One encrypted value that can no longer be read.
#[derive(Debug, Clone, Serialize)]
pub struct IntegrityFailure {
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    /// account | api_key | env_var | database | ssh | cloud | license | smtp |
    /// custom_field | secret_history | attachment
    pub scope: String,
    /// password | totp_secret | key_value | value | passphrase | secret |
    /// license_key | data
    pub field: String,
    /// Error class only: never the plaintext or the ciphertext.
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrityReport {
    /// Number of non-NULL ciphertexts that were decryption attempts.
    pub checked_fields: usize,
    pub failures: Vec<IntegrityFailure>,
    /// `"ok"` or the joined messages of `PRAGMA integrity_check`.
    pub sqlite_integrity: String,
    pub foreign_key_violations: Vec<String>,
    pub checked_attachments: bool,
    pub checked_at: i64,
    pub duration_ms: u64,
}

/// One encrypted column pair, described with static identifiers so no caller
/// string ever reaches the SQL text.
struct EncryptedField {
    table: &'static str,
    ciphertext: &'static str,
    nonce: &'static str,
    scope: &'static str,
    field: &'static str,
    /// Attachments are opt-in: scanning them dominates the runtime.
    attachments: bool,
}

const ENCRYPTED_FIELDS: &[EncryptedField] = &[
    EncryptedField {
        table: "account_items",
        ciphertext: "password_encrypted",
        nonce: "password_nonce",
        scope: "account",
        field: "password",
        attachments: false,
    },
    EncryptedField {
        table: "account_items",
        ciphertext: "totp_secret_encrypted",
        nonce: "totp_secret_nonce",
        scope: "account",
        field: "totp_secret",
        attachments: false,
    },
    EncryptedField {
        table: "api_key_items",
        ciphertext: "key_value_encrypted",
        nonce: "key_value_nonce",
        scope: "api_key",
        field: "key_value",
        attachments: false,
    },
    EncryptedField {
        table: "env_var_items",
        ciphertext: "value_encrypted",
        nonce: "value_nonce",
        scope: "env_var",
        field: "value",
        attachments: false,
    },
    EncryptedField {
        table: "database_items",
        ciphertext: "password_encrypted",
        nonce: "password_nonce",
        scope: "database",
        field: "password",
        attachments: false,
    },
    EncryptedField {
        table: "ssh_items",
        ciphertext: "password_encrypted",
        nonce: "password_nonce",
        scope: "ssh",
        field: "password",
        attachments: false,
    },
    EncryptedField {
        table: "ssh_items",
        ciphertext: "passphrase_encrypted",
        nonce: "passphrase_nonce",
        scope: "ssh",
        field: "passphrase",
        attachments: false,
    },
    EncryptedField {
        table: "cloud_items",
        ciphertext: "secret_encrypted",
        nonce: "secret_nonce",
        scope: "cloud",
        field: "secret",
        attachments: false,
    },
    EncryptedField {
        table: "license_items",
        ciphertext: "license_key_encrypted",
        nonce: "license_key_nonce",
        scope: "license",
        field: "license_key",
        attachments: false,
    },
    EncryptedField {
        table: "smtp_items",
        ciphertext: "password_encrypted",
        nonce: "password_nonce",
        scope: "smtp",
        field: "password",
        attachments: false,
    },
    EncryptedField {
        table: "custom_fields",
        ciphertext: "value_encrypted",
        nonce: "value_nonce",
        scope: "custom_field",
        field: "value",
        attachments: false,
    },
    EncryptedField {
        table: "secret_history",
        ciphertext: "value_encrypted",
        nonce: "value_nonce",
        scope: "secret_history",
        field: "value",
        attachments: false,
    },
    EncryptedField {
        table: "item_attachments",
        ciphertext: "data_encrypted",
        nonce: "data_nonce",
        scope: "attachment",
        field: "data",
        attachments: true,
    },
];

fn db_err(e: rusqlite::Error) -> VaultError {
    VaultError::DatabaseError(e.to_string())
}

/// Decrypt every encrypted column (plus SQLite's own structural checks) and
/// report what could not be read. Trashed items are scanned as well: a corrupt
/// entry should surface before the user restores it.
///
/// The scan is O(rows) and uses one prepared statement per column pair.
pub fn check_vault_integrity(
    conn: &Connection,
    key: &[u8; 32],
    include_attachments: bool,
) -> Result<IntegrityReport> {
    let started = Instant::now();

    let sqlite_integrity = check_sqlite_integrity(conn)?;
    let foreign_key_violations = check_foreign_keys(conn)?;

    let mut checked_fields = 0usize;
    let mut failures: Vec<IntegrityFailure> = Vec::new();
    for spec in ENCRYPTED_FIELDS {
        if spec.attachments && !include_attachments {
            continue;
        }
        scan_field(conn, key, spec, &mut checked_fields, &mut failures)?;
    }

    Ok(IntegrityReport {
        checked_fields,
        failures,
        sqlite_integrity,
        foreign_key_violations,
        checked_attachments: include_attachments,
        checked_at: chrono_timestamp(),
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

fn check_sqlite_integrity(conn: &Connection) -> Result<String> {
    let mut stmt = conn
        .prepare("PRAGMA integrity_check")
        .map_err(db_err)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(db_err)?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(db_err)?);
    }

    // A healthy database reports exactly one row: "ok".
    if messages.len() == 1 && messages[0] == "ok" {
        return Ok("ok".to_string());
    }
    Ok(messages.join("; "))
}

/// Violations are reported, not raised: a dangling foreign key is a finding the
/// user needs to see, not a reason to abort the whole check.
fn check_foreign_keys(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("PRAGMA foreign_key_check")
        .map_err(db_err)?;
    let rows = stmt
        .query_map([], |row| {
            let table: String = row.get(0)?;
            let rowid: Option<i64> = row.get(1)?;
            let parent: String = row.get(2)?;
            Ok(format!(
                "{} rowid {} -> {}",
                table,
                rowid
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "null".to_string()),
                parent
            ))
        })
        .map_err(db_err)?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(db_err)?);
    }
    Ok(out)
}

fn scan_field(
    conn: &Connection,
    key: &[u8; 32],
    spec: &EncryptedField,
    checked_fields: &mut usize,
    failures: &mut Vec<IntegrityFailure>,
) -> Result<()> {
    // Table and column names come from the const table above; `items` is joined
    // without a `deleted_at` filter so trash entries are verified too.
    let sql = format!(
        "SELECT d.item_id, i.title, i.type, d.{ciphertext}, d.{nonce} \
         FROM {table} d JOIN items i ON i.id = d.item_id",
        ciphertext = spec.ciphertext,
        nonce = spec.nonce,
        table = spec.table,
    );

    let mut stmt = conn.prepare(&sql).map_err(db_err)?;
    let mut rows = stmt.query([]).map_err(db_err)?;
    while let Some(row) = rows.next().map_err(db_err)? {
        let item_id: String = row.get(0).map_err(db_err)?;
        let item_title: String = row.get(1).map_err(db_err)?;
        let item_type: String = row.get(2).map_err(db_err)?;
        let ciphertext: Option<Vec<u8>> = row.get(3).map_err(db_err)?;
        let nonce: Option<Vec<u8>> = row.get(4).map_err(db_err)?;

        // A secret that was never set has neither half; nothing to verify.
        if ciphertext.is_none() && nonce.is_none() {
            continue;
        }
        // Only rows that actually hold ciphertext count as an attempt.
        if ciphertext.is_some() {
            *checked_fields += 1;
        }

        if let Some(reason) = inspect(ciphertext, nonce, key) {
            failures.push(IntegrityFailure {
                item_id,
                item_title,
                item_type,
                scope: spec.scope.to_string(),
                field: spec.field.to_string(),
                reason,
            });
        }
    }
    Ok(())
}

/// Return `None` when the value decrypts, otherwise the error class. Binary
/// payloads (attachments) are not required to be UTF-8: AES-GCM already
/// authenticates them.
fn inspect(
    ciphertext: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    key: &[u8; 32],
) -> Option<String> {
    let (ciphertext, nonce) = match (ciphertext, nonce) {
        (Some(ciphertext), Some(nonce)) => (ciphertext, nonce),
        // Half a pair can never be decrypted; report it instead of treating the
        // secret as unset.
        (Some(_), None) => return Some("ciphertext/nonce mismatch: nonce is NULL".to_string()),
        (None, Some(_)) => {
            return Some("ciphertext/nonce mismatch: ciphertext is NULL".to_string())
        }
        (None, None) => return None,
    };

    let nonce: [u8; 12] = match nonce.as_slice().try_into() {
        Ok(nonce) => nonce,
        Err(_) => {
            return Some(format!(
                "invalid nonce length: {} bytes (expected 12)",
                nonce.len()
            ))
        }
    };

    match crate::crypto::decrypt(key, &nonce, &ciphertext) {
        Ok(_) => None,
        // `CryptoError` only describes the AEAD failure, never the payload.
        Err(e) => Some(e.to_string()),
    }
}
