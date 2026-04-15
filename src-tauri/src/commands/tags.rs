use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::crypto::keyring::Keyring;
use crate::db::models::Tag;
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn tag_create(
    name: String,
    color: Option<String>,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<Tag, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    let now = Utc::now().to_rfc3339();
    let tag = Tag {
        id: Uuid::new_v4().to_string(),
        name,
        color: color.unwrap_or_else(|| "#6366f1".into()),
        created_at: now.clone(),
        updated_at: now,
    };
    db.create_tag(&tag)?;
    Ok(tag)
}

#[tauri::command]
pub fn tag_update(
    id: String,
    name: String,
    color: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.update_tag(&id, &name, &color)?)
}

#[tauri::command]
pub fn tag_delete(id: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<bool, AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    Ok(db.delete_tag(&id)?)
}

#[tauri::command]
pub fn tag_list(db: State<'_, Database>) -> Result<Vec<Tag>, AppError> {
    Ok(db.list_tags()?)
}

#[tauri::command]
pub fn tag_add_to_entry(
    entry_id: String,
    tag_id: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<(), AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    db.add_tag_to_entry(&entry_id, &tag_id)?;
    Ok(())
}

#[tauri::command]
pub fn tag_remove_from_entry(
    entry_id: String,
    tag_id: String,
    db: State<'_, Database>,
    keyring: State<'_, Keyring>,
) -> Result<(), AppError> {
    if !keyring.is_unlocked() {
        return Err(AppError::VaultLocked);
    }
    db.remove_tag_from_entry(&entry_id, &tag_id)?;
    Ok(())
}
