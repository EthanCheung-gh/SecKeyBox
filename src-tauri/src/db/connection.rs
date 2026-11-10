use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

use crate::error::{Result, VaultError};

pub struct DbConnection(pub Mutex<Connection>);

pub fn get_db_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("vault.db")
}

pub fn open_connection(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    enable_foreign_keys(&conn)?;
    Ok(conn)
}

/// The schema relies on ON DELETE CASCADE (groups -> items -> detail rows).
/// SQLite only enforces foreign keys when the per-connection pragma is on,
/// so every new connection must enable it explicitly.
pub fn enable_foreign_keys(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn is_vault_initialized(conn: &Connection) -> Result<bool> {
    let result: std::result::Result<String, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'password_hash'",
        [],
        |row| row.get(0),
    );
    Ok(result.is_ok())
}

pub fn set_password_hash(conn: &Connection, hash: &str, salt: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('password_hash', ?)",
        [hash],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('password_salt', ?)",
        [salt],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn get_password_hash(conn: &Connection) -> Result<Option<String>> {
    let result: std::result::Result<String, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'password_hash'",
        [],
        |row| row.get(0),
    );
    Ok(result.ok())
}

pub fn get_password_salt(conn: &Connection) -> Result<Option<String>> {
    let result: std::result::Result<String, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'password_salt'",
        [],
        |row| row.get(0),
    );
    Ok(result.ok())
}

/// Re-encrypt one encrypted column pair of a detail table with a new key.
/// `id_cols` are the primary-key columns used in the WHERE clause
/// (env_var_items uses a composite key).
fn reencrypt_column(
    conn: &Connection,
    table: &str,
    id_cols: &[&str],
    ct_col: &str,
    nonce_col: &str,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<()> {
    let select_sql = format!(
        "SELECT {}, {}, {} FROM {}",
        id_cols.join(", "),
        ct_col,
        nonce_col,
        table
    );
    let mut stmt = conn
        .prepare(&select_sql)
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let id_count = id_cols.len();
    let rows: Vec<(Vec<rusqlite::types::Value>, Vec<u8>, Vec<u8>)> = stmt
        .query_map([], |row| {
            let mut ids = Vec::with_capacity(id_count);
            for i in 0..id_count {
                ids.push(row.get::<_, rusqlite::types::Value>(i)?);
            }
            let ct: Option<Vec<u8>> = row.get(id_count)?;
            let nonce: Option<Vec<u8>> = row.get(id_count + 1)?;
            Ok((ids, ct, nonce))
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .filter_map(|r| r.ok())
        // NULL ciphertext (optional secret) stays NULL — nothing to rotate.
        .filter_map(|(ids, ct, nonce)| match (ct, nonce) {
            (Some(ct), Some(nonce)) => Some((ids, ct, nonce)),
            _ => None,
        })
        .collect();
    drop(stmt);

    let where_clause = id_cols
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{} = ?{}", c, i + 3))
        .collect::<Vec<_>>()
        .join(" AND ");
    let update_sql = format!(
        "UPDATE {} SET {} = ?1, {} = ?2 WHERE {}",
        table, ct_col, nonce_col, where_clause
    );

    for (ids, ct, nonce) in rows {
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&nonce);
        let plaintext = crate::crypto::decrypt(old_key, &nonce_arr, &ct)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        let reencrypted = crate::crypto::encrypt(new_key, &plaintext)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        let mut params: Vec<rusqlite::types::Value> = vec![
            rusqlite::types::Value::Blob(reencrypted.ciphertext.to_vec()),
            rusqlite::types::Value::Blob(reencrypted.nonce.to_vec()),
        ];
        params.extend(ids);
        conn.execute(&update_sql, rusqlite::params_from_iter(params))
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

/// Re-encrypt every sensitive field from `old_key` to `new_key` inside a
/// single transaction, so a failure leaves the vault readable with the old
/// key (no partial rewrite).
pub fn reencrypt_all(conn: &Connection, old_key: &[u8; 32], new_key: &[u8; 32]) -> Result<()> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    reencrypt_column(&tx, "account_items", &["item_id"], "password_encrypted", "password_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "api_key_items", &["item_id"], "key_value_encrypted", "key_value_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "env_var_items", &["item_id", "key"], "value_encrypted", "value_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "database_items", &["item_id"], "password_encrypted", "password_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "ssh_items", &["item_id"], "password_encrypted", "password_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "ssh_items", &["item_id"], "passphrase_encrypted", "passphrase_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "cloud_items", &["item_id"], "secret_encrypted", "secret_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "license_items", &["item_id"], "license_key_encrypted", "license_key_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "smtp_items", &["item_id"], "password_encrypted", "password_nonce", old_key, new_key)?;
    reencrypt_column(&tx, "secret_history", &["id"], "value_encrypted", "value_nonce", old_key, new_key)?;

    tx.commit()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}
