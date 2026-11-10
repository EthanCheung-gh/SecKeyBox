use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, VaultError};

use super::audit::log_event;
use super::types::*;
use super::util::{chrono_timestamp, hex_encode, item_title, with_tx};

/// Items may never be created *inside* the trash group: it only ever holds
/// soft-deleted rows, and a live item parked there would be invisible to every
/// active list while still being readable by id. All creation paths — the
/// Tauri commands and both importers — funnel through the `create_*` helpers,
/// so this is the single checkpoint.
pub fn ensure_group_can_hold_items(group_id: &str) -> Result<()> {
    if group_id == TRASH_GROUP_ID {
        return Err(VaultError::Validation(
            "Items cannot be created in the trash group".to_string(),
        ));
    }
    Ok(())
}

/// A missing or trashed item header must surface as `ItemNotFound` (the IPC
/// layer maps that to `not_found`), not as a generic database error.
fn item_header_missing(error: rusqlite::Error) -> VaultError {
    match error {
        rusqlite::Error::QueryReturnedNoRows => VaultError::ItemNotFound,
        other => VaultError::DatabaseError(other.to_string()),
    }
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
             WHERE i.deleted_at IS NULL
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

/// One item summary (same projection as get_all_items).
pub fn get_item_summary(conn: &Connection, id: &str) -> Result<Option<ItemSummary>> {
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
             WHERE i.id = ?1 AND i.deleted_at IS NULL",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let mut rows = stmt
        .query_map([id], |row| {
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
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    match rows.next() {
        Some(Ok(summary)) => Ok(Some(summary)),
        Some(Err(e)) => Err(VaultError::DatabaseError(e.to_string())),
        None => Ok(None),
    }
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
             WHERE i.group_id = ?1 AND i.deleted_at IS NULL
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

/// 按 query 过滤全部条目摘要（search_items 命令的数据层实现）。
///
/// 匹配字段：标题、列表副标题（与 [`get_all_items`] 的表达式一致），以及
/// 各类条目的用户名类字段（account.username / api_key.key_name / env_var.key /
/// database.username / ssh.username / cloud.access_key_id /
/// license.bound_email / smtp.username）。
///
/// - 空白 query 返回全部条目（与 `get_all_items` 相同）。
/// - query 中的 LIKE 通配符（`%`、`_`、`\`）会被转义为字面量，不参与匹配语义。
/// - SQLite 的 LIKE 仅对 ASCII 字符大小写不敏感（引擎默认行为）。
pub fn search_items(conn: &Connection, query: &str) -> Result<Vec<ItemSummary>> {
    let query = query.trim();
    if query.is_empty() {
        return get_all_items(conn);
    }

    // 转义 LIKE 通配符并包上 %..%，ESCAPE '\' 声明转义字符。
    let mut pattern = String::with_capacity(query.len() + 2);
    pattern.push('%');
    for c in query.chars() {
        if c == '\\' || c == '%' || c == '_' {
            pattern.push('\\');
        }
        pattern.push(c);
    }
    pattern.push('%');

    let mut stmt = conn
        .prepare(
            "SELECT id, title, subtitle, icon, type, is_favorite, group_id, created_at, updated_at
             FROM (
                 SELECT i.id AS id, i.title AS title,
                     COALESCE(a.username, ak.key_name, ev.key,
                         db.host || COALESCE(':' || db.port, ''),
                         s.username || '@' || s.host,
                         c.provider || ' · ' || c.access_key_id,
                         l.bound_email,
                         sm.host || COALESCE(':' || sm.port, ''),
                         '') AS subtitle,
                     COALESCE(a.username, ak.key_name, ev.key, db.username,
                         s.username, c.access_key_id, l.bound_email, sm.username,
                         '') AS username,
                     i.icon AS icon, i.type AS type, i.is_favorite AS is_favorite,
                     i.group_id AS group_id, i.created_at AS created_at,
                     i.updated_at AS updated_at
                 FROM items i
                 LEFT JOIN account_items a ON i.id = a.item_id
                 LEFT JOIN api_key_items ak ON i.id = ak.item_id
                 LEFT JOIN env_var_items ev ON i.id = ev.item_id AND ev.sort_order = 0
                 LEFT JOIN database_items db ON i.id = db.item_id
                 LEFT JOIN ssh_items s ON i.id = s.item_id
                 LEFT JOIN cloud_items c ON i.id = c.item_id
                 LEFT JOIN license_items l ON i.id = l.item_id
                 LEFT JOIN smtp_items sm ON i.id = sm.item_id
                 WHERE i.deleted_at IS NULL
             )
             WHERE title LIKE ?1 ESCAPE '\\'
                OR subtitle LIKE ?1 ESCAPE '\\'
                OR username LIKE ?1 ESCAPE '\\'
             ORDER BY title",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let items = stmt
        .query_map([&pattern], |row| {
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
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(item_header_missing)?;

    let account: (
        String,
        Vec<u8>,
        [u8; 12],
        Option<String>,
        Option<String>,
        Option<Vec<u8>>,
        Option<Vec<u8>>,
    ) = conn
        .query_row(
            "SELECT username, password_encrypted, password_nonce, website, notes,
                    totp_secret_encrypted, totp_secret_nonce
             FROM account_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(2)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    nonce,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let password = String::from_utf8(
        crate::crypto::decrypt(key, &account.2, &account.1)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    )
    .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in password".to_string()))?;

    let totp_secret = match (account.5, account.6) {
        (Some(ct), Some(nb)) => {
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&nb);
            Some(String::from_utf8(
                crate::crypto::decrypt(key, &nonce, &ct)
                    .map_err(|e| VaultError::CryptoError(e.to_string()))?,
            )
            .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in totp secret".to_string()))?)
        }
        _ => None,
    };

    let secret_history = load_secret_history(conn, id, key)?;

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
        totp_secret,
        secret_history,
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
             FROM items WHERE id = ?1 AND deleted_at IS NULL",
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
        .map_err(item_header_missing)?;

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
    let secret_history = load_secret_history(conn, id, key)?;


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
        secret_history,
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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, key_value.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    tx.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'api_key', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    tx.execute(
        "INSERT INTO api_key_items (item_id, key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, key_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), endpoint, auth_method, notes, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    log_event(tx, "item.create", Some(&id), Some(title), Some("api_key"));
    Ok(id)
    })
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
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();
    if key_value.is_some() {
        archive_secret(conn, "api_key_items", id, "key_value_encrypted", "key_value_nonce", "key_value")?;
    }

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

    log_event(conn, "item.update", Some(id), Some(title), Some("api_key"));
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
    totp_secret: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, password.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    tx.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'account', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let totp_encrypted = match totp_secret {
        Some(s) if !s.is_empty() => {
            let e = crate::crypto::encrypt(key, s.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?;
            Some((e.ciphertext.to_vec(), e.nonce.to_vec()))
        }
        _ => None,
    };

    tx.execute(
        "INSERT INTO account_items (item_id, username, password_encrypted, password_nonce, website, notes, totp_secret_encrypted, totp_secret_nonce)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            username,
            encrypted.ciphertext.to_vec(),
            encrypted.nonce.to_vec(),
            website,
            notes,
            totp_encrypted.as_ref().map(|e| e.0.clone()),
            totp_encrypted.as_ref().map(|e| e.1.clone()),
        ],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    log_event(tx, "item.create", Some(&id), Some(title), Some("account"));
    Ok(id)
    })
}

pub fn update_account_item(
    conn: &Connection,
    id: &str,
    title: &str,
    username: &str,
    password: Option<&str>,
    website: Option<&str>,
    notes: Option<&str>,
    totp_secret: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();

    if password.is_some() {
        archive_secret(conn, "account_items", id, "password_encrypted", "password_nonce", "password")?;
    }

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    // TOTP secret: None = keep current value, Some("") = clear, Some(s) = set.
    // Each fragment starts with a comma and carries no trailing comma so the
    // statement stays valid for all three cases (a trailing comma produced
    // "... NULL, WHERE ..." and broke every edit).
    let totp_update = match totp_secret {
        None => String::new(),
        Some("") => ", totp_secret_encrypted = NULL, totp_secret_nonce = NULL".to_string(),
        Some(s) => {
            let e = crate::crypto::encrypt(key, s.as_bytes())
                .map_err(|e| VaultError::CryptoError(e.to_string()))?;
            format!(
                ", totp_secret_encrypted = X'{}', totp_secret_nonce = X'{}'",
                hex_encode(&e.ciphertext),
                hex_encode(&e.nonce)
            )
        }
    };

    if let Some(pwd) = password {
        let encrypted = crate::crypto::encrypt(key, pwd.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            &format!(
                "UPDATE account_items SET username = ?1, password_encrypted = ?2, password_nonce = ?3, website = ?4, notes = ?5{totp_update} WHERE item_id = ?6"
            ),
            params![username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), website, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            &format!(
                "UPDATE account_items SET username = ?1, website = ?2, notes = ?3{totp_update} WHERE item_id = ?4"
            ),
            params![username, website, notes, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    log_event(conn, "item.update", Some(id), Some(title), Some("account"));
    Ok(())
}

/// Soft delete: the item is stamped with `deleted_at`, remembers its group in
/// `original_group_id` and is parked in the hidden trash group. Detail rows,
/// attachments, custom fields, tag links and secret history stay in place so
/// the item can be restored; only `purge_item` removes them for good.
///
/// Returns [`VaultError::ItemNotFound`] for an unknown id *and* for an item
/// that is already in the trash (a second delete is a no-op, not a re-stamp).
pub fn delete_item(conn: &Connection, id: &str) -> Result<()> {
    let title = item_title(conn, id);
    let now = chrono_timestamp();

    let affected = conn
        .execute(
            "UPDATE items
             SET deleted_at = ?1, original_group_id = group_id, group_id = ?2, updated_at = ?1
             WHERE id = ?3 AND deleted_at IS NULL",
            params![now, TRASH_GROUP_ID, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if affected == 0 {
        return Err(VaultError::ItemNotFound);
    }

    log_event(conn, "item.trash", Some(id), title.as_deref(), None);
    Ok(())
}

/// Reject trashed and unknown items for every mutation path that must not
/// touch the trash (updates, custom fields, tags, attachments).
pub(super) fn ensure_active_item(conn: &Connection, id: &str) -> Result<()> {
    let active: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if active == 0 {
        return Err(VaultError::ItemNotFound);
    }
    Ok(())
}

pub fn toggle_favorite(conn: &Connection, id: &str) -> Result<()> {
    let affected = conn
        .execute(
            "UPDATE items SET is_favorite = NOT is_favorite WHERE id = ?1 AND deleted_at IS NULL",
            [id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if affected == 0 {
        return Err(VaultError::ItemNotFound);
    }
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
             FROM items WHERE id = ?1 AND deleted_at IS NULL",
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
        .map_err(item_header_missing)?;

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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    tx.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'env_var', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    for (i, (var_key, var_value)) in variables.iter().enumerate() {
        let encrypted = crate::crypto::encrypt(key, var_value.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        tx.execute(
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

    log_event(tx, "item.create", Some(&id), Some(title), Some("env_var"));
    Ok(id)
    })
}

pub fn update_env_var_item(
    conn: &Connection,
    id: &str,
    title: &str,
    variables: &[(String, String)],
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    ensure_active_item(conn, id)?;
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

/// Archive the current ciphertext of a secret column into secret_history
/// before it gets overwritten. No-op when the column is NULL (nothing to
/// archive). The stored ciphertext is the old one under the current key,
/// so history stays readable after a master-password change (reencrypt_all
/// rotates history rows together with everything else).
fn archive_secret(
    conn: &Connection,
    table: &str,
    item_id: &str,
    enc_col: &str,
    nonce_col: &str,
    field: &str,
) -> Result<()> {
    conn.execute(
        &format!(
            "INSERT INTO secret_history (item_id, field, value_encrypted, value_nonce, changed_at)
             SELECT ?1, ?2, {enc_col}, {nonce_col}, strftime('%s','now')
             FROM {table}
             WHERE item_id = ?1 AND {enc_col} IS NOT NULL"
        ),
        params![item_id, field],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Load (decrypted) secret history for an item, newest first.
fn load_secret_history(conn: &Connection, item_id: &str, key: &[u8; 32]) -> Result<Vec<SecretHistoryEntry>> {
    let mut stmt = conn
        .prepare(
            "SELECT field, value_encrypted, value_nonce, changed_at FROM secret_history
             WHERE item_id = ?1 ORDER BY changed_at DESC, id DESC LIMIT 20",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let rows: Vec<(String, Vec<u8>, [u8; 12], i64)> = stmt
        .query_map([item_id], |row| {
            let nonce_bytes: Vec<u8> = row.get(2)?;
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&nonce_bytes);
            Ok((row.get(0)?, row.get(1)?, nonce, row.get(3)?))
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    let mut out = Vec::with_capacity(rows.len());
    for (field, ct, nonce, changed_at) in rows {
        let value = String::from_utf8(
            crate::crypto::decrypt(key, &nonce, &ct)
                .map_err(|e| VaultError::CryptoError(e.to_string()))?,
        )
        .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in history value".to_string()))?;
        out.push(SecretHistoryEntry { field, value, changed_at });
    }
    Ok(out)
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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    tx.execute(
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

    tx.execute(
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

    log_event(tx, "item.create", Some(&id), Some(title), Some("database"));
    Ok(id)
    })
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
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();
    if password.is_some() {
        archive_secret(conn, "database_items", id, "password_encrypted", "password_nonce", "password")?;
    }

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

    log_event(conn, "item.update", Some(id), Some(title), Some("database"));
    Ok(())
}

pub fn get_database_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<DatabaseItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(item_header_missing)?;

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
    let secret_history = load_secret_history(conn, id, key)?;


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
        secret_history,
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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    tx.execute(
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

    tx.execute(
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

    log_event(tx, "item.create", Some(&id), Some(title), Some("ssh"));
    Ok(id)
    })
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
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();
    if password.is_some() {
        archive_secret(conn, "ssh_items", id, "password_encrypted", "password_nonce", "password")?;
    }
    if passphrase.is_some() {
        archive_secret(conn, "ssh_items", id, "passphrase_encrypted", "passphrase_nonce", "passphrase")?;
    }

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

    log_event(conn, "item.update", Some(id), Some(title), Some("ssh"));
    Ok(())
}

pub fn get_ssh_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<SshItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(item_header_missing)?;

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
    let secret_history = load_secret_history(conn, id, key)?;


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
        secret_history,
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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, secret.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    tx.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'cloud', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    tx.execute(
        "INSERT INTO cloud_items (item_id, provider, access_key_id, secret_encrypted, secret_nonce, region, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, provider, access_key_id, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), region, notes],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    log_event(tx, "item.create", Some(&id), Some(title), Some("cloud"));
    Ok(id)
    })
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
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();
    if secret.is_some() {
        archive_secret(conn, "cloud_items", id, "secret_encrypted", "secret_nonce", "secret")?;
    }

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

    log_event(conn, "item.update", Some(id), Some(title), Some("cloud"));
    Ok(())
}

pub fn get_cloud_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<CloudItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(item_header_missing)?;

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
    let secret_history = load_secret_history(conn, id, key)?;


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
        secret_history,
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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, license_key.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    tx.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'license', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    tx.execute(
        "INSERT INTO license_items (item_id, software_name, license_key_encrypted, license_key_nonce, bound_email, expiry_date, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, software_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), bound_email, expiry_date, notes],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    log_event(tx, "item.create", Some(&id), Some(title), Some("license"));
    Ok(id)
    })
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
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();
    if license_key.is_some() {
        archive_secret(conn, "license_items", id, "license_key_encrypted", "license_key_nonce", "license_key")?;
    }

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

    log_event(conn, "item.update", Some(id), Some(title), Some("license"));
    Ok(())
}

pub fn get_license_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<LicenseItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(item_header_missing)?;

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
    let secret_history = load_secret_history(conn, id, key)?;


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
        secret_history,
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
    ensure_group_can_hold_items(group_id)?;
    with_tx(conn, |tx| {
        let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, password.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    tx.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
         VALUES (?1, ?2, 'smtp', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    tx.execute(
        "INSERT INTO smtp_items (item_id, host, port, encryption, username, password_encrypted, password_nonce, from_address, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, host, port, encryption, username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), from_address, notes],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    log_event(tx, "item.create", Some(&id), Some(title), Some("smtp"));
    Ok(id)
    })
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
    ensure_active_item(conn, id)?;
    let now = chrono_timestamp();
    if password.is_some() {
        archive_secret(conn, "smtp_items", id, "password_encrypted", "password_nonce", "password")?;
    }

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

    log_event(conn, "item.update", Some(id), Some(title), Some("smtp"));
    Ok(())
}

pub fn get_smtp_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<SmtpItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(item_header_missing)?;

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
    let secret_history = load_secret_history(conn, id, key)?;


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
        secret_history,
    })
}
