use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub id: String,
    pub item_id: String,
    pub filename: String,
    pub content_type: Option<String>,
    pub size: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub is_favorite: bool,
    pub group_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub username: String,
    pub password: String,
    pub website: Option<String>,
    pub notes: Option<String>,
    /// Base32 TOTP secret for two-factor codes; None = not set.
    pub totp_secret: Option<String>,
    /// Previous values of the secret fields (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

/// One archived (still-encrypted) previous value of a secret field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretHistoryEntry {
    pub field: String,
    pub value: String,
    pub changed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemDetail {
    #[serde(rename = "account")]
    Account(AccountItemDetail),
    #[serde(rename = "api_key")]
    ApiKey(ApiKeyItemDetail),
    #[serde(rename = "env_var")]
    EnvVar(EnvVarItemDetail),
    #[serde(rename = "database")]
    Database(DatabaseItemDetail),
    #[serde(rename = "ssh")]
    Ssh(SshItemDetail),
    #[serde(rename = "cloud")]
    Cloud(CloudItemDetail),
    #[serde(rename = "license")]
    License(LicenseItemDetail),
    #[serde(rename = "smtp")]
    Smtp(SmtpItemDetail),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub key_name: String,
    pub key_value: String,
    pub endpoint: Option<String>,
    pub auth_method: Option<String>,
    pub rotation_date: Option<i64>,
    pub notes: Option<String>,
    /// Previous secret values (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub variables: Vec<EnvVarPair>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub db_type: String,
    pub host: String,
    pub port: Option<i64>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub connection_url: Option<String>,
    pub notes: Option<String>,
    /// Previous secret values (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub host: String,
    pub port: Option<i64>,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub passphrase: Option<String>,
    pub notes: Option<String>,
    /// Previous secret values (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub provider: String,
    pub access_key_id: String,
    pub secret: String,
    pub region: Option<String>,
    pub notes: Option<String>,
    /// Previous secret values (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub software_name: String,
    pub license_key: String,
    pub bound_email: Option<String>,
    pub expiry_date: Option<i64>,
    pub notes: Option<String>,
    /// Previous secret values (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub host: String,
    pub port: Option<i64>,
    pub encryption: Option<String>,
    pub username: Option<String>,
    pub password: String,
    pub from_address: Option<String>,
    pub notes: Option<String>,
    /// Previous secret values (newest first).
    pub secret_history: Vec<SecretHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub event: String,
    pub item_id: Option<String>,
    pub item_title: Option<String>,
    pub detail: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: i64,
}
