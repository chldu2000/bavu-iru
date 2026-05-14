use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
	#[error("Vault is locked")]
	VaultLocked,
	#[error("Vault is already unlocked")]
	VaultAlreadyUnlocked,
	#[error("Vault not found")]
	VaultNotFound,
	#[error("Vault already exists")]
	VaultAlreadyExists,
	#[error("Invalid password")]
	InvalidPassword,
	#[error("Database error: {0}")]
	Database(#[from] rusqlite::Error),
	#[error("Encryption error: {0}")]
	Encryption(String),
	#[error("Key derivation error: {0}")]
	Kdf(String),
	#[error("Base64 decode error: {0}")]
	Base64(#[from] base64::DecodeError),
	#[error("Clipboard error: {0}")]
	Clipboard(String),
	#[error("Import error: {0}")]
	Import(String),
	#[error("Export error: {0}")]
	Export(String),
}

impl From<aes_gcm::Error> for AppError {
	fn from(_: aes_gcm::Error) -> Self {
		AppError::Encryption("AES-GCM operation failed".into())
	}
}

impl From<argon2::Error> for AppError {
	fn from(e: argon2::Error) -> Self {
		AppError::Kdf(format!("{:?}", e))
	}
}

impl From<csv::Error> for AppError {
	fn from(e: csv::Error) -> Self {
		AppError::Import(e.to_string())
	}
}

impl serde::Serialize for AppError {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}
