//! Item trash (soft delete).
//!
//! Deleting an item no longer removes it: `deleted_at` is stamped,
//! `original_group_id` remembers where it came from and `group_id` is pointed
//! at the hidden [`TRASH_GROUP_ID`] group so every active-item read path can
//! filter on `deleted_at IS NULL` alone. Detail rows, attachments, custom
//! fields, tags and secret history survive until the item is purged.

use rusqlite::{params, Connection};

use crate::error::{Result, VaultError};

use super::audit::log_event;
use super::schema::insert_builtin_groups;
use super::types::{TrashItemSummary, TRASH_GROUP_ID};
use super::util::{chrono_timestamp, item_title};

/// Default retention window written to `vault_config('trash_retention_days')`.
pub const DEFAULT_TRASH_RETENTION_DAYS: i64 = 30;
/// Upper bound for the configured retention window (~10 years).
pub const MAX_TRASH_RETENTION_DAYS: i64 = 3650;
const RETENTION_CONFIG_KEY: &str = "trash_retention_days";

// ---------- listing ----------

/// Soft-deleted items, newest deletion first. `original_group_name` is NULL
/// once the original group has been deleted.
pub fn list_trash(conn: &Connection) -> Result<Vec<TrashItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, i.type, i.icon, i.deleted_at, i.original_group_id, g.name
             FROM items i
             LEFT JOIN groups g ON g.id = i.original_group_id
             WHERE i.deleted_at IS NOT NULL
             ORDER BY i.deleted_at DESC, i.title",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let items = stmt
        .query_map([], |row| {
            Ok(TrashItemSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                item_type: row.get(2)?,
                icon: row.get(3)?,
                deleted_at: row.get(4)?,
                original_group_id: row.get(5)?,
                original_group_name: row.get(6)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(items)
}

pub fn get_trash_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM items WHERE deleted_at IS NOT NULL", [], |row| {
        row.get(0)
    })
    .map_err(|e| VaultError::DatabaseError(e.to_string()))
}

// ---------- restore ----------

/// Built-in group an item type belongs to (the restore fallback).
fn builtin_group_for_type(item_type: &str) -> &'static str {
    match item_type {
        "account" => "built-in-accounts",
        "api_key" => "built-in-api-keys",
        "env_var" => "built-in-env-vars",
        "database" => "built-in-databases",
        "ssh" => "built-in-servers",
        "cloud" => "built-in-cloud",
        "license" => "built-in-licenses",
        "smtp" => "built-in-smtp",
        // Unknown types still need a home; fall back to the accounts group.
        _ => "built-in-accounts",
    }
}

fn group_exists(conn: &Connection, id: &str) -> Result<bool> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM groups WHERE id = ?1", [id], |row| row.get(0))
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(count > 0)
}

/// Pick the group a restored item should land in: the original group when it
/// still exists, then the built-in group for its type, then any non-trash
/// group, and finally a freshly re-created set of built-in groups.
fn resolve_restore_group(
    conn: &Connection,
    item_type: &str,
    original_group_id: Option<&str>,
) -> Result<String> {
    if let Some(group_id) = original_group_id {
        if group_id != TRASH_GROUP_ID && group_exists(conn, group_id)? {
            return Ok(group_id.to_string());
        }
    }

    let builtin = builtin_group_for_type(item_type);
    if group_exists(conn, builtin)? {
        return Ok(builtin.to_string());
    }

    let any_group: Option<String> = conn
        .query_row(
            "SELECT id FROM groups WHERE id != ?1 ORDER BY sort_order LIMIT 1",
            [TRASH_GROUP_ID],
            |row| row.get(0),
        )
        .ok();
    if let Some(group_id) = any_group {
        return Ok(group_id);
    }

    // Every group (including the built-ins) is gone; recreate the defaults.
    insert_builtin_groups(conn)?;
    if group_exists(conn, builtin)? {
        return Ok(builtin.to_string());
    }
    Err(VaultError::DatabaseError(
        "No group available to restore the item into".to_string(),
    ))
}

/// Move one trashed item back into a live group.
pub fn restore_item(conn: &Connection, id: &str) -> Result<()> {
    let row: std::result::Result<(String, String, Option<String>), rusqlite::Error> = conn
        .query_row(
            "SELECT title, type, original_group_id FROM items
             WHERE id = ?1 AND deleted_at IS NOT NULL",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

    let (title, item_type, original_group_id) = match row {
        Ok(row) => row,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(VaultError::ItemNotFound),
        Err(e) => return Err(VaultError::DatabaseError(e.to_string())),
    };

    let group_id = resolve_restore_group(conn, &item_type, original_group_id.as_deref())?;
    let now = chrono_timestamp();

    let affected = conn
        .execute(
            "UPDATE items SET group_id = ?1, deleted_at = NULL, original_group_id = NULL,
                    updated_at = ?2
             WHERE id = ?3 AND deleted_at IS NOT NULL",
            params![group_id, now, id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if affected == 0 {
        return Err(VaultError::ItemNotFound);
    }

    log_event(conn, "item.restore", Some(id), Some(&title), None);
    Ok(())
}

// ---------- permanent delete ----------

/// Permanently delete one trashed item. Only trashed rows can be purged, so a
/// mis-targeted id can never hard-delete a live item. Detail rows, attachments,
/// custom fields, tag links and secret history go with it (ON DELETE CASCADE).
pub fn purge_item(conn: &Connection, id: &str) -> Result<()> {
    let title = item_title(conn, id);
    let affected = conn
        .execute("DELETE FROM items WHERE id = ?1 AND deleted_at IS NOT NULL", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if affected == 0 {
        return Err(VaultError::ItemNotFound);
    }
    log_event(conn, "item.purge", Some(id), title.as_deref(), None);
    Ok(())
}

/// Permanently delete everything in the trash; returns the number of items.
pub fn empty_trash(conn: &Connection) -> Result<usize> {
    let count = get_trash_count(conn)?;
    if count > 0 {
        conn.execute("DELETE FROM items WHERE deleted_at IS NOT NULL", [])
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    // Always recorded: emptying the trash is an explicit user action.
    log_event(conn, "trash.empty", None, None, Some(&count.to_string()));
    Ok(count as usize)
}

// ---------- automatic purge ----------

/// Configured retention window in days. `0` disables the automatic purge.
/// Negative values are meaningless and are treated as "unset" (the 30-day
/// default) rather than as `0`, which would silently disable the purge;
/// oversized values clamp to [`MAX_TRASH_RETENTION_DAYS`], and a missing or
/// malformed entry falls back to the default.
pub fn trash_retention_days(conn: &Connection) -> i64 {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM vault_config WHERE key = ?1",
            [RETENTION_CONFIG_KEY],
            |row| row.get(0),
        )
        .ok();
    match raw.and_then(|value| value.trim().parse::<i64>().ok()) {
        Some(days) if days < 0 => DEFAULT_TRASH_RETENTION_DAYS,
        Some(days) => days.min(MAX_TRASH_RETENTION_DAYS),
        None => DEFAULT_TRASH_RETENTION_DAYS,
    }
}

/// Persist the retention window. Validation lives here (not only in the
/// command) so every caller shares one range check.
pub fn set_trash_retention_days(conn: &Connection, days: i64) -> Result<()> {
    if !(0..=MAX_TRASH_RETENTION_DAYS).contains(&days) {
        return Err(VaultError::Validation(format!(
            "Trash retention must be between 0 and {MAX_TRASH_RETENTION_DAYS} days"
        )));
    }
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES (?1, ?2)",
        params![RETENTION_CONFIG_KEY, days.to_string()],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Drop trash entries older than the retention window. Runs on every unlock
/// (after the daily backup), so the audit entry is only written when something
/// was actually removed — otherwise every unlock would append a noise row.
pub fn purge_expired_trash(conn: &Connection) -> Result<usize> {
    let days = trash_retention_days(conn);
    if days == 0 {
        return Ok(0);
    }
    let cutoff = chrono_timestamp() - days * 86400;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM items WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
            [cutoff],
            |row| row.get(0),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if count == 0 {
        return Ok(0);
    }

    conn.execute(
        "DELETE FROM items WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
        [cutoff],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    log_event(conn, "trash.purge_expired", None, None, Some(&count.to_string()));
    Ok(count as usize)
}
