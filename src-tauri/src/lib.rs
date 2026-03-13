mod error;
mod crypto;
mod db;
mod state;
mod commands;

pub use error::{VaultError, Result};
pub use crypto::{derive_key, verify_password, encrypt, decrypt, SecureKey, EncryptedData};
pub use state::VaultState;
pub use db::{
    init_schema, insert_builtin_groups, run_migrations,
    Group, ItemSummary, AccountItemDetail, ApiKeyItemDetail, EnvVarItemDetail, EnvVarPair, ItemDetail,
    DbConnection, open_connection, get_db_path,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use commands::*;
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let db_path = get_db_path(&app.handle());
            let conn = if db_path.exists() {
                open_connection(&db_path).expect("Failed to open database")
            } else {
                rusqlite::Connection::open_in_memory().expect("Failed to create in-memory DB")
            };
            
            app.manage(DbConnection(std::sync::Mutex::new(conn)));
            app.manage(VaultState::new());
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_vault_initialized,
            initialize_vault,
            unlock_vault,
            lock_vault,
            is_vault_unlocked,
            get_groups,
            create_new_group,
            update_existing_group,
            delete_existing_group,
            get_all_items_cmd,
            get_items_by_group_cmd,
            get_item_detail,
            create_new_account_item,
            update_existing_account_item,
            create_new_api_key_item,
            update_existing_api_key_item,
            create_new_env_var_item,
            update_existing_env_var_item,
            delete_existing_item,
            toggle_item_favorite,
            copy_to_clipboard,
            clear_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}