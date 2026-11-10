use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, VaultError};

use super::audit::log_event;
use super::types::{Group, TRASH_GROUP_ID};
use super::util::{chrono_timestamp, with_tx};

/// All user-visible groups. The hidden trash group is excluded, so the
/// sidebar never offers it and it cannot be renamed away.
pub fn get_all_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, icon, parent_id, sort_order, created_at, updated_at 
             FROM groups WHERE id != ?1 ORDER BY sort_order",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let groups = stmt
        .query_map([TRASH_GROUP_ID], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                parent_id: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(groups)
}

pub fn create_group(
    conn: &Connection,
    name: &str,
    icon: Option<&str>,
    parent_id: Option<&str>,
    sort_order: Option<i32>,
) -> Result<Group> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();
    let sort_order = sort_order.unwrap_or(999);

    conn.execute(
        "INSERT INTO groups (id, name, icon, parent_id, sort_order, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, icon, parent_id, sort_order, now, now],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(Group {
        id,
        name: name.to_string(),
        icon: icon.map(|s| s.to_string()),
        parent_id: parent_id.map(|s| s.to_string()),
        sort_order,
        created_at: now,
        updated_at: now,
    })
}

fn ensure_not_trash_group(id: &str) -> Result<()> {
    if id == TRASH_GROUP_ID {
        return Err(VaultError::Validation(
            "The trash group is managed by the app and cannot be modified".to_string(),
        ));
    }
    Ok(())
}

pub fn update_group(
    conn: &Connection,
    id: &str,
    name: &str,
    icon: Option<&str>,
    sort_order: Option<i32>,
) -> Result<()> {
    ensure_not_trash_group(id)?;
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE groups SET name = ?1, icon = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, icon, sort_order, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Delete a group without destroying its contents: items still in the group
/// are moved to the trash first (inside the same transaction), so their detail
/// rows survive the `groups -> items ON DELETE CASCADE`. Any group — built-in
/// or custom — can be deleted, except the trash group itself.
pub fn delete_group(conn: &Connection, id: &str) -> Result<()> {
    ensure_not_trash_group(id)?;

    with_tx(conn, |tx| {
        let now = chrono_timestamp();

        // `deleted_at IS NULL` keeps already-trashed items pointing at the
        // trash group; only live items of this group are moved.
        let trashed = tx
            .execute(
                "UPDATE items
                 SET deleted_at = ?1, original_group_id = group_id, group_id = ?2, updated_at = ?1
                 WHERE group_id = ?3 AND deleted_at IS NULL",
                params![now, TRASH_GROUP_ID, id],
            )
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

        let name = tx
            .query_row("SELECT name FROM groups WHERE id = ?1", [id], |row| {
                row.get::<_, String>(0)
            })
            .ok();

        // Nothing left to cascade: the group holds no live items any more.
        let deleted = tx
            .execute("DELETE FROM groups WHERE id = ?1", [id])
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        if deleted == 0 {
            // Unknown group: report it instead of writing an audit entry for
            // something that never happened.
            return Err(VaultError::ItemNotFound);
        }

        log_event(
            tx,
            "group.delete",
            None,
            name.as_deref(),
            Some(&format!("{trashed} item(s) moved to trash")),
        );
        Ok(())
    })
}
