# SecKeyBox Phase 2: Windows Hello Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan.

**Goal:** Add Windows Hello biometric authentication for quick unlock.

**Architecture:** Use tauri-plugin-biometric for authentication, store encrypted master password in Windows Credential Manager.

**Tech Stack:** Tauri 2.x, Rust, tauri-plugin-biometric, keyring

---

## File Structure

```
src-tauri/
├── Cargo.toml              # MODIFY: Add tauri-plugin-biometric, keyring
├── src/
│   ├── lib.rs              # MODIFY: Add biometric plugin
│   └── commands/
│       └── biometric.rs    # NEW: setup_biometric, unlock_with_biometric

src/
└── components/
    └── unlock/
        └── UnlockScreen.tsx  # MODIFY: Add Windows Hello button
```

---

## Chunk 1: Backend Setup

### Task 1.1: Add dependencies

- [ ] Add to Cargo.toml:
```toml
tauri-plugin-biometric = "2"
keyring = "3"
```

### Task 1.2: Create biometric commands

- [ ] Create `src-tauri/src/commands/biometric.rs`:
```rust
use crate::error::{Result, VaultError};
use crate::state::VaultState;
use tauri::State;
use tauri_plugin_biometric::Biometric;

#[tauri::command]
pub async fn is_biometric_available(app: tauri::AppHandle) -> Result<bool> {
    let biometric = Biometric::new(app);
    Ok(biometric.status().await?.is_available())
}

#[tauri::command]
pub async fn setup_biometric_unlock(
    app: tauri::AppHandle,
    vault_state: State<VaultState>,
) -> Result<()> {
    let biometric = Biometric::new(app);
    biometric.authenticate("Enable Windows Hello for SecKeyBox").await?;
    
    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let entry = keyring::Entry::new("SecKeyBox", "master_key")?;
    entry.set_password(&String::from_utf8_lossy(key.as_bytes()))?;
    
    Ok(())
}

#[tauri::command]
pub async fn unlock_with_biometric(
    app: tauri::AppHandle,
    vault_state: State<VaultState>,
) -> Result<bool> {
    let biometric = Biometric::new(app);
    biometric.authenticate("Unlock SecKeyBox").await?;
    
    let entry = keyring::Entry::new("SecKeyBox", "master_key")?;
    let password = entry.get_password()?;
    
    // Store in vault state
    vault_state.set_master_key(password.as_bytes());
    
    Ok(true)
}
```

### Task 1.3: Register commands and plugin

- [ ] Add biometric module to mod.rs
- [ ] Add plugin to lib.rs
- [ ] Register commands in invoke_handler

---

## Chunk 2: Frontend Integration

### Task 2.1: Update UnlockScreen

- [ ] Add Windows Hello button
- [ ] Check biometric availability on mount
- [ ] Call unlock_with_biometric on click

---

## Chunk 3: Build and Test

- [ ] Verify Rust builds
- [ ] Verify frontend builds
- [ ] Manual testing