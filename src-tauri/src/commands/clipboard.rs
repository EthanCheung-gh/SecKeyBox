use tauri_plugin_clipboard_manager::ClipboardExt;
use crate::error::{Result, VaultError};

#[tauri::command]
pub async fn copy_to_clipboard(app_handle: tauri::AppHandle, text: String) -> Result<()> {
    app_handle
        .clipboard()
        .write_text(&text)
        .map_err(|e| VaultError::DatabaseError(format!("Clipboard error: {}", e)))?;
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