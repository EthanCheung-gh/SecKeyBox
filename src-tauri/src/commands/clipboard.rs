use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::db::{log_event, DbConnection};
use crate::error::{Result, VaultError};

/// Copy a value to the clipboard. When the caller knows which item and field
/// the value belongs to, an activity entry is recorded (never the value).
#[tauri::command]
pub async fn copy_to_clipboard(
    app_handle: tauri::AppHandle,
    db: State<'_, DbConnection>,
    text: String,
    item_id: Option<String>,
    item_title: Option<String>,
    field: Option<String>,
) -> Result<()> {
    app_handle
        .clipboard()
        .write_text(&text)
        .map_err(|e| VaultError::DatabaseError(format!("Clipboard error: {}", e)))?;

    if let Some(id) = item_id {
        if let Ok(conn) = db.inner().0.lock() {
            log_event(&conn, "item.copy", Some(&id), item_title.as_deref(), field.as_deref());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard(app_handle: tauri::AppHandle) -> Result<()> {
    app_handle
        .clipboard()
        .write_text("")
        .map_err(|e| VaultError::DatabaseError(format!("Clipboard error: {}", e)))?;
    Ok(())
}