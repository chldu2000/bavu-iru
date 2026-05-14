mod crypto;
mod db;
mod commands;
mod error;
mod security;
mod tray;

use commands::clipboard::ClipboardState;
use crypto::keyring::Keyring;
use db::repository::Database;
use security::autolock::create_lock_screen_listener;
use tauri::{Emitter, Listener, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	tauri::Builder::default()
		.plugin(tauri_plugin_log::Builder::default().build())
		.plugin(tauri_plugin_clipboard_manager::init())
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
			app.manage(ClipboardState::new());

			// Start system lock screen listener
			let lock_listener = create_lock_screen_listener();
			let app_handle = app.handle().clone();
			lock_listener.start_listening(Box::new(move || {
				let keyring = app_handle.state::<Keyring>();
				if keyring.is_unlocked() {
					keyring.lock();
					let _ = app_handle.emit("vault-locked", ());
				}
			}));

			// Create system tray
			tray::create_tray(app)?;

			// Update tray state when vault unlocks
			let tray_handle = app.handle().clone();
			app.listen("vault-unlocked", move |_| {
				tray::update_tray_state(&tray_handle, true);
			});

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![
			commands::vault::vault_setup,
			commands::vault::vault_unlock,
			commands::vault::vault_lock,
			commands::vault::vault_status,
			commands::vault::settings_get,
			commands::vault::settings_set,
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
			commands::clipboard::clipboard_copy,
			commands::clipboard::clipboard_clear,
				commands::import_export::export_vault,
				commands::import_export::preview_import,
				commands::import_export::import_vault,
				commands::import_export::check_integrity,
		])
		.on_window_event(|window, event| {
			if let tauri::WindowEvent::CloseRequested { api, .. } = event {
				// Hide window instead of closing — keep app in system tray
				let _ = window.hide();
				api.prevent_close();
			}
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
