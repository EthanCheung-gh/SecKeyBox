use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, VaultError};

use super::audit::log_event;
use super::items::get_item_summary;
use super::types::{CustomField, ItemSummary, Tag};
use super::util::{chrono_timestamp, item_title};

// ---------- custom fields & tags ----------

pub fn list_custom_fields(conn: &Connection, item_id: &str, key: &[u8; 32]) -> Result<Vec<CustomField>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, field_key, value_encrypted, value_nonce
             FROM custom_fields WHERE item_id = ?1 ORDER BY sort_order, field_key",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let mut out = Vec::new();
    for row in rows {
        let (id, field_key, ciphertext, nonce) = row.map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        let nonce_bytes: [u8; 12] = nonce
            .as_slice()
            .try_into()
            .map_err(|_| VaultError::CryptoError("Invalid custom field nonce".to_string()))?;
        let plain = crate::crypto::decrypt(key, &nonce_bytes, &ciphertext)?;
        let value = String::from_utf8(plain)
            .map_err(|_| VaultError::CryptoError("Invalid UTF-8 in custom field".to_string()))?;
        out.push(CustomField { id, key: field_key, value });
    }
    Ok(out)
}

/// Replace all custom fields for an item. Empty keys are rejected; an empty
/// value is allowed (a blank note-style field).
pub fn save_custom_fields(
    conn: &Connection,
    item_id: &str,
    fields: &[(String, String)],
    key: &[u8; 32],
) -> Result<()> {
    let exists: i64 = conn
        .query_row("SELECT COUNT(*) FROM items WHERE id = ?1", [item_id], |row| row.get(0))
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if exists == 0 {
        return Err(VaultError::ItemNotFound);
    }

    conn.execute("DELETE FROM custom_fields WHERE item_id = ?1", [item_id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    for (index, (field_key, value)) in fields.iter().enumerate() {
        let trimmed = field_key.trim();
        if trimmed.is_empty() {
            return Err(VaultError::Validation("Custom field key must not be empty".to_string()));
        }
        let encrypted = crate::crypto::encrypt(key, value.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        conn.execute(
            "INSERT INTO custom_fields (id, item_id, field_key, value_encrypted, value_nonce, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Uuid::new_v4().to_string(),
                item_id,
                trimmed,
                encrypted.ciphertext,
                encrypted.nonce.to_vec(),
                index as i64
            ],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    log_event(conn, "item.update", Some(item_id), item_title(conn, item_id).as_deref(), Some("custom_fields"));
    Ok(())
}

pub fn get_all_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, created_at FROM tags ORDER BY name")
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, created_at: row.get(3)? })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))
}

pub fn create_tag(conn: &Connection, name: &str, color: Option<&str>) -> Result<Tag> {
    let name = name.trim();
    if name.is_empty() {
        return Err(VaultError::Validation("Tag name must not be empty".to_string()));
    }
    let existing: Option<String> = conn
        .query_row("SELECT id FROM tags WHERE name = ?1", [name], |row| row.get(0))
        .ok();
    if let Some(id) = existing {
        // Idempotent: return the existing tag instead of failing.
        return conn
            .query_row("SELECT id, name, color, created_at FROM tags WHERE id = ?1", [id], |row| {
                Ok(Tag { id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, created_at: row.get(3)? })
            })
            .map_err(|e| VaultError::DatabaseError(e.to_string()));
    }

    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        color: color.map(|c| c.to_string()),
        created_at: chrono_timestamp(),
    };
    conn.execute(
        "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![tag.id, tag.name, tag.color, tag.created_at],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    log_event(conn, "tag.create", None, Some(name), None);
    Ok(tag)
}

pub fn delete_tag(conn: &Connection, id: &str) -> Result<()> {
    let name = conn
        .query_row("SELECT name FROM tags WHERE id = ?1", [id], |row| row.get::<_, String>(0))
        .ok();
    let affected = conn
        .execute("DELETE FROM tags WHERE id = ?1", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if affected == 0 {
        return Err(VaultError::ItemNotFound);
    }
    log_event(conn, "tag.delete", None, name.as_deref(), None);
    Ok(())
}

pub fn get_item_tag_ids(conn: &Connection, item_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT tag_id FROM item_tags WHERE item_id = ?1")
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let rows = stmt
        .query_map([item_id], |row| row.get::<_, String>(0))
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))
}

pub fn set_item_tags(conn: &Connection, item_id: &str, tag_ids: &[String]) -> Result<()> {
    conn.execute("DELETE FROM item_tags WHERE item_id = ?1", [item_id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    for tag_id in tag_ids {
        conn.execute(
            "INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)",
            params![item_id, tag_id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    Ok(())
}

pub fn get_items_by_tag(conn: &Connection, tag_id: &str) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id FROM items i
             JOIN item_tags it ON it.item_id = i.id
             WHERE it.tag_id = ?1
             ORDER BY i.title",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let ids: Vec<String> = stmt
        .query_map([tag_id], |row| row.get(0))
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    let mut out = Vec::new();
    for id in ids {
        if let Some(summary) = get_item_summary(conn, &id)? {
            out.push(summary);
        }
    }
    Ok(out)
}
