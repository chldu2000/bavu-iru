use argon2::{Argon2, Algorithm, Version, Params};
use rand::RngCore;
use zeroize::Zeroize;

const SALT_LEN: usize = 32;
const KEY_LEN: usize = 32;
const TIME_COST: u32 = 3;
const MEMORY_COST: u32 = 65536; // 64 MiB
const PARALLELISM: u32 = 4;

pub struct KdfParams {
	pub salt: [u8; SALT_LEN],
	pub time_cost: u32,
	pub memory_cost: u32,
	pub parallelism: u32,
}

impl Default for KdfParams {
	fn default() -> Self {
		let mut salt = [0u8; SALT_LEN];
		rand::thread_rng().fill_bytes(&mut salt);
		Self {
			salt,
			time_cost: TIME_COST,
			memory_cost: MEMORY_COST,
			parallelism: PARALLELISM,
		}
	}
}

/// Derive a 256-bit key from a password using Argon2id.
/// The caller is responsible for zeroizing the returned key when done.
pub fn derive_key(password: &str, params: &KdfParams) -> Result<[u8; KEY_LEN], argon2::Error> {
	let argon2 = Argon2::new(
		Algorithm::Argon2id,
		Version::V0x13,
		Params::new(params.memory_cost, params.time_cost, params.parallelism, Some(KEY_LEN))?,
	);

	let mut key = [0u8; KEY_LEN];
	argon2.hash_password_into(password.as_bytes(), &params.salt, &mut key)?;
	Ok(key)
}

/// Zeroize a key in place.
pub fn zero_key(key: &mut [u8]) {
	key.zeroize();
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_derive_key_deterministic() {
		let params = KdfParams::default();
		let key1 = derive_key("testpassword", &params).unwrap();
		let key2 = derive_key("testpassword", &params).unwrap();
		assert_eq!(key1, key2);
	}

	#[test]
	fn test_different_salts_produce_different_keys() {
		let params1 = KdfParams::default();
		let mut params2 = KdfParams::default();
		params2.salt = [0u8; SALT_LEN]; // different salt
		let key1 = derive_key("testpassword", &params1).unwrap();
		let key2 = derive_key("testpassword", &params2).unwrap();
		assert_ne!(key1, key2);
	}
}
