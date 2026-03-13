use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{get_all_groups, get_all_items, DbConnection, Group};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: i64,
    pub app: String,
    pub groups: Vec<Group>,
    pub items: Vec<ExportItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportItem {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub encrypted_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub groups_imported: usize,
    pub items_imported: usize,
    pub groups_skipped: usize,
    pub items_skipped: usize,
}

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tauri::command]
pub fn export_vault(db: State<DbConnection>, vault_state: State<VaultState>) -> Result<String> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let groups = get_all_groups(&conn)?;
    let item_summaries = get_all_items(&conn)?;

    let mut items = Vec::new();
    for summary in item_summaries {
        let encrypted_data = match summary.item_type.as_str() {
            "account" => {
                let row: (String, Vec<u8>, Vec<u8>, Option<String>, Option<String>) = conn
                    .query_row(
                        "SELECT username, password_encrypted, password_nonce, website, notes 
                         FROM account_items WHERE item_id = ?1",
                        [&summary.id],
                        |row| {
                            Ok((
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                            ))
                        },
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

                serde_json::json!({
                    "username": row.0,
                    "password_encrypted": STANDARD.encode(&row.1),
                    "password_nonce": STANDARD.encode(&row.2),
                    "website": row.3,
                    "notes": row.4,
                })
            }
            "api_key" => {
                let row: (
                    String,
                    Vec<u8>,
                    Vec<u8>,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                    Option<i64>,
                ) = conn
                    .query_row(
                        "SELECT key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date 
                         FROM api_key_items WHERE item_id = ?1",
                        [&summary.id],
                        |row| {
                            Ok((
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                                row.get(5)?,
                                row.get(6)?,
                            ))
                        },
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

                serde_json::json!({
                    "key_name": row.0,
                    "key_value_encrypted": STANDARD.encode(&row.1),
                    "key_value_nonce": STANDARD.encode(&row.2),
                    "endpoint": row.3,
                    "auth_method": row.4,
                    "notes": row.5,
                    "rotation_date": row.6,
                })
            }
            "env_var" => {
                let mut stmt = conn
                    .prepare(
                        "SELECT key, value_encrypted, value_nonce FROM env_var_items WHERE item_id = ?1 ORDER BY sort_order",
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

                let variables: Vec<serde_json::Value> = stmt
                    .query_map([&summary.id], |row| {
                        Ok(serde_json::json!({
                            "key": row.get::<_, String>(0)?,
                            "value_encrypted": STANDARD.encode(row.get::<_, Vec<u8>>(1)?),
                            "value_nonce": STANDARD.encode(row.get::<_, Vec<u8>>(2)?),
                        }))
                    })
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

                let notes: Option<String> = conn
                    .query_row(
                        "SELECT notes FROM env_var_items WHERE item_id = ?1 LIMIT 1",
                        [&summary.id],
                        |row| row.get(0),
                    )
                    .ok();

                serde_json::json!({
                    "variables": variables,
                    "notes": notes,
                })
            }
            _ => continue,
        };

        items.push(ExportItem {
            id: summary.id,
            group_id: summary.group_id,
            title: summary.title,
            icon: summary.icon,
            item_type: summary.item_type,
            is_favorite: summary.is_favorite,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
            encrypted_data,
        });
    }

    let export = ExportData {
        version: "1.0".to_string(),
        exported_at: chrono_timestamp(),
        app: "SecKeyBox".to_string(),
        groups,
        items,
    };

    serde_json::to_string_pretty(&export)
        .map_err(|e| VaultError::DatabaseError(format!("Failed to serialize export: {}", e)))
}

#[tauri::command]
pub fn import_vault(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    data: String,
    mode: String,
) -> Result<ImportResult> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    let export: ExportData = serde_json::from_str(&data)
        .map_err(|e| VaultError::DatabaseError(format!("Invalid export format: {}", e)))?;

    if export.app != "SecKeyBox" {
        return Err(VaultError::DatabaseError(
            "Not a SecKeyBox export file".to_string(),
        ));
    }

    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    if mode == "replace" {
        conn.execute("DELETE FROM items", [])
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        conn.execute("DELETE FROM groups WHERE id NOT LIKE 'built-in-%'", [])
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    let mut result = ImportResult {
        groups_imported: 0,
        items_imported: 0,
        groups_skipped: 0,
        items_skipped: 0,
    };

    let existing_groups: Vec<String> = conn
        .prepare("SELECT id FROM groups")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| row.get(0))
                .and_then(|ids| ids.collect())
        })
        .unwrap_or_default();

    for group in export.groups {
        if existing_groups.contains(&group.id) && mode == "merge" {
            result.groups_skipped += 1;
            continue;
        }

        conn.execute(
            "INSERT OR REPLACE INTO groups (id, name, icon, parent_id, sort_order, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                group.id,
                group.name,
                group.icon,
                group.parent_id,
                group.sort_order,
                group.created_at,
                group.updated_at
            ],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

        result.groups_imported += 1;
    }

    let existing_items: Vec<String> = conn
        .prepare("SELECT id FROM items")
        .and_then(|mut stmt| {
            stmt.query_map([], |row| row.get(0))
                .and_then(|ids| ids.collect())
        })
        .unwrap_or_default();

    for item in export.items {
        if existing_items.contains(&item.id) && mode == "merge" {
            result.items_skipped += 1;
            continue;
        }

        conn.execute(
            "INSERT OR REPLACE INTO items (id, group_id, title, icon, type, is_favorite, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                item.id,
                item.group_id,
                item.title,
                item.icon,
                item.item_type,
                item.is_favorite as i32,
                item.created_at,
                item.updated_at
            ],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

        match item.item_type.as_str() {
            "account" => {
                let username = item.encrypted_data["username"].as_str().unwrap_or("");
                let password_encrypted = STANDARD
                    .decode(
                        item.encrypted_data["password_encrypted"]
                            .as_str()
                            .unwrap_or(""),
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
                let password_nonce = STANDARD
                    .decode(item.encrypted_data["password_nonce"].as_str().unwrap_or(""))
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
                let website = item.encrypted_data["website"].as_str();
                let notes = item.encrypted_data["notes"].as_str();

                conn.execute(
                    "INSERT OR REPLACE INTO account_items (item_id, username, password_encrypted, password_nonce, website, notes) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    rusqlite::params![
                        item.id, username, password_encrypted, password_nonce, website, notes
                    ],
                )
                .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
            }
            "api_key" => {
                let key_name = item.encrypted_data["key_name"].as_str().unwrap_or("");
                let key_value_encrypted = STANDARD
                    .decode(
                        item.encrypted_data["key_value_encrypted"]
                            .as_str()
                            .unwrap_or(""),
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
                let key_value_nonce = STANDARD
                    .decode(
                        item.encrypted_data["key_value_nonce"]
                            .as_str()
                            .unwrap_or(""),
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
                let endpoint = item.encrypted_data["endpoint"].as_str();
                let auth_method = item.encrypted_data["auth_method"].as_str();
                let notes = item.encrypted_data["notes"].as_str();
                let rotation_date = item.encrypted_data["rotation_date"].as_i64();

                conn.execute(
                    "INSERT OR REPLACE INTO api_key_items (item_id, key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        item.id,
                        key_name,
                        key_value_encrypted,
                        key_value_nonce,
                        endpoint,
                        auth_method,
                        notes,
                        rotation_date
                    ],
                )
                .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
            }
            "env_var" => {
                let variables = item.encrypted_data["variables"].as_array().ok_or_else(|| {
                    VaultError::DatabaseError("Invalid env_var format".to_string())
                })?;
                let notes = item.encrypted_data["notes"].as_str();

                for (sort_order, var) in variables.iter().enumerate() {
                    let key = var["key"].as_str().unwrap_or("");
                    let value_encrypted = STANDARD
                        .decode(var["value_encrypted"].as_str().unwrap_or(""))
                        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
                    let value_nonce = STANDARD
                        .decode(var["value_nonce"].as_str().unwrap_or(""))
                        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

                    conn.execute(
                        "INSERT INTO env_var_items (item_id, key, value_encrypted, value_nonce, sort_order, notes) 
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![
                            item.id,
                            key,
                            value_encrypted,
                            value_nonce,
                            sort_order as i32,
                            notes
                        ],
                    )
                    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
                }
            }
            _ => {}
        }

        result.items_imported += 1;
    }

    Ok(result)
}
