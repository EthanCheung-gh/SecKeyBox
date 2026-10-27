//! Custom fields and tags: per-item metadata that is not part of any
//! credential template. Field values are encrypted; tag names are not.

use serde::Deserialize;
use tauri::State;

use crate::db::{
    create_tag, delete_tag, get_all_tags, get_item_tag_ids, get_items_by_tag,
    list_custom_fields, save_custom_fields, set_item_tags, CustomField, DbConnection, ItemSummary,
    Tag,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

#[derive(Debug, Deserialize)]
pub struct CustomFieldInput {
    pub key: String,
    pub value: String,
}

fn require_key(vault_state: &VaultState) -> Result<crate::crypto::SecureKey> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    vault_state.get_master_key().ok_or(VaultError::VaultLocked)
}

#[tauri::command]
pub fn list_custom_fields_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
) -> Result<Vec<CustomField>> {
    let key = require_key(&vault_state)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    list_custom_fields(&conn, &item_id, key.as_bytes())
}

#[tauri::command]
pub fn save_custom_fields_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
    fields: Vec<CustomFieldInput>,
) -> Result<()> {
    let key = require_key(&vault_state)?;
    let pairs: Vec<(String, String)> = fields
        .into_iter()
        .map(|field| (field.key, field.value))
        .collect();
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    save_custom_fields(&conn, &item_id, &pairs, key.as_bytes())
}

// ---------- tags ----------

#[tauri::command]
pub fn get_tags(db: State<DbConnection>, vault_state: State<VaultState>) -> Result<Vec<Tag>> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_tags(&conn)
}

#[tauri::command]
pub fn create_new_tag(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    name: String,
    color: Option<String>,
) -> Result<Tag> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_tag(&conn, &name, color.as_deref())
}

#[tauri::command]
pub fn delete_existing_tag(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_tag(&conn, &id)
}

#[tauri::command]
pub fn get_item_tags(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
) -> Result<Vec<String>> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_item_tag_ids(&conn, &item_id)
}

#[tauri::command]
pub fn set_item_tags_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
    tag_ids: Vec<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    set_item_tags(&conn, &item_id, &tag_ids)
}

#[tauri::command]
pub fn get_items_by_tag_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    tag_id: String,
) -> Result<Vec<ItemSummary>> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_items_by_tag(&conn, &tag_id)
}
