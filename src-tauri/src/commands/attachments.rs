use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use tauri::State;

use crate::db::{
    add_attachment, delete_attachment, get_attachment_data, list_attachments, log_event,
    item_title, AttachmentMeta, DbConnection,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

#[derive(Debug, Serialize)]
pub struct AttachmentData {
    pub filename: String,
    pub content_type: Option<String>,
    pub data_base64: String,
}

/// Attachments are only available while the vault is unlocked: their contents
/// are encrypted with the in-memory master key.
#[tauri::command]
pub fn add_attachment_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
    filename: String,
    content_type: Option<String>,
    data_base64: String,
) -> Result<AttachmentMeta> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let data = BASE64
        .decode(data_base64.as_bytes())
        .map_err(|e| VaultError::Validation(format!("Invalid attachment payload: {}", e)))?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let meta = add_attachment(
        &conn,
        &item_id,
        &filename,
        content_type.as_deref(),
        &data,
        key.as_bytes(),
    )?;
    log_event(
        &conn,
        "attachment.add",
        Some(&item_id),
        item_title(&conn, &item_id).as_deref(),
        Some(&filename),
    );
    Ok(meta)
}

#[tauri::command]
pub fn list_attachments_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
) -> Result<Vec<AttachmentMeta>> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    list_attachments(&conn, &item_id)
}

#[tauri::command]
pub fn get_attachment_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
    attachment_id: String,
) -> Result<AttachmentData> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    let (meta, data) = get_attachment_data(&conn, &item_id, &attachment_id, key.as_bytes())?;
    Ok(AttachmentData {
        filename: meta.filename,
        content_type: meta.content_type,
        data_base64: BASE64.encode(&data),
    })
}

/// Write a decrypted attachment straight to disk, so large files never need
/// a base64 round-trip through the webview.
#[tauri::command]
pub fn save_attachment_to_path(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
    attachment_id: String,
    path: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let data = {
        let conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        get_attachment_data(&conn, &item_id, &attachment_id, key.as_bytes())?.1
    };
    std::fs::write(&path, data)
        .map_err(|e| VaultError::Validation(format!("Failed to write {}: {}", path, e)))?;
    Ok(())
}

#[tauri::command]
pub fn delete_attachment_cmd(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    item_id: String,
    attachment_id: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_attachment(&conn, &item_id, &attachment_id)?;
    log_event(
        &conn,
        "attachment.delete",
        Some(&item_id),
        item_title(&conn, &item_id).as_deref(),
        None,
    );
    Ok(())
}
