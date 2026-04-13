use std::sync::Mutex;
use zeroize::Zeroize;

/// In-memory key storage. The derived master key lives here
/// and is zeroized on lock/vault close.
pub struct Keyring {
	inner: Mutex<Option<[u8; 32]>>,
}

impl Keyring {
	pub fn new() -> Self {
		Self {
			inner: Mutex::new(None),
		}
	}

	/// Store the derived master key. Any previous key is zeroized first.
	pub fn set(&self, key: [u8; 32]) {
		let mut guard = self.inner.lock().unwrap();
		if let Some(mut old) = guard.take() {
			old.zeroize();
		}
		*guard = Some(key);
	}

	/// Access the key for encryption/decryption operations.
	pub fn with_key<F, R>(&self, f: F) -> Option<R>
	where
		F: FnOnce(&[u8; 32]) -> R,
	{
		let guard = self.inner.lock().unwrap();
		guard.as_ref().map(f)
	}

	/// Check if a key is currently stored (vault is unlocked).
	pub fn is_unlocked(&self) -> bool {
		self.inner.lock().unwrap().is_some()
	}

	/// Lock the vault: zeroize and remove the key from memory.
	pub fn lock(&self) {
		let mut guard = self.inner.lock().unwrap();
		if let Some(mut key) = guard.take() {
			key.zeroize();
		}
	}
}

impl Default for Keyring {
	fn default() -> Self {
		Self::new()
	}
}
