use tauri::State;

use crate::crypto::keyring::Keyring;
use crate::db::models::Entry;
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn entry_list(db: State<'_, Database>) -> Result<Vec<Entry>, AppError> {
	Ok(db.list_entries()?)
}

#[tauri::command]
pub fn entry_get(id: String, db: State<'_, Database>) -> Result<Option<Entry>, AppError> {
	Ok(db.get_entry(&id)?)
}

#[tauri::command]
pub fn entry_create(entry: Entry, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<(), AppError> {
	if !keyring.is_unlocked() {
		return Err(AppError::VaultLocked);
	}
	db.create_entry(&entry)?;
	Ok(())
}

#[tauri::command]
pub fn entry_update(entry: Entry, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
	if !keyring.is_unlocked() {
		return Err(AppError::VaultLocked);
	}
	Ok(db.update_entry(&entry)?)
}

#[tauri::command]
pub fn entry_delete(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
	if !keyring.is_unlocked() {
		return Err(AppError::VaultLocked);
	}
	Ok(db.delete_entry(&id)?)
}

#[tauri::command]
pub fn toggle_favorite(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
	if !keyring.is_unlocked() {
		return Err(AppError::VaultLocked);
	}
	Ok(db.toggle_favorite(&id)?)
}
