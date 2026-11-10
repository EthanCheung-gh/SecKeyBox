use serde::Serialize;
use tauri::State;

use crate::db::{DbConnection, IntegrityReport};
use crate::error::{Result, VaultError};
use crate::state::VaultState;

pub const DEFAULT_AUTO_LOCK_MINUTES: i64 = 5;
const CONFIG_KEY: &str = "auto_lock_minutes";

#[derive(Debug, Clone, Serialize)]
pub struct SecuritySettings {
    pub auto_lock_minutes: i64,
    /// Days a trashed item is kept before the automatic purge; `0` disables it.
    pub trash_retention_days: i64,
}

fn read_auto_lock_minutes(conn: &rusqlite::Connection) -> Option<i64> {
    let value: String = conn
        .query_row(
            "SELECT value FROM vault_config WHERE key = ?1",
            [CONFIG_KEY],
            |row| row.get(0),
        )
        .ok()?;
    value.trim().parse::<i64>().ok()
}

/// Report user interaction. The frontend throttles calls to roughly one per
/// 10s; the backend just records the timestamp used by the idle watchdog.
#[tauri::command]
pub fn record_activity(vault_state: State<VaultState>) {
    vault_state.touch_activity();
}

/// Current security policy. Falls back to the default when the DB has not
/// been initialized yet (e.g. the setup screen).
#[tauri::command]
pub fn get_security_settings(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
) -> Result<SecuritySettings> {
    let (minutes, trash_retention_days) = {
        let conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        (
            read_auto_lock_minutes(&conn).unwrap_or(DEFAULT_AUTO_LOCK_MINUTES),
            crate::db::trash_retention_days(&conn),
        )
    };
    let minutes = minutes.max(0);
    vault_state.set_auto_lock_minutes(minutes);
    Ok(SecuritySettings {
        auto_lock_minutes: minutes,
        trash_retention_days,
    })
}

/// Persist the idle-lock policy and apply it to the running watchdog.
/// `0` disables the idle lock entirely.
#[tauri::command]
pub fn set_auto_lock_minutes(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    minutes: i64,
) -> Result<()> {
    if minutes < 0 || minutes > 24 * 60 {
        return Err(VaultError::Validation(
            "Auto-lock minutes must be between 0 and 1440".to_string(),
        ));
    }
    {
        let conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        conn.execute(
            "INSERT OR REPLACE INTO vault_config (key, value) VALUES (?1, ?2)",
            rusqlite::params![CONFIG_KEY, minutes.to_string()],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    vault_state.set_auto_lock_minutes(minutes);
    // Treat the policy change itself as activity so shortening the timeout
    // does not immediately lock the vault under the user's cursor.
    vault_state.touch_activity();
    Ok(())
}

/// Full-vault integrity self-check: decrypt every encrypted column, then run
/// SQLite's own `integrity_check` / `foreign_key_check`.
///
/// Attachments make the scan slow, so it runs on the blocking pool instead of
/// the async runtime. The master key is copied out of the state before `app`
/// moves into the task.
#[tauri::command]
pub async fn check_vault_integrity(
    app: tauri::AppHandle,
    include_attachments: bool,
) -> Result<IntegrityReport> {
    use tauri::Manager;

    let key: [u8; 32] = {
        let vault_state = app.state::<VaultState>();
        if !vault_state.is_unlocked() {
            return Err(VaultError::VaultLocked);
        }
        let secure_key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
        *secure_key.as_bytes()
    };

    let report = tauri::async_runtime::spawn_blocking(move || {
        let db = app.state::<DbConnection>();
        let conn = db
            .inner()
            .0
            .lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
        crate::db::check_vault_integrity(&conn, &key, include_attachments)
    })
    .await
    .map_err(|e| VaultError::DatabaseError(format!("Integrity check failed: {}", e)))??;

    Ok(report)
}

