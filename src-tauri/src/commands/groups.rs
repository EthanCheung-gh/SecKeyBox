use tauri::State;
use crate::db::{DbConnection, get_all_groups, create_group, update_group, delete_group, Group};
use crate::error::{Result, VaultError};

#[tauri::command]
pub fn get_groups(db: State<DbConnection>) -> Result<Vec<Group>> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_groups(&conn)
}

#[tauri::command]
pub fn create_new_group(
    db: State<DbConnection>,
    name: String,
    icon: Option<String>,
    parent_id: Option<String>,
    sort_order: Option<i32>,
) -> Result<Group> {
    if name.is_empty() || name.len() > 50 {
        return Err(VaultError::DatabaseError("Group name must be 1-50 characters".to_string()));
    }

    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_group(&conn, &name, icon.as_deref(), parent_id.as_deref(), sort_order)
}

#[tauri::command]
pub fn update_existing_group(
    db: State<DbConnection>,
    id: String,
    name: String,
    icon: Option<String>,
    sort_order: Option<i32>,
) -> Result<()> {
    if name.is_empty() || name.len() > 50 {
        return Err(VaultError::DatabaseError("Group name must be 1-50 characters".to_string()));
    }

    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_group(&conn, &id, &name, icon.as_deref(), sort_order)
}

#[tauri::command]
pub fn delete_existing_group(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_group(&conn, &id)
}