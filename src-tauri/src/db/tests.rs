use rusqlite::Connection;

use super::*;
use crate::crypto::{decrypt, derive_key};
use crate::db::{init_schema, insert_builtin_groups, run_migrations};

fn seeded_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    crate::db::insert_builtin_groups(&conn).unwrap();
    enable_foreign_keys(&conn).unwrap();
    conn
}

/// Create one item of every encrypted category and return their ids.
fn seed_all_types(conn: &Connection, key: &[u8; 32]) -> Vec<String> {        let mut ids = Vec::new();
    ids.push(
        create_account_item(conn, "built-in-accounts", "Acc", "user1", "secret-acc", None, None, None, key)
            .unwrap(),
    );
    ids.push(
        create_api_key_item(conn, "built-in-api-keys", "Key", "K", "secret-api", None, None, None, key)
            .unwrap(),
    );
    ids.push(
        create_env_var_item(
            conn,
            "built-in-env-vars",
            "Env",
            &[("A".to_string(), "secret-env".to_string())],
            None,
            key,
        )
        .unwrap(),
    );
    ids.push(
        create_database_item(
            conn, "built-in-databases", "DB", "postgresql", "h", Some(5432), None,
            Some("u"), Some("secret-db"), None, None, key,
        )
        .unwrap(),
    );
    ids.push(
        create_ssh_item(
            conn, "built-in-servers", "SSH", "h", Some(22), "u",
            Some("secret-ssh"), None, Some("secret-passphrase"), None, key,
        )
        .unwrap(),
    );
    ids.push(
        create_cloud_item(conn, "built-in-cloud", "Cloud", "aws", "AKIA", "secret-cloud", None, None, key)
            .unwrap(),
    );
    ids.push(
        create_license_item(conn, "built-in-licenses", "Lic", "Software", "secret-license", None, None, None, key)
            .unwrap(),
    );
    ids.push(
        create_smtp_item(conn, "built-in-smtp", "SMTP", "h", Some(587), None, None, "secret-smtp", None, None, key)
            .unwrap(),
    );
    ids
}

#[test]
fn reencrypt_all_rotates_every_encrypted_field() {
    let conn = seeded_conn();
    let old_key = derive_key("old-password-1", None).unwrap().key;
    let new_key = derive_key("new-password-2", None).unwrap().key;

    let ids = seed_all_types(&conn, &old_key);

    // Sanity: readable with the old key before rotation.
    assert_eq!(
        get_account_item_detail(&conn, &ids[0], &old_key).unwrap().password,
        "secret-acc"
    );

    reencrypt_all(&conn, &old_key, &new_key).unwrap();

    // Every category decrypts correctly with the new key.
    assert_eq!(get_account_item_detail(&conn, &ids[0], &new_key).unwrap().password, "secret-acc");
    assert_eq!(get_api_key_item_detail(&conn, &ids[1], &new_key).unwrap().key_value, "secret-api");
    assert_eq!(
        get_env_var_item_detail(&conn, &ids[2], &new_key).unwrap().variables[0].value,
        "secret-env"
    );
    assert_eq!(
        get_database_item_detail(&conn, &ids[3], &new_key).unwrap().password.unwrap(),
        "secret-db"
    );
    let ssh = get_ssh_item_detail(&conn, &ids[4], &new_key).unwrap();
    assert_eq!(ssh.password.unwrap(), "secret-ssh");
    assert_eq!(ssh.passphrase.unwrap(), "secret-passphrase");
    assert_eq!(get_cloud_item_detail(&conn, &ids[5], &new_key).unwrap().secret, "secret-cloud");
    assert_eq!(
        get_license_item_detail(&conn, &ids[6], &new_key).unwrap().license_key,
        "secret-license"
    );
    assert_eq!(get_smtp_item_detail(&conn, &ids[7], &new_key).unwrap().password, "secret-smtp");

    // The old key must no longer decrypt the stored ciphertext.
    let (ct, nonce): (Vec<u8>, Vec<u8>) = conn
        .query_row(
            "SELECT password_encrypted, password_nonce FROM account_items WHERE item_id = ?1",
            [&ids[0]],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(&nonce);
    assert!(decrypt(&old_key, &nonce_arr, &ct).is_err());
}

#[test]
fn reencrypt_all_leaves_null_secrets_null() {
    let conn = seeded_conn();
    let old_key = derive_key("old-password-1", None).unwrap().key;
    let new_key = derive_key("new-password-2", None).unwrap().key;

    // Database item without a password — both columns are NULL.
    let id = create_database_item(
        &conn, "built-in-databases", "DB", "redis", "h", None, None, None, None, None, None, &old_key,
    )
    .unwrap();

    reencrypt_all(&conn, &old_key, &new_key).unwrap();

    let (ct, nonce): (Option<Vec<u8>>, Option<Vec<u8>>) = conn
        .query_row(
            "SELECT password_encrypted, password_nonce FROM database_items WHERE item_id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(ct.is_none() && nonce.is_none());
    assert!(get_database_item_detail(&conn, &id, &new_key).unwrap().password.is_none());
}

#[test]
fn reencrypt_all_rolls_back_on_corrupt_ciphertext() {
    let conn = seeded_conn();
    let old_key = derive_key("old-password-1", None).unwrap().key;
    let new_key = derive_key("new-password-2", None).unwrap().key;

    let ids = seed_all_types(&conn, &old_key);

    // Corrupt one row so decryption fails mid-transaction.
    conn.execute(
        "UPDATE smtp_items SET password_encrypted = x'DEADBEEF' WHERE item_id = ?1",
        [&ids[7]],
    )
    .unwrap();

    assert!(reencrypt_all(&conn, &old_key, &new_key).is_err());

    // Untouched rows must still be readable with the OLD key (rollback).
    assert_eq!(
        get_account_item_detail(&conn, &ids[0], &old_key).unwrap().password,
        "secret-acc"
    );
    assert!(get_account_item_detail(&conn, &ids[0], &new_key).is_err());
}

/// Simulate the full app lifecycle against a real db file:
/// first launch (initialize + create item) → close → second launch
/// (unlock verifies password, re-derives key from stored salt, opens item).
#[test]
fn second_launch_can_open_items_created_in_first_launch() {
    let dir = std::env::temp_dir().join(format!("seckeybox-lifecycle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("vault.db");

    let password = "lifecycle-pw";

    // ---- first launch: initialize vault and create a license item ----
    {
        let conn = Connection::open(&db_path).unwrap();
        enable_foreign_keys(&conn).unwrap();
        init_schema(&conn).unwrap();
        run_migrations(&conn).unwrap();
        insert_builtin_groups(&conn).unwrap();

        let derived = derive_key(password, None).unwrap();
        set_password_hash(&conn, &derived.hash, &derived.salt).unwrap();

        create_license_item(
            &conn,
            "built-in-licenses",
            "Lens",
            "Lens IDE",
            "lens-license-key",
            Some("dev@example.com"),
            Some(1_800_000_000),
            None,
            &derived.key,
        )
        .unwrap();
    } // connection dropped, like closing the app

    // ---- second launch: unlock and open the item detail ----
    {
        let conn = Connection::open(&db_path).unwrap();
        enable_foreign_keys(&conn).unwrap();

        let version_before: String = conn
            .query_row(
                "SELECT value FROM vault_config WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "<missing>".to_string());
        let detail_rows_before: i64 = conn
            .query_row("SELECT COUNT(*) FROM license_items", [], |r| r.get(0))
            .unwrap();

        run_migrations(&conn).unwrap();

        // Migrations must not remove anything at the current version
        // (this regressed when the stale version read re-ran the v4
        // rebuild and cascade-deleted the detail rows).
        let detail_rows_after: i64 = conn
            .query_row("SELECT COUNT(*) FROM license_items", [], |r| r.get(0))
            .unwrap();
        let version_after: String = conn
            .query_row(
                "SELECT value FROM vault_config WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "<missing>".to_string());
        assert_eq!(
            detail_rows_before, detail_rows_after,
            "run_migrations wiped license_items! version before={version_before} after={version_after}"
        );

        let stored_hash = get_password_hash(&conn).unwrap().unwrap();
        let stored_salt = get_password_salt(&conn).unwrap().unwrap();
        assert!(crate::crypto::verify_password(password, &stored_hash).unwrap());

        let key = derive_key(password, Some(&stored_salt)).unwrap().key;

        let items = get_all_items(&conn).unwrap();
        assert_eq!(items.len(), 1);
        let id = items[0].id.clone();

        let detail = get_license_item_detail(&conn, &id, &key).unwrap();
        assert_eq!(detail.title, "Lens");
        assert_eq!(detail.license_key, "lens-license-key");
        assert_eq!(detail.expiry_date, Some(1_800_000_000));
    }

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn delete_group_cascades_items_and_details_for_any_group() {        let conn = seeded_conn();
    let key = derive_key("some-password-1", None).unwrap().key;

    // Custom group with an item.
    let custom = create_group(&conn, "Team", None, None, None).unwrap();
    let custom_item =
        create_account_item(&conn, &custom.id, "Team Acc", "u", "pw", None, None, None, &key).unwrap();

    // Built-in group with an item.
    let builtin_item =
        create_account_item(&conn, "built-in-accounts", "Acc2", "u", "pw", None, None, None, &key)
            .unwrap();

    delete_group(&conn, &custom.id).unwrap();
    assert!(get_all_groups(&conn).unwrap().iter().all(|g| g.id != custom.id));
    assert!(get_all_items(&conn).unwrap().iter().all(|i| i.id != custom_item));
    let orphans: i64 = conn
        .query_row("SELECT COUNT(*) FROM account_items WHERE item_id = ?1", [&custom_item], |r| r.get(0))
        .unwrap();
    assert_eq!(orphans, 0, "detail row must cascade with its item");

    // Built-in groups are deletable too.
    delete_group(&conn, "built-in-accounts").unwrap();
    assert!(get_all_groups(&conn).unwrap().iter().all(|g| g.id != "built-in-accounts"));
    assert!(get_all_items(&conn).unwrap().iter().all(|i| i.id != builtin_item));
}

#[test]
fn account_update_handles_all_totp_branches() {
    let conn = seeded_conn();
    let key = derive_key("totp-branch-password", None).unwrap().key;
    let id = create_account_item(
        &conn,
        "built-in-accounts",
        "GitHub",
        "octocat",
        "old-pass",
        None,
        None,
        None,
        &key,
    )
    .unwrap();

    // None = keep the current secret. This used to fail with
    // "... NULL, WHERE ..." because the SQL fragment kept a trailing comma.
    update_account_item(&conn, &id, "GitHub", "octocat", None, None, None, None, &key).unwrap();
    assert!(get_account_item_detail(&conn, &id, &key)
        .unwrap()
        .totp_secret
        .is_none());

    // Some(secret) = set it.
    update_account_item(
        &conn,
        &id,
        "GitHub",
        "octocat",
        None,
        None,
        None,
        Some("JBSWY3DPEHPK3PXP"),
        &key,
    )
    .unwrap();
    assert_eq!(
        get_account_item_detail(&conn, &id, &key)
            .unwrap()
            .totp_secret
            .as_deref(),
        Some("JBSWY3DPEHPK3PXP")
    );

    // Some("") = clear it.
    update_account_item(&conn, &id, "GitHub", "octocat", None, None, None, Some(""), &key)
        .unwrap();
    assert!(get_account_item_detail(&conn, &id, &key)
        .unwrap()
        .totp_secret
        .is_none());

    // Password rotation still works alongside a TOTP set.
    update_account_item(
        &conn,
        &id,
        "GitHub",
        "octocat",
        Some("new-pass"),
        None,
        None,
        Some("JBSWY3DPEHPK3PXP"),
        &key,
    )
    .unwrap();
    let detail = get_account_item_detail(&conn, &id, &key).unwrap();
    assert_eq!(detail.password, "new-pass");
    assert_eq!(detail.totp_secret.as_deref(), Some("JBSWY3DPEHPK3PXP"));
}

#[test]
fn custom_fields_are_encrypted_and_replaced() {
    let conn = seeded_conn();
    let key = derive_key("cf-password", None).unwrap().key;
    let id = create_account_item(&conn, "built-in-accounts", "Host", "u", "p", None, None, None, &key).unwrap();

    save_custom_fields(&conn, &id, &[("region".to_string(), "cn-north-1".to_string())], &key).unwrap();
    let fields = list_custom_fields(&conn, &id, &key).unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].key, "region");
    assert_eq!(fields[0].value, "cn-north-1");

    // Stored as ciphertext, not plaintext.
    let stored: Vec<u8> = conn
        .query_row("SELECT value_encrypted FROM custom_fields WHERE item_id = ?1", [&id], |r| r.get(0))
        .unwrap();
    assert!(!stored.windows(11).any(|w| w == b"cn-north-1"));

    // Full replacement.
    save_custom_fields(
        &conn,
        &id,
        &[("a".to_string(), "1".to_string()), ("b".to_string(), "2".to_string())],
        &key,
    )
    .unwrap();
    assert_eq!(list_custom_fields(&conn, &id, &key).unwrap().len(), 2);

    // Empty keys are rejected.
    assert!(save_custom_fields(&conn, &id, &[(" ".to_string(), "x".to_string())], &key).is_err());

    // Cascade with the item.
    delete_item(&conn, &id).unwrap();
    let remaining: i64 = conn
        .query_row("SELECT COUNT(*) FROM custom_fields WHERE item_id = ?1", [&id], |r| r.get(0))
        .unwrap();
    assert_eq!(remaining, 0);
}

#[test]
fn tags_attach_across_groups_and_cascade() {
    let conn = seeded_conn();
    let key = derive_key("tag-password", None).unwrap().key;
    let account = create_account_item(&conn, "built-in-accounts", "A", "u", "p", None, None, None, &key).unwrap();
    let database = create_database_item(
        &conn, "built-in-databases", "D", "postgres", "db.local", None, None, None, None, None, None, &key,
    )
    .unwrap();

    let tag = create_tag(&conn, "prod", None).unwrap();
    let again = create_tag(&conn, "prod", None).unwrap();
    assert_eq!(tag.id, again.id, "creating an existing tag is idempotent");

    set_item_tags(&conn, &account, &[tag.id.clone()]).unwrap();
    set_item_tags(&conn, &database, &[tag.id.clone()]).unwrap();
    assert_eq!(get_item_tag_ids(&conn, &account).unwrap(), vec![tag.id.clone()]);
    assert_eq!(get_items_by_tag(&conn, &tag.id).unwrap().len(), 2);

    delete_tag(&conn, &tag.id).unwrap();
    assert!(get_items_by_tag(&conn, &tag.id).unwrap().is_empty());
    assert!(get_item_tag_ids(&conn, &account).unwrap().is_empty());
}

#[test]
fn audit_log_records_events_newest_first() {
    let conn = seeded_conn();
    log_event(&conn, "vault.unlock", None, None, None);
    log_event(&conn, "item.copy", Some("id-1"), Some("GitHub"), Some("password"));

    let entries = list_audit_log(&conn, 10).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].event, "item.copy");
    assert_eq!(entries[0].item_id.as_deref(), Some("id-1"));
    assert_eq!(entries[0].item_title.as_deref(), Some("GitHub"));
    assert_eq!(entries[0].detail.as_deref(), Some("password"));
    assert_eq!(entries[1].event, "vault.unlock");
    assert!(entries[1].item_title.is_none());

    // Deleting an item keeps its title in the activity log.
    let key = derive_key("audit-password", None).unwrap().key;
    let id = create_account_item(&conn, "built-in-accounts", "Doomed", "u", "pw", None, None, None, &key).unwrap();
    delete_item(&conn, &id).unwrap();
    let entries = list_audit_log(&conn, 10).unwrap();
    assert_eq!(entries[0].event, "item.delete");
    assert_eq!(entries[0].item_title.as_deref(), Some("Doomed"));
}

#[test]
fn attachments_are_encrypted_and_scoped_to_their_item() {
    let conn = seeded_conn();
    let key = derive_key("attachments-password", None).unwrap().key;
    let item_id = create_account_item(
        &conn,
        "built-in-accounts",
        "SSH host",
        "u",
        "pw",
        None,
        None,
        None,
        &key,
    )
    .unwrap();
    let other_id = create_account_item(
        &conn,
        "built-in-accounts",
        "Other",
        "u",
        "pw",
        None,
        None,
        None,
        &key,
    )
    .unwrap();

    let payload: &[u8] = b"-----BEGIN OPENSSH PRIVATE KEY-----\nnot-a-real-key\n";
    let meta = add_attachment(
        &conn,
        &item_id,
        "id_ed25519",
        Some("text/plain"),
        payload,
        &key,
    )
    .unwrap();
    assert_eq!(meta.size, payload.len() as i64);

    // Contents are stored encrypted, not as plaintext.
    let stored: Vec<u8> = conn
        .query_row(
            "SELECT data_encrypted FROM item_attachments WHERE id = ?1",
            [&meta.id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!stored
        .windows(payload.len())
        .any(|window| window == payload));

    let listed = list_attachments(&conn, &item_id).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].filename, "id_ed25519");

    let (got_meta, data) = get_attachment_data(&conn, &item_id, &meta.id, &key).unwrap();
    assert_eq!(got_meta.filename, "id_ed25519");
    assert_eq!(data, payload);

    // Scoped lookup: an attachment is not readable through another item.
    assert!(get_attachment_data(&conn, &other_id, &meta.id, &key).is_err());
    assert!(delete_attachment(&conn, &other_id, &meta.id).is_err());

    // Attachments cascade with their item.
    delete_item(&conn, &item_id).unwrap();
    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM item_attachments WHERE item_id = ?1",
            [&item_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 0);
}
