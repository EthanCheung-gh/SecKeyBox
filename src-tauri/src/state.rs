use std::sync::RwLock;
use crate::crypto::SecureKey;

fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct VaultState {
    master_key: RwLock<Option<SecureKey>>,
    is_unlocked: RwLock<bool>,
    /// Epoch seconds of the last reported user interaction; 0 while locked.
    last_activity: RwLock<i64>,
    /// Idle minutes before the watchdog locks the vault; 0 disables it.
    auto_lock_minutes: RwLock<i64>,
}

impl VaultState {
    pub fn new() -> Self {
        Self {
            master_key: RwLock::new(None),
            is_unlocked: RwLock::new(false),
            last_activity: RwLock::new(0),
            auto_lock_minutes: RwLock::new(5),
        }
    }

    pub fn unlock(&self, key: SecureKey) {
        let mut master_key = self.master_key.write().unwrap();
        *master_key = Some(key);
        let mut is_unlocked = self.is_unlocked.write().unwrap();
        *is_unlocked = true;
        drop(is_unlocked);
        *self.last_activity.write().unwrap() = now_epoch_secs();
    }

    pub fn lock(&self) {
        let mut master_key = self.master_key.write().unwrap();
        *master_key = None;
        let mut is_unlocked = self.is_unlocked.write().unwrap();
        *is_unlocked = false;
        drop(is_unlocked);
        *self.last_activity.write().unwrap() = 0;
    }

    pub fn is_unlocked(&self) -> bool {
        *self.is_unlocked.read().unwrap()
    }

    pub fn get_master_key(&self) -> Option<SecureKey> {
        let master_key = self.master_key.read().unwrap();
        master_key.as_ref().map(|k| SecureKey::new(*k.as_bytes()))
    }

    /// Refresh the idle timer. No-op while locked so a late activity event
    /// cannot resurrect a locked vault's timer.
    pub fn touch_activity(&self) {
        if !self.is_unlocked() {
            return;
        }
        *self.last_activity.write().unwrap() = now_epoch_secs();
    }

    pub fn last_activity(&self) -> i64 {
        *self.last_activity.read().unwrap()
    }

    pub fn auto_lock_minutes(&self) -> i64 {
        *self.auto_lock_minutes.read().unwrap()
    }

    pub fn set_auto_lock_minutes(&self, minutes: i64) {
        *self.auto_lock_minutes.write().unwrap() = minutes.max(0);
    }

    pub fn now_epoch_secs(&self) -> i64 {
        now_epoch_secs()
    }
}

impl Default for VaultState {
    fn default() -> Self {
        Self::new()
    }
}
