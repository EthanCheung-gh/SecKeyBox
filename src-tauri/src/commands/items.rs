use crate::db::{
    create_account_item, create_api_key_item, delete_item, get_account_item_detail, get_all_items,
    get_api_key_item_detail, get_items_by_group, toggle_favorite, update_account_item,
    update_api_key_item, DbConnection, ItemDetail, ItemSummary,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;
use tauri::State;

#[tauri::command]
pub fn get_all_items_cmd(db: State<DbConnection>) -> Result<Vec<ItemSummary>> {
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_items(&conn)
}

#[tauri::command]
pub fn get_items_by_group_cmd(
    db: State<DbConnection>,
    group_id: String,
) -> Result<Vec<ItemSummary>> {
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_items_by_group(&conn, &group_id)
}

#[tauri::command]
pub fn get_item_detail(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<ItemDetail> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let item_type: String = conn
        .query_row("SELECT type FROM items WHERE id = ?1", [&id], |row| {
            row.get(0)
        })
        .map_err(|_| VaultError::ItemNotFound)?;

    match item_type.as_str() {
        "account" => {
            get_account_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::Account(d))
        }
        "api_key" => {
            get_api_key_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::ApiKey(d))
        }
        _ => Err(VaultError::ItemNotFound),
    }
}

#[tauri::command]
pub fn create_new_account_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    username: String,
    password: String,
    website: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Username must be 1-100 characters".to_string(),
        ));
    }
    if password.is_empty() || password.len() > 1000 {
        return Err(VaultError::DatabaseError(
            "Password must be 1-1000 characters".to_string(),
        ));
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    create_account_item(
        &conn,
        &group_id,
        &title,
        &username,
        &password,
        website.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_account_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    username: String,
    password: Option<String>,
    website: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Username must be 1-100 characters".to_string(),
        ));
    }
    if let Some(ref pwd) = password {
        if pwd.is_empty() || pwd.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be 1-1000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    update_account_item(
        &conn,
        &id,
        &title,
        &username,
        password.as_deref(),
        website.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn create_new_api_key_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    key_name: String,
    key_value: String,
    endpoint: Option<String>,
    auth_method: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if key_name.is_empty() || key_name.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Key name must be 1-100 characters".to_string(),
        ));
    }
    if key_value.is_empty() || key_value.len() > 50000 {
        return Err(VaultError::DatabaseError(
            "Key value must be 1-50000 characters".to_string(),
        ));
    }
    if let Some(ref e) = endpoint {
        if e.len() > 500 {
            return Err(VaultError::DatabaseError(
                "Endpoint must be under 500 characters".to_string(),
            ));
        }
    }
    if let Some(ref am) = auth_method {
        if am.len() > 20 {
            return Err(VaultError::DatabaseError(
                "Auth method must be under 20 characters".to_string(),
            ));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Notes must be under 5000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    create_api_key_item(
        &conn,
        &group_id,
        &title,
        &key_name,
        &key_value,
        endpoint.as_deref(),
        auth_method.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_api_key_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    key_name: String,
    key_value: Option<String>,
    endpoint: Option<String>,
    auth_method: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if key_name.is_empty() || key_name.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Key name must be 1-100 characters".to_string(),
        ));
    }
    if let Some(ref kv) = key_value {
        if kv.is_empty() || kv.len() > 50000 {
            return Err(VaultError::DatabaseError(
                "Key value must be 1-50000 characters".to_string(),
            ));
        }
    }
    if let Some(ref e) = endpoint {
        if e.len() > 500 {
            return Err(VaultError::DatabaseError(
                "Endpoint must be under 500 characters".to_string(),
            ));
        }
    }
    if let Some(ref am) = auth_method {
        if am.len() > 20 {
            return Err(VaultError::DatabaseError(
                "Auth method must be under 20 characters".to_string(),
            ));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Notes must be under 5000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    update_api_key_item(
        &conn,
        &id,
        &title,
        &key_name,
        key_value.as_deref(),
        endpoint.as_deref(),
        auth_method.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn delete_existing_item(db: State<DbConnection>, id: String) -> Result<()> {
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_item(&conn, &id)
}

#[tauri::command]
pub fn toggle_item_favorite(db: State<DbConnection>, id: String) -> Result<()> {
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    toggle_favorite(&conn, &id)
}
