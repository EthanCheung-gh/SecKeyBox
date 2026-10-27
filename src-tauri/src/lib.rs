mod error;
mod crypto;
mod db;
mod state;
mod commands;
mod ipc;

pub use error::{VaultError, Result};
pub use crypto::{derive_key, verify_password, encrypt, decrypt, SecureKey, EncryptedData};
pub use state::VaultState;
pub use db::{
    init_schema, insert_builtin_groups, run_migrations,
    Group, ItemSummary, AccountItemDetail, ApiKeyItemDetail, EnvVarItemDetail, EnvVarPair, ItemDetail,
    DatabaseItemDetail, SshItemDetail, CloudItemDetail, LicenseItemDetail, SmtpItemDetail,
    DbConnection, open_connection, get_db_path, AttachmentMeta,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use commands::*;
    use tauri::{Emitter, Manager};
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let db_path = get_db_path(&app.handle());
            eprintln!("[DEBUG] setup: db_path = {:?}", db_path);
            eprintln!("[DEBUG] setup: db_path.exists() = {}", db_path.exists());

            let conn = if db_path.exists() {
                eprintln!("[DEBUG] setup: opening existing database file");
                open_connection(&db_path).expect("Failed to open database")
            } else {
                eprintln!("[DEBUG] setup: no database, using in-memory (will create file on init)");
                let conn =
                    rusqlite::Connection::open_in_memory().expect("Failed to create in-memory DB");
                crate::db::enable_foreign_keys(&conn).expect("Failed to enable foreign keys");
                conn
            };

            app.manage(DbConnection(std::sync::Mutex::new(conn)));
            app.manage(VaultState::new());

            // Global shortcut toggles the quick-search palette window.
            // tauri-plugin-global-shortcut 2.x delivers presses through the
            // handler API; the old `global-shortcut://shortcut` event is not
            // emitted any more, so listening for it silently did nothing.
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                if let Some(window) = app.get_webview_window("palette") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }) {
                eprintln!("[WARN] Failed to register global shortcut: {:?}", e);
            }

            // Exit app when main window is closed
            let app_handle = app.handle().clone();
            if let Some(main_window) = app_handle.get_webview_window("main") {
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        use tauri_plugin_clipboard_manager::ClipboardExt;
                        let _ = app_handle.clipboard().write_text("");
                        app_handle.exit(0);
                    }
                });
            }

            // Idle watchdog: locks the vault and clears the clipboard after the
            // configured idle period. Runs in Rust so it still fires when the
            // webview is hidden or its timers are throttled.
            let watchdog_app = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    let state = watchdog_app.state::<VaultState>();
                    let minutes = state.auto_lock_minutes();
                    if minutes <= 0 || !state.is_unlocked() {
                        continue;
                    }
                    let idle = state.now_epoch_secs() - state.last_activity();
                    if idle < minutes * 60 {
                        continue;
                    }
                    state.lock();
                    drop(state);

                    // Record why the vault locked (idle timeout) for the
                    // activity log; ignore failures so the watchdog survives.
                    {
                        let db = watchdog_app.state::<DbConnection>();
                        if let Ok(conn) = db.inner().0.lock() {
                            crate::db::log_event(
                                &conn,
                                "vault.lock_timeout",
                                None,
                                None,
                                Some(&format!("{} min idle", minutes)),
                            );
                        }
                    }

                    let clipboard_app = watchdog_app.clone();
                    let _ = watchdog_app.run_on_main_thread(move || {
                        use tauri_plugin_clipboard_manager::ClipboardExt;
                        let _ = clipboard_app.clipboard().write_text("");
                    });
                    let _ = watchdog_app.emit("vault-locked", "timeout");
                }
            });

            // Local socket for the `seckeybox` CLI (read-only, unlocked only).
            #[cfg(unix)]
            ipc::spawn(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_vault_initialized,
            initialize_vault,
            unlock_vault,
            lock_vault,
            is_vault_unlocked,
            change_master_password,
            security_audit,
            import_csv,
            import_env,
            get_groups,
            create_new_group,
            update_existing_group,
            delete_existing_group,
            get_all_items_cmd,
            get_items_by_group_cmd,
            get_item_detail,
            get_totp_code,
            create_new_account_item,
            update_existing_account_item,
            create_new_api_key_item,
            update_existing_api_key_item,
            create_new_env_var_item,
            update_existing_env_var_item,
            create_new_database_item,
            update_existing_database_item,
            create_new_ssh_item,
            update_existing_ssh_item,
            create_new_cloud_item,
            update_existing_cloud_item,
            create_new_license_item,
            update_existing_license_item,
            create_new_smtp_item,
            update_existing_smtp_item,
            delete_existing_item,
            toggle_item_favorite,
            copy_to_clipboard,
            clear_clipboard,
            search_items,
            export_vault,
            import_vault,
            record_activity,
            get_security_settings,
            get_activity_log,
            set_auto_lock_minutes,
            add_attachment_cmd,
            list_attachments_cmd,
            get_attachment_cmd,
            save_attachment_to_path,
            delete_attachment_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}