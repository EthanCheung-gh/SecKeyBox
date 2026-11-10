use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, VaultError};

use super::types::AttachmentMeta;
use super::util::chrono_timestamp;

/// Largest attachment accepted by the backend (bytes).
pub const MAX_ATTACHMENT_BYTES: usize = 5 * 1024 * 1024;

// ---------- attachments ----------

/// Store an encrypted attachment for an item. Contents are AES-GCM encrypted
/// with the vault key; only the metadata columns stay readable.
pub fn add_attachment(
    conn: &Connection,
    item_id: &str,
    filename: &str,
    content_type: Option<&str>,
    data: &[u8],
    key: &[u8; 32],
) -> Result<AttachmentMeta> {
    if data.is_empty() {
        return Err(VaultError::Validation(
            "Attachment is empty".to_string(),
        ));
    }
    if data.len() > MAX_ATTACHMENT_BYTES {
        return Err(VaultError::Validation(format!(
            "Attachment exceeds the {} MB limit",
            MAX_ATTACHMENT_BYTES / (1024 * 1024)
        )));
    }

    let exists: i64 = conn
        .query_row("SELECT COUNT(*) FROM items WHERE id = ?1", [item_id], |row| {
            row.get(0)
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if exists == 0 {
        return Err(VaultError::ItemNotFound);
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();
    let encrypted =
        crate::crypto::encrypt(key, data).map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO item_attachments (id, item_id, filename, content_type, size, data_encrypted, data_nonce, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            item_id,
            filename,
            content_type,
            data.len() as i64,
            encrypted.ciphertext,
            encrypted.nonce.to_vec(),
            now
        ],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(AttachmentMeta {
        id,
        item_id: item_id.to_string(),
        filename: filename.to_string(),
        content_type: content_type.map(|s| s.to_string()),
        size: data.len() as i64,
        created_at: now,
    })
}

pub fn list_attachments(conn: &Connection, item_id: &str) -> Result<Vec<AttachmentMeta>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, filename, content_type, size, created_at
             FROM item_attachments WHERE item_id = ?1 ORDER BY created_at, filename",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let rows = stmt
        .query_map([item_id], |row| {
            Ok(AttachmentMeta {
                id: row.get(0)?,
                item_id: row.get(1)?,
                filename: row.get(2)?,
                content_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))
}

/// Decrypt one attachment. `item_id` scopes the lookup so a stale attachment
/// id from another item cannot be read.
pub fn get_attachment_data(
    conn: &Connection,
    item_id: &str,
    attachment_id: &str,
    key: &[u8; 32],
) -> Result<(AttachmentMeta, Vec<u8>)> {
    let row = conn.query_row(
        "SELECT id, item_id, filename, content_type, size, created_at, data_nonce, data_encrypted
         FROM item_attachments WHERE id = ?1 AND item_id = ?2",
        params![attachment_id, item_id],
        |row| {
            let meta = AttachmentMeta {
                id: row.get(0)?,
                item_id: row.get(1)?,
                filename: row.get(2)?,
                content_type: row.get(3)?,
                size: row.get(4)?,
                created_at: row.get(5)?,
            };
            let nonce: Vec<u8> = row.get(6)?;
            let ciphertext: Vec<u8> = row.get(7)?;
            Ok((meta, nonce, ciphertext))
        },
    );

    let (meta, nonce, ciphertext) = match row {
        Ok(value) => value,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Err(VaultError::ItemNotFound),
        Err(e) => return Err(VaultError::DatabaseError(e.to_string())),
    };

    let nonce_bytes: [u8; 12] = nonce
        .as_slice()
        .try_into()
        .map_err(|_| VaultError::CryptoError("Invalid attachment nonce".to_string()))?;

    let data = crate::crypto::decrypt(key, &nonce_bytes, &ciphertext)?;
    Ok((meta, data))
}

pub fn delete_attachment(conn: &Connection, item_id: &str, attachment_id: &str) -> Result<()> {
    let affected = conn
        .execute(
            "DELETE FROM item_attachments WHERE id = ?1 AND item_id = ?2",
            params![attachment_id, item_id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    if affected == 0 {
        return Err(VaultError::ItemNotFound);
    }
    Ok(())
}
