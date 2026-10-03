use crate::db::{
    create_account_item, create_api_key_item, create_cloud_item, create_database_item,
    create_env_var_item, create_license_item, create_smtp_item, create_ssh_item, delete_item,
    get_account_item_detail, get_all_items, get_api_key_item_detail, get_cloud_item_detail,
    get_database_item_detail, get_env_var_item_detail, get_items_by_group,
    get_license_item_detail, get_smtp_item_detail, get_ssh_item_detail, toggle_favorite,
    update_account_item, update_api_key_item, update_cloud_item, update_database_item,
    update_env_var_item, update_license_item, update_smtp_item, update_ssh_item, DbConnection,
    ItemDetail, ItemSummary,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarPairInput {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub group_id: String,
    pub title: String,
    pub key_name: String,
    pub key_value: String,
    pub endpoint: Option<String>,
    pub auth_method: Option<String>,
    pub notes: Option<String>,
}

#[tauri::command]
pub fn get_all_items_cmd(db: State<DbConnection>) -> Result<Vec<ItemSummary>> {
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_items(&conn)
}

#[tauri::command]
pub fn get_items_by_group_cmd(
    db: State<DbConnection>,
    group_id: String,
) -> Result<Vec<ItemSummary>> {
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_items_by_group(&conn, &group_id)
}

#[tauri::command]
pub fn get_item_detail(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<ItemDetail> {
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

    let item_type: String = conn
        .query_row("SELECT type FROM items WHERE id = ?1", [&id], |row| {
            row.get(0)
        })
        .map_err(|_| VaultError::ItemNotFound)?;

    match item_type.as_str() {
        "account" => {
            get_account_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::Account(d))
        }
        "api_key" => {
            get_api_key_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::ApiKey(d))
        }
        "env_var" => {
            get_env_var_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::EnvVar(d))
        }
        "database" => {
            get_database_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::Database(d))
        }
        "ssh" => get_ssh_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::Ssh(d)),
        "cloud" => {
            get_cloud_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::Cloud(d))
        }
        "license" => {
            get_license_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::License(d))
        }
        "smtp" => get_smtp_item_detail(&conn, &id, key.as_bytes()).map(|d| ItemDetail::Smtp(d)),
        _ => Err(VaultError::ItemNotFound),
    }
}

#[tauri::command]
pub fn create_new_account_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    username: String,
    password: String,
    website: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    eprintln!(
        "[DEBUG] create_new_account_item: called with group_id={}, title={}",
        group_id, title
    );

    if !vault_state.is_unlocked() {
        eprintln!("[DEBUG] create_new_account_item: vault is locked!");
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        eprintln!("[DEBUG] create_new_account_item: invalid title");
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if username.is_empty() || username.len() > 100 {
        eprintln!("[DEBUG] create_new_account_item: invalid username");
        return Err(VaultError::DatabaseError(
            "Username must be 1-100 characters".to_string(),
        ));
    }
    if password.is_empty() || password.len() > 1000 {
        eprintln!("[DEBUG] create_new_account_item: invalid password");
        return Err(VaultError::DatabaseError(
            "Password must be 1-1000 characters".to_string(),
        ));
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;

    eprintln!("[DEBUG] create_new_account_item: got master key");

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    eprintln!("[DEBUG] create_new_account_item: got db connection, calling create_account_item");

    let result = create_account_item(
        &conn,
        &group_id,
        &title,
        &username,
        &password,
        website.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    );

    eprintln!("[DEBUG] create_new_account_item: result = {:?}", result);
    result
}

#[tauri::command]
pub fn update_existing_account_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    username: String,
    password: Option<String>,
    website: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Username must be 1-100 characters".to_string(),
        ));
    }
    if let Some(ref pwd) = password {
        if pwd.is_empty() || pwd.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be 1-1000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    update_account_item(
        &conn,
        &id,
        &title,
        &username,
        password.as_deref(),
        website.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn create_new_api_key_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    request: CreateApiKeyRequest,
) -> Result<String> {
    eprintln!(
        "[DEBUG] create_new_api_key_item: group_id={}, title={}, key_name={}",
        request.group_id, request.title, request.key_name
    );

    if !vault_state.is_unlocked() {
        eprintln!("[DEBUG] create_new_api_key_item: vault locked");
        return Err(VaultError::VaultLocked);
    }

    if request.title.is_empty() || request.title.len() > 100 {
        eprintln!("[DEBUG] create_new_api_key_item: invalid title");
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if request.key_name.is_empty() || request.key_name.len() > 100 {
        eprintln!("[DEBUG] create_new_api_key_item: invalid key_name");
        return Err(VaultError::DatabaseError(
            "Key name must be 1-100 characters".to_string(),
        ));
    }
    if request.key_value.is_empty() || request.key_value.len() > 50000 {
        eprintln!("[DEBUG] create_new_api_key_item: invalid key_value");
        return Err(VaultError::DatabaseError(
            "Key value must be 1-50000 characters".to_string(),
        ));
    }
    if let Some(ref e) = request.endpoint {
        if e.len() > 500 {
            return Err(VaultError::DatabaseError(
                "Endpoint must be under 500 characters".to_string(),
            ));
        }
    }
    if let Some(ref am) = request.auth_method {
        if am.len() > 20 {
            return Err(VaultError::DatabaseError(
                "Auth method must be under 20 characters".to_string(),
            ));
        }
    }
    if let Some(ref n) = request.notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Notes must be under 5000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;

    eprintln!("[DEBUG] create_new_api_key_item: got master key");

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    eprintln!("[DEBUG] create_new_api_key_item: calling create_api_key_item");

    let result = create_api_key_item(
        &conn,
        &request.group_id,
        &request.title,
        &request.key_name,
        &request.key_value,
        request.endpoint.as_deref(),
        request.auth_method.as_deref(),
        request.notes.as_deref(),
        key.as_bytes(),
    );

    eprintln!("[DEBUG] create_new_api_key_item: result = {:?}", result);
    result
}

#[tauri::command]
pub fn update_existing_api_key_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    key_name: String,
    key_value: Option<String>,
    endpoint: Option<String>,
    auth_method: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if key_name.is_empty() || key_name.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Key name must be 1-100 characters".to_string(),
        ));
    }
    if let Some(ref kv) = key_value {
        if kv.is_empty() || kv.len() > 50000 {
            return Err(VaultError::DatabaseError(
                "Key value must be 1-50000 characters".to_string(),
            ));
        }
    }
    if let Some(ref e) = endpoint {
        if e.len() > 500 {
            return Err(VaultError::DatabaseError(
                "Endpoint must be under 500 characters".to_string(),
            ));
        }
    }
    if let Some(ref am) = auth_method {
        if am.len() > 20 {
            return Err(VaultError::DatabaseError(
                "Auth method must be under 20 characters".to_string(),
            ));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Notes must be under 5000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    update_api_key_item(
        &conn,
        &id,
        &title,
        &key_name,
        key_value.as_deref(),
        endpoint.as_deref(),
        auth_method.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn delete_existing_item(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_item(&conn, &id)
}

#[tauri::command]
pub fn toggle_item_favorite(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    toggle_favorite(&conn, &id)
}

#[tauri::command]
pub fn create_new_env_var_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    variables: Vec<EnvVarPairInput>,
    notes: Option<String>,
) -> Result<String> {
    eprintln!(
        "[DEBUG] create_new_env_var_item: group_id={}, title={}, variables count={}",
        group_id,
        title,
        variables.len()
    );

    if !vault_state.is_unlocked() {
        eprintln!("[DEBUG] create_new_env_var_item: vault locked");
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        eprintln!("[DEBUG] create_new_env_var_item: invalid title");
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if variables.is_empty() {
        eprintln!("[DEBUG] create_new_env_var_item: no variables");
        return Err(VaultError::DatabaseError(
            "At least one variable is required".to_string(),
        ));
    }
    for var in &variables {
        if var.key.is_empty() || var.key.len() > 100 {
            return Err(VaultError::DatabaseError(
                "Variable key must be 1-100 characters".to_string(),
            ));
        }
        if var.value.is_empty() || var.value.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Variable value must be 1-5000 characters".to_string(),
            ));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Notes must be under 5000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let vars: Vec<(String, String)> = variables
        .iter()
        .map(|v| (v.key.clone(), v.value.clone()))
        .collect();

    create_env_var_item(
        &conn,
        &group_id,
        &title,
        &vars,
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_env_var_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    variables: Vec<EnvVarPairInput>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError(
            "Title must be 1-100 characters".to_string(),
        ));
    }
    if variables.is_empty() {
        return Err(VaultError::DatabaseError(
            "At least one variable is required".to_string(),
        ));
    }
    for var in &variables {
        if var.key.is_empty() || var.key.len() > 100 {
            return Err(VaultError::DatabaseError(
                "Variable key must be 1-100 characters".to_string(),
            ));
        }
        if var.value.is_empty() || var.value.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Variable value must be 1-5000 characters".to_string(),
            ));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError(
                "Notes must be under 5000 characters".to_string(),
            ));
        }
    }

    let key = vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)?;
    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    let vars: Vec<(String, String)> = variables
        .iter()
        .map(|v| (v.key.clone(), v.value.clone()))
        .collect();

    update_env_var_item(&conn, &id, &title, &vars, notes.as_deref(), key.as_bytes())
}

// ---------- shared validators for the new item categories ----------

fn check_unlocked_and_lock(
    vault_state: &State<VaultState>,
) -> std::result::Result<crate::crypto::SecureKey, VaultError> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }
    vault_state
        .get_master_key()
        .ok_or(VaultError::VaultLocked)
}

fn check_len(value: &str, max: usize, label: &str) -> Result<()> {
    if value.is_empty() || value.len() > max {
        return Err(VaultError::DatabaseError(format!(
            "{} must be 1-{} characters",
            label, max
        )));
    }
    Ok(())
}

fn check_opt_len(value: &Option<String>, max: usize, label: &str) -> Result<()> {
    if let Some(ref v) = value {
        if v.len() > max {
            return Err(VaultError::DatabaseError(format!(
                "{} must be under {} characters",
                label, max
            )));
        }
    }
    Ok(())
}

fn check_port(port: &Option<i64>) -> Result<()> {
    if let Some(p) = port {
        if !(0..=65535).contains(p) {
            return Err(VaultError::DatabaseError(
                "Port must be between 0 and 65535".to_string(),
            ));
        }
    }
    Ok(())
}

// ---------- database ----------

#[tauri::command]
pub fn create_new_database_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    db_type: String,
    host: String,
    port: Option<i64>,
    database_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    connection_url: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&db_type, 20, "Database type")?;
    check_len(&host, 255, "Host")?;
    check_port(&port)?;
    check_opt_len(&username, 100, "Username")?;
    if let Some(ref p) = password {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&connection_url, 2000, "Connection URL")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_database_item(
        &conn,
        &group_id,
        &title,
        &db_type,
        &host,
        port,
        database_name.as_deref(),
        username.as_deref(),
        password.as_deref(),
        connection_url.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_database_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    db_type: String,
    host: String,
    port: Option<i64>,
    database_name: Option<String>,
    username: Option<String>,
    password: Option<String>,
    connection_url: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&db_type, 20, "Database type")?;
    check_len(&host, 255, "Host")?;
    check_port(&port)?;
    check_opt_len(&username, 100, "Username")?;
    if let Some(ref p) = password {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&connection_url, 2000, "Connection URL")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_database_item(
        &conn,
        &id,
        &title,
        &db_type,
        &host,
        port,
        database_name.as_deref(),
        username.as_deref(),
        password.as_deref(),
        connection_url.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

// ---------- ssh ----------

#[tauri::command]
pub fn create_new_ssh_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    host: String,
    port: Option<i64>,
    username: String,
    password: Option<String>,
    key_path: Option<String>,
    passphrase: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&host, 255, "Host")?;
    check_port(&port)?;
    check_len(&username, 100, "Username")?;
    if let Some(ref p) = password {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&key_path, 500, "Key path")?;
    if let Some(ref p) = passphrase {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Passphrase must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_ssh_item(
        &conn,
        &group_id,
        &title,
        &host,
        port,
        &username,
        password.as_deref(),
        key_path.as_deref(),
        passphrase.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_ssh_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    host: String,
    port: Option<i64>,
    username: String,
    password: Option<String>,
    key_path: Option<String>,
    passphrase: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&host, 255, "Host")?;
    check_port(&port)?;
    check_len(&username, 100, "Username")?;
    if let Some(ref p) = password {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&key_path, 500, "Key path")?;
    if let Some(ref p) = passphrase {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Passphrase must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_ssh_item(
        &conn,
        &id,
        &title,
        &host,
        port,
        &username,
        password.as_deref(),
        key_path.as_deref(),
        passphrase.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

// ---------- cloud ----------

#[tauri::command]
pub fn create_new_cloud_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    provider: String,
    access_key_id: String,
    secret: String,
    region: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&provider, 30, "Provider")?;
    check_len(&access_key_id, 200, "Access Key ID")?;
    check_len(&secret, 1000, "Secret")?;
    check_opt_len(&region, 50, "Region")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_cloud_item(
        &conn,
        &group_id,
        &title,
        &provider,
        &access_key_id,
        &secret,
        region.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_cloud_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    provider: String,
    access_key_id: String,
    secret: Option<String>,
    region: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&provider, 30, "Provider")?;
    check_len(&access_key_id, 200, "Access Key ID")?;
    if let Some(ref s) = secret {
        if s.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Secret must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&region, 50, "Region")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_cloud_item(
        &conn,
        &id,
        &title,
        &provider,
        &access_key_id,
        secret.as_deref(),
        region.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

// ---------- license ----------

#[tauri::command]
pub fn create_new_license_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    software_name: String,
    license_key: String,
    bound_email: Option<String>,
    expiry_date: Option<i64>,
    notes: Option<String>,
) -> Result<String> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&software_name, 100, "Software name")?;
    check_len(&license_key, 1000, "License key")?;
    check_opt_len(&bound_email, 200, "Bound email")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_license_item(
        &conn,
        &group_id,
        &title,
        &software_name,
        &license_key,
        bound_email.as_deref(),
        expiry_date,
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_license_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    software_name: String,
    license_key: Option<String>,
    bound_email: Option<String>,
    expiry_date: Option<i64>,
    notes: Option<String>,
) -> Result<()> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&software_name, 100, "Software name")?;
    if let Some(ref k) = license_key {
        if k.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "License key must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&bound_email, 200, "Bound email")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_license_item(
        &conn,
        &id,
        &title,
        &software_name,
        license_key.as_deref(),
        bound_email.as_deref(),
        expiry_date,
        notes.as_deref(),
        key.as_bytes(),
    )
}

// ---------- smtp ----------

#[tauri::command]
pub fn create_new_smtp_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    host: String,
    port: Option<i64>,
    encryption: Option<String>,
    username: Option<String>,
    password: String,
    from_address: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&host, 255, "Host")?;
    check_port(&port)?;
    check_opt_len(&encryption, 10, "Encryption")?;
    check_opt_len(&username, 100, "Username")?;
    check_len(&password, 1000, "Password")?;
    check_opt_len(&from_address, 200, "From address")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_smtp_item(
        &conn,
        &group_id,
        &title,
        &host,
        port,
        encryption.as_deref(),
        username.as_deref(),
        &password,
        from_address.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_smtp_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    host: String,
    port: Option<i64>,
    encryption: Option<String>,
    username: Option<String>,
    password: Option<String>,
    from_address: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    let key = check_unlocked_and_lock(&vault_state)?;
    check_len(&title, 100, "Title")?;
    check_len(&host, 255, "Host")?;
    check_port(&port)?;
    check_opt_len(&encryption, 10, "Encryption")?;
    check_opt_len(&username, 100, "Username")?;
    if let Some(ref p) = password {
        if p.len() > 1000 {
            return Err(VaultError::DatabaseError(
                "Password must be under 1000 characters".to_string(),
            ));
        }
    }
    check_opt_len(&from_address, 200, "From address")?;
    check_opt_len(&notes, 5000, "Notes")?;

    let conn = db
        .inner()
        .0
        .lock()
        .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_smtp_item(
        &conn,
        &id,
        &title,
        &host,
        port,
        encryption.as_deref(),
        username.as_deref(),
        password.as_deref(),
        from_address.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}
