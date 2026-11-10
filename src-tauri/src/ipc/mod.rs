//! Local-socket IPC for the `seckeybox` CLI.
//!
//! Protocol: one JSON request per line, one JSON response per line.
//!
//! Safety model:
//! - Read-only. The protocol can never unlock the vault or mutate it.
//! - A locked vault answers `{"ok":false,"error":"locked"}` for every
//!   command that touches secrets.
//! - The socket lives in the user's runtime/data directory and is chmod
//!   0600, so only the same user can connect.
//! - Contents are never logged or written to disk.

use serde_json::{json, Value};
use std::path::PathBuf;
use tauri::Manager;

use crate::db::{get_all_items, search_items, DbConnection};
use crate::state::VaultState;

pub const PROTOCOL_VERSION: &str = "1";

/// Where the CLI and the app meet. Kept in sync with `cli/src/main.rs`.
pub fn socket_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime.is_empty() {
            return PathBuf::from(runtime).join("seckeybox.sock");
        }
    }
    PathBuf::from(app_data_dir()).join("seckeybox.sock")
}

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn app_data_dir() -> String {
    #[cfg(target_os = "macos")]
    {
        format!("{}/Library/Application Support/com.seckeybox.app", home_dir())
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            if !data_home.is_empty() {
                return format!("{}/com.seckeybox.app", data_home);
            }
        }
        format!("{}/.local/share/com.seckeybox.app", home_dir())
    }
}

#[cfg(unix)]
pub fn spawn(app: tauri::AppHandle) {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    std::thread::spawn(move || {
        let path = socket_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // A leftover socket from a crashed run would make bind() fail.
        let _ = std::fs::remove_file(&path);

        let listener = match UnixListener::bind(&path) {
            Ok(listener) => listener,
            Err(e) => {
                eprintln!("[WARN] CLI socket bind failed at {:?}: {}", path, e);
                return;
            }
        };
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        eprintln!("[INFO] CLI socket listening at {:?}", path);

        for connection in listener.incoming() {
            match connection {
                Ok(stream) => {
                    let app = app.clone();
                    std::thread::spawn(move || {
                        let _ = handle_client(stream, &app);
                    });
                }
                Err(_) => continue,
            }
        }
    });
}

#[cfg(unix)]
fn handle_client(
    mut stream: std::os::unix::net::UnixStream,
    app: &tauri::AppHandle,
) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader, Read, Write};

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    // Cap the request at 64 KiB so a hostile/local client cannot exhaust memory.
    reader.by_ref().take(64 * 1024).read_line(&mut line)?;

    let response = respond(app, &line);
    let mut payload = serde_json::to_string(&response)
        .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"internal\"}".to_string());
    payload.push('\n');
    stream.write_all(payload.as_bytes())?;
    stream.flush()
}

fn error(code: &str) -> Value {
    json!({ "ok": false, "error": code })
}

fn respond(app: &tauri::AppHandle, line: &str) -> Value {
    respond_with(
        &app.state::<DbConnection>().inner().0,
        &app.state::<VaultState>(),
        line,
    )
}

/// Protocol handler shared by the socket listener and the unit tests.
/// `db` is the connection mutex; `vault` carries unlock state and the key.
pub fn respond_with(db: &std::sync::Mutex<rusqlite::Connection>, vault: &VaultState, line: &str) -> Value {
    let request: Value = match serde_json::from_str(line.trim()) {
        Ok(value) => value,
        Err(_) => return error("bad_request"),
    };
    let command = request.get("cmd").and_then(Value::as_str).unwrap_or("");
    let unlocked = vault.is_unlocked();
    let version = env!("CARGO_PKG_VERSION");

    match command {
        "status" => json!({
            "ok": true,
            "unlocked": unlocked,
            "version": version,
            "protocol": PROTOCOL_VERSION,
        }),
        "list" => {
            if !unlocked {
                return error("locked");
            }
            let conn = match db.lock() {
                Ok(conn) => conn,
                Err(_) => return error("internal"),
            };
            let items = match request.get("query").and_then(Value::as_str) {
                Some(query) if !query.trim().is_empty() => search_items(&conn, query.trim()),
                _ => get_all_items(&conn),
            };
            match items {
                Ok(items) => json!({
                    "ok": true,
                    "items": items
                        .iter()
                        .map(|item| json!({
                            "id": item.id,
                            "title": item.title,
                            "type": item.item_type,
                            "group_id": item.group_id,
                        }))
                        .collect::<Vec<_>>(),
                }),
                Err(e) => json!({ "ok": false, "error": e.to_string() }),
            }
        }
        "get" => {
            if !unlocked {
                return error("locked");
            }
            match read_item(db, vault, &request) {
                Ok(value) => value,
                Err(value) => value,
            }
        }
        "totp" => {
            if !unlocked {
                return error("locked");
            }
            totp_response(db, vault, &request)
        }
        _ => error("unknown_command"),
    }
}

/// Resolve a request to a decrypted item and its fields.
fn read_item(
    db: &std::sync::Mutex<rusqlite::Connection>,
    vault: &VaultState,
    request: &Value,
) -> std::result::Result<Value, Value> {
    let key = vault.get_master_key().ok_or_else(|| error("locked"))?;
    let conn = db.lock().map_err(|_| error("internal"))?;

    let item_id = match resolve_item_id(&conn, request)? {
        Ok(id) => id,
        Err(payload) => return Err(payload),
    };

    let detail = crate::commands::load_item_detail(&conn, &item_id, key.as_bytes())
        .map_err(|e| json!({ "ok": false, "error": e.to_string() }))?;

    let (item_type, title, id, fields) = detail_payload(&detail)
        .map_err(|e| json!({ "ok": false, "error": e.to_string() }))?;

    Ok(json!({
        "ok": true,
        "item": { "id": id, "title": title, "type": item_type, "fields": fields },
    }))
}

fn totp_response(
    db: &std::sync::Mutex<rusqlite::Connection>,
    vault: &VaultState,
    request: &Value,
) -> Value {
    let key = match vault.get_master_key() {
        Some(key) => key,
        None => return error("locked"),
    };
    let conn = match db.lock() {
        Ok(conn) => conn,
        Err(_) => return error("internal"),
    };

    let item_id = match resolve_item_id(&conn, request) {
        Ok(Ok(id)) => id,
        Ok(Err(payload)) => return payload,
        Err(_) => return error("internal"),
    };

    let detail = match crate::db::get_account_item_detail(&conn, &item_id, key.as_bytes()) {
        Ok(detail) => detail,
        Err(_) => return error("not_found"),
    };
    let secret = match detail.totp_secret {
        Some(secret) => secret,
        None => return error("no_totp_secret"),
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let code = match crate::crypto::generate(&secret, timestamp) {
        Ok(code) => code,
        Err(e) => return json!({ "ok": false, "error": e.to_string() }),
    };

    json!({
        "ok": true,
        "code": code,
        "seconds_remaining": crate::crypto::seconds_remaining(timestamp),
    })
}

/// Resolve by explicit id, or by unique title. Ambiguous titles return the
/// candidate list so the CLI can ask for `--id`.
fn resolve_item_id(
    conn: &rusqlite::Connection,
    request: &Value,
) -> std::result::Result<std::result::Result<String, Value>, Value> {
    if let Some(id) = request.get("id").and_then(Value::as_str) {
        return Ok(Ok(id.to_string()));
    }
    let name = match request.get("name").and_then(Value::as_str) {
        Some(name) if !name.trim().is_empty() => name.trim(),
        _ => return Ok(Err(error("name_required"))),
    };

    let mut stmt = match conn.prepare("SELECT id, title, type FROM items WHERE title = ?1") {
        Ok(stmt) => stmt,
        Err(_) => return Err(error("internal")),
    };
    let matches: Vec<(String, String, String)> = match stmt
        .query_map([name], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
    {
        Ok(rows) => rows.filter_map(|row| row.ok()).collect(),
        Err(_) => return Err(error("internal")),
    };

    match matches.len() {
        0 => Ok(Err(error("not_found"))),
        1 => Ok(Ok(matches[0].0.clone())),
        _ => {
            let candidates: Vec<Value> = matches
                .iter()
                .map(|(id, title, item_type)| {
                    json!({ "id": id, "title": title, "type": item_type })
                })
                .collect();
            Ok(Err(json!({ "ok": false, "error": "ambiguous", "candidates": candidates })))
        }
    }
}

type DetailPayload = (String, String, String, Value);

/// Serialize an item detail into (type, title, id, fields). Structural keys
/// and notes/history are dropped; env_vars become a flat key -> value map.
fn detail_payload(detail: &crate::db::ItemDetail) -> crate::Result<DetailPayload> {
    let value = serde_json::to_value(detail)
        .map_err(|e| crate::error::VaultError::Validation(e.to_string()))?;
    let mut object = match value {
        Value::Object(object) => object,
        _ => return Err(crate::error::VaultError::Validation("invalid detail".into())),
    };

    let item_type = object
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let title = object
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    for key in [
        "type",
        "id",
        "group_id",
        "title",
        "icon",
        "is_favorite",
        "created_at",
        "updated_at",
        "notes",
        "secret_history",
    ] {
        object.remove(key);
    }

    // env_var stores variables as an array; flatten it so `--field KEY` works.
    if item_type == "env_var" {
        let mut flat = serde_json::Map::new();
        if let Some(Value::Array(variables)) = object.remove("variables") {
            for variable in variables {
                if let (Some(key), Some(value)) = (
                    variable.get("key").and_then(Value::as_str),
                    variable.get("value").and_then(Value::as_str),
                ) {
                    flat.insert(key.to_string(), Value::String(value.to_string()));
                }
            }
        }
        object = flat;
    }

    Ok((item_type, title, id, Value::Object(object)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{derive_key, SecureKey};
    use crate::db::{create_account_item, create_env_var_item, enable_foreign_keys, init_schema, insert_builtin_groups};
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn setup() -> (Mutex<Connection>, VaultState, [u8; 32]) {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        insert_builtin_groups(&conn).unwrap();
        enable_foreign_keys(&conn).unwrap();
        let key = derive_key("test-master-password", None).unwrap().key;
        (Mutex::new(conn), VaultState::new(), key)
    }

    fn call(db: &Mutex<Connection>, vault: &VaultState, request: Value) -> Value {
        respond_with(db, vault, &request.to_string())
    }

    #[test]
    fn locked_vault_refuses_every_secret_command() {
        let (db, vault, _key) = setup();
        let status = call(&db, &vault, json!({ "cmd": "status" }));
        assert_eq!(status["unlocked"], json!(false));

        for request in [
            json!({ "cmd": "list" }),
            json!({ "cmd": "get", "name": "anything" }),
            json!({ "cmd": "totp", "name": "anything" }),
        ] {
            let response = call(&db, &vault, request);
            assert_eq!(response["ok"], json!(false));
            assert_eq!(response["error"], json!("locked"));
        }
    }

    #[test]
    fn get_by_title_returns_decrypted_fields() {
        let (db, vault, key) = setup();
        {
            let conn = db.lock().unwrap();
            create_account_item(
                &conn,
                "built-in-accounts",
                "GitHub",
                "octocat",
                "s3cret-pass",
                Some("https://github.com"),
                None,
                Some("JBSWY3DPEHPK3PXP"),
                &key,
            )
            .unwrap();
        }
        vault.unlock(SecureKey::new(key));

        let response = call(&db, &vault, json!({ "cmd": "get", "name": "GitHub" }));
        assert_eq!(response["ok"], json!(true));
        assert_eq!(response["item"]["type"], json!("account"));
        assert_eq!(response["item"]["fields"]["password"], json!("s3cret-pass"));
        assert_eq!(response["item"]["fields"]["username"], json!("octocat"));
        assert_eq!(response["item"]["fields"]["website"], json!("https://github.com"));

        let totp = call(&db, &vault, json!({ "cmd": "totp", "name": "GitHub" }));
        assert_eq!(totp["ok"], json!(true));
        assert_eq!(totp["code"].as_str().unwrap().len(), 6);
    }

    #[test]
    fn duplicate_titles_are_ambiguous_with_candidates() {
        let (db, vault, key) = setup();
        {
            let conn = db.lock().unwrap();
            for password in ["one-pass", "two-pass"] {
                create_account_item(
                    &conn,
                    "built-in-accounts",
                    "Shared",
                    "user",
                    password,
                    None,
                    None,
                    None,
                    &key,
                )
                .unwrap();
            }
        }
        vault.unlock(SecureKey::new(key));

        let response = call(&db, &vault, json!({ "cmd": "get", "name": "Shared" }));
        assert_eq!(response["ok"], json!(false));
        assert_eq!(response["error"], json!("ambiguous"));
        assert_eq!(response["candidates"].as_array().unwrap().len(), 2);

        // --id disambiguates.
        let id = response["candidates"][0]["id"].as_str().unwrap().to_string();
        let by_id = call(&db, &vault, json!({ "cmd": "get", "id": id }));
        assert_eq!(by_id["ok"], json!(true));
    }

    #[test]
    fn list_searches_and_env_vars_flatten_to_fields() {
        let (db, vault, key) = setup();
        let env_item_id;
        {
            let conn = db.lock().unwrap();
            create_account_item(
                &conn,
                "built-in-accounts",
                "GitHub",
                "octocat",
                "pw",
                None,
                None,
                None,
                &key,
            )
            .unwrap();
            env_item_id = create_env_var_item(
                &conn,
                "built-in-env-vars",
                "prod env",
                &[
                    ("DATABASE_URL".to_string(), "postgres://prod".to_string()),
                    ("REDIS_URL".to_string(), "redis://prod".to_string()),
                ],
                None,
                &key,
            )
            .unwrap();
        }
        vault.unlock(SecureKey::new(key));

        let list = call(&db, &vault, json!({ "cmd": "list", "query": "Git" }));
        assert_eq!(list["ok"], json!(true));
        assert_eq!(list["items"].as_array().unwrap().len(), 1);
        assert_eq!(list["items"][0]["title"], json!("GitHub"));

        let env = call(&db, &vault, json!({ "cmd": "get", "name": "prod env" }));
        assert_eq!(env["ok"], json!(true));
        assert_eq!(env["item"]["type"], json!("env_var"));
        assert_eq!(
            env["item"]["fields"]["DATABASE_URL"],
            json!("postgres://prod")
        );
        assert!(env["item"]["fields"]["variables"].is_null());
        assert!(env_item_id.len() > 0);
    }

    #[test]
    fn unknown_names_and_commands_are_rejected() {
        let (db, vault, key) = setup();
        vault.unlock(SecureKey::new(key));

        let missing = call(&db, &vault, json!({ "cmd": "get", "name": "nope" }));
        assert_eq!(missing["error"], json!("not_found"));

        let unknown = call(&db, &vault, json!({ "cmd": "delete-everything" }));
        assert_eq!(unknown["error"], json!("unknown_command"));

        let bad = respond_with(&db, &vault, "not json");
        assert_eq!(bad["error"], json!("bad_request"));
    }
}
