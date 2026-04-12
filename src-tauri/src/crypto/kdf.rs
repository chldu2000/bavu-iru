use argon2::{
    password_hash::rand_core::{OsRng, RngCore},
    Argon2, Params, Version,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use zeroize::Zeroizing;

const ARGON2_TIME_COST: u32 = 3;
const ARGON2_MEMORY_COST_KIB: u32 = 65536; // 64 MiB
const ARGON2_PARALLELISM: u32 = 4;
const SALT_LENGTH: usize = 32;
const KEY_LENGTH: usize = 32; // 256 bits

pub struct KdfModule;

impl KdfModule {
    pub fn generate_salt() -> [u8; SALT_LENGTH] {
        let mut salt = [0u8; SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    pub fn derive_key(password: &str, salt: &[u8; SALT_LENGTH]) -> Result<Zeroizing<[u8; KEY_LENGTH]>, argon2::Error> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            Params::new(
                ARGON2_MEMORY_COST_KIB,
                ARGON2_TIME_COST,
                ARGON2_PARALLELISM,
                Some(KEY_LENGTH),
            )?,
        );

        let mut key = Zeroizing::new([0u8; KEY_LENGTH]);
        argon2.hash_password_into(password.as_bytes(), salt, &mut *key)?;

        Ok(key)
    }

    pub fn salt_to_string(salt: &[u8; SALT_LENGTH]) -> String {
        BASE64.encode(salt)
    }

    pub fn salt_from_string(s: &str) -> Result<[u8; SALT_LENGTH], String> {
        let decoded = BASE64.decode(s).map_err(|e| format!("Failed to decode salt: {}", e))?;
        if decoded.len() != SALT_LENGTH {
            return Err(format!("Invalid salt length: {}", decoded.len()));
        }
        let mut salt = [0u8; SALT_LENGTH];
        salt.copy_from_slice(&decoded);
        Ok(salt)
    }
}