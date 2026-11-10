use rusqlite::{params, Connection};

use crate::error::{Result, VaultError};

use super::types::AuditEntry;
use super::util::chrono_timestamp;

// ---------- audit log ----------

/// Number of activity entries kept; older rows are pruned on insert.
const AUDIT_LOG_LIMIT: i64 = 2000;

/// Append one activity entry. Never store secret values here: `detail` is a
/// short, non-sensitive descriptor (field name, import source, ...).
pub fn log_event(
    conn: &Connection,
    event: &str,
    item_id: Option<&str>,
    item_title: Option<&str>,
    detail: Option<&str>,
) {
    let _ = conn.execute(
        "INSERT INTO audit_log (event, item_id, item_title, detail, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![event, item_id, item_title, detail, chrono_timestamp()],
    );
    let _ = conn.execute(
        "DELETE FROM audit_log WHERE id NOT IN (SELECT id FROM audit_log ORDER BY id DESC LIMIT ?1)",
        [AUDIT_LOG_LIMIT],
    );
}

/// Most recent entries first.
pub fn list_audit_log(conn: &Connection, limit: i64) -> Result<Vec<AuditEntry>> {
    let limit = limit.clamp(1, AUDIT_LOG_LIMIT);
    let mut stmt = conn
        .prepare(
            "SELECT id, event, item_id, item_title, detail, created_at
             FROM audit_log ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let rows = stmt
        .query_map([limit], |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                event: row.get(1)?,
                item_id: row.get(2)?,
                item_title: row.get(3)?,
                detail: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))
}
