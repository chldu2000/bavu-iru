use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use tauri::State;

use crate::crypto::keyring::Keyring;
use crate::crypto::kdf::{self, KdfParams};
use crate::db::repository::Database;
use crate::error::AppError;

#[tauri::command]
pub fn vault_setup(password: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<(), AppError> {
	if keyring.is_unlocked() {
		return Err(AppError::VaultAlreadyUnlocked);
	}

	// Check if vault already exists
	let existing = db.get_meta("kdf_salt")?;
	if existing.is_some() {
		return Err(AppError::VaultAlreadyExists);
	}

	let params = KdfParams::default();
	let key = kdf::derive_key(&password, &params)?;

	// Store KDF params (salt is not secret)
	db.set_meta("kdf_salt", &BASE64.encode(params.salt))?;
	db.set_meta("kdf_time_cost", &params.time_cost.to_string())?;
	db.set_meta("kdf_memory_cost", &params.memory_cost.to_string())?;
	db.set_meta("kdf_parallelism", &params.parallelism.to_string())?;

	// Verify roundtrip: store a known verification token
	let verify_token = b"bavu-iru-vault-verify";
	let encrypted = crate::crypto::cipher::encrypt(&key, verify_token)?;
	db.set_meta("vault_verify", &BASE64.encode(&encrypted))?;

	keyring.set(key);
	Ok(())
}

#[tauri::command]
pub fn vault_unlock(password: String, db: State<'_, Database>, keyring: State<'_, Keyring>) -> Result<(), AppError> {
	if keyring.is_unlocked() {
		return Err(AppError::VaultAlreadyUnlocked);
	}

	let salt_b64 = db.get_meta("kdf_salt")?.ok_or(AppError::VaultNotFound)?;
	let salt = BASE64.decode(&salt_b64)?;

	let time_cost: u32 = db.get_meta("kdf_time_cost")?.unwrap_or_default().parse().unwrap_or(3);
	let memory_cost: u32 = db.get_meta("kdf_memory_cost")?.unwrap_or_default().parse().unwrap_or(65536);
	let parallelism: u32 = db.get_meta("kdf_parallelism")?.unwrap_or_default().parse().unwrap_or(4);

	let mut salt_arr = [0u8; 32];
	salt_arr.copy_from_slice(&salt[..32]);

	let params = KdfParams {
		salt: salt_arr,
		time_cost,
		memory_cost,
		parallelism,
	};
	let key = kdf::derive_key(&password, &params)?;

	// Verify the key is correct
	let verify_b64 = db.get_meta("vault_verify")?.ok_or(AppError::VaultNotFound)?;
	let verify_data = BASE64.decode(&verify_b64)?;
	let decrypted = crate::crypto::cipher::decrypt(&key, &verify_data)?;
	if decrypted != b"bavu-iru-vault-verify" {
		return Err(AppError::InvalidPassword);
	}

	keyring.set(key);
	Ok(())
}

#[tauri::command]
pub fn vault_lock(keyring: State<'_, Keyring>) -> Result<(), AppError> {
	keyring.lock();
	Ok(())
}

#[tauri::command]
pub fn vault_status(keyring: State<'_, Keyring>) -> bool {
	keyring.is_unlocked()
}
