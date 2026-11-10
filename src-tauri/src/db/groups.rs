use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, VaultError};

use super::types::Group;
use super::util::chrono_timestamp;

pub fn get_all_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, icon, parent_id, sort_order, created_at, updated_at 
             FROM groups ORDER BY sort_order",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let groups = stmt
        .query_map([], |row| {
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

pub fn update_group(
    conn: &Connection,
    id: &str,
    name: &str,
    icon: Option<&str>,
    sort_order: Option<i32>,
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE groups SET name = ?1, icon = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, icon, sort_order, now, id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn delete_group(conn: &Connection, id: &str) -> Result<()> {
    // Any group — built-in or custom — can be deleted. Items in the group are
    // removed by the groups->items ON DELETE CASCADE (foreign keys are
    // enabled per connection), which in turn cascades to the detail tables.
    conn.execute("DELETE FROM groups WHERE id = ?1", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}
