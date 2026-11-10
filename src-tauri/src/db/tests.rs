use rusqlite::Connection;

use super::*;
use crate::crypto::{decrypt, derive_key};
use crate::db::{init_schema, insert_builtin_groups, run_migrations};
use crate::error::VaultError;

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
fn delete_group_trashes_items_and_keeps_their_details() {
    let conn = seeded_conn();
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
    // The item is no longer active, but it is not destroyed either.
    assert!(get_all_items(&conn).unwrap().iter().all(|i| i.id != custom_item));
    let detail_rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM account_items WHERE item_id = ?1", [&custom_item], |r| r.get(0))
        .unwrap();
    assert_eq!(detail_rows, 1, "detail row must survive a group delete");

    let trash = list_trash(&conn).unwrap();
    assert_eq!(trash.len(), 1);
    assert_eq!(trash[0].id, custom_item);
    assert_eq!(trash[0].original_group_id.as_deref(), Some(custom.id.as_str()));
    assert!(
        trash[0].original_group_name.is_none(),
        "the deleted group has no name to show"
    );

    // Restoring falls back to the built-in group of the item type.
    restore_item(&conn, &custom_item).unwrap();
    let items = get_all_items(&conn).unwrap();
    let restored = items.iter().find(|i| i.id == custom_item).unwrap();
    assert_eq!(restored.group_id, "built-in-accounts");
    assert_eq!(
        get_account_item_detail(&conn, &custom_item, &key).unwrap().password,
        "pw"
    );

    // Built-in groups are deletable too; their items are trashed as well.
    delete_group(&conn, "built-in-accounts").unwrap();
    assert!(get_all_groups(&conn).unwrap().iter().all(|g| g.id != "built-in-accounts"));
    assert!(get_all_items(&conn).unwrap().iter().all(|i| i.id != builtin_item));
    assert!(list_trash(&conn).unwrap().iter().any(|i| i.id == builtin_item));
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

    // Custom fields survive a soft delete so a restore keeps them, and only go
    // away with a permanent purge.
    save_custom_fields(&conn, &id, &[("region".to_string(), "cn-north-1".to_string())], &key)
        .unwrap();
    delete_item(&conn, &id).unwrap();
    let kept: i64 = conn
        .query_row("SELECT COUNT(*) FROM custom_fields WHERE item_id = ?1", [&id], |r| r.get(0))
        .unwrap();
    assert_eq!(kept, 1, "trashing must keep custom fields for a later restore");

    purge_item(&conn, &id).unwrap();
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
    assert_eq!(entries[0].event, "item.trash");
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

    // Attachments survive a soft delete (a restore must keep them intact) and
    // only cascade away on a permanent purge.
    delete_item(&conn, &item_id).unwrap();
    let kept: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM item_attachments WHERE item_id = ?1",
            [&item_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(kept, 1, "trashing must keep attachments for a later restore");

    purge_item(&conn, &item_id).unwrap();
    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM item_attachments WHERE item_id = ?1",
            [&item_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(remaining, 0);
}

// ---------- trash / soft delete ----------

#[test]
fn soft_delete_hides_item_and_restore_returns_it_to_its_group() {
    let conn = seeded_conn();
    let key = derive_key("trash-password-1", None).unwrap().key;
    let group = create_group(&conn, "Work", None, None, None).unwrap();
    let id = create_account_item(
        &conn, &group.id, "GitHub", "octocat", "pw", None, None, None, &key,
    )
    .unwrap();

    delete_item(&conn, &id).unwrap();

    // Every active read path must hide the trashed item.
    assert!(get_all_items(&conn).unwrap().is_empty());
    assert!(search_items(&conn, "GitHub").unwrap().is_empty());
    assert!(get_items_by_group(&conn, &group.id).unwrap().is_empty());
    assert!(get_item_summary(&conn, &id).unwrap().is_none());
    assert_eq!(get_trash_count(&conn).unwrap(), 1);

    let trash = list_trash(&conn).unwrap();
    assert_eq!(trash.len(), 1);
    assert_eq!(trash[0].id, id);
    assert_eq!(trash[0].title, "GitHub");
    assert_eq!(trash[0].item_type, "account");
    assert_eq!(trash[0].original_group_id.as_deref(), Some(group.id.as_str()));
    assert_eq!(trash[0].original_group_name.as_deref(), Some("Work"));
    assert!(trash[0].deleted_at > 0);

    // The row and its detail survived; only the detail getter is guarded.
    let details: i64 = conn
        .query_row("SELECT COUNT(*) FROM account_items WHERE item_id = ?1", [&id], |r| r.get(0))
        .unwrap();
    assert_eq!(details, 1);
    assert!(matches!(
        get_account_item_detail(&conn, &id, &key),
        Err(VaultError::ItemNotFound)
    ));

    restore_item(&conn, &id).unwrap();
    let items = get_all_items(&conn).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].group_id, group.id);
    assert_eq!(get_trash_count(&conn).unwrap(), 0);
    assert_eq!(get_account_item_detail(&conn, &id, &key).unwrap().password, "pw");
}

#[test]
fn restore_falls_back_to_any_group_when_builtin_groups_are_gone() {
    let conn = seeded_conn();
    let key = derive_key("fallback-password", None).unwrap().key;
    let custom = create_group(&conn, "Team", None, None, None).unwrap();
    let id = create_account_item(&conn, &custom.id, "Acc", "u", "pw", None, None, None, &key)
        .unwrap();

    // Drop the group the item lived in and its type fallback.
    delete_group(&conn, &custom.id).unwrap();
    delete_group(&conn, "built-in-accounts").unwrap();
    assert_eq!(get_trash_count(&conn).unwrap(), 1);

    restore_item(&conn, &id).unwrap();
    let restored = get_item_summary(&conn, &id).unwrap().unwrap();
    let groups = get_all_groups(&conn).unwrap();
    assert_eq!(restored.group_id, "built-in-api-keys", "first remaining non-trash group");
    assert!(groups.iter().any(|g| g.id == restored.group_id));

    // Unknown/never-trashed ids are rejected.
    assert!(matches!(restore_item(&conn, &id), Err(VaultError::ItemNotFound)));
}

#[test]
fn purge_item_and_empty_trash_cascade_details() {
    let conn = seeded_conn();
    let key = derive_key("purge-password-1", None).unwrap().key;
    let account =
        create_account_item(&conn, "built-in-accounts", "A", "u", "pw", None, None, None, &key)
            .unwrap();
    let api_key =
        create_api_key_item(&conn, "built-in-api-keys", "B", "K", "secret", None, None, None, &key)
            .unwrap();
    let live =
        create_cloud_item(&conn, "built-in-cloud", "C", "aws", "AKIA", "s", None, None, &key).unwrap();

    delete_item(&conn, &account).unwrap();
    delete_item(&conn, &api_key).unwrap();
    assert_eq!(get_trash_count(&conn).unwrap(), 2);

    purge_item(&conn, &account).unwrap();
    let account_rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM account_items WHERE item_id = ?1", [&account], |r| r.get(0))
        .unwrap();
    assert_eq!(account_rows, 0, "purge must cascade the detail row");
    assert_eq!(get_trash_count(&conn).unwrap(), 1);

    // Only trashed rows may be purged, and only once.
    assert!(matches!(purge_item(&conn, &account), Err(VaultError::ItemNotFound)));
    assert!(matches!(purge_item(&conn, &live), Err(VaultError::ItemNotFound)));
    assert!(get_item_summary(&conn, &live).unwrap().is_some(), "live item untouched");

    assert_eq!(empty_trash(&conn).unwrap(), 1);
    let api_rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM api_key_items WHERE item_id = ?1", [&api_key], |r| r.get(0))
        .unwrap();
    assert_eq!(api_rows, 0);
    assert_eq!(get_trash_count(&conn).unwrap(), 0);
    assert_eq!(empty_trash(&conn).unwrap(), 0);

    let events: Vec<String> = list_audit_log(&conn, 50)
        .unwrap()
        .into_iter()
        .map(|entry| entry.event)
        .collect();
    for event in ["item.trash", "item.purge", "trash.empty"] {
        assert!(events.iter().any(|e| e == event), "missing audit event {event}");
    }
}

#[test]
fn purge_expired_trash_respects_the_retention_window() {
    let conn = seeded_conn();
    let key = derive_key("retention-password", None).unwrap().key;
    let old =
        create_account_item(&conn, "built-in-accounts", "Old", "u", "pw", None, None, None, &key)
            .unwrap();
    let recent =
        create_account_item(&conn, "built-in-accounts", "Recent", "u", "pw", None, None, None, &key)
            .unwrap();

    delete_item(&conn, &old).unwrap();
    delete_item(&conn, &recent).unwrap();
    conn.execute(
        "UPDATE items SET deleted_at = strftime('%s','now') - 31*86400 WHERE id = ?1",
        [&old],
    )
    .unwrap();
    conn.execute(
        "UPDATE items SET deleted_at = strftime('%s','now') - 29*86400 WHERE id = ?1",
        [&recent],
    )
    .unwrap();

    assert_eq!(trash_retention_days(&conn), 30, "defaults to 30 days");
    assert_eq!(purge_expired_trash(&conn).unwrap(), 1);
    let remaining = list_trash(&conn).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, recent);

    // Retention 0 disables the automatic purge entirely.
    set_retention(&conn, "0");
    assert_eq!(trash_retention_days(&conn), 0);
    conn.execute(
        "UPDATE items SET deleted_at = strftime('%s','now') - 400*86400 WHERE id = ?1",
        [&recent],
    )
    .unwrap();
    assert_eq!(purge_expired_trash(&conn).unwrap(), 0);
    assert_eq!(get_trash_count(&conn).unwrap(), 1);

    // Absurd values are clamped instead of disabling the purge silently.
    set_retention(&conn, "99999");
    assert_eq!(trash_retention_days(&conn), MAX_TRASH_RETENTION_DAYS);
    set_retention(&conn, "not-a-number");
    assert_eq!(trash_retention_days(&conn), DEFAULT_TRASH_RETENTION_DAYS);
}

fn set_retention(conn: &Connection, value: &str) {
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('trash_retention_days', ?1)",
        [value],
    )
    .unwrap();
}

#[test]
fn trashed_items_reject_updates_reads_and_side_writes() {
    let conn = seeded_conn();
    let key = derive_key("guard-password", None).unwrap().key;
    let id = create_account_item(&conn, "built-in-accounts", "Acc", "u", "pw", None, None, None, &key)
        .unwrap();
    let tag = create_tag(&conn, "prod", None).unwrap();

    delete_item(&conn, &id).unwrap();

    assert!(matches!(
        update_account_item(&conn, &id, "T", "u", None, None, None, None, &key),
        Err(VaultError::ItemNotFound)
    ));
    assert!(matches!(toggle_favorite(&conn, &id), Err(VaultError::ItemNotFound)));
    assert!(matches!(
        get_account_item_detail(&conn, &id, &key),
        Err(VaultError::ItemNotFound)
    ));
    assert!(
        matches!(delete_item(&conn, &id), Err(VaultError::ItemNotFound)),
        "a second delete must not re-stamp the trash entry"
    );
    assert!(matches!(
        save_custom_fields(&conn, &id, &[("k".to_string(), "v".to_string())], &key),
        Err(VaultError::ItemNotFound)
    ));
    assert!(matches!(
        set_item_tags(&conn, &id, &[tag.id.clone()]),
        Err(VaultError::ItemNotFound)
    ));
    assert!(matches!(
        add_attachment(&conn, &id, "id_ed25519", None, b"key", &key),
        Err(VaultError::ItemNotFound)
    ));

    // Restoring clears both trash columns and re-enables the item.
    restore_item(&conn, &id).unwrap();
    let restored = get_item_summary(&conn, &id).unwrap().unwrap();
    assert_eq!(restored.group_id, "built-in-accounts");
    let (deleted_at, original): (Option<i64>, Option<String>) = conn
        .query_row(
            "SELECT deleted_at, original_group_id FROM items WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(deleted_at.is_none() && original.is_none());
    toggle_favorite(&conn, &id).unwrap();
}

#[test]
fn trash_group_is_hidden_and_immutable() {
    let conn = seeded_conn();

    assert!(
        get_all_groups(&conn).unwrap().iter().all(|g| g.id != TRASH_GROUP_ID),
        "the trash group must not show up in the sidebar"
    );
    assert!(get_all_groups(&conn).unwrap().len() == 8);

    assert!(matches!(
        update_group(&conn, TRASH_GROUP_ID, "Renamed", None, None),
        Err(VaultError::Validation(_))
    ));
    assert!(matches!(
        delete_group(&conn, TRASH_GROUP_ID),
        Err(VaultError::Validation(_))
    ));

    // The row itself is still there for soft-deleted items to point at.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM groups WHERE id = ?1", [TRASH_GROUP_ID], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn schema_v9_is_idempotent_and_adds_trash_columns() {
    let conn = Connection::open_in_memory().unwrap();
    for _ in 0..2 {
        init_schema(&conn).unwrap();
        run_migrations(&conn).unwrap();
        insert_builtin_groups(&conn).unwrap();
    }

    assert_eq!(get_schema_version(&conn).unwrap(), 9);
    for column in ["deleted_at", "original_group_id"] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('items') WHERE name = ?1",
                [column],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1, "items.{column} is missing");
    }
    let index: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_items_deleted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(index, 1);
    let groups: i64 = conn
        .query_row("SELECT COUNT(*) FROM groups", [], |r| r.get(0))
        .unwrap();
    assert_eq!(groups, 9, "8 built-in groups plus the trash group");

    // Re-running the guarded `version < 9` branch (a v8 database that already
    // has the columns, e.g. after init_schema) must not fail either.
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('schema_version', '8')",
        [],
    )
    .unwrap();
    run_migrations(&conn).unwrap();
    assert_eq!(get_schema_version(&conn).unwrap(), 9);
}

#[test]
fn migration_adds_trash_columns_to_a_v8_database() {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    insert_builtin_groups(&conn).unwrap();
    enable_foreign_keys(&conn).unwrap();

    // Rebuild `items` without the v9 columns to mimic a database created by
    // the previous schema version, then drop the trash group.
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = OFF;
        CREATE TABLE items_v8 (
            id TEXT PRIMARY KEY,
            group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
            type TEXT NOT NULL CHECK(type IN ('account', 'api_key', 'env_var', 'database', 'ssh', 'cloud', 'license', 'smtp')),
            title TEXT NOT NULL,
            icon TEXT,
            is_favorite INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        INSERT INTO items_v8 (id, group_id, type, title, icon, is_favorite, created_at, updated_at)
            SELECT id, group_id, type, title, icon, is_favorite, created_at, updated_at FROM items;
        DROP TABLE items;
        ALTER TABLE items_v8 RENAME TO items;
        DROP INDEX IF EXISTS idx_items_deleted;
        DELETE FROM groups WHERE id = 'built-in-trash';
        INSERT OR REPLACE INTO vault_config (key, value) VALUES ('schema_version', '8');
        PRAGMA foreign_keys = ON;
        "#,
    )
    .unwrap();

    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM pragma_table_info('items') WHERE name = 'deleted_at'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(before, 0, "v8 baseline must not have the trash column");

    run_migrations(&conn).unwrap();

    for column in ["deleted_at", "original_group_id"] {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('items') WHERE name = ?1",
                [column],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(exists, 1, "items.{column} was not added by the migration");
    }
    let index: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_items_deleted'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(index, 1);
    let trash_group: i64 = conn
        .query_row("SELECT COUNT(*) FROM groups WHERE id = 'built-in-trash'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(trash_group, 1, "migration must (re)create the trash group");
    assert_eq!(get_schema_version(&conn).unwrap(), 9);
}
