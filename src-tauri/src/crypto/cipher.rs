use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use argon2::password_hash::rand_core::RngCore;

const NONCE_LENGTH: usize = 12;

pub struct CipherModule;

impl CipherModule {
    pub fn generate_nonce() -> [u8; NONCE_LENGTH] {
        let mut nonce = [0u8; NONCE_LENGTH];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<String, cipher::Error> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_e| cipher::Error::KeyLength)?;

        let nonce_bytes = Self::generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| cipher::Error::Encryption)?;

        // Prepend nonce to ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend(ciphertext);

        Ok(BASE64.encode(&combined))
    }

    pub fn decrypt(key: &[u8; 32], encoded: &str) -> Result<Vec<u8>, cipher::Error> {
        let combined = BASE64.decode(encoded)
            .map_err(|_| cipher::Error::Decode)?;

        if combined.len() < NONCE_LENGTH {
            return Err(cipher::Error::Decode);
        }

        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LENGTH);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| cipher::Error::KeyLength)?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| cipher::Error::Decryption)
    }
}

pub mod cipher {
    #[derive(Debug)]
    pub enum Error {
        KeyLength,
        Encryption,
        Decryption,
        Decode,
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::KeyLength => write!(f, "Invalid key length"),
                Error::Encryption => write!(f, "Encryption failed"),
                Error::Decryption => write!(f, "Decryption failed"),
                Error::Decode => write!(f, "Base64 decode failed"),
            }
        }
    }

    impl std::error::Error for Error {}
}