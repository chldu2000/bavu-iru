use aes_gcm::{
	aead::{Aead, KeyInit, OsRng},
	Aes256Gcm, Nonce,
};
use rand::RngCore;

const NONCE_LEN: usize = 12;

/// Encrypt plaintext using AES-256-GCM.
/// Returns `[nonce (12 bytes) || ciphertext + auth_tag]`.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
	let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| aes_gcm::Error)?;
	let mut nonce_bytes = [0u8; NONCE_LEN];
	OsRng.fill_bytes(&mut nonce_bytes);
	let nonce = Nonce::from_slice(&nonce_bytes);

	let ciphertext = cipher.encrypt(nonce, plaintext)?;

	let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
	output.extend_from_slice(&nonce_bytes);
	output.extend_from_slice(&ciphertext);
	Ok(output)
}

/// Decrypt data encrypted by `encrypt`.
/// Expects input format: `[nonce (12 bytes) || ciphertext + auth_tag]`.
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
	if data.len() < NONCE_LEN {
		return Err(aes_gcm::Error);
	}
	let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
	let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| aes_gcm::Error)?;
	let nonce = Nonce::from_slice(nonce_bytes);
	cipher.decrypt(nonce, ciphertext)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_encrypt_decrypt_roundtrip() {
		let key = [42u8; 32];
		let plaintext = b"hello, world!";
		let encrypted = encrypt(&key, plaintext).unwrap();
		let decrypted = decrypt(&key, &encrypted).unwrap();
		assert_eq!(decrypted, plaintext);
	}

	#[test]
	fn test_different_nonces_per_encryption() {
		let key = [42u8; 32];
		let plaintext = b"same data";
		let e1 = encrypt(&key, plaintext).unwrap();
		let e2 = encrypt(&key, plaintext).unwrap();
		// Nonces (first 12 bytes) should differ
		assert_ne!(&e1[..12], &e2[..12]);
		// Full ciphertexts should differ
		assert_ne!(e1, e2);
	}

	#[test]
	fn test_wrong_key_fails() {
		let key1 = [1u8; 32];
		let key2 = [2u8; 32];
		let encrypted = encrypt(&key1, b"secret").unwrap();
		assert!(decrypt(&key2, &encrypted).is_err());
	}
}
