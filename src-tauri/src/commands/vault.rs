use tauri::State;

use crate::crypto::{derive_key, verify_password, SecureKey};
use crate::db::{
    get_db_path, get_password_hash, get_password_salt, init_schema, insert_builtin_groups,
    log_event, open_connection, reencrypt_all, run_migrations, set_password_hash, DbConnection,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

#[tauri::command]
pub fn is_vault_initialized(app_handle: tauri::AppHandle, db: State<DbConnection>) -> Result<bool> {
    let db_path = get_db_path(&app_handle);

    if !db_path.exists() {
        return Ok(false);
    }

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

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

    {
        let mut db_conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        *db_conn = conn;
    }

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<bool> {
    let conn = db.inner().0.lock().map_err(|e| {
        eprintln!("[ERROR] unlock_vault: failed to acquire db lock: {:?}", e);
        VaultError::DatabaseError("Lock poisoned".to_string())
    })?;

    if let Err(e) = run_migrations(&conn) {
        eprintln!("[ERROR] unlock_vault: migrations failed: {:?}", e);
        return Err(e);
    }

    let stored_hash = get_password_hash(&conn)?;
    let stored_salt = get_password_salt(&conn)?;

    let stored_hash = stored_hash.ok_or(VaultError::VaultNotInitialized)?;
    let stored_salt = stored_salt.ok_or(VaultError::VaultNotInitialized)?;

    if !verify_password(&master_password, &stored_hash)? {
        return Err(VaultError::InvalidPassword);
    }

    let derived = derive_key(&master_password, Some(&stored_salt))?;
    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);
    log_event(&conn, "vault.unlock", None, None, None);
    // One snapshot per day, produced after the vault is readable.
    crate::commands::backup::ensure_daily_backup(&app_handle, &conn);

    // Drop trash entries past the retention window. Runs after the backup so
    // the snapshot still contains them; a failure must never block unlocking.
    match crate::db::purge_expired_trash(&conn) {
        Ok(0) => {}
        Ok(purged) => eprintln!("[INFO] unlock_vault: purged {purged} expired trash item(s)"),
        Err(e) => eprintln!("[WARN] unlock_vault: trash auto-purge failed: {:?}", e),
    }

    Ok(true)
}

#[tauri::command]
pub fn lock_vault(db: State<DbConnection>, vault_state: State<VaultState>) -> Result<()> {
    if let Ok(conn) = db.inner().0.lock() {
        log_event(&conn, "vault.lock", None, None, None);
    }
    vault_state.lock();
    Ok(())
}

#[tauri::command]
pub fn is_vault_unlocked(vault_state: State<VaultState>) -> bool {
    vault_state.is_unlocked()
}

/// Change the master password: verify the current one, derive a new key with
/// a fresh salt and re-encrypt every sensitive field, then swap the stored
/// hash/salt and the in-memory key. The re-encryption runs in one
/// transaction, so a failure leaves the vault fully readable with the old
/// password.
#[tauri::command]
pub fn change_master_password(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    current_password: String,
    new_password: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    if new_password.len() < 8 {
        return Err(VaultError::DatabaseError(
            "New password must be at least 8 characters".to_string(),
        ));
    }

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let stored_hash = get_password_hash(&conn)?.ok_or(VaultError::VaultNotInitialized)?;
    if !verify_password(&current_password, &stored_hash)? {
        return Err(VaultError::InvalidPassword);
    }

    let old_key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;

    // Fresh salt for the new password.
    let derived = derive_key(&new_password, None)?;

    reencrypt_all(&conn, old_key.as_bytes(), &derived.key)?;
    set_password_hash(&conn, &derived.hash, &derived.salt)?;
    log_event(&conn, "vault.password_changed", None, None, None);

    vault_state.unlock(SecureKey::new(derived.key));
    Ok(())
}
