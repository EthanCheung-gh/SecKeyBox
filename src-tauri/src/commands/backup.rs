//! Local snapshot rotation.
//!
//! Snapshots are consistent copies of the encrypted database produced with
//! SQLite's `VACUUM INTO` (safe while the app is running), stored under
//! `app_data_dir/backups`. Ten most recent files are kept; one snapshot per
//! day is created automatically when the vault unlocks.

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::db::{get_db_path, log_event, open_connection, DbConnection};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

const MAX_BACKUPS: usize = 10;
const BACKUP_PREFIX: &str = "vault-";

#[derive(Debug, Clone, Serialize)]
pub struct BackupInfo {
    /// File name inside the backups directory (the handle used by the API).
    pub file: String,
    pub created_at: i64,
    pub size: i64,
}

fn backups_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    get_db_path(app_handle)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("backups")
}

fn list_dir(dir: &Path) -> Vec<(String, i64, i64)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(ts) = parse_backup_name(&name) else {
            continue;
        };
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_file() {
            out.push((name, ts, meta.len() as i64));
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}

/// `vault-<epoch>.db` -> epoch.
fn parse_backup_name(name: &str) -> Option<i64> {
    let stem = name.strip_prefix(BACKUP_PREFIX)?.strip_suffix(".db")?;
    if stem.is_empty() || !stem.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    stem.parse().ok()
}

fn prune(dir: &Path) {
    for (name, _, _) in list_dir(dir).into_iter().skip(MAX_BACKUPS) {
        let _ = std::fs::remove_file(dir.join(name));
    }
}

fn snapshot(conn: &rusqlite::Connection, dir: &Path) -> Result<BackupInfo> {
    std::fs::create_dir_all(dir)
        .map_err(|e| VaultError::Validation(format!("Cannot create backups directory: {}", e)))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let file = format!("{}{}.db", BACKUP_PREFIX, now);
    let path = dir.join(&file);
    let _ = std::fs::remove_file(&path);

    conn.execute("VACUUM INTO ?1", [path.to_string_lossy().to_string()])
        .map_err(|e| VaultError::DatabaseError(format!("Backup failed: {}", e)))?;

    let size = std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0);
    prune(dir);
    Ok(BackupInfo {
        file,
        created_at: now,
        size,
    })
}

/// Create a snapshot on demand.
#[tauri::command]
pub fn create_backup(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
) -> Result<BackupInfo> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    let info = snapshot(&conn, &backups_dir(&app_handle))?;
    log_event(&conn, "vault.backup", None, None, Some(&info.file));
    Ok(info)
}

#[tauri::command]
pub fn list_backups(app_handle: tauri::AppHandle) -> Result<Vec<BackupInfo>> {
    Ok(list_dir(&backups_dir(&app_handle))
        .into_iter()
        .map(|(file, created_at, size)| BackupInfo {
            file,
            created_at,
            size,
        })
        .collect())
}

#[tauri::command]
pub fn delete_backup(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    file: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    if parse_backup_name(&file).is_none() {
        return Err(VaultError::Validation("Invalid backup name".to_string()));
    }
    let path = backups_dir(&app_handle).join(&file);
    std::fs::remove_file(&path)
        .map_err(|e| VaultError::Validation(format!("Cannot delete backup: {}", e)))?;
    if let Ok(conn) = db.inner().0.lock() {
        log_event(&conn, "vault.backup_delete", None, None, Some(&file));
    }
    Ok(())
}

/// Restore a snapshot over the live database.
///
/// Order matters: verify the snapshot, take a pre-restore snapshot, lock the
/// vault, swap files atomically, then reopen the connection. The user must
/// unlock again afterwards.
#[tauri::command]
pub fn restore_backup(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    file: String,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    if parse_backup_name(&file).is_none() {
        return Err(VaultError::Validation("Invalid backup name".to_string()));
    }

    let dir = backups_dir(&app_handle);
    let source = dir.join(&file);
    let db_path = get_db_path(&app_handle);

    // 1. The snapshot must be a healthy SQLite database.
    {
        let candidate = open_connection(&source)?;
        let check: String = candidate
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| VaultError::DatabaseError(format!("Backup is unreadable: {}", e)))?;
        if check != "ok" {
            return Err(VaultError::Validation(format!(
                "Backup failed integrity check: {}",
                check
            )));
        }
    }

    // 2. Keep a snapshot of the current state first.
    {
        let mut conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        snapshot(&conn, &dir)?;
        log_event(&conn, "vault.restore", None, None, Some(&file));

        // 3. Drop the live connection so the file can be replaced.
        *conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    vault_state.lock();

    // 4. Copy next to the target, then rename (atomic on the same filesystem).
    let staged = db_path.with_extension("db.restore");
    std::fs::copy(&source, &staged)
        .map_err(|e| VaultError::Validation(format!("Cannot stage restore: {}", e)))?;
    std::fs::rename(&staged, &db_path)
        .map_err(|e| VaultError::Validation(format!("Cannot replace database: {}", e)))?;

    // 5. Reopen so the app is usable again (locked).
    let reopened = open_connection(&db_path)?;
    {
        let mut conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        *conn = reopened;
    }

    Ok(())
}

/// Keep at most one snapshot per calendar day; called after a successful
/// unlock. Failures are ignored so they never block unlocking.
pub fn ensure_daily_backup(app_handle: &tauri::AppHandle, conn: &rusqlite::Connection) {
    let dir = backups_dir(app_handle);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let today_start = now - (now % 86_400);
    if list_dir(&dir).iter().any(|(_, ts, _)| *ts >= today_start) {
        return;
    }
    if let Ok(info) = snapshot(conn, &dir) {
        log_event(conn, "vault.backup", None, None, Some(&info.file));
    }
}

#[cfg(test)]
mod tests {
    use super::parse_backup_name;

    #[test]
    fn backup_names_are_strictly_parsed() {
        assert_eq!(parse_backup_name("vault-1700000000.db"), Some(1_700_000_000));
        // Path traversal and unexpected names are rejected.
        assert_eq!(parse_backup_name("../vault-1700000000.db"), None);
        assert_eq!(parse_backup_name("vault-1700000000.db.tmp"), None);
        assert_eq!(parse_backup_name("vault-.db"), None);
        assert_eq!(parse_backup_name("seckeybox.db"), None);
        assert_eq!(parse_backup_name("vault-12a.db"), None);
    }
}
