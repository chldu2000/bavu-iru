mod crypto;
mod db;
mod commands;
mod error;
mod security;

use crypto::keyring::Keyring;
use db::repository::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_log::Builder::default().build())
		.setup(|app| {
			// Open database in app data directory
			let app_dir = app
				.path()
				.app_data_dir()
				.expect("Failed to resolve app data dir");
			std::fs::create_dir_all(&app_dir).ok();
			let db_path = app_dir.join("vault.db");
			let database = Database::open(&db_path).expect("Failed to open database");
			let keyring = Keyring::new();

			app.manage(database);
			app.manage(keyring);
			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			commands::vault::vault_setup,
			commands::vault::vault_unlock,
			commands::vault::vault_lock,
			commands::vault::vault_status,
			commands::entries::entry_list,
			commands::entries::entry_get,
			commands::entries::entry_create,
			commands::entries::entry_update,
			commands::entries::entry_delete,
			commands::entries::toggle_favorite,
			commands::folders::folder_create,
			commands::folders::folder_rename,
			commands::folders::folder_delete,
			commands::folders::folder_list,
			commands::tags::tag_create,
			commands::tags::tag_update,
			commands::tags::tag_delete,
			commands::tags::tag_list,
			commands::tags::tag_add_to_entry,
			commands::tags::tag_remove_from_entry,
			commands::generator::generate_password,
			commands::strength::evaluate_password_strength,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
