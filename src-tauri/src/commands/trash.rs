//! Trash commands: list / count / restore / purge / empty.
//!
//! Every command requires an unlocked vault. Listing and counting are pure
//! metadata reads; restoring and purging need the same guard so a locked vault
//! cannot be mutated through the trash API.

use tauri::State;

use crate::db::{self, DbConnection, TrashItemSummary};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

fn lock_db<'a>(
    db: &'a State<DbConnection>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>> {
    db.inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))
}

#[tauri::command]
pub fn list_trash(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
) -> Result<Vec<TrashItemSummary>> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::list_trash(&conn)
}

#[tauri::command]
pub fn get_trash_count(db: State<DbConnection>, vault_state: State<VaultState>) -> Result<i64> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::get_trash_count(&conn)
}

#[tauri::command]
pub fn restore_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::restore_item(&conn, &id)
}

#[tauri::command]
pub fn purge_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::purge_item(&conn, &id)
}

#[tauri::command]
pub fn empty_trash(db: State<DbConnection>, vault_state: State<VaultState>) -> Result<usize> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::empty_trash(&conn)
}

/// Update the automatic-purge window (`0` disables it). The read-only value is
/// reported through `get_security_settings`.
#[tauri::command]
pub fn set_trash_retention_days(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    days: i64,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::set_trash_retention_days(&conn, days)
}
