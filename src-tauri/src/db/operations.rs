use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Manager;
use uuid::Uuid;

use crate::error::{Result, VaultError};

pub struct DbConnection(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub is_favorite: bool,
    pub group_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub username: String,
    pub password: String,
    pub website: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemDetail {
    #[serde(rename = "account")]
    Account(AccountItemDetail),
    #[serde(rename = "api_key")]
    ApiKey(ApiKeyItemDetail),
    #[serde(rename = "env_var")]
    EnvVar(EnvVarItemDetail),
    #[serde(rename = "database")]
    Database(DatabaseItemDetail),
    #[serde(rename = "ssh")]
    Ssh(SshItemDetail),
    #[serde(rename = "cloud")]
    Cloud(CloudItemDetail),
    #[serde(rename = "license")]
    License(LicenseItemDetail),
    #[serde(rename = "smtp")]
    Smtp(SmtpItemDetail),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub key_name: String,
    pub key_value: String,
    pub endpoint: Option<String>,
    pub auth_method: Option<String>,
    pub rotation_date: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub variables: Vec<EnvVarPair>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub db_type: String,
    pub host: String,
    pub port: Option<i64>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub host: String,
    pub port: Option<i64>,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub passphrase: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub provider: String,
    pub access_key_id: String,
    pub secret: String,
    pub region: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub software_name: String,
    pub license_key: String,
    pub bound_email: Option<String>,
    pub expiry_date: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub host: String,
    pub port: Option<i64>,
    pub encryption: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub from_address: Option<String>,
    pub notes: Option<String>,
}

pub fn get_db_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("vault.db")
}

pub fn open_connection(path: &std::path::Path) -> Result<Connection> {
    Connection::open(path).map_err(|e| VaultError::DatabaseError(e.to_string()))
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

    tx.commit()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_all_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, icon, parent_id, sort_order, created_at, updated_at 
             FROM groups ORDER BY sort_order",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let groups = stmt
        .query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                parent_id: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(groups)
}

pub fn create_group(
    conn: &Connection,
    name: &str,
    icon: Option<&str>,
    parent_id: Option<&str>,
    sort_order: Option<i32>,
) -> Result<Group> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();
    let sort_order = sort_order.unwrap_or(999);

    conn.execute(
        "INSERT INTO groups (id, name, icon, parent_id, sort_order, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, icon, parent_id, sort_order, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(Group {
        id,
        name: name.to_string(),
        icon: icon.map(|s| s.to_string()),
        parent_id: parent_id.map(|s| s.to_string()),
        sort_order,
        created_at: now,
        updated_at: now,
    })
}

pub fn update_group(
    conn: &Connection,
    id: &str,
    name: &str,
    icon: Option<&str>,
    sort_order: Option<i32>,
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE groups SET name = ?1, icon = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, icon, sort_order, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn delete_group(conn: &Connection, id: &str) -> Result<()> {
    if id.starts_with("built-in-") {
        return Err(VaultError::GroupNotEmpty);
    }

    conn.execute("DELETE FROM groups WHERE id = ?1", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn get_all_items(conn: &Connection) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title,
                COALESCE(a.username, ak.key_name, ev.key,
                    db.host || COALESCE(':' || db.port, ''),
                    s.username || '@' || s.host,
                    c.provider || ' · ' || c.access_key_id,
                    l.bound_email,
                    sm.host || COALESCE(':' || sm.port, ''),
                    '') as subtitle,
                i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at
             FROM items i
             LEFT JOIN account_items a ON i.id = a.item_id
             LEFT JOIN api_key_items ak ON i.id = ak.item_id
             LEFT JOIN env_var_items ev ON i.id = ev.item_id AND ev.sort_order = 0
             LEFT JOIN database_items db ON i.id = db.item_id
             LEFT JOIN ssh_items s ON i.id = s.item_id
             LEFT JOIN cloud_items c ON i.id = c.item_id
             LEFT JOIN license_items l ON i.id = l.item_id
             LEFT JOIN smtp_items sm ON i.id = sm.item_id
             ORDER BY i.title",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let items = stmt
        .query_map([], |row| {
            Ok(ItemSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                icon: row.get(3)?,
                item_type: row.get(4)?,
                is_favorite: row.get::<_, i32>(5)? != 0,
                group_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(items)
}

pub fn get_items_by_group(conn: &Connection, group_id: &str) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title,
                COALESCE(a.username, ak.key_name, ev.key,
                    db.host || COALESCE(':' || db.port, ''),
                    s.username || '@' || s.host,
                    c.provider || ' · ' || c.access_key_id,
                    l.bound_email,
                    sm.host || COALESCE(':' || sm.port, ''),
                    '') as subtitle,
                i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at
             FROM items i
             LEFT JOIN account_items a ON i.id = a.item_id
             LEFT JOIN api_key_items ak ON i.id = ak.item_id
             LEFT JOIN env_var_items ev ON i.id = ev.item_id AND ev.sort_order = 0
             LEFT JOIN database_items db ON i.id = db.item_id
             LEFT JOIN ssh_items s ON i.id = s.item_id
             LEFT JOIN cloud_items c ON i.id = c.item_id
             LEFT JOIN license_items l ON i.id = l.item_id
             LEFT JOIN smtp_items sm ON i.id = sm.item_id
             WHERE i.group_id = ?1
             ORDER BY i.title",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let items = stmt
        .query_map([group_id], |row| {
            Ok(ItemSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                icon: row.get(3)?,
                item_type: row.get(4)?,
                is_favorite: row.get::<_, i32>(5)? != 0,
                group_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(items)
}

pub fn get_account_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<AccountItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let account: (String, Vec<u8>, [u8; 12], Option<String>, Option<String>) = conn
        .query_row(
            "SELECT username, password_encrypted, password_nonce, website, notes 
             FROM account_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(2)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((row.get(0)?, row.get(1)?, nonce, row.get(3)?, row.get(4)?))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let password = String::from_utf8(
        crate::crypto::decrypt(key, &account.2, &account.1)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    )
    .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in password".to_string()))?;

    Ok(AccountItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        username: account.0,
        password,
        website: account.3,
        notes: account.4,
    })
}

pub fn get_api_key_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<ApiKeyItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at 
             FROM items WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get::<_, i32>(4)? != 0,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let api_key: (String, Vec<u8>, [u8; 12], Option<String>, Option<String>, Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date
             FROM api_key_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(2)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((
                    row.get(0)?, row.get(1)?, nonce, row.get(3)?,
                    row.get(4)?, row.get(5)?, row.get(6)?
                ))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let key_value = String::from_utf8(
        crate::crypto::decrypt(key, &api_key.2, &api_key.1)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    )
    .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in key value".to_string()))?;

    Ok(ApiKeyItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        key_name: api_key.0,
        key_value,
        endpoint: api_key.3,
        auth_method: api_key.4,
        notes: api_key.5,
        rotation_date: api_key.6,
    })
}

pub fn create_api_key_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    key_name: &str,
    key_value: &str,
    endpoint: Option<&str>,
    auth_method: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, key_value.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'api_key', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO api_key_items (item_id, key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, key_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), endpoint, auth_method, notes, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_api_key_item(
    conn: &Connection,
    id: &str,
    title: &str,
    key_name: &str,
    key_value: Option<&str>,
    endpoint: Option<&str>,
    auth_method: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(kv) = key_value {
        let encrypted = crate::crypto::encrypt(key, kv.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "UPDATE api_key_items SET key_name = ?1, key_value_encrypted = ?2, key_value_nonce = ?3, endpoint = ?4, auth_method = ?5, notes = ?6, rotation_date = ?7 WHERE item_id = ?8",
            params![key_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), endpoint, auth_method, notes, now, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE api_key_items SET key_name = ?1, endpoint = ?2, auth_method = ?3, notes = ?4 WHERE item_id = ?5",
            params![key_name, endpoint, auth_method, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn create_account_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    username: &str,
    password: &str,
    website: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, password.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'account', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO account_items (item_id, username, password_encrypted, password_nonce, website, notes) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), website, notes],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_account_item(
    conn: &Connection,
    id: &str,
    title: &str,
    username: &str,
    password: Option<&str>,
    website: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(pwd) = password {
        let encrypted = crate::crypto::encrypt(key, pwd.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "UPDATE account_items SET username = ?1, password_encrypted = ?2, password_nonce = ?3, website = ?4, notes = ?5 WHERE item_id = ?6",
            params![username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), website, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE account_items SET username = ?1, website = ?2, notes = ?3 WHERE item_id = ?4",
            params![username, website, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn delete_item(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM items WHERE id = ?1", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn toggle_favorite(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE items SET is_favorite = NOT is_favorite WHERE id = ?1",
        [id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_env_var_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<EnvVarItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at 
             FROM items WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get::<_, i32>(4)? != 0,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let notes: Option<String> = conn
        .query_row(
            "SELECT notes FROM env_var_items WHERE item_id = ?1 LIMIT 1",
            [id],
            |row| row.get(0),
        )
        .unwrap_or(None);

    let mut stmt = conn
        .prepare(
            "SELECT key, value_encrypted, value_nonce FROM env_var_items 
             WHERE item_id = ?1 ORDER BY sort_order",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let variables = stmt
        .query_map([id], |row| {
            let var_key: String = row.get(0)?;
            let value_encrypted: Vec<u8> = row.get(1)?;
            let nonce_bytes: Vec<u8> = row.get(2)?;
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&nonce_bytes);

            let decrypted = crate::crypto::decrypt(key, &nonce, &value_encrypted)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;

            Ok(EnvVarPair {
                key: var_key,
                value: String::from_utf8(decrypted).map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(EnvVarItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        variables,
        notes,
    })
}

pub fn create_env_var_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    variables: &[(String, String)],
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'env_var', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    for (i, (var_key, var_value)) in variables.iter().enumerate() {
        let encrypted = crate::crypto::encrypt(key, var_value.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "INSERT INTO env_var_items (item_id, key, value_encrypted, value_nonce, sort_order, notes) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                var_key,
                encrypted.ciphertext.to_vec(),
                encrypted.nonce.to_vec(),
                i as i32,
                notes
            ],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(id)
}

pub fn update_env_var_item(
    conn: &Connection,
    id: &str,
    title: &str,
    variables: &[(String, String)],
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute("DELETE FROM env_var_items WHERE item_id = ?1", params![id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    for (i, (var_key, var_value)) in variables.iter().enumerate() {
        let encrypted = crate::crypto::encrypt(key, var_value.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "INSERT INTO env_var_items (item_id, key, value_encrypted, value_nonce, sort_order, notes) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                var_key,
                encrypted.ciphertext.to_vec(),
                encrypted.nonce.to_vec(),
                i as i32,
                notes
            ],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// ---------- database ----------

pub fn create_database_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    db_type: &str,
    host: &str,
    port: Option<i64>,
    database_name: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    connection_url: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'database', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let encrypted = match password {
        Some(p) => {
            let e = crate::crypto::encrypt(key, p.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?;
            Some((e.ciphertext.to_vec(), e.nonce.to_vec()))
        }
        None => None,
    };

    conn.execute(
        "INSERT INTO database_items (item_id, db_type, host, port, database_name, username, password_encrypted, password_nonce, connection_url, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            db_type,
            host,
            port,
            database_name,
            username,
            encrypted.as_ref().map(|e| e.0.clone()),
            encrypted.as_ref().map(|e| e.1.clone()),
            connection_url,
            notes
        ],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_database_item(
    conn: &Connection,
    id: &str,
    title: &str,
    db_type: &str,
    host: &str,
    port: Option<i64>,
    database_name: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    connection_url: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(p) = password {
        let encrypted = crate::crypto::encrypt(key, p.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        conn.execute(
            "UPDATE database_items SET db_type = ?1, host = ?2, port = ?3, database_name = ?4, username = ?5, password_encrypted = ?6, password_nonce = ?7, connection_url = ?8, notes = ?9 WHERE item_id = ?10",
            params![db_type, host, port, database_name, username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), connection_url, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE database_items SET db_type = ?1, host = ?2, port = ?3, database_name = ?4, username = ?5, connection_url = ?6, notes = ?7 WHERE item_id = ?8",
            params![db_type, host, port, database_name, username, connection_url, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn get_database_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<DatabaseItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let db: (String, String, Option<i64>, Option<String>, Option<String>, Option<Vec<u8>>, Option<Vec<u8>>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT db_type, host, port, database_name, username, password_encrypted, password_nonce, connection_url, notes FROM database_items WHERE item_id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let password = match (db.5, db.6) {
        (Some(ciphertext), Some(nonce_bytes)) => {
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&nonce_bytes);
            Some(String::from_utf8(
                crate::crypto::decrypt(key, &nonce, &ciphertext)
                    .map_err(|e| VaultError::CryptoError(e.to_string()))?,
            )
            .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in password".to_string()))?)
        }
        _ => None,
    };

    Ok(DatabaseItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        db_type: db.0,
        host: db.1,
        port: db.2,
        database_name: db.3,
        username: db.4,
        password,
        connection_url: db.7,
        notes: db.8,
    })
}

// ---------- ssh ----------

pub fn create_ssh_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    host: &str,
    port: Option<i64>,
    username: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    passphrase: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'ssh', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let enc_password = match password {
        Some(p) => {
            let e = crate::crypto::encrypt(key, p.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?;
            Some((e.ciphertext.to_vec(), e.nonce.to_vec()))
        }
        None => None,
    };
    let enc_passphrase = match passphrase {
        Some(p) => {
            let e = crate::crypto::encrypt(key, p.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?;
            Some((e.ciphertext.to_vec(), e.nonce.to_vec()))
        }
        None => None,
    };

    conn.execute(
        "INSERT INTO ssh_items (item_id, host, port, username, password_encrypted, password_nonce, key_path, passphrase_encrypted, passphrase_nonce, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            host,
            port,
            username,
            enc_password.as_ref().map(|e| e.0.clone()),
            enc_password.as_ref().map(|e| e.1.clone()),
            key_path,
            enc_passphrase.as_ref().map(|e| e.0.clone()),
            enc_passphrase.as_ref().map(|e| e.1.clone()),
            notes
        ],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_ssh_item(
    conn: &Connection,
    id: &str,
    title: &str,
    host: &str,
    port: Option<i64>,
    username: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    passphrase: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let enc_password = match password {
        Some(p) => Some(
            crate::crypto::encrypt(key, p.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?,
        ),
        None => None,
    };
    let enc_passphrase = match passphrase {
        Some(p) => Some(
            crate::crypto::encrypt(key, p.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?,
        ),
        None => None,
    };

    // rusqlite params! does not support dynamic arity; branch explicitly.
    match (enc_password, enc_passphrase) {
        (Some(pw), Some(pp)) => {
            conn.execute(
                "UPDATE ssh_items SET host = ?1, port = ?2, username = ?3, key_path = ?4, notes = ?5, password_encrypted = ?6, password_nonce = ?7, passphrase_encrypted = ?8, passphrase_nonce = ?9 WHERE item_id = ?10",
                params![host, port, username, key_path, notes, pw.ciphertext.to_vec(), pw.nonce.to_vec(), pp.ciphertext.to_vec(), pp.nonce.to_vec(), id],
            )
        }
        (Some(pw), None) => {
            conn.execute(
                "UPDATE ssh_items SET host = ?1, port = ?2, username = ?3, key_path = ?4, notes = ?5, password_encrypted = ?6, password_nonce = ?7 WHERE item_id = ?8",
                params![host, port, username, key_path, notes, pw.ciphertext.to_vec(), pw.nonce.to_vec(), id],
            )
        }
        (None, Some(pp)) => {
            conn.execute(
                "UPDATE ssh_items SET host = ?1, port = ?2, username = ?3, key_path = ?4, notes = ?5, passphrase_encrypted = ?6, passphrase_nonce = ?7 WHERE item_id = ?8",
                params![host, port, username, key_path, notes, pp.ciphertext.to_vec(), pp.nonce.to_vec(), id],
            )
        }
        (None, None) => {
            conn.execute(
                "UPDATE ssh_items SET host = ?1, port = ?2, username = ?3, key_path = ?4, notes = ?5 WHERE item_id = ?6",
                params![host, port, username, key_path, notes, id],
            )
        }
    }
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn get_ssh_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<SshItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let ssh: (String, Option<i64>, String, Option<Vec<u8>>, Option<Vec<u8>>, Option<String>, Option<Vec<u8>>, Option<Vec<u8>>, Option<String>) = conn
        .query_row(
            "SELECT host, port, username, password_encrypted, password_nonce, key_path, passphrase_encrypted, passphrase_nonce, notes FROM ssh_items WHERE item_id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let decrypt_opt = |ciphertext: &Option<Vec<u8>>, nonce_bytes: &Option<Vec<u8>>, label: &str| -> Result<Option<String>> {
        match (ciphertext, nonce_bytes) {
            (Some(ct), Some(nb)) => {
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(nb);
                Ok(Some(String::from_utf8(
                    crate::crypto::decrypt(key, &nonce, ct)
                        .map_err(|e| VaultError::CryptoError(e.to_string()))?,
                )
                .map_err(|_| VaultError::CryptoError(format!("Invalid UTF-8 in {}", label)))?))
            }
            _ => Ok(None),
        }
    };

    Ok(SshItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        host: ssh.0,
        port: ssh.1,
        username: ssh.2,
        password: decrypt_opt(&ssh.3, &ssh.4, "password")?,
        key_path: ssh.5,
        passphrase: decrypt_opt(&ssh.6, &ssh.7, "passphrase")?,
        notes: ssh.8,
    })
}

// ---------- cloud ----------

pub fn create_cloud_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    provider: &str,
    access_key_id: &str,
    secret: &str,
    region: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, secret.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'cloud', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO cloud_items (item_id, provider, access_key_id, secret_encrypted, secret_nonce, region, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, provider, access_key_id, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), region, notes],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_cloud_item(
    conn: &Connection,
    id: &str,
    title: &str,
    provider: &str,
    access_key_id: &str,
    secret: Option<&str>,
    region: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(s) = secret {
        let encrypted = crate::crypto::encrypt(key, s.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        conn.execute(
            "UPDATE cloud_items SET provider = ?1, access_key_id = ?2, secret_encrypted = ?3, secret_nonce = ?4, region = ?5, notes = ?6 WHERE item_id = ?7",
            params![provider, access_key_id, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), region, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE cloud_items SET provider = ?1, access_key_id = ?2, region = ?3, notes = ?4 WHERE item_id = ?5",
            params![provider, access_key_id, region, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn get_cloud_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<CloudItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let cloud: (String, String, Vec<u8>, [u8; 12], Option<String>, Option<String>) = conn
        .query_row(
            "SELECT provider, access_key_id, secret_encrypted, secret_nonce, region, notes FROM cloud_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(3)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, nonce, row.get(4)?, row.get(5)?))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let secret = String::from_utf8(
        crate::crypto::decrypt(key, &cloud.3, &cloud.2)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    )
    .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in secret".to_string()))?;

    Ok(CloudItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        provider: cloud.0,
        access_key_id: cloud.1,
        secret,
        region: cloud.4,
        notes: cloud.5,
    })
}

// ---------- license ----------

pub fn create_license_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    software_name: &str,
    license_key: &str,
    bound_email: Option<&str>,
    expiry_date: Option<i64>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, license_key.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'license', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO license_items (item_id, software_name, license_key_encrypted, license_key_nonce, bound_email, expiry_date, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, software_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), bound_email, expiry_date, notes],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_license_item(
    conn: &Connection,
    id: &str,
    title: &str,
    software_name: &str,
    license_key: Option<&str>,
    bound_email: Option<&str>,
    expiry_date: Option<i64>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(k) = license_key {
        let encrypted = crate::crypto::encrypt(key, k.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        conn.execute(
            "UPDATE license_items SET software_name = ?1, license_key_encrypted = ?2, license_key_nonce = ?3, bound_email = ?4, expiry_date = ?5, notes = ?6 WHERE item_id = ?7",
            params![software_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), bound_email, expiry_date, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE license_items SET software_name = ?1, bound_email = ?2, expiry_date = ?3, notes = ?4 WHERE item_id = ?5",
            params![software_name, bound_email, expiry_date, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn get_license_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<LicenseItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let license: (String, Vec<u8>, [u8; 12], Option<String>, Option<i64>, Option<String>) = conn
        .query_row(
            "SELECT software_name, license_key_encrypted, license_key_nonce, bound_email, expiry_date, notes FROM license_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(2)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((row.get(0)?, row.get(1)?, nonce, row.get(3)?, row.get(4)?, row.get(5)?))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let license_key = String::from_utf8(
        crate::crypto::decrypt(key, &license.2, &license.1)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    )
    .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in license key".to_string()))?;

    Ok(LicenseItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        software_name: license.0,
        license_key,
        bound_email: license.3,
        expiry_date: license.4,
        notes: license.5,
    })
}

// ---------- smtp ----------

pub fn create_smtp_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    host: &str,
    port: Option<i64>,
    encryption: Option<&str>,
    username: Option<&str>,
    password: &str,
    from_address: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, password.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'smtp', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO smtp_items (item_id, host, port, encryption, username, password_encrypted, password_nonce, from_address, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, host, port, encryption, username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), from_address, notes],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_smtp_item(
    conn: &Connection,
    id: &str,
    title: &str,
    host: &str,
    port: Option<i64>,
    encryption: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    from_address: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(p) = password {
        let encrypted = crate::crypto::encrypt(key, p.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        conn.execute(
            "UPDATE smtp_items SET host = ?1, port = ?2, encryption = ?3, username = ?4, password_encrypted = ?5, password_nonce = ?6, from_address = ?7, notes = ?8 WHERE item_id = ?9",
            params![host, port, encryption, username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), from_address, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE smtp_items SET host = ?1, port = ?2, encryption = ?3, username = ?4, from_address = ?5, notes = ?6 WHERE item_id = ?7",
            params![host, port, encryption, username, from_address, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn get_smtp_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<SmtpItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let smtp: (String, Option<i64>, Option<String>, Option<String>, Vec<u8>, [u8; 12], Option<String>, Option<String>) = conn
        .query_row(
            "SELECT host, port, encryption, username, password_encrypted, password_nonce, from_address, notes FROM smtp_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(5)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, nonce, row.get(6)?, row.get(7)?))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let password = String::from_utf8(
        crate::crypto::decrypt(key, &smtp.5, &smtp.4)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    )
    .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in password".to_string()))?;

    Ok(SmtpItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        host: smtp.0,
        port: smtp.1,
        encryption: smtp.2,
        username: smtp.3,
        password,
        from_address: smtp.6,
        notes: smtp.7,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{decrypt, derive_key};

    fn seeded_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::init_schema(&conn).unwrap();
        crate::db::insert_builtin_groups(&conn).unwrap();
        conn
    }

    /// Create one item of every encrypted category and return their ids.
    fn seed_all_types(conn: &Connection, key: &[u8; 32]) -> Vec<String> {
        let mut ids = Vec::new();
        ids.push(
            create_account_item(conn, "built-in-accounts", "Acc", "user1", "secret-acc", None, None, key)
                .unwrap(),
        );
        ids.push(
            create_api_key_item(conn, "built-in-api-keys", "Key", "K", "secret-api", None, None, None, key)
                .unwrap(),
        );
        ids.push(
            create_env_var_item(
                conn,
                "built-in-env-vars",
                "Env",
                &[("A".to_string(), "secret-env".to_string())],
                None,
                key,
            )
            .unwrap(),
        );
        ids.push(
            create_database_item(
                conn, "built-in-databases", "DB", "postgresql", "h", Some(5432), None,
                Some("u"), Some("secret-db"), None, None, key,
            )
            .unwrap(),
        );
        ids.push(
            create_ssh_item(
                conn, "built-in-servers", "SSH", "h", Some(22), "u",
                Some("secret-ssh"), None, Some("secret-passphrase"), None, key,
            )
            .unwrap(),
        );
        ids.push(
            create_cloud_item(conn, "built-in-cloud", "Cloud", "aws", "AKIA", "secret-cloud", None, None, key)
                .unwrap(),
        );
        ids.push(
            create_license_item(conn, "built-in-licenses", "Lic", "Software", "secret-license", None, None, None, key)
                .unwrap(),
        );
        ids.push(
            create_smtp_item(conn, "built-in-smtp", "SMTP", "h", Some(587), None, None, "secret-smtp", None, None, key)
                .unwrap(),
        );
        ids
    }

    #[test]
    fn reencrypt_all_rotates_every_encrypted_field() {
        let conn = seeded_conn();
        let old_key = derive_key("old-password-1", None).unwrap().key;
        let new_key = derive_key("new-password-2", None).unwrap().key;

        let ids = seed_all_types(&conn, &old_key);

        // Sanity: readable with the old key before rotation.
        assert_eq!(
            get_account_item_detail(&conn, &ids[0], &old_key).unwrap().password,
            "secret-acc"
        );

        reencrypt_all(&conn, &old_key, &new_key).unwrap();

        // Every category decrypts correctly with the new key.
        assert_eq!(get_account_item_detail(&conn, &ids[0], &new_key).unwrap().password, "secret-acc");
        assert_eq!(get_api_key_item_detail(&conn, &ids[1], &new_key).unwrap().key_value, "secret-api");
        assert_eq!(
            get_env_var_item_detail(&conn, &ids[2], &new_key).unwrap().variables[0].value,
            "secret-env"
        );
        assert_eq!(
            get_database_item_detail(&conn, &ids[3], &new_key).unwrap().password.unwrap(),
            "secret-db"
        );
        let ssh = get_ssh_item_detail(&conn, &ids[4], &new_key).unwrap();
        assert_eq!(ssh.password.unwrap(), "secret-ssh");
        assert_eq!(ssh.passphrase.unwrap(), "secret-passphrase");
        assert_eq!(get_cloud_item_detail(&conn, &ids[5], &new_key).unwrap().secret, "secret-cloud");
        assert_eq!(
            get_license_item_detail(&conn, &ids[6], &new_key).unwrap().license_key,
            "secret-license"
        );
        assert_eq!(get_smtp_item_detail(&conn, &ids[7], &new_key).unwrap().password, "secret-smtp");

        // The old key must no longer decrypt the stored ciphertext.
        let (ct, nonce): (Vec<u8>, Vec<u8>) = conn
            .query_row(
                "SELECT password_encrypted, password_nonce FROM account_items WHERE item_id = ?1",
                [&ids[0]],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&nonce);
        assert!(decrypt(&old_key, &nonce_arr, &ct).is_err());
    }

    #[test]
    fn reencrypt_all_leaves_null_secrets_null() {
        let conn = seeded_conn();
        let old_key = derive_key("old-password-1", None).unwrap().key;
        let new_key = derive_key("new-password-2", None).unwrap().key;

        // Database item without a password — both columns are NULL.
        let id = create_database_item(
            &conn, "built-in-databases", "DB", "redis", "h", None, None, None, None, None, None, &old_key,
        )
        .unwrap();

        reencrypt_all(&conn, &old_key, &new_key).unwrap();

        let (ct, nonce): (Option<Vec<u8>>, Option<Vec<u8>>) = conn
            .query_row(
                "SELECT password_encrypted, password_nonce FROM database_items WHERE item_id = ?1",
                [&id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert!(ct.is_none() && nonce.is_none());
        assert!(get_database_item_detail(&conn, &id, &new_key).unwrap().password.is_none());
    }

    #[test]
    fn reencrypt_all_rolls_back_on_corrupt_ciphertext() {
        let conn = seeded_conn();
        let old_key = derive_key("old-password-1", None).unwrap().key;
        let new_key = derive_key("new-password-2", None).unwrap().key;

        let ids = seed_all_types(&conn, &old_key);

        // Corrupt one row so decryption fails mid-transaction.
        conn.execute(
            "UPDATE smtp_items SET password_encrypted = x'DEADBEEF' WHERE item_id = ?1",
            [&ids[7]],
        )
        .unwrap();

        assert!(reencrypt_all(&conn, &old_key, &new_key).is_err());

        // Untouched rows must still be readable with the OLD key (rollback).
        assert_eq!(
            get_account_item_detail(&conn, &ids[0], &old_key).unwrap().password,
            "secret-acc"
        );
        assert!(get_account_item_detail(&conn, &ids[0], &new_key).is_err());
    }
}
