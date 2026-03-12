# SecKeyBox Phase 1 MVP Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a functional Windows password vault with master password protection, three-column UI, and account/password management.

**Architecture:** Tauri 2.x desktop app with Rust backend for crypto/database operations and React frontend for UI. Data stored in encrypted SQLite database. Master password derives encryption key via Argon2id.

**Tech Stack:** Tauri 2.x, React 18, TypeScript, Tailwind CSS, Zustand, Rust, SQLite, AES-256-GCM, Argon2id

---

## File Structure

```
SecKeyBox/
├── src/                              # React frontend
│   ├── components/
│   │   ├── ui/                       # Shadcn/ui components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── input.tsx
│   │   │   ├── toast.tsx
│   │   │   └── tooltip.tsx
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── ItemList.tsx
│   │   │   └── DetailPanel.tsx
│   │   ├── unlock/
│   │   │   ├── UnlockScreen.tsx
│   │   │   └── SetupScreen.tsx
│   │   └── modals/
│   │       ├── AddEditItemModal.tsx
│   │       ├── AddGroupModal.tsx
│   │       ├── EditGroupModal.tsx
│   │       └── DeleteConfirmModal.tsx
│   ├── hooks/
│   │   └── useActivityTracker.ts
│   ├── lib/
│   │   ├── tauri.ts
│   │   └── utils.ts
│   ├── stores/
│   │   ├── vault.ts
│   │   └── ui.ts
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   ├── kdf.rs
│   │   │   └── cipher.rs
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── operations.rs
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── vault.rs
│   │   │   ├── groups.rs
│   │   │   ├── items.rs
│   │   │   └── clipboard.rs
│   │   ├── state.rs
│   │   ├── error.rs
│   │   ├── lib.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
└── index.html
```

---

## Chunk 1: Project Initialization

### Task 1.1: Create Tauri + React Project

**Files:**
- Create: `package.json`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `vite.config.ts`
- Create: `tailwind.config.js`
- Create: `postcss.config.js`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/index.css`
- Create: `src/App.tsx`

- [ ] **Step 1: Initialize git repository**

Run: `git init`
Expected: Git repository initialized

Create `.gitignore`:

```
# Dependencies
node_modules/

# Build outputs
dist/
target/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local
```

- [ ] **Step 2: Initialize npm project with package.json**

Create `package.json`:

```json
{
  "name": "seckeybox",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-clipboard-manager": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "lucide-react": "^0.460.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "zustand": "^5.0.0",
    "clsx": "^2.1.1",
    "tailwind-merge": "^2.5.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.20",
    "postcss": "^8.4.45",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.6.0",
    "vite": "^5.4.0"
  }
}
```

- [ ] **Step 3: Create TypeScript configuration**

Create `tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

Create `tsconfig.node.json`:

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 4: Create Vite configuration**

Create `vite.config.ts`:

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
```

- [ ] **Step 5: Create Tailwind configuration**

Create `tailwind.config.js`:

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          100: '#dbeafe',
          200: '#bfdbfe',
          300: '#93c5fd',
          400: '#60a5fa',
          500: '#3b82f6',
          600: '#2563eb',
          700: '#1d4ed8',
          800: '#1e40af',
          900: '#1e3a8a',
        },
      },
      borderRadius: {
        lg: '0.5rem',
        md: '0.375rem',
      },
    },
  },
  plugins: [],
};
```

Create `postcss.config.js`:

```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

- [ ] **Step 6: Create index.html**

Create `index.html`:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>SecKeyBox</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 7: Create React entry point**

Create `src/main.tsx`:

```typescript
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

Create `src/index.css`:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
  font-family: 'Segoe UI Variable', 'Segoe UI', system-ui, -apple-system, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

body {
  margin: 0;
  min-height: 100vh;
}

#root {
  min-height: 100vh;
}
```

Create `src/App.tsx`:

```typescript
function App() {
  return (
    <div className="min-h-screen bg-gray-100 flex items-center justify-center">
      <h1 className="text-2xl font-semibold text-gray-800">SecKeyBox</h1>
    </div>
  );
}

export default App;
```

- [ ] **Step 8: Install frontend dependencies**

Run: `npm install`
Expected: Dependencies installed successfully

- [ ] **Step 9: Verify frontend build works**

Run: `npm run build`
Expected: Build completes without errors

- [ ] **Step 10: Commit frontend setup**

```bash
git add -A
git commit -m "feat: initialize React + TypeScript + Tailwind frontend"
```

### Task 1.2: Initialize Tauri Backend

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/build.rs`

- [ ] **Step 1: Create Tauri directory structure**

Run: `mkdir src-tauri\src src-tauri\icons` (Windows) or `mkdir -p src-tauri/src src-tauri/icons` (Unix)
Expected: Directories created

- [ ] **Step 2: Create Cargo.toml**

Create `src-tauri/Cargo.toml`:

```toml
[package]
name = "seckeybox"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-clipboard-manager = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
rusqlite = { version = "0.32", features = ["bundled"] }
argon2 = "0.5"
aes-gcm = "0.10"
rand = "0.8"
zeroize = "1"
thiserror = "2"
parking_lot = "0.12"

[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
```

- [ ] **Step 3: Create tauri.conf.json**

Create `src-tauri/tauri.conf.json`:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "SecKeyBox",
  "version": "0.1.0",
  "identifier": "com.seckeybox.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "SecKeyBox",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "decorations": true,
        "transparent": false,
        "center": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "clipboard-manager": {
      "all": true
    }
  }
}
```

- [ ] **Step 4: Create build.rs**

Create `src-tauri/build.rs`:

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 5: Create main.rs**

Create `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    seckeybox::run()
}
```

- [ ] **Step 6: Create lib.rs (initial)**

Create `src-tauri/src/lib.rs`:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: Skip icon generation (use defaults)**

Skip icon generation for now - Tauri will use default icons. Custom icons can be added later with `npx tauri icon <source-image.png>`.

Note: To add custom icons in future, prepare a 1024x1024 PNG and run `npx tauri icon path/to/icon.png`

- [ ] **Step 8: Test Tauri dev mode**

Run: `npm run tauri dev`
Expected: App window opens showing "SecKeyBox" heading

- [ ] **Step 9: Commit Tauri setup**

```bash
git add -A
git commit -m "feat: initialize Tauri 2.x backend with clipboard plugin"
```

### Task 1.3: Create Type Definitions

**Files:**
- Create: `src/types/index.ts`

- [ ] **Step 1: Create TypeScript type definitions**

Create `src/types/index.ts`:

```typescript
export interface Group {
  id: string;
  name: string;
  icon?: string;
  parent_id?: string;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface ItemSummary {
  id: string;
  title: string;
  subtitle: string;
  icon?: string;
  type: 'account';
  is_favorite: boolean;
}

export interface ItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'account';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  username: string;
  password: string;
  website?: string;
  notes?: string;
}

export interface CreateAccountItemInput {
  group_id: string;
  title: string;
  username: string;
  password: string;
  website?: string;
  notes?: string;
}

export interface UpdateAccountItemInput {
  id: string;
  title: string;
  username: string;
  password?: string;
  website?: string;
  notes?: string;
}

export interface CreateGroupInput {
  name: string;
  icon?: string;
  parent_id?: string;
  sort_order?: number;
}

export interface UpdateGroupInput {
  id: string;
  name: string;
  icon?: string;
  sort_order?: number;
}

export type VaultError =
  | 'InvalidPassword'
  | 'VaultLocked'
  | 'ItemNotFound'
  | 'GroupNotEmpty'
  | 'CryptoError'
  | 'DatabaseError';
```

- [ ] **Step 2: Commit type definitions**

```bash
git add -A
git commit -m "feat: add TypeScript type definitions for vault data models"
```

---

## Chunk 2: Rust Backend - Crypto Module

### Task 2.1: Create Error Types

**Files:**
- Create: `src-tauri/src/error.rs`

- [ ] **Step 1: Create error module**

Create `src-tauri/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Vault is locked")]
    VaultLocked,

    #[error("Item not found")]
    ItemNotFound,

    #[error("Group not empty")]
    GroupNotEmpty,

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Vault not initialized")]
    VaultNotInitialized,

    #[error("Vault already initialized")]
    VaultAlreadyInitialized,
}

pub type Result<T> = std::result::Result<T, VaultError>;
```

- [ ] **Step 2: Commit error types**

```bash
git add -A
git commit -m "feat: add Rust error types for vault operations"
```

### Task 2.2: Create Key Derivation Function (KDF)

**Files:**
- Create: `src-tauri/src/crypto/mod.rs`
- Create: `src-tauri/src/crypto/kdf.rs`

- [ ] **Step 1: Create crypto module structure**

Create `src-tauri/src/crypto/mod.rs`:

```rust
mod kdf;
mod cipher;

pub use kdf::*;
pub use cipher::*;
```

- [ ] **Step 2: Create KDF implementation**

Create `src-tauri/src/crypto/kdf.rs`:

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use zeroize::Zeroize;

use crate::error::{Result, VaultError};

const MEMORY_COST: u32 = 64 * 1024; // 64MB in KiB (Argon2 m_cost unit)
const TIME_COST: u32 = 3;
const PARALLELISM: u32 = 4;

pub struct DerivedKey {
    pub key: [u8; 32],
    pub salt: String,
    pub hash: String,
}

impl Drop for DerivedKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

pub fn derive_key(password: &str, salt: Option<&str>) -> Result<DerivedKey> {
    let salt = match salt {
        Some(s) => SaltString::from_b64(s).map_err(|e| VaultError::CryptoError(e.to_string()))?,
        None => SaltString::generate(&mut OsRng),
    };

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST, TIME_COST, PARALLELISM, None)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    );

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let key_bytes = hash
        .hash
        .ok_or_else(|| VaultError::CryptoError("Failed to generate hash".to_string()))?
        .as_bytes();

    let mut key = [0u8; 32];
    key.copy_from_slice(&key_bytes[..32]);

    Ok(DerivedKey {
        key,
        salt: salt.to_string(),
        hash: hash.to_string(),
    })
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(MEMORY_COST, TIME_COST, PARALLELISM, None)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?,
    );

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

- [ ] **Step 3: Commit KDF module**

```bash
git add -A
git commit -m "feat: implement Argon2id key derivation for master password"
```

### Task 2.3: Create AES-256-GCM Cipher

**Files:**
- Create: `src-tauri/src/crypto/cipher.rs`

- [ ] **Step 1: Create cipher implementation**

Create `src-tauri/src/crypto/cipher.rs`:

```rust
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use zeroize::Zeroize;

use crate::error::{Result, VaultError};

const NONCE_SIZE: usize = 12;

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_SIZE],
}

pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<EncryptedData> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes,
    })
}

pub fn decrypt(key: &[u8; 32], nonce: &[u8; NONCE_SIZE], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    Ok(plaintext)
}

pub struct SecureKey {
    key: [u8; 32],
}

impl SecureKey {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl Drop for SecureKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}
```

- [ ] **Step 2: Commit cipher module**

```bash
git add -A
git commit -m "feat: implement AES-256-GCM encryption for sensitive data"
```

### Task 2.4: Create Vault State

**Files:**
- Create: `src-tauri/src/state.rs`

- [ ] **Step 1: Create vault state module**

Create `src-tauri/src/state.rs`:

```rust
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
```

- [ ] **Step 2: Commit state module**

```bash
git add -A
git commit -m "feat: add vault state management with secure key storage"
```

### Task 2.5: Update lib.rs with New Modules

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update lib.rs to include crypto modules**

Update `src-tauri/src/lib.rs`:

```rust
mod error;
mod crypto;
mod state;

pub use error::{VaultError, Result};
pub use crypto::{derive_key, verify_password, encrypt, decrypt, SecureKey, EncryptedData};
pub use state::VaultState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify Rust code compiles**

Run: `cd src-tauri && cargo check`
Expected: Compilation succeeds without errors

- [ ] **Step 3: Commit lib.rs update**

```bash
git add -A
git commit -m "feat: integrate crypto modules into lib.rs"
```

---

## Chunk 3: Rust Backend - Database Module

### Task 3.1: Create Database Schema

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: Create database module structure**

Create `src-tauri/src/db/mod.rs`:

```rust
mod schema;
mod operations;

pub use schema::*;
pub use operations::*;
```

- [ ] **Step 2: Create database schema**

Create `src-tauri/src/db/schema.rs`:

```rust
use rusqlite::Connection;
use crate::error::Result;

const SCHEMA_VERSION: i32 = 1;

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS groups (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            icon TEXT,
            parent_id TEXT REFERENCES groups(id),
            sort_order INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
            type TEXT NOT NULL CHECK(type IN ('account', 'api_key', 'env_var')),
            title TEXT NOT NULL,
            icon TEXT,
            is_favorite INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS account_items (
            item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            username TEXT NOT NULL,
            password_encrypted BLOB NOT NULL,
            password_nonce BLOB NOT NULL,
            website TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS api_key_items (
            item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            key_name TEXT NOT NULL,
            key_value_encrypted BLOB NOT NULL,
            key_value_nonce BLOB NOT NULL,
            endpoint TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS env_var_items (
            item_id TEXT REFERENCES items(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value_encrypted BLOB NOT NULL,
            value_nonce BLOB NOT NULL,
            sort_order INTEGER DEFAULT 0,
            PRIMARY KEY (item_id, key)
        );

        CREATE TABLE IF NOT EXISTS vault_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_items_group ON items(group_id);
        CREATE INDEX IF NOT EXISTS idx_items_type ON items(type);
        CREATE INDEX IF NOT EXISTS idx_items_favorite ON items(is_favorite);
        "#,
    ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;

    set_schema_version(conn, SCHEMA_VERSION)?;
    
    Ok(())
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('schema_version', ?)",
        [version.to_string()],
    ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_schema_version(conn: &Connection) -> Result<i32> {
    let version: Result<i32, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'schema_version'",
        [],
        |row| row.get(0),
    );
    
    match version {
        Ok(v) => Ok(v),
        Err(_) => Ok(0),
    }
}

pub fn insert_builtin_groups(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        INSERT OR IGNORE INTO groups (id, name, icon, sort_order, created_at, updated_at)
        VALUES 
            ('built-in-accounts', 'Accounts', '🔑', 0, strftime('%s', 'now'), strftime('%s', 'now')),
            ('built-in-api-keys', 'API Keys', '🔧', 1, strftime('%s', 'now'), strftime('%s', 'now')),
            ('built-in-env-vars', 'Environment Variables', '📦', 2, strftime('%s', 'now'), strftime('%s', 'now'));
        "#,
    ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;
    
    Ok(())
}
```

- [ ] **Step 3: Commit schema module**

```bash
git add -A
git commit -m "feat: add SQLite database schema with groups and items tables"
```

### Task 3.2: Create Database Operations

**Files:**
- Create: `src-tauri/src/db/operations.rs`

- [ ] **Step 1: Create database operations**

Create `src-tauri/src/db/operations.rs`:

```rust
use rusqlite::{Connection, params};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::error::{Result, VaultError};

pub struct DbConnection(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: Option<String>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub username: String,
    pub password: String,
    pub website: Option<String>,
    pub notes: Option<String>,
}

pub fn get_db_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("vault.db")
}

pub fn open_connection(path: &std::path::Path) -> Result<Connection> {
    Connection::open(path).map_err(|e| VaultError::DatabaseError(e.to_string()))
}

// Vault operations

pub fn is_vault_initialized(conn: &Connection) -> Result<bool> {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'password_hash'",
        [],
        |row| row.get(0),
    );
    Ok(result.is_ok())
}

pub fn set_password_hash(conn: &Connection, hash: &str, salt: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('password_hash', ?)",
        [hash],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    
    conn.execute(
        "INSERT OR REPLACE INTO vault_config (key, value) VALUES ('password_salt', ?)",
        [salt],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    
    Ok(())
}

pub fn get_password_hash(conn: &Connection) -> Result<Option<String>> {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'password_hash'",
        [],
        |row| row.get(0),
    );
    Ok(result.ok())
}

pub fn get_password_salt(conn: &Connection) -> Result<Option<String>> {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM vault_config WHERE key = 'password_salt'",
        [],
        |row| row.get(0),
    );
    Ok(result.ok())
}

// Group operations

pub fn get_all_groups(conn: &Connection) -> Result<Vec<Group>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, icon, parent_id, sort_order, created_at, updated_at 
             FROM groups ORDER BY sort_order"
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let groups = stmt
        .query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                parent_id: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(groups)
}

pub fn create_group(
    conn: &Connection,
    name: &str,
    icon: Option<&str>,
    parent_id: Option<&str>,
    sort_order: Option<i32>,
) -> Result<Group> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();
    let sort_order = sort_order.unwrap_or(999);

    conn.execute(
        "INSERT INTO groups (id, name, icon, parent_id, sort_order, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, name, icon, parent_id, sort_order, now, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(Group {
        id,
        name: name.to_string(),
        icon: icon.map(|s| s.to_string()),
        parent_id: parent_id.map(|s| s.to_string()),
        sort_order,
        created_at: now,
        updated_at: now,
    })
}

pub fn update_group(
    conn: &Connection,
    id: &str,
    name: &str,
    icon: Option<&str>,
    sort_order: Option<i32>,
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE groups SET name = ?1, icon = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, icon, sort_order, now, id],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

pub fn delete_group(conn: &Connection, id: &str) -> Result<()> {
    if id.starts_with("built-in-") {
        return Err(VaultError::GroupNotEmpty);
    }

    conn.execute("DELETE FROM groups WHERE id = ?1", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(())
}

// Item operations

pub fn get_all_items(conn: &Connection) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, a.username, i.icon, i.type, i.is_favorite 
             FROM items i 
             LEFT JOIN account_items a ON i.id = a.item_id 
             ORDER BY i.title"
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let items = stmt
        .query_map([], |row| {
            Ok(ItemSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                icon: row.get(3)?,
                item_type: row.get(4)?,
                is_favorite: row.get::<_, i32>(5)? != 0,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(items)
}

pub fn get_items_by_group(conn: &Connection, group_id: &str) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, a.username, i.icon, i.type, i.is_favorite 
             FROM items i 
             LEFT JOIN account_items a ON i.id = a.item_id 
             WHERE i.group_id = ?1
             ORDER BY i.title"
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let items = stmt
        .query_map([group_id], |row| {
            Ok(ItemSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                icon: row.get(3)?,
                item_type: row.get(4)?,
                is_favorite: row.get::<_, i32>(5)? != 0,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(items)
}

pub fn get_account_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<AccountItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at FROM items WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?)),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let account: (String, Vec<u8>, [u8; 12], Option<String>, Option<String>) = conn
        .query_row(
            "SELECT username, password_encrypted, password_nonce, website, notes 
             FROM account_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(2)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((row.get(0)?, row.get(1)?, nonce, row.get(3)?, row.get(4)?))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let password = String::from_utf8(
        crate::crypto::decrypt(key, &account.2, &account.1)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?
    ).map_err(|_| VaultError::CryptoError("Invalid UTF-8 in password".to_string()))?;

    Ok(AccountItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        username: account.0,
        password,
        website: account.3,
        notes: account.4,
    })
}

pub fn create_account_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    username: &str,
    password: &str,
    website: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, password.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'account', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO account_items (item_id, username, password_encrypted, password_nonce, website, notes) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), website, notes],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}

pub fn update_account_item(
    conn: &Connection,
    id: &str,
    title: &str,
    username: &str,
    password: Option<&str>,
    website: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(pwd) = password {
        let encrypted = crate::crypto::encrypt(key, pwd.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "UPDATE account_items SET username = ?1, password_encrypted = ?2, password_nonce = ?3, website = ?4, notes = ?5 WHERE item_id = ?6",
            params![username, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), website, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE account_items SET username = ?1, website = ?2, notes = ?3 WHERE item_id = ?4",
            params![username, website, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

pub fn delete_item(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM items WHERE id = ?1", [id])
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn toggle_favorite(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "UPDATE items SET is_favorite = NOT is_favorite WHERE id = ?1",
        [id],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    Ok(())
}

// Helper functions

fn chrono_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

- [ ] **Step 2: Commit database operations**

```bash
git add -A
git commit -m "feat: add database CRUD operations for groups and account items"
```

### Task 3.3: Update lib.rs with Database Module

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update lib.rs to include database module**

Update `src-tauri/src/lib.rs`:

```rust
mod error;
mod crypto;
mod db;
mod state;

pub use error::{VaultError, Result};
pub use crypto::{derive_key, verify_password, encrypt, decrypt, SecureKey, EncryptedData};
pub use state::VaultState;
pub use db::{init_schema, insert_builtin_groups, Group, ItemSummary, AccountItemDetail};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify Rust code compiles**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds without errors

- [ ] **Step 3: Commit lib.rs update**

```bash
git add -A
git commit -m "feat: integrate database module into lib.rs"
```

---

## Chunk 4: Rust Backend - Tauri Commands

### Task 4.1: Create Commands Module Structure

**Files:**
- Create: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Create commands module**

Create `src-tauri/src/commands/mod.rs`:

```rust
mod vault;
mod groups;
mod items;
mod clipboard;

pub use vault::*;
pub use groups::*;
pub use items::*;
pub use clipboard::*;
```

- [ ] **Step 2: Commit commands module structure**

```bash
git add -A
git commit -m "feat: create commands module structure"
```

### Task 4.2: Create Vault Commands

**Files:**
- Create: `src-tauri/src/commands/vault.rs`

- [ ] **Step 1: Create vault commands**

Create `src-tauri/src/commands/vault.rs`:

```rust
use tauri::State;
use rusqlite::Connection;

use crate::state::VaultState;
use crate::db::{DbConnection, get_db_path, open_connection, init_schema, insert_builtin_groups, is_vault_initialized, set_password_hash, get_password_hash, get_password_salt};
use crate::crypto::{derive_key, verify_password, SecureKey};
use crate::error::{Result, VaultError};

#[tauri::command]
pub fn is_vault_initialized(app_handle: tauri::AppHandle) -> Result<bool> {
    let db_path = get_db_path(&app_handle);
    
    if !db_path.exists() {
        return Ok(false);
    }
    
    let conn = open_connection(&db_path)?;
    crate::db::is_vault_initialized(&conn)
}

#[tauri::command]
pub fn initialize_vault(
    app_handle: tauri::AppHandle,
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<()> {
    if master_password.len() < 8 {
        return Err(VaultError::InvalidPassword);
    }

    let db_path = get_db_path(&app_handle);
    
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    let conn = open_connection(&db_path)?;
    init_schema(&conn)?;
    insert_builtin_groups(&conn)?;

    if crate::db::is_vault_initialized(&conn)? {
        return Err(VaultError::VaultAlreadyInitialized);
    }

    let derived = derive_key(&master_password, None)?;
    set_password_hash(&conn, &derived.hash, &derived.salt)?;

    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);

    let mut db_conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    *db_conn = conn;

    Ok(())
}

#[tauri::command]
pub fn unlock_vault(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<bool> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    let stored_hash = get_password_hash(&conn)?.ok_or(VaultError::VaultNotInitialized)?;
    let stored_salt = get_password_salt(&conn)?.ok_or(VaultError::VaultNotInitialized)?;
    
    if !verify_password(&master_password, &stored_hash)? {
        return Err(VaultError::InvalidPassword);
    }

    let derived = derive_key(&master_password, Some(&stored_salt))?;
    let secure_key = SecureKey::new(derived.key);
    vault_state.unlock(secure_key);

    Ok(true)
}

#[tauri::command]
pub fn lock_vault(vault_state: State<VaultState>) -> Result<()> {
    vault_state.lock();
    Ok(())
}

#[tauri::command]
pub fn is_vault_unlocked(vault_state: State<VaultState>) -> bool {
    vault_state.is_unlocked()
}
```

- [ ] **Step 2: Commit vault commands**

```bash
git add -A
git commit -m "feat: add Tauri vault commands (init, unlock, lock)"
```

### Task 4.3: Create Group Commands

**Files:**
- Create: `src-tauri/src/commands/groups.rs`

- [ ] **Step 1: Create group commands**

Create `src-tauri/src/commands/groups.rs`:

```rust
use tauri::State;
use crate::db::{DbConnection, get_all_groups, create_group, update_group, delete_group, Group};
use crate::error::{Result, VaultError};

#[tauri::command]
pub fn get_groups(db: State<DbConnection>) -> Result<Vec<Group>> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_groups(&conn)
}

#[tauri::command]
pub fn create_new_group(
    db: State<DbConnection>,
    name: String,
    icon: Option<String>,
    parent_id: Option<String>,
    sort_order: Option<i32>,
) -> Result<Group> {
    if name.is_empty() || name.len() > 50 {
        return Err(VaultError::DatabaseError("Group name must be 1-50 characters".to_string()));
    }

    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    create_group(&conn, &name, icon.as_deref(), parent_id.as_deref(), sort_order)
}

#[tauri::command]
pub fn update_existing_group(
    db: State<DbConnection>,
    id: String,
    name: String,
    icon: Option<String>,
    sort_order: Option<i32>,
) -> Result<()> {
    if name.is_empty() || name.len() > 50 {
        return Err(VaultError::DatabaseError("Group name must be 1-50 characters".to_string()));
    }

    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    update_group(&conn, &id, &name, icon.as_deref(), sort_order)
}

#[tauri::command]
pub fn delete_existing_group(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_group(&conn, &id)
}
```

- [ ] **Step 2: Commit group commands**

```bash
git add -A
git commit -m "feat: add Tauri group commands (CRUD operations)"
```

### Task 4.4: Create Item Commands

**Files:**
- Create: `src-tauri/src/commands/items.rs`

- [ ] **Step 1: Create item commands**

Create `src-tauri/src/commands/items.rs`:

```rust
use tauri::State;
use crate::state::VaultState;
use crate::db::{DbConnection, get_all_items, get_items_by_group, get_account_item_detail, create_account_item, update_account_item, delete_item, toggle_favorite, ItemSummary, AccountItemDetail};
use crate::error::{Result, VaultError};

#[tauri::command]
pub fn get_all_items_cmd(db: State<DbConnection>) -> Result<Vec<ItemSummary>> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_all_items(&conn)
}

#[tauri::command]
pub fn get_items_by_group_cmd(db: State<DbConnection>, group_id: String) -> Result<Vec<ItemSummary>> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    get_items_by_group(&conn, &group_id)
}

#[tauri::command]
pub fn get_item_detail(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<AccountItemDetail> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    get_account_item_detail(&conn, &id, key.as_bytes())
}

#[tauri::command]
pub fn create_new_account_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    username: String,
    password: String,
    website: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError("Username must be 1-100 characters".to_string()));
    }
    if password.is_empty() || password.len() > 1000 {
        return Err(VaultError::DatabaseError("Password must be 1-1000 characters".to_string()));
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    create_account_item(
        &conn,
        &group_id,
        &title,
        &username,
        &password,
        website.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn update_existing_account_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    username: String,
    password: Option<String>,
    website: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if username.is_empty() || username.len() > 100 {
        return Err(VaultError::DatabaseError("Username must be 1-100 characters".to_string()));
    }
    if let Some(ref pwd) = password {
        if pwd.is_empty() || pwd.len() > 1000 {
            return Err(VaultError::DatabaseError("Password must be 1-1000 characters".to_string()));
        }
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    update_account_item(
        &conn,
        &id,
        &title,
        &username,
        password.as_deref(),
        website.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}

#[tauri::command]
pub fn delete_existing_item(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    delete_item(&conn, &id)
}

#[tauri::command]
pub fn toggle_item_favorite(db: State<DbConnection>, id: String) -> Result<()> {
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    toggle_favorite(&conn, &id)
}
```

- [ ] **Step 2: Commit item commands**

```bash
git add -A
git commit -m "feat: add Tauri item commands (CRUD for account items)"
```

### Task 4.5: Create Clipboard Commands

**Files:**
- Create: `src-tauri/src/commands/clipboard.rs`

- [ ] **Step 1: Create clipboard commands**

Create `src-tauri/src/commands/clipboard.rs`:

```rust
use tauri::Manager;
use crate::error::{Result, VaultError};

#[tauri::command]
pub async fn copy_to_clipboard(app_handle: tauri::AppHandle, text: String) -> Result<()> {
    app_handle
        .clipboard()
        .write_text(&text)
        .map_err(|e| VaultError::DatabaseError(format!("Clipboard error: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn clear_clipboard(app_handle: tauri::AppHandle) -> Result<()> {
    app_handle
        .clipboard()
        .write_text("")
        .map_err(|e| VaultError::DatabaseError(format!("Clipboard error: {}", e)))?;
    Ok(())
}
```

- [ ] **Step 2: Commit clipboard commands**

```bash
git add -A
git commit -m "feat: add Tauri clipboard commands"
```

### Task 4.6: Update lib.rs with Commands and State

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update lib.rs with full integration**

Update `src-tauri/src/lib.rs`:

```rust
mod error;
mod crypto;
mod db;
mod state;
mod commands;

pub use error::{VaultError, Result};
pub use crypto::{derive_key, verify_password, encrypt, decrypt, SecureKey, EncryptedData};
pub use state::VaultState;
pub use db::{
    init_schema, insert_builtin_groups, 
    Group, ItemSummary, AccountItemDetail,
    DbConnection, open_connection, get_db_path,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use commands::*;
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let db_path = get_db_path(&app.handle());
            let conn = if db_path.exists() {
                open_connection(&db_path).expect("Failed to open database")
            } else {
                rusqlite::Connection::open_in_memory().expect("Failed to create in-memory DB")
            };
            
            app.manage(DbConnection(std::sync::Mutex::new(conn)));
            app.manage(VaultState::new());
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_vault_initialized,
            initialize_vault,
            unlock_vault,
            lock_vault,
            is_vault_unlocked,
            get_groups,
            create_new_group,
            update_existing_group,
            delete_existing_group,
            get_all_items_cmd,
            get_items_by_group_cmd,
            get_item_detail,
            create_new_account_item,
            update_existing_account_item,
            delete_existing_item,
            toggle_item_favorite,
            copy_to_clipboard,
            clear_clipboard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify Rust code compiles**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compilation succeeds without errors

- [ ] **Step 3: Commit final backend integration**

```bash
git add -A
git commit -m "feat: integrate all Tauri commands and state management"
```
```