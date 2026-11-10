//! Duplicate-item commands: detect groups, archive the extras.
//!
//! Both commands need an unlocked vault: detection reads account/credential
//! metadata and the merge mutates it. Detection itself never decrypts anything.

use tauri::State;

use crate::db::{self, DbConnection, DuplicateGroup, MergeReport};
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
pub fn find_duplicates(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
) -> Result<Vec<DuplicateGroup>> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::find_duplicates(&conn)
}

#[tauri::command]
pub fn archive_duplicates(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    keep_id: String,
    duplicate_ids: Vec<String>,
) -> Result<MergeReport> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = lock_db(&db)?;
    db::archive_duplicates(&conn, &keep_id, &duplicate_ids)
}
