use rusqlite::Connection;

use crate::error::{Result, VaultError};

pub(crate) fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Run `f` inside a transaction so an item is never left behind without its
/// detail row (a half-written creation used to produce an items row whose
/// missing detail made the item und-openable after relaunch).
pub(crate) fn with_tx<T>(
    conn: &Connection,
    f: impl FnOnce(&rusqlite::Transaction) -> Result<T>,
) -> Result<T> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    let out = f(&tx)?;
    tx.commit()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(out)
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02X}"));
    }
    out
}

/// Look up an item title (for activity entries written after a delete).
pub(crate) fn item_title(conn: &Connection, id: &str) -> Option<String> {
    conn.query_row("SELECT title FROM items WHERE id = ?1", [id], |row| row.get(0))
        .ok()
}
