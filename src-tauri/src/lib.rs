pub mod crypto;
pub mod db;
pub mod commands;
pub mod security;

pub use commands::{
    setup_vault, unlock_vault, lock_vault, is_vault_unlocked, is_vault_initialized,
    create_entry, update_entry, delete_entry, get_entry, list_entries, search_entries,
    copy_to_clipboard,
};
pub use commands::vault::VaultState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    log::info!("Starting BavuIru password manager");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(VaultState::new())
        .invoke_handler(tauri::generate_handler![
            setup_vault,
            unlock_vault,
            lock_vault,
            is_vault_unlocked,
            is_vault_initialized,
            create_entry,
            update_entry,
            delete_entry,
            get_entry,
            list_entries,
            search_entries,
            copy_to_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}