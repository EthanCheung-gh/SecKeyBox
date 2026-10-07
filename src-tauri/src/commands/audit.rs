use std::collections::HashMap;

use rusqlite::Connection;
use tauri::State;

use crate::db::{
    get_account_item_detail, get_all_items, get_api_key_item_detail, get_cloud_item_detail,
    get_database_item_detail, get_license_item_detail, get_smtp_item_detail,
    get_ssh_item_detail, DbConnection,
};
use crate::error::{Result, VaultError};
use crate::state::VaultState;
use serde::Serialize;

/// Minimum accepted password length (characters).
const WEAK_LENGTH: usize = 12;
/// Rotations older than this are flagged.
const STALE_ROTATION_DAYS: i64 = 180;
/// Licenses expiring within this window are flagged.
const EXPIRY_SOON_DAYS: i64 = 30;

#[derive(Debug, Serialize)]
pub struct AuditFinding {
    pub item_id: String,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct ReusedSecretGroup {
    pub secret_hint: String,
    pub items: Vec<AuditFinding>,
}

#[derive(Debug, Serialize)]
pub struct SecurityAudit {
    pub weak_passwords: Vec<AuditFinding>,
    pub reused_secrets: Vec<ReusedSecretGroup>,
    pub stale_rotations: Vec<AuditFinding>,
    pub missing_2fa: Vec<AuditFinding>,
    pub expiring_licenses: Vec<AuditFinding>,
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn mask_hint(s: &str) -> String {
    if s.len() <= 4 {
        "••••".to_string()
    } else {
        format!("•••{}", &s[s.len() - 4..])
    }
}

fn charset_classes(s: &str) -> usize {
    let lower = s.chars().any(|c| c.is_ascii_lowercase());
    let upper = s.chars().any(|c| c.is_ascii_uppercase());
    let digit = s.chars().any(|c| c.is_ascii_digit());
    let symbol = s.chars().any(|c| c.is_ascii_punctuation() || c == ' ');
    [lower, upper, digit, symbol].iter().filter(|&&b| b).count()
}

/// Aggregate a local security and hygiene report over the whole vault.
/// Requires an unlocked vault (secrets are decrypted to evaluate them).
#[tauri::command]
pub fn security_audit(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
) -> Result<SecurityAudit> {
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

    run_audit(&conn, key.as_bytes())
}

/// Free-function core so tests can drive the audit without Tauri state.
pub fn run_audit(conn: &Connection, key: &[u8; 32]) -> Result<SecurityAudit> {
    let items = get_all_items(conn)?;
    let ts = now();
    let mut audit = SecurityAudit {
        weak_passwords: Vec::new(),
        reused_secrets: Vec::new(),
        stale_rotations: Vec::new(),
        missing_2fa: Vec::new(),
        expiring_licenses: Vec::new(),
    };
    let mut usage: HashMap<String, Vec<AuditFinding>> = HashMap::new();

    let mut track = |secret: String, title: &str, item_id: &str, label: &str| {
        usage.entry(secret).or_default().push(AuditFinding {
            item_id: item_id.to_string(),
            title: title.to_string(),
            detail: label.to_string(),
        });
    };

    for item in &items {
        let finding = |detail: String| AuditFinding {
            item_id: item.id.clone(),
            title: item.title.clone(),
            detail,
        };
        match item.item_type.as_str() {
            "account" => {
                let d = get_account_item_detail(&conn, &item.id, key)?;                let weak = d.password.len() < WEAK_LENGTH || charset_classes(&d.password) < 3;
                if weak {
                    audit.weak_passwords.push(finding(format!(
                        "{} 位 / {} 种字符类型",
                        d.password.len(),
                        charset_classes(&d.password)
                    )));
                }
                track(d.password.clone(), &item.title, &item.id, "账号密码");
                if d.totp_secret.as_deref().unwrap_or("").is_empty() {
                    audit
                        .missing_2fa
                        .push(finding(format!("用户名 {}", d.username)));
                }
            }
            "api_key" => {
                let d = get_api_key_item_detail(&conn, &item.id, key)?;
                track(d.key_value.clone(), &item.title, &item.id, &format!("API Key {}", d.key_name));
                match d.rotation_date {
                    None => audit.stale_rotations.push(finding("从未记录轮换".to_string())),
                    Some(rot) => {
                        let age = (ts - rot) / 86400;
                        if age > STALE_ROTATION_DAYS {
                            audit.stale_rotations.push(finding(format!("{age} 天未轮换")));
                        }
                    }
                }
            }
            "database" => {
                let d = get_database_item_detail(&conn, &item.id, key)?;
                if let Some(pw) = d.password.clone() {
                    if pw.len() < WEAK_LENGTH || charset_classes(&pw) < 3 {
                        audit
                            .weak_passwords
                            .push(finding(format!("数据库密码 {} 位 / {} 种字符类型", pw.len(), charset_classes(&pw))));
                    }
                    track(pw, &item.title, &item.id, "数据库密码");
                }
            }
            "ssh" => {
                let d = get_ssh_item_detail(&conn, &item.id, key)?;
                if let Some(pw) = d.password.clone() {
                    if pw.len() < WEAK_LENGTH || charset_classes(&pw) < 3 {
                        audit
                            .weak_passwords
                            .push(finding(format!("SSH 密码 {} 位 / {} 种字符类型", pw.len(), charset_classes(&pw))));
                    }
                    track(pw, &item.title, &item.id, "SSH 密码");
                }
            }
            "cloud" => {
                let d = get_cloud_item_detail(&conn, &item.id, key)?;
                track(d.secret.clone(), &item.title, &item.id, "云 Secret");
            }
            "license" => {
                let d = get_license_item_detail(&conn, &item.id, key)?;
                track(d.license_key.clone(), &item.title, &item.id, "许可证密钥");
                if let Some(exp) = d.expiry_date {
                    let days = (exp - ts) / 86400;
                    if days < EXPIRY_SOON_DAYS {
                        audit.expiring_licenses.push(finding(if days < 0 {
                            format!("已过期 {} 天", -days)
                        } else {
                            format!("{} 天后过期", days)
                        }));
                    }
                }
            }
            "smtp" => {
                let d = get_smtp_item_detail(&conn, &item.id, key)?;
                track(d.password.clone(), &item.title, &item.id, "SMTP 密码");
            }
            _ => {}
        }
    }

    audit.reused_secrets = usage
        .into_iter()
        .filter(|(_, items)| items.len() > 1)
        .map(|(secret, items)| ReusedSecretGroup {
            secret_hint: mask_hint(&secret),
            items,
        })
        .collect();
    audit.reused_secrets.sort_by(|a, b| b.items.len().cmp(&a.items.len()));

    Ok(audit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::derive_key;
    use crate::db::{
        create_account_item, create_api_key_item, create_license_item, enable_foreign_keys,
        init_schema, insert_builtin_groups,
    };
    use rusqlite::Connection;

    fn setup() -> (Connection, [u8; 32]) {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        insert_builtin_groups(&conn).unwrap();
        enable_foreign_keys(&conn).unwrap();
        let key = derive_key("audit-pw-1", None).unwrap().key;
        (conn, key)
    }

    #[test]
    fn audit_flags_weak_reused_and_missing_2fa() {
        let (conn, key) = setup();
        // two accounts share the same short weak password
        create_account_item(&conn, "built-in-accounts", "GitHub", "octo", "123456", None, None, None, &key).unwrap();
        create_account_item(&conn, "built-in-accounts", "GitLab", "ethan", "123456", None, None, None, &key).unwrap();

        let audit = run_audit(&conn, &key).unwrap();
        assert_eq!(audit.weak_passwords.len(), 2);
        assert_eq!(audit.missing_2fa.len(), 2);
        assert_eq!(audit.reused_secrets.len(), 1);
        assert_eq!(audit.reused_secrets[0].items.len(), 2);
        assert_eq!(audit.reused_secrets[0].secret_hint, "•••3456");
    }

    #[test]
    fn audit_flags_stale_rotation_and_expiring_license() {
        let (conn, key) = setup();
        create_api_key_item(&conn, "built-in-api-keys", "OpenAI", "K", "sk-strong-enough-2026", None, None, None, &key).unwrap();
        conn.execute(
            "UPDATE api_key_items SET rotation_date = strftime('%s','now') - 400*86400",
            [],
        )
        .unwrap();
        let soon = chrono_now_plus(10);
        create_license_item(&conn, "built-in-licenses", "Old Tool", "Tool", "LIC-1", None, Some(soon), None, &key).unwrap();

        let audit = run_audit(&conn, &key).unwrap();
        assert_eq!(audit.stale_rotations.len(), 1);
        assert!(audit.stale_rotations[0].detail.contains("400"));
        assert_eq!(audit.expiring_licenses.len(), 1);
        assert!(audit.expiring_licenses[0].detail.contains("10"));
    }

    #[test]
    fn audit_clean_vault_has_no_findings() {
        let (conn, key) = setup();
        create_account_item(
            &conn,
            "built-in-accounts",
            "Strong",
            "user",
            "Zq7#kLm2vNp9xWt4",
            None,
            None,
            Some("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"),
            &key,
        )
        .unwrap();

        let audit = run_audit(&conn, &key).unwrap();
        assert!(audit.weak_passwords.is_empty());
        assert!(audit.missing_2fa.is_empty());
        assert!(audit.reused_secrets.is_empty());
    }

    fn chrono_now_plus(days: i64) -> i64 {
        now() + days * 86400
    }
}
