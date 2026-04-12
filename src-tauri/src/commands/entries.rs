use crate::commands::vault::VaultState;
use crate::crypto::CipherModule;
use crate::db::Entry;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEntryInput {
    pub title: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub folder_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEntryInput {
    pub id: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub folder_id: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[tauri::command]
pub fn create_entry(input: CreateEntryInput, state: State<'_, VaultState>) -> Result<Entry, String> {
    let key = state.keyring.get_key().ok_or("Vault is locked")?;

    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock.as_ref().ok_or("Vault not initialized")?;

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let entry = Entry {
        id: id.clone(),
        folder_id: input.folder_id,
        title: input.title,
        username: input.username,
        password: input.password,
        url: input.url,
        notes: input.notes,
        custom_fields: None,
        tags: input.tags.map(|t| serde_json::to_string(&t).unwrap_or_default()),
        strength: None,
        expires_at: None,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    let encrypted_data = CipherModule::encrypt(&key, serde_json::to_vec(&entry).unwrap().as_slice())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    repo.insert_entry(&id, entry.folder_id.as_deref(), &encrypted_data, &now, &now)
        .map_err(|e| format!("Failed to insert entry: {}", e))?;

    Ok(entry)
}

#[tauri::command]
pub fn update_entry(input: UpdateEntryInput, state: State<'_, VaultState>) -> Result<Entry, String> {
    let key = state.keyring.get_key().ok_or("Vault is locked")?;

    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock.as_ref().ok_or("Vault not initialized")?;

    let existing = repo.get_entry(&input.id)
        .map_err(|e| format!("Failed to get entry: {}", e))?
        .ok_or("Entry not found")?;

    let encrypted_data = existing.2;

    // Decrypt existing entry
    let decrypted = CipherModule::decrypt(&key, &encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    let mut entry: Entry = serde_json::from_slice(&decrypted)
        .map_err(|e| format!("Failed to parse entry: {}", e))?;

    // Update fields
    if let Some(title) = input.title {
        entry.title = title;
    }
    if let Some(username) = input.username {
        entry.username = Some(username);
    }
    if let Some(password) = input.password {
        entry.password = Some(password);
    }
    if let Some(url) = input.url {
        entry.url = Some(url);
    }
    if let Some(notes) = input.notes {
        entry.notes = Some(notes);
    }
    if let Some(folder_id) = input.folder_id {
        entry.folder_id = Some(folder_id);
    }
    if let Some(tags) = input.tags {
        entry.tags = Some(serde_json::to_string(&tags).unwrap_or_default());
    }

    let now = Utc::now().to_rfc3339();
    entry.updated_at = now.clone();

    let new_encrypted = CipherModule::encrypt(&key, serde_json::to_vec(&entry).unwrap().as_slice())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    repo.update_entry(&input.id, entry.folder_id.as_deref(), &new_encrypted, &now)
        .map_err(|e| format!("Failed to update entry: {}", e))?;

    Ok(entry)
}

#[tauri::command]
pub fn delete_entry(id: &str, state: State<'_, VaultState>) -> Result<(), String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock.as_ref().ok_or("Vault not initialized")?;

    repo.delete_entry(id)
        .map_err(|e| format!("Failed to delete entry: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_entry(id: &str, state: State<'_, VaultState>) -> Result<Entry, String> {
    let key = state.keyring.get_key().ok_or("Vault is locked")?;

    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock.as_ref().ok_or("Vault not initialized")?;

    let (_entry_id, _folder_id, encrypted_data, _created_at, _updated_at) = repo.get_entry(id)
        .map_err(|e| format!("Failed to get entry: {}", e))?
        .ok_or("Entry not found")?;

    let decrypted = CipherModule::decrypt(&key, &encrypted_data)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    let entry: Entry = serde_json::from_slice(&decrypted)
        .map_err(|e| format!("Failed to parse entry: {}", e))?;

    Ok(entry)
}

#[tauri::command]
pub fn list_entries(state: State<'_, VaultState>) -> Result<Vec<Entry>, String> {
    let key = state.keyring.get_key().ok_or("Vault is locked")?;

    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock.as_ref().ok_or("Vault not initialized")?;

    let entries_data = repo.list_entries()
        .map_err(|e| format!("Failed to list entries: {}", e))?;

    let mut entries = Vec::new();
    for (_id, _folder_id, encrypted_data, _created_at, _updated_at) in entries_data {
        match CipherModule::decrypt(&key, &encrypted_data) {
            Ok(decrypted) => {
                if let Ok(entry) = serde_json::from_slice::<Entry>(&decrypted) {
                    entries.push(entry);
                }
            }
            Err(_) => continue, // Skip entries that fail to decrypt
        }
    }

    Ok(entries)
}

#[tauri::command]
pub fn search_entries(query: &str, state: State<'_, VaultState>) -> Result<Vec<Entry>, String> {
    let entries = list_entries(state)?;
    let query_lower = query.to_lowercase();

    let filtered: Vec<Entry> = entries.into_iter().filter(|entry| {
        entry.title.to_lowercase().contains(&query_lower)
            || entry.username.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false)
            || entry.url.as_ref().map(|u| u.to_lowercase().contains(&query_lower)).unwrap_or(false)
            || entry.notes.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false)
    }).collect();

    Ok(filtered)
}