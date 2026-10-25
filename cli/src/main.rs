//! `seckeybox` — command-line client for the running SecKeyBox app.
//!
//! The app listens on a local socket (chmod 0600) and answers one JSON object
//! per line. This CLI never decrypts anything itself: it only works while the
//! app is running and the vault is unlocked, which keeps the master key in one
//! process.
//!
//! Exit codes: 0 success, 1 locked, 2 not found / bad usage, 3 cannot connect.

use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::ExitCode;

const PROTOCOL_VERSION: &str = "1";

struct CliError {
    code: u8,
    message: String,
}

impl CliError {
    fn new(code: u8, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
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

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn socket_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        if !runtime.is_empty() {
            return PathBuf::from(runtime).join("seckeybox.sock");
        }
    }
    PathBuf::from(app_data_dir()).join("seckeybox.sock")
}

fn print_help() {
    println!(
        "seckeybox-cli {version} — read secrets from a running SecKeyBox vault

USAGE:
  seckeybox-cli status
  seckeybox-cli list [QUERY] [--json]
  seckeybox-cli get <NAME> [--field FIELD] [--json]
  seckeybox-cli get --id <ID> [--field FIELD] [--json]
  seckeybox-cli totp <NAME> [--json]

EXAMPLES:
  export DB_PASSWORD=$(seckeybox-cli get \"prod database\")
  seckeybox-cli get \"GitHub\" --field username
  seckeybox-cli totp \"GitHub\"

The vault must be unlocked in the app; the CLI cannot unlock it.
Exit codes: 0 ok, 1 locked, 2 not found / usage, 3 app not reachable.",
        version = env!("CARGO_PKG_VERSION")
    );
}

#[cfg(unix)]
fn send(request: &Value) -> Result<Value, CliError> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;

    let path = socket_path();
    let mut stream = UnixStream::connect(&path).map_err(|e| {
        CliError::new(
            3,
            format!(
                "cannot connect to SecKeyBox ({}): {}. Is the app running and unlocked?",
                path.display(),
                e
            ),
        )
    })?;

    let mut payload = serde_json::to_string(request)
        .map_err(|e| CliError::new(2, format!("failed to encode request: {}", e)))?;
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .map_err(|e| CliError::new(3, format!("failed to send request: {}", e)))?;

    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .map_err(|e| CliError::new(3, format!("failed to read response: {}", e)))?;
    if line.trim().is_empty() {
        return Err(CliError::new(3, "the app closed the connection without a reply"));
    }
    serde_json::from_str(line.trim())
        .map_err(|e| CliError::new(3, format!("invalid response from the app: {}", e)))
}

#[cfg(windows)]
fn send(_request: &Value) -> Result<Value, CliError> {
    Err(CliError::new(
        3,
        "the SecKeyBox CLI is not supported on Windows yet (the app uses a named pipe that this build does not implement)",
    ))
}

fn check_ok(response: &Value) -> Result<(), CliError> {
    if response.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(());
    }
    let code = response.get("error").and_then(Value::as_str).unwrap_or("unknown");
    match code {
        "locked" => Err(CliError::new(
            1,
            "the vault is locked — unlock SecKeyBox first",
        )),
        "not_found" => Err(CliError::new(2, "no item matched")),
        "ambiguous" => {
            let mut message = String::from("several items match; pass --id with one of:\n");
            if let Some(candidates) = response.get("candidates").and_then(Value::as_array) {
                for candidate in candidates {
                    message.push_str(&format!(
                        "  {}\t{}\t{}\n",
                        candidate.get("id").and_then(Value::as_str).unwrap_or(""),
                        candidate.get("type").and_then(Value::as_str).unwrap_or(""),
                        candidate.get("title").and_then(Value::as_str).unwrap_or("")
                    ));
                }
            }
            Err(CliError::new(2, message))
        }
        "no_totp_secret" => Err(CliError::new(
            2,
            "this item has no TOTP secret configured",
        )),
        "name_required" => Err(CliError::new(2, "pass an item name or --id")),
        other => Err(CliError::new(2, format!("request failed: {}", other))),
    }
}

/// Field used when `get` is called without `--field`.
fn primary_value(item_type: &str, fields: &Value) -> Option<String> {
    let get = |name: &str| -> Option<String> {
        fields
            .get(name)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    match item_type {
        "account" => get("password"),
        "api_key" => get("key_value"),
        "env_var" => {
            let map = fields.as_object()?;
            if map.len() == 1 {
                map.values().next().and_then(Value::as_str).map(str::to_string)
            } else {
                None
            }
        }
        "database" => get("password").or_else(|| get("connection_url")),
        "ssh" => get("password").or_else(|| get("passphrase")),
        "cloud" => get("secret"),
        "license" => get("license_key"),
        "smtp" => get("password"),
        _ => None,
    }
}

fn field_value(fields: &Value, field: &str) -> Option<String> {
    match fields.get(field) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(Value::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

struct ParsedArgs {
    positional: Vec<String>,
    id: Option<String>,
    field: Option<String>,
    json: bool,
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, CliError> {
    let mut parsed = ParsedArgs {
        positional: Vec::new(),
        id: None,
        field: None,
        json: false,
    };
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => parsed.json = true,
            "--id" => {
                parsed.id = Some(
                    iter.next()
                        .ok_or_else(|| CliError::new(2, "--id requires a value"))?
                        .clone(),
                )
            }
            "--field" | "-f" => {
                parsed.field = Some(
                    iter.next()
                        .ok_or_else(|| CliError::new(2, "--field requires a value"))?
                        .clone(),
                )
            }
            other if other.starts_with('-') => {
                return Err(CliError::new(2, format!("unknown option: {}", other)))
            }
            other => parsed.positional.push(other.to_string()),
        }
    }
    Ok(parsed)
}

fn run(args: &[String]) -> Result<(), CliError> {
    let command = args.first().map(String::as_str).unwrap_or("");
    match command {
        "" | "help" | "--help" | "-h" => {
            print_help();
            return Ok(());
        }
        "--version" | "-V" => {
            println!("seckeybox-cli {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        _ => {}
    }

    let parsed = parse_args(&args[1..])?;

    match command {
        "status" => {
            let response = send(&json!({ "cmd": "status", "protocol": PROTOCOL_VERSION }))?;
            check_ok(&response)?;
            if parsed.json {
                println!("{}", response);
            } else {
                let unlocked = response
                    .get("unlocked")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                println!("{}", if unlocked { "unlocked" } else { "locked" });
            }
            Ok(())
        }
        "list" => {
            let query = parsed.positional.first().cloned().unwrap_or_default();
            let response = send(&json!({ "cmd": "list", "query": query }))?;
            check_ok(&response)?;
            let items = response
                .get("items")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if parsed.json {
                println!("{}", Value::Array(items));
            } else {
                for item in items {
                    println!(
                        "{}\t{}\t{}",
                        item.get("id").and_then(Value::as_str).unwrap_or(""),
                        item.get("type").and_then(Value::as_str).unwrap_or(""),
                        item.get("title").and_then(Value::as_str).unwrap_or("")
                    );
                }
            }
            Ok(())
        }
        "get" => {
            let name = parsed
                .positional
                .first()
                .cloned()
                .ok_or_else(|| CliError::new(2, "get requires an item name (or --id)"))?;
            let response = send(&json!({
                "cmd": "get",
                "name": name,
                "id": parsed.id,
            }))?;
            check_ok(&response)?;

            let item = response
                .get("item")
                .ok_or_else(|| CliError::new(3, "the app returned no item"))?;
            if parsed.json {
                println!("{}", item);
                return Ok(());
            }

            let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
            let fields = item.get("fields").cloned().unwrap_or(Value::Null);

            let value = match parsed.field.as_deref() {
                Some(field) => field_value(&fields, field).ok_or_else(|| {
                    let available: Vec<String> = fields
                        .as_object()
                        .map(|map| map.keys().cloned().collect())
                        .unwrap_or_default();
                    CliError::new(
                        2,
                        format!("field '{}' not found; available: {}", field, available.join(", ")),
                    )
                })?,
                None => primary_value(item_type, &fields).ok_or_else(|| {
                    let available: Vec<String> = fields
                        .as_object()
                        .map(|map| map.keys().cloned().collect())
                        .unwrap_or_default();
                    CliError::new(
                        2,
                        format!(
                            "no default secret for this item; use --field (available: {})",
                            available.join(", ")
                        ),
                    )
                })?,
            };
            println!("{}", value);
            Ok(())
        }
        "totp" => {
            let name = parsed
                .positional
                .first()
                .cloned()
                .ok_or_else(|| CliError::new(2, "totp requires an item name (or --id)"))?;
            let response = send(&json!({
                "cmd": "totp",
                "name": name,
                "id": parsed.id,
            }))?;
            check_ok(&response)?;
            if parsed.json {
                println!("{}", response);
            } else {
                println!(
                    "{}",
                    response.get("code").and_then(Value::as_str).unwrap_or("")
                );
            }
            Ok(())
        }
        other => Err(CliError::new(2, format!("unknown command: {}", other))),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("seckeybox-cli: {}", error.message);
            ExitCode::from(error.code)
        }
    }
}
