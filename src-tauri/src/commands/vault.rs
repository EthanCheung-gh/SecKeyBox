use tauri::State;

use crate::crypto::{derive_key, verify_password, SecureKey};
use crate::db::{
    get_db_path, get_password_hash, get_password_salt, init_schema, insert_builtin_groups,
    open_connection, run_migrations, set_password_hash, DbConnection,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

#[tauri::command]
pub fn is_vault_initialized(app_handle: tauri::AppHandle) -> Result<bool> {
    let db_path = get_db_path(&app_handle);

    if !db_path.exists() {
        return Ok(false);
    }

    let conn = open_connection(&db_path)?;
    crate::db::is_vault_initialized(&conn)
}

#[tauri::command]
pub fn initialize_vault(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<()> {
    if master_password.len() < 8 {
        return Err(VaultError::InvalidPassword);
    }

    let db_path = get_db_path(&app_handle);

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    let conn = open_connection(&db_path)?;
    init_schema(&conn)?;
    run_migrations(&conn)?;
    insert_builtin_groups(&conn)?;

    if crate::db::is_vault_initialized(&conn)? {
        return Err(VaultError::VaultAlreadyInitialized);
    }

    let derived = derive_key(&master_password, None)?;
    set_password_hash(&conn, &derived.hash, &derived.salt)?;

    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);

    let mut db_conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    *db_conn = conn;

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<bool> {
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    run_migrations(&conn)?;

    let stored_hash = get_password_hash(&conn)?.ok_or(VaultError::VaultNotInitialized)?;
    let stored_salt = get_password_salt(&conn)?.ok_or(VaultError::VaultNotInitialized)?;

    if !verify_password(&master_password, &stored_hash)? {
        return Err(VaultError::InvalidPassword);
    }

    let derived = derive_key(&master_password, Some(&stored_salt))?;
    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);

    Ok(true)
}

#[tauri::command]
pub fn lock_vault(vault_state: State<VaultState>) -> Result<()> {
    vault_state.lock();
    Ok(())
}

#[tauri::command]
pub fn is_vault_unlocked(vault_state: State<VaultState>) -> bool {
    vault_state.is_unlocked()
}
