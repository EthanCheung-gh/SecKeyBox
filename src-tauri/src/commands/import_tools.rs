//! Bulk import from other password managers and .env files.
//!
//! Supported CSV shapes: Chrome, Bitwarden, 1Password, LastPass and a
//! generic name,url,username,password,notes layout. Bitwarden's folder
//! column (or any explicit folder/group column) becomes a group per unique
//! folder name. Bitwarden's login_totp column feeds the new account TOTP.

use tauri::State;

use crate::db::{
    create_account_item, create_env_var_item, get_all_groups, DbConnection,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ImportToolsResult {
    pub groups_imported: usize,
    pub items_imported: usize,
    pub items_skipped: usize,
}

/// A row normalized from any of the supported CSV shapes.
struct CsvEntry {
    title: String,
    url: Option<String>,
    username: Option<String>,
    password: String,
    totp: Option<String>,
    notes: Option<String>,
    folder: Option<String>,
}

fn header_index(header: &csv::StringRecord, aliases: &[&str]) -> Option<usize> {
    for (i, col) in header.iter().enumerate() {
        let col = col.trim().to_ascii_lowercase();
        if aliases.iter().any(|a| *a == col) {
            return Some(i);
        }
    }
    None
}

fn entry_from_csv(header: &csv::StringRecord, rec: &csv::StringRecord) -> Option<CsvEntry> {
    let get = |aliases: &[&str]| -> Option<String> {
        header_index(header, aliases)
            .and_then(|i| rec.get(i))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let title = get(&["name", "title"])?;
    let password = get(&["password", "login_password"])?;
    Some(CsvEntry {
        title,
        url: get(&["url", "login_uri", "website", "hostname"]),
        username: get(&["username", "login_username", "user_name"]),
        password,
        totp: get(&["login_totp", "totp", "otpauth"]),
        notes: get(&["notes", "note", "extra", "comments"]),
        folder: get(&["folder", "grouping", "group"]),
    })
}

/// Import accounts from a CSV export of a supported password manager.
pub fn import_csv_rows(
    conn: &rusqlite::Connection,
    key: &[u8; 32],
    data: &str,
    group_id: &str,
) -> Result<ImportToolsResult> {
    let mut rdr = csv::Reader::from_reader(data.as_bytes());
    let Some(header) = rdr.headers().ok().cloned() else {
        return Err(VaultError::DatabaseError("Empty CSV file".to_string()));
    };

    let mut result = ImportToolsResult {
        groups_imported: 0,
        items_imported: 0,
        items_skipped: 0,
    };

    for rec in rdr.records() {
        let rec = rec.map_err(|e| VaultError::DatabaseError(e.to_string()))?;
        let Some(entry) = entry_from_csv(&header, &rec) else {
            result.items_skipped += 1;
            continue;
        };

        let target_group = match entry.folder.as_deref() {
            Some(folder) if !folder.is_empty() => {
                let existing = get_all_groups(conn)?
                    .into_iter()
                    .find(|g| g.name == folder);
                match existing {
                    Some(g) => g.id,
                    None => {
                        let group =
                            crate::db::create_group(conn, folder, Some("📂"), None, None)?;
                        result.groups_imported += 1;
                        group.id
                    }
                }
            }
            _ => group_id.to_string(),
        };

        create_account_item(
            conn,
            &target_group,
            &entry.title,
            entry.username.as_deref().unwrap_or(""),
            &entry.password,
            entry.url.as_deref(),
            entry.notes.as_deref(),
            entry.totp.as_deref(),
            key,
        )
        .map_err(|e| VaultError::DatabaseError(format!("row '{}': {}", entry.title, e)))?;
        result.items_imported += 1;
    }

    Ok(result)
}

#[tauri::command]
pub fn import_csv(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    data: String,
    group_id: String,
) -> Result<ImportToolsResult> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    import_csv_rows(&conn, key.as_bytes(), &data, &group_id)
}

/// Import KEY=VALUE pairs as one env-var item.
pub fn import_env_rows(
    conn: &rusqlite::Connection,
    key: &[u8; 32],
    group_id: &str,
    title: &str,
    content: &str,
) -> Result<ImportToolsResult> {
    let mut pairs = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((k, v)) = line.split_once('=') else { continue };
        let key = k.trim();
        let mut value = v.trim();
        for q in ['"', '\''] {
            if value.len() >= 2 && value.starts_with(q) && value.ends_with(q) {
                value = &value[1..value.len() - 1];
            }
        }
        if !key.is_empty() {
            pairs.push((key.to_string(), value.to_string()));
        }
    }
    if pairs.is_empty() {
        return Err(VaultError::DatabaseError(
            "No KEY=VALUE pairs found in the file".to_string(),
        ));
    }

    create_env_var_item(conn, group_id, title, &pairs, None, key)?;

    Ok(ImportToolsResult {
        groups_imported: 0,
        items_imported: 1,
        items_skipped: 0,
    })
}

#[tauri::command]
pub fn import_env(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    content: String,
) -> Result<ImportToolsResult> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    import_env_rows(&conn, key.as_bytes(), &group_id, &title, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::derive_key;
    use crate::db::{enable_foreign_keys, get_all_items, init_schema, insert_builtin_groups};
    use rusqlite::Connection;

    const CHROME_CSV: &str = "name,url,username,password,note\nGitHub,https://github.com,octo,gh-pass-1,work account\n";
    const BITWARDEN_CSV: &str = "folder,name,login_uri,login_username,login_password,login_totp,notes\nSocial,X (Twitter),https://x.com,user1,x-pass-123,JBSWY3DPEHPK3PXP,social notes\nWork,GitLab,https://gitlab.com,ethan,gl-pass-456,,gitlab\n";

    fn setup() -> (Connection, [u8; 32]) {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        insert_builtin_groups(&conn).unwrap();
        enable_foreign_keys(&conn).unwrap();
        let key = derive_key("import-pw-1", None).unwrap().key;
        (conn, key)
    }

    #[test]
    fn import_chrome_csv() {
        let (conn, key) = setup();
        let result = import_csv_rows(&conn, &key, CHROME_CSV, "built-in-accounts").unwrap();
        assert_eq!(result.items_imported, 1);
        assert_eq!(result.items_skipped, 0);

        let items = get_all_items(&conn).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "GitHub");
        assert_eq!(items[0].subtitle, "octo");
    }

    #[test]
    fn import_bitwarden_csv_creates_folders_as_groups() {
        let (conn, key) = setup();
        let result = import_csv_rows(&conn, &key, BITWARDEN_CSV, "built-in-accounts").unwrap();
        assert_eq!(result.items_imported, 2);
        assert_eq!(result.groups_imported, 2, "Social + Work groups created");

        let items = get_all_items(&conn).unwrap();
        assert_eq!(items.len(), 2);
        let groups = get_all_groups(&conn).unwrap();
        assert!(groups.iter().any(|g| g.name == "Social"));
        assert!(groups.iter().any(|g| g.name == "Work"));

        // Bitwarden's login_totp is stored as the account TOTP secret
        let social = items.iter().find(|i| i.title == "X (Twitter)").unwrap();
        let detail = crate::db::get_account_item_detail(&conn, &social.id, &key).unwrap();
        assert_eq!(detail.totp_secret.as_deref(), Some("JBSWY3DPEHPK3PXP"));
    }

    #[test]
    fn import_env_parses_pairs() {
        let (conn, key) = setup();
        let content = "# comment\nFOO=bar\nexport BAZ=\"quoted value\"\nnot a pair\n";
        let result = import_env_rows(&conn, &key, "built-in-env-vars", "App Env", content).unwrap();
        assert_eq!(result.items_imported, 1);

        let items = get_all_items(&conn).unwrap();
        assert_eq!(items[0].item_type, "env_var");
        let detail = crate::db::get_env_var_item_detail(&conn, &items[0].id, &key).unwrap();
        assert_eq!(detail.variables.len(), 2);
        assert_eq!(detail.variables[0].key, "FOO");
        assert_eq!(detail.variables[0].value, "bar");
        assert_eq!(detail.variables[1].key, "BAZ");
        assert_eq!(detail.variables[1].value, "quoted value");
    }

    #[test]
    fn import_env_rejects_empty_content() {
        let (conn, key) = setup();
        assert!(import_env_rows(&conn, &key, "built-in-env-vars", "Empty", "# nothing\n").is_err());
    }
}
