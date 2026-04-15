use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::crypto::keyring::Keyring;
use crate::db::models::Folder;
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn folder_create(
    name: String,
    parent_id: Option<String>,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<Folder, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    let now = Utc::now().to_rfc3339();
    let folder = Folder {
        id: Uuid::new_v4().to_string(),
        name,
        parent_id,
        sort_order: 0,
        created_at: now.clone(),
        updated_at: now,
    };
    db.create_folder(&folder)?;
    Ok(folder)
}

#[tauri::command]
pub fn folder_rename(id: String, name: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.rename_folder(&id, &name)?)
}

#[tauri::command]
pub fn folder_delete(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.delete_folder(&id)?)
}

#[tauri::command]
pub fn folder_list(db: State<'_, Database>) -> Result<Vec<Folder>, AppError> {
    Ok(db.list_folders()?)
}
