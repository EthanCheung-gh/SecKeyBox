use crate::db::{
    create_account_item, create_api_key_item, create_env_var_item, delete_item,
    get_account_item_detail, get_all_items, get_api_key_item_detail, get_env_var_item_detail,
    get_items_by_group, toggle_favorite, update_account_item, update_api_key_item,
    update_env_var_item, DbConnection, ItemDetail, ItemSummary,
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
