use std::sync::RwLock;
use crate::crypto::SecureKey;

pub struct VaultState {
    master_key: RwLock<Option<SecureKey>>,
    is_unlocked: RwLock<bool>,
}

impl VaultState {
    pub fn new() -> Self {
        Self {
            master_key: RwLock::new(None),
            is_unlocked: RwLock::new(false),
        }
    }

    pub fn unlock(&self, key: SecureKey) {
        let mut master_key = self.master_key.write().unwrap();
        *master_key = Some(key);
        let mut is_unlocked = self.is_unlocked.write().unwrap();
        *is_unlocked = true;
    }

    pub fn lock(&self) {
        let mut master_key = self.master_key.write().unwrap();
        *master_key = None;
        let mut is_unlocked = self.is_unlocked.write().unwrap();
        *is_unlocked = false;
    }

    pub fn is_unlocked(&self) -> bool {
        *self.is_unlocked.read().unwrap()
    }

    pub fn get_master_key(&self) -> Option<SecureKey> {
        let master_key = self.master_key.read().unwrap();
        master_key.as_ref().map(|k| SecureKey::new(*k.as_bytes()))
    }
}

impl Default for VaultState {
    fn default() -> Self {
        Self::new()
    }
}