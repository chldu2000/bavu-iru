use crate::crypto::{KdfModule, Keyring};
use crate::db::Repository;
use chrono::Utc;
use std::sync::Mutex;
use tauri::State;

pub struct VaultState {
    pub keyring: Keyring,
    pub repository: Mutex<Option<Repository>>,
}

impl VaultState {
    pub fn new() -> Self {
        Self {
            keyring: Keyring::new(),
            repository: Mutex::new(None),
        }
    }
}

impl Default for VaultState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub fn setup_vault(password: &str, state: State<'_, VaultState>) -> Result<(), String> {
    let salt = KdfModule::generate_salt();
    let key = KdfModule::derive_key(password, &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Create repository
    let repo = Repository::new("vault.db")
        .map_err(|e| format!("Failed to create database: {}", e))?;

    // Store salt and metadata
    repo.set_meta("kdf_salt", &KdfModule::salt_to_string(&salt))
        .map_err(|e| format!("Failed to store salt: {}", e))?;
    repo.set_meta("version", "1.0")
        .map_err(|e| format!("Failed to store version: {}", e))?;
    repo.set_meta("created_at", &Utc::now().to_rfc3339())
        .map_err(|e| format!("Failed to store created_at: {}", e))?;
    repo.set_meta("last_modified", &Utc::now().to_rfc3339())
        .map_err(|e| format!("Failed to store last_modified: {}", e))?;

    // Unlock keyring
    state.keyring.unlock(key);

    // Store repository
    let mut repo_lock = state.repository.lock().unwrap();
    *repo_lock = Some(repo);

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(password: &str, state: State<'_, VaultState>) -> Result<(), String> {
    let repo_lock = state.repository.lock().unwrap();
    let repo = repo_lock.as_ref().ok_or("Vault not initialized")?;

    let salt_str = repo.get_meta("kdf_salt")
        .map_err(|e| format!("Failed to get salt: {}", e))?
        .ok_or("Salt not found")?;

    let salt = KdfModule::salt_from_string(&salt_str)
        .map_err(|e| format!("Invalid salt: {}", e))?;

    let key = KdfModule::derive_key(password, &salt)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Test decryption with the key by trying to decrypt existing entries
    // For now, just unlock if key derivation succeeds
    drop(repo_lock);

    state.keyring.unlock(key);

    Ok(())
}

#[tauri::command]
pub fn lock_vault(state: State<'_, VaultState>) -> Result<(), String> {
    state.keyring.lock();
    Ok(())
}

#[tauri::command]
pub fn is_vault_unlocked(state: State<'_, VaultState>) -> bool {
    state.keyring.is_unlocked()
}

#[tauri::command]
pub fn is_vault_initialized(state: State<'_, VaultState>) -> Result<bool, String> {
    let repo_lock = state.repository.lock().unwrap();
    if let Some(repo) = repo_lock.as_ref() {
        let has_salt = repo.get_meta("kdf_salt")
            .map_err(|e| format!("Failed to get meta: {}", e))?
            .is_some();
        Ok(has_salt)
    } else {
        Ok(false)
    }
}