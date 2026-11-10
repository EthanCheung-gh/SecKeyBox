//! Duplicate item detection and safe archive/merge.
//!
//! Items are grouped by an identity key built only from structured,
//! unencrypted columns — never from a decrypted secret and never from a fuzzy
//! title match — so detection is deterministic and stays free of false
//! positives. Merging is always user-confirmed: the kept item only gains the
//! values it was missing, the duplicate's metadata is carried over to it, and
//! each duplicate is soft-deleted into the trash so the merge stays reversible
//! until the trash is purged.

use std::collections::{BTreeMap, HashSet};

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, VaultError};

use super::audit::log_event;
use super::items::delete_item;
use super::types::{DuplicateCandidate, DuplicateGroup, MergeReport};
use super::util::{chrono_timestamp, item_title, with_tx};

/// Upper bound for one merge request. A single confirmation should stay a
/// reviewable action, not a bulk operation.
const MAX_MERGE_DUPLICATES: usize = 20;

// ---------- identity keys ----------

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalize_opt(value: Option<&str>) -> String {
    value.map(normalize).unwrap_or_default()
}

fn port_or_zero(port: Option<i64>) -> i64 {
    port.unwrap_or(0)
}

/// Host part of a website, extracted without a URL parser: drop the scheme,
/// any leading `//`, then the path / query / fragment / port, then a leading
/// `www.`. `https://www.GitHub.com/a?b=1` and `github.com:443` both become
/// `github.com`. An unparsable value degrades to an empty host, which pushes
/// the account identity onto the title instead.
fn website_host(website: Option<&str>) -> String {
    let Some(raw) = website else {
        return String::new();
    };
    let mut host = normalize(raw);
    if let Some((_, rest)) = host.split_once("://") {
        host = rest.to_string();
    }
    if let Some(rest) = host.strip_prefix("//") {
        host = rest.to_string();
    }
    let end = host
        .find(|c: char| matches!(c, '/' | '?' | '#' | ':'))
        .unwrap_or(host.len());
    host.truncate(end);
    let host = host.strip_prefix("www.").unwrap_or(&host);
    host.to_string()
}

fn optional_row<T>(row: std::result::Result<T, rusqlite::Error>) -> Result<Option<T>> {
    match row {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(VaultError::DatabaseError(e.to_string())),
    }
}

/// Identity key for an item of known type. `None` when the detail row is gone:
/// an item without a detail row cannot be grouped and cannot be merged safely.
fn identity_key(
    conn: &Connection,
    id: &str,
    item_type: &str,
    title: &str,
) -> Result<Option<String>> {
    let key = match item_type {
        "account" => {
            let row: std::result::Result<(String, Option<String>), rusqlite::Error> = conn
                .query_row(
                    "SELECT username, website FROM account_items WHERE item_id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                );
            let Some((username, website)) = optional_row(row)? else {
                return Ok(None);
            };
            let username = normalize(&username);
            let host = website_host(website.as_deref());
            if username.is_empty() && host.is_empty() {
                normalize(title)
            } else {
                format!("{username}|{host}")
            }
        }
        "api_key" => {
            let row: std::result::Result<(String, Option<String>), rusqlite::Error> = conn
                .query_row(
                    "SELECT key_name, endpoint FROM api_key_items WHERE item_id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                );
            let Some((key_name, endpoint)) = optional_row(row)? else {
                return Ok(None);
            };
            let key_name = normalize(&key_name);
            let endpoint = normalize_opt(endpoint.as_deref());
            if key_name.is_empty() && endpoint.is_empty() {
                normalize(title)
            } else {
                format!("{key_name}|{endpoint}")
            }
        }
        // An env-var item is one named set of variables: the title *is* its
        // identity, and its variable names are free-form.
        "env_var" => normalize(title),
        "database" => {
            let row: std::result::Result<
                (String, String, Option<i64>, Option<String>, Option<String>),
                rusqlite::Error,
            > = conn.query_row(
                "SELECT db_type, host, port, database_name, username
                 FROM database_items WHERE item_id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            );
            let Some((db_type, host, port, database_name, username)) = optional_row(row)? else {
                return Ok(None);
            };
            format!(
                "{}|{}|{}|{}|{}",
                normalize(&db_type),
                normalize(&host),
                port_or_zero(port),
                normalize_opt(database_name.as_deref()),
                normalize_opt(username.as_deref())
            )
        }
        "ssh" => {
            let row: std::result::Result<(String, String, Option<i64>), rusqlite::Error> = conn
                .query_row(
                    "SELECT username, host, port FROM ssh_items WHERE item_id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
            let Some((username, host, port)) = optional_row(row)? else {
                return Ok(None);
            };
            format!(
                "{}|{}|{}",
                normalize(&username),
                normalize(&host),
                port_or_zero(port)
            )
        }
        "cloud" => {
            let row: std::result::Result<(String, String), rusqlite::Error> = conn.query_row(
                "SELECT provider, access_key_id FROM cloud_items WHERE item_id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            );
            let Some((provider, access_key_id)) = optional_row(row)? else {
                return Ok(None);
            };
            format!(
                "{}|{}",
                normalize(&provider),
                normalize(&access_key_id)
            )
        }
        "license" => {
            let row: std::result::Result<(String, Option<String>), rusqlite::Error> = conn
                .query_row(
                    "SELECT software_name, bound_email FROM license_items WHERE item_id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                );
            let Some((software_name, bound_email)) = optional_row(row)? else {
                return Ok(None);
            };
            format!(
                "{}|{}",
                normalize(&software_name),
                normalize_opt(bound_email.as_deref())
            )
        }
        "smtp" => {
            let row: std::result::Result<(String, Option<i64>, Option<String>), rusqlite::Error> =
                conn.query_row(
                    "SELECT host, port, username FROM smtp_items WHERE item_id = ?1",
                    [id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                );
            let Some((host, port, username)) = optional_row(row)? else {
                return Ok(None);
            };
            format!(
                "{}|{}|{}",
                normalize(&host),
                port_or_zero(port),
                normalize_opt(username.as_deref())
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(key))
}

/// `(item_type, identity key)` of one active item. `None` for an unknown or
/// trashed id — the trash is never part of duplicate detection.
pub fn identity_for_item(conn: &Connection, id: &str) -> Result<Option<(String, String)>> {
    let row: std::result::Result<(String, String), rusqlite::Error> = conn.query_row(
        "SELECT type, title FROM items WHERE id = ?1 AND deleted_at IS NULL",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );
    let Some((item_type, title)) = optional_row(row)? else {
        return Ok(None);
    };
    let key = identity_key(conn, id, &item_type, &title)?;
    Ok(key.map(|key| (item_type, key)))
}

// ---------- detection ----------

/// Every group of active items that share an identity key. Ordered by type and
/// key; members are ordered oldest first (the natural pick for "keep").
pub fn find_duplicates(conn: &Connection) -> Result<Vec<DuplicateGroup>> {
    // Collect the headers first: the per-item identity lookup must not run
    // inside an open statement.
    let headers: Vec<(String, String, String, String, i64, i64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, type, title, group_id, created_at, updated_at
                 FROM items WHERE deleted_at IS NULL",
            )
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?
    };

    let mut groups: BTreeMap<(String, String), Vec<DuplicateCandidate>> = BTreeMap::new();
    for (id, item_type, title, group_id, created_at, updated_at) in headers {
        let Some(key) = identity_key(conn, &id, &item_type, &title)? else {
            continue;
        };
        groups
            .entry((item_type, key))
            .or_default()
            .push(DuplicateCandidate {
                id,
                title,
                group_id,
                created_at,
                updated_at,
            });
    }

    let mut out = Vec::new();
    for ((item_type, key), mut items) in groups {
        if items.len() < 2 {
            continue;
        }
        items.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.title.cmp(&b.title))
        });
        out.push(DuplicateGroup {
            key,
            item_type,
            items,
        });
    }
    Ok(out)
}

// ---------- archive / merge ----------

/// Reject a merge request *before* anything is written. Returns the keep item's
/// type and the duplicate ids in merge order (oldest first) so empty-field
/// filling is deterministic.
fn validate_merge_request(
    conn: &Connection,
    keep_id: &str,
    duplicate_ids: &[String],
) -> Result<(String, Vec<String>)> {
    if duplicate_ids.is_empty() {
        return Err(VaultError::Validation(
            "Select at least one duplicate to archive".to_string(),
        ));
    }
    if duplicate_ids.len() > MAX_MERGE_DUPLICATES {
        return Err(VaultError::Validation(format!(
            "At most {MAX_MERGE_DUPLICATES} duplicates can be archived at once"
        )));
    }

    let mut seen: HashSet<&str> = HashSet::with_capacity(duplicate_ids.len());
    for duplicate in duplicate_ids {
        if duplicate == keep_id {
            return Err(VaultError::Validation(
                "The kept item cannot also be archived as a duplicate".to_string(),
            ));
        }
        if !seen.insert(duplicate.as_str()) {
            return Err(VaultError::Validation(
                "The same duplicate was selected twice".to_string(),
            ));
        }
    }

    // Unknown or trashed ids are `ItemNotFound`; mismatched identities are a
    // client mistake and must never reach the mutation phase.
    let keep_identity = identity_for_item(conn, keep_id)?.ok_or(VaultError::ItemNotFound)?;
    let (item_type, keep_key) = keep_identity;
    let mut ordered: Vec<(i64, String)> = Vec::with_capacity(duplicate_ids.len());
    for duplicate in duplicate_ids {
        let identity = identity_for_item(conn, duplicate)?.ok_or(VaultError::ItemNotFound)?;
        if identity.0 != item_type || identity.1 != keep_key {
            return Err(VaultError::Validation(
                "Selected items do not share the same identity".to_string(),
            ));
        }
        let created_at: i64 = conn
            .query_row(
                "SELECT created_at FROM items WHERE id = ?1",
                [duplicate],
                |row| row.get(0),
            )
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        ordered.push((created_at, duplicate.clone()));
    }
    ordered.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((item_type, ordered.into_iter().map(|(_, id)| id).collect()))
}

/// Copy a plain text column from `duplicate_id` only when the keep value is
/// empty. Never overwrites a non-empty keep value.
fn fill_text(
    conn: &Connection,
    table: &str,
    column: &str,
    keep_id: &str,
    duplicate_id: &str,
) -> Result<()> {
    conn.execute(
        &format!(
            "UPDATE {table}
             SET {column} = COALESCE(NULLIF({column}, ''), (
                 SELECT {column} FROM {table}
                 WHERE item_id = ?2 AND {column} IS NOT NULL AND {column} <> ''
             ))
             WHERE item_id = ?1"
        ),
        params![keep_id, duplicate_id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Copy a numeric/timestamp column from `duplicate_id` only when the keep value
/// is NULL.
fn fill_number(
    conn: &Connection,
    table: &str,
    column: &str,
    keep_id: &str,
    duplicate_id: &str,
) -> Result<()> {
    conn.execute(
        &format!(
            "UPDATE {table}
             SET {column} = COALESCE({column}, (
                 SELECT {column} FROM {table}
                 WHERE item_id = ?2 AND {column} IS NOT NULL
             ))
             WHERE item_id = ?1"
        ),
        params![keep_id, duplicate_id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Copy an encrypted ciphertext/nonce pair when the keep ciphertext is NULL.
/// Both rows live under the same master key, so the ciphertext is copied
/// verbatim — no decrypt/re-encrypt, and no plaintext ever leaves the DB.
fn fill_encrypted(
    conn: &Connection,
    table: &str,
    ciphertext_column: &str,
    nonce_column: &str,
    keep_id: &str,
    duplicate_id: &str,
) -> Result<()> {
    conn.execute(
        &format!(
            "UPDATE {table}
             SET {ciphertext_column} = COALESCE({ciphertext_column}, (
                     SELECT {ciphertext_column} FROM {table}
                     WHERE item_id = ?2 AND {ciphertext_column} IS NOT NULL
                 )),
                 {nonce_column} = COALESCE({nonce_column}, (
                     SELECT {nonce_column} FROM {table}
                     WHERE item_id = ?2 AND {nonce_column} IS NOT NULL
                 ))
             WHERE item_id = ?1"
        ),
        params![keep_id, duplicate_id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Env-var notes live on the variable rows (one per key), not on the item.
fn fill_env_var_notes(
    conn: &Connection,
    keep_id: &str,
    duplicate_id: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE env_var_items
         SET notes = COALESCE(notes, (
             SELECT notes FROM env_var_items
             WHERE item_id = ?2 AND notes IS NOT NULL AND notes <> ''
             LIMIT 1
         ))
         WHERE item_id = ?1",
        params![keep_id, duplicate_id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Add every variable the keep item does not have yet. The keep value always
/// wins on a key collision, and the duplicate's ciphertext is copied as-is.
fn merge_env_vars(conn: &Connection, keep_id: &str, duplicate_id: &str) -> Result<usize> {
    conn.execute(
        "INSERT OR IGNORE INTO env_var_items
             (item_id, key, value_encrypted, value_nonce, sort_order, notes)
         SELECT ?1, key, value_encrypted, value_nonce, sort_order, notes
         FROM env_var_items WHERE item_id = ?2",
        params![keep_id, duplicate_id],
    )
    .map_err(|e| VaultError::DatabaseError(e.to_string()))
}

/// Fill the keep item's empty fields from one duplicate. Returns the number of
/// env-var rows added (always 0 for the other types).
fn fill_missing_fields(
    conn: &Connection,
    keep_id: &str,
    duplicate_id: &str,
    item_type: &str,
) -> Result<usize> {
    match item_type {
        "account" => {
            fill_text(conn, "account_items", "website", keep_id, duplicate_id)?;
            fill_text(conn, "account_items", "notes", keep_id, duplicate_id)?;
            fill_encrypted(
                conn,
                "account_items",
                "totp_secret_encrypted",
                "totp_secret_nonce",
                keep_id,
                duplicate_id,
            )?;
        }
        "api_key" => {
            fill_text(conn, "api_key_items", "endpoint", keep_id, duplicate_id)?;
            fill_text(conn, "api_key_items", "auth_method", keep_id, duplicate_id)?;
            fill_text(conn, "api_key_items", "notes", keep_id, duplicate_id)?;
            fill_number(conn, "api_key_items", "rotation_date", keep_id, duplicate_id)?;
        }
        "database" => {
            fill_text(conn, "database_items", "database_name", keep_id, duplicate_id)?;
            fill_text(conn, "database_items", "username", keep_id, duplicate_id)?;
            fill_text(conn, "database_items", "connection_url", keep_id, duplicate_id)?;
            fill_text(conn, "database_items", "notes", keep_id, duplicate_id)?;
            fill_number(conn, "database_items", "port", keep_id, duplicate_id)?;
            fill_encrypted(
                conn,
                "database_items",
                "password_encrypted",
                "password_nonce",
                keep_id,
                duplicate_id,
            )?;
        }
        "ssh" => {
            fill_text(conn, "ssh_items", "key_path", keep_id, duplicate_id)?;
            fill_text(conn, "ssh_items", "notes", keep_id, duplicate_id)?;
            fill_number(conn, "ssh_items", "port", keep_id, duplicate_id)?;
            fill_encrypted(
                conn,
                "ssh_items",
                "password_encrypted",
                "password_nonce",
                keep_id,
                duplicate_id,
            )?;
            fill_encrypted(
                conn,
                "ssh_items",
                "passphrase_encrypted",
                "passphrase_nonce",
                keep_id,
                duplicate_id,
            )?;
        }
        "cloud" => {
            fill_text(conn, "cloud_items", "region", keep_id, duplicate_id)?;
            fill_text(conn, "cloud_items", "notes", keep_id, duplicate_id)?;
        }
        "license" => {
            fill_text(conn, "license_items", "bound_email", keep_id, duplicate_id)?;
            fill_text(conn, "license_items", "notes", keep_id, duplicate_id)?;
            fill_number(conn, "license_items", "expiry_date", keep_id, duplicate_id)?;
        }
        "smtp" => {
            fill_text(conn, "smtp_items", "encryption", keep_id, duplicate_id)?;
            fill_text(conn, "smtp_items", "username", keep_id, duplicate_id)?;
            fill_text(conn, "smtp_items", "from_address", keep_id, duplicate_id)?;
            fill_text(conn, "smtp_items", "notes", keep_id, duplicate_id)?;
            fill_number(conn, "smtp_items", "port", keep_id, duplicate_id)?;
        }
        "env_var" => {
            fill_env_var_notes(conn, keep_id, duplicate_id)?;
            return merge_env_vars(conn, keep_id, duplicate_id);
        }
        _ => {}
    }
    Ok(0)
}

/// Copy the duplicate's custom fields the keep item does not have. Existing
/// keep fields always win, and each moved field becomes a fresh row so the
/// duplicate keeps its own copy in the trash.
fn move_custom_fields(conn: &Connection, keep_id: &str, duplicate_id: &str) -> Result<usize> {
    let rows: Vec<(String, Vec<u8>, Vec<u8>, i64)> = {
        let mut stmt = conn
            .prepare(
                "SELECT field_key, value_encrypted, value_nonce, sort_order
                 FROM custom_fields WHERE item_id = ?1 ORDER BY sort_order, field_key",
            )
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        let rows = stmt
            .query_map([duplicate_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?
    };

    let mut moved = 0;
    for (field_key, ciphertext, nonce, sort_order) in rows {
        let existing: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM custom_fields WHERE item_id = ?1 AND field_key = ?2",
                params![keep_id, field_key],
                |row| row.get(0),
            )
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        if existing > 0 {
            continue;
        }
        conn.execute(
            "INSERT INTO custom_fields
                 (id, item_id, field_key, value_encrypted, value_nonce, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Uuid::new_v4().to_string(),
                keep_id,
                field_key,
                ciphertext,
                nonce,
                sort_order
            ],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        moved += 1;
    }
    Ok(moved)
}

/// Archive `duplicate_ids` into `keep_id`: fill the keep item's empty fields,
/// carry the duplicates' tags / attachments / custom fields / secret history
/// over, then move the duplicates to the trash and record one audit entry.
/// Everything runs in a single transaction — a failure at any step leaves the
/// vault untouched.
pub fn archive_duplicates(
    conn: &Connection,
    keep_id: &str,
    duplicate_ids: &[String],
) -> Result<MergeReport> {
    let (item_type, ordered) = validate_merge_request(conn, keep_id, duplicate_ids)?;

    with_tx(conn, |tx| {
        let mut report = MergeReport {
            kept_id: keep_id.to_string(),
            moved_tags: 0,
            moved_attachments: 0,
            moved_custom_fields: 0,
            moved_history: 0,
            trashed: 0,
        };

        for duplicate_id in &ordered {
            // Oldest first: the earliest duplicate fills the most fields and
            // later ones only contribute what is still empty.
            fill_missing_fields(tx, keep_id, duplicate_id, &item_type)?;
            report.moved_tags += tx
                .execute(
                    "INSERT OR IGNORE INTO item_tags (item_id, tag_id)
                     SELECT ?1, tag_id FROM item_tags WHERE item_id = ?2",
                    params![keep_id, duplicate_id],
                )
                .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
            report.moved_attachments += tx
                .execute(
                    "UPDATE item_attachments SET item_id = ?1 WHERE item_id = ?2",
                    params![keep_id, duplicate_id],
                )
                .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
            report.moved_custom_fields += move_custom_fields(tx, keep_id, duplicate_id)?;
            report.moved_history += tx
                .execute(
                    "UPDATE secret_history SET item_id = ?1 WHERE item_id = ?2",
                    params![keep_id, duplicate_id],
                )
                .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        }

        tx.execute(
            "UPDATE items SET updated_at = ?1 WHERE id = ?2",
            params![chrono_timestamp(), keep_id],
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

        // Soft delete through the shared path so the duplicates obey the same
        // trash invariants (original group remembered, `item.trash` audited).
        for duplicate_id in &ordered {
            delete_item(tx, duplicate_id)?;
            report.trashed += 1;
        }

        let title = item_title(tx, keep_id);
        log_event(
            tx,
            "item.duplicate_merged",
            Some(keep_id),
            title.as_deref(),
            Some(&format!("merged {} duplicate(s)", report.trashed)),
        );
        Ok(report)
    })
}
