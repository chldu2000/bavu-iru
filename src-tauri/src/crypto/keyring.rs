use std::sync::RwLock;
use zeroize::Zeroizing;

pub struct Keyring {
    master_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
}

impl Keyring {
    pub fn new() -> Self {
        Self {
            master_key: RwLock::new(None),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.master_key.read().unwrap().is_some()
    }

    pub fn lock(&self) {
        let mut key = self.master_key.write().unwrap();
        *key = None;
    }

    pub fn unlock(&self, key: Zeroizing<[u8; 32]>) {
        let mut master_key = self.master_key.write().unwrap();
        *master_key = Some(key);
    }

    pub fn get_key(&self) -> Option<Zeroizing<[u8; 32]>> {
        let key = self.master_key.read().unwrap();
        key.clone()
    }
}

impl Default for Keyring {
    fn default() -> Self {
        Self::new()
    }
}