use tauri::State;
use crate::state::VaultState;
use crate::db::{DbConnection, get_all_items, get_items_by_group, get_account_item_detail, create_account_item, update_account_item, delete_item, toggle_favorite, ItemSummary, AccountItemDetail};
use crate::error::{Result, VaultError};

#[tauri::command]
pub fn get_all_items_cmd(db: State<DbConnection>) -> Result<Vec<ItemSummary>> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_items(&conn)
}

#[tauri::command]
pub fn get_items_by_group_cmd(db: State<DbConnection>, group_id: String) -> Result<Vec<ItemSummary>> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_items_by_group(&conn, &group_id)
}

#[tauri::command]
pub fn get_item_detail(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<AccountItemDetail> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    get_account_item_detail(&conn, &id, key.as_bytes())
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
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError("Username must be 1-100 characters".to_string()));
    }
    if password.is_empty() || password.len() > 1000 {
        return Err(VaultError::DatabaseError("Password must be 1-1000 characters".to_string()));
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
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
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError("Username must be 1-100 characters".to_string()));
    }
    if let Some(ref pwd) = password {
        if pwd.is_empty() || pwd.len() > 1000 {
            return Err(VaultError::DatabaseError("Password must be 1-1000 characters".to_string()));
        }
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
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
pub fn delete_existing_item(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_item(&conn, &id)
}

#[tauri::command]
pub fn toggle_item_favorite(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    toggle_favorite(&conn, &id)
}