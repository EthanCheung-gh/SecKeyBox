use tauri::State;

use crate::crypto::{derive_key, verify_password, SecureKey};
use crate::db::{
    get_db_path, get_password_hash, get_password_salt, init_schema, insert_builtin_groups,
    open_connection, run_migrations, set_password_hash, DbConnection,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

#[tauri::command]
pub fn is_vault_initialized(app_handle: tauri::AppHandle, db: State<DbConnection>) -> Result<bool> {
    let db_path = get_db_path(&app_handle);
    eprintln!("[DEBUG] is_vault_initialized: db_path = {:?}", db_path);
    eprintln!(
        "[DEBUG] is_vault_initialized: exists = {}",
        db_path.exists()
    );

    if !db_path.exists() {
        return Ok(false);
    }

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let result = crate::db::is_vault_initialized(&conn);
    eprintln!("[DEBUG] is_vault_initialized: result = {:?}", result);
    result
}

#[tauri::command]
pub fn initialize_vault(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<()> {
    eprintln!("[DEBUG] initialize_vault: called");

    if master_password.len() < 8 {
        return Err(VaultError::InvalidPassword);
    }

    let db_path = get_db_path(&app_handle);
    eprintln!("[DEBUG] initialize_vault: db_path = {:?}", db_path);

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        eprintln!("[DEBUG] initialize_vault: created parent dir");
    }

    let conn = open_connection(&db_path)?;
    eprintln!(
        "[DEBUG] initialize_vault: opened connection, file exists = {}",
        db_path.exists()
    );

    init_schema(&conn)?;
    run_migrations(&conn)?;
    insert_builtin_groups(&conn)?;
    eprintln!("[DEBUG] initialize_vault: schema initialized");

    if crate::db::is_vault_initialized(&conn)? {
        eprintln!("[DEBUG] initialize_vault: already initialized!");
        return Err(VaultError::VaultAlreadyInitialized);
    }

    let derived = derive_key(&master_password, None)?;
    eprintln!("[DEBUG] initialize_vault: derived hash = {}", derived.hash);
    eprintln!("[DEBUG] initialize_vault: derived salt = {}", derived.salt);
    set_password_hash(&conn, &derived.hash, &derived.salt)?;
    eprintln!("[DEBUG] initialize_vault: password hash set");

    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);

    {
        let mut db_conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        *db_conn = conn;
    }

    eprintln!(
        "[DEBUG] initialize_vault: success, db file exists = {}",
        db_path.exists()
    );
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<bool> {
    eprintln!(
        "[DEBUG] unlock_vault: called, password length = {}",
        master_password.len()
    );

    let conn = db.inner().0.lock().map_err(|e| {
        eprintln!("[DEBUG] unlock_vault: lock error = {:?}", e);
        VaultError::DatabaseError("Lock poisoned".to_string())
    })?;

    eprintln!("[DEBUG] unlock_vault: got db lock, running migrations");

    // Debug: check what's in vault_config
    let mut stmt = conn
        .prepare("SELECT key, value FROM vault_config")
        .map_err(|e| {
            eprintln!("[DEBUG] unlock_vault: prepare error = {:?}", e);
            VaultError::DatabaseError(e.to_string())
        })?;
    let configs: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| {
            eprintln!("[DEBUG] unlock_vault: query error = {:?}", e);
            VaultError::DatabaseError(e.to_string())
        })?
        .filter_map(|r| r.ok())
        .collect();
    eprintln!("[DEBUG] unlock_vault: vault_config = {:?}", configs);

    if let Err(e) = run_migrations(&conn) {
        eprintln!("[DEBUG] unlock_vault: migrations error = {:?}", e);
        return Err(e);
    }
    eprintln!("[DEBUG] unlock_vault: migrations done");

    let stored_hash = get_password_hash(&conn)?;
    eprintln!("[DEBUG] unlock_vault: stored_hash = {:?}", stored_hash);

    let stored_salt = get_password_salt(&conn)?;
    eprintln!("[DEBUG] unlock_vault: stored_salt = {:?}", stored_salt);

    let stored_hash = stored_hash.ok_or(VaultError::VaultNotInitialized)?;
    let stored_salt = stored_salt.ok_or(VaultError::VaultNotInitialized)?;

    eprintln!(
        "[DEBUG] unlock_vault: verifying password with hash len={}",
        stored_hash.len()
    );

    let verify_result = verify_password(&master_password, &stored_hash);
    eprintln!("[DEBUG] unlock_vault: verify_result = {:?}", verify_result);

    if !verify_result? {
        eprintln!("[DEBUG] unlock_vault: password verification failed");
        return Err(VaultError::InvalidPassword);
    }

    eprintln!("[DEBUG] unlock_vault: password verified, deriving key...");

    let derived = derive_key(&master_password, Some(&stored_salt))?;
    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);

    eprintln!("[DEBUG] unlock_vault: success");
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
