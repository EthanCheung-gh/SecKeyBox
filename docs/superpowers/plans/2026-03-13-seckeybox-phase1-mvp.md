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
  group_id: string;
  created_at: number;
  updated_at: number;
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
    pub group_id: String,
    pub created_at: i64,
    pub updated_at: i64,
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
            "SELECT i.id, i.title, a.username, i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at 
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
                group_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
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
            "SELECT i.id, i.title, a.username, i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at 
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
                group_id: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
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

---

## Chunk 5: React Frontend - Core Infrastructure

### Task 5.1: Create Tauri IPC Wrapper

**Files:**
- Create: `src/lib/tauri.ts`

- [ ] **Step 1: Create Tauri invoke wrapper**

Create `src/lib/tauri.ts`:

```typescript
import { invoke } from '@tauri-apps/api/core';

export async function isVaultInitialized(): Promise<boolean> {
  return invoke('is_vault_initialized');
}

export async function initializeVault(masterPassword: string): Promise<void> {
  return invoke('initialize_vault', { masterPassword });
}

export async function unlockVault(masterPassword: string): Promise<boolean> {
  return invoke('unlock_vault', { masterPassword });
}

export async function lockVault(): Promise<void> {
  return invoke('lock_vault');
}

export async function isVaultUnlocked(): Promise<boolean> {
  return invoke('is_vault_unlocked');
}

export async function getGroups() {
  return invoke('get_groups');
}

export async function createNewGroup(name: string, icon?: string, parentId?: string, sortOrder?: number) {
  return invoke('create_new_group', { name, icon, parentId, sortOrder });
}

export async function updateExistingGroup(id: string, name: string, icon?: string, sortOrder?: number) {
  return invoke('update_existing_group', { id, name, icon, sortOrder });
}

export async function deleteExistingGroup(id: string) {
  return invoke('delete_existing_group', { id });
}

export async function getAllItems() {
  return invoke('get_all_items_cmd');
}

export async function getItemsByGroup(groupId: string) {
  return invoke('get_items_by_group_cmd', { groupId });
}

export async function getItemDetail(id: string) {
  return invoke('get_item_detail', { id });
}

export async function createNewAccountItem(
  groupId: string,
  title: string,
  username: string,
  password: string,
  website?: string,
  notes?: string
) {
  return invoke('create_new_account_item', { groupId, title, username, password, website, notes });
}

export async function updateExistingAccountItem(
  id: string,
  title: string,
  username: string,
  password?: string,
  website?: string,
  notes?: string
) {
  return invoke('update_existing_account_item', { id, title, username, password, website, notes });
}

export async function deleteExistingItem(id: string) {
  return invoke('delete_existing_item', { id });
}

export async function toggleItemFavorite(id: string) {
  return invoke('toggle_item_favorite', { id });
}

export async function copyToClipboard(text: string) {
  return invoke('copy_to_clipboard', { text });
}

export async function clearClipboard() {
  return invoke('clear_clipboard');
}
```

- [ ] **Step 2: Commit Tauri wrapper**

```bash
git add -A
git commit -m "feat: add Tauri IPC wrapper for all backend commands"
```

### Task 5.2: Create Utility Functions

**Files:**
- Create: `src/lib/utils.ts`

- [ ] **Step 1: Create utility functions**

Create `src/lib/utils.ts`:

```typescript
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatTimestamp(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString();
}

export function truncate(str: string, length: number): string {
  if (str.length <= length) return str;
  return str.slice(0, length) + '...';
}
```

- [ ] **Step 2: Commit utilities**

```bash
git add -A
git commit -m "feat: add utility functions (cn, formatTimestamp, truncate)"
```

### Task 5.3: Create Zustand Stores

**Files:**
- Create: `src/stores/vault.ts`
- Create: `src/stores/ui.ts`

- [ ] **Step 1: Create vault store**

Create `src/stores/vault.ts`:

```typescript
import { create } from 'zustand';
import type { Group, ItemSummary, ItemDetail } from '@/types';
import * as api from '@/lib/tauri';

interface VaultState {
  isInitialized: boolean | null;
  isUnlocked: boolean;
  groups: Group[];
  items: ItemSummary[];
  selectedItem: ItemDetail | null;
  isLoading: boolean;
  error: string | null;
  
  checkInitialized: () => Promise<void>;
  initialize: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  loadGroups: () => Promise<void>;
  loadItems: (groupId?: string) => Promise<void>;
  selectItem: (id: string) => Promise<void>;
  clearSelection: () => void;
  createGroup: (name: string, icon?: string) => Promise<void>;
  updateGroup: (id: string, name: string, icon?: string) => Promise<void>;
  deleteGroup: (id: string) => Promise<void>;
  createItem: (data: {
    groupId: string;
    title: string;
    username: string;
    password: string;
    website?: string;
    notes?: string;
  }) => Promise<void>;
  updateItem: (id: string, data: {
    title: string;
    username: string;
    password?: string;
    website?: string;
    notes?: string;
  }) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  clearError: () => void;
}

export const useVaultStore = create<VaultState>((set, get) => ({
  isInitialized: null,
  isUnlocked: false,
  groups: [],
  items: [],
  selectedItem: null,
  isLoading: false,
  error: null,

  checkInitialized: async () => {
    try {
      const initialized = await api.isVaultInitialized();
      set({ isInitialized: initialized });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  initialize: async (password) => {
    set({ isLoading: true, error: null });
    try {
      await api.initializeVault(password);
      set({ isInitialized: true, isUnlocked: true, isLoading: false });
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  unlock: async (password) => {
    set({ isLoading: true, error: null });
    try {
      await api.unlockVault(password);
      set({ isUnlocked: true, isLoading: false });
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  lock: async () => {
    await api.lockVault();
    set({ isUnlocked: false, items: [], selectedItem: null });
  },

  loadGroups: async () => {
    try {
      const groups = await api.getGroups();
      set({ groups });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  loadItems: async (groupId) => {
    try {
      const items = groupId 
        ? await api.getItemsByGroup(groupId)
        : await api.getAllItems();
      set({ items });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  selectItem: async (id) => {
    try {
      const item = await api.getItemDetail(id);
      set({ selectedItem: item });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearSelection: () => set({ selectedItem: null }),

  createGroup: async (name, icon) => {
    try {
      await api.createNewGroup(name, icon);
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateGroup: async (id, name, icon) => {
    try {
      await api.updateExistingGroup(id, name, icon);
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteGroup: async (id) => {
    try {
      await api.deleteExistingGroup(id);
      await get().loadGroups();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createItem: async (data) => {
    try {
      await api.createNewAccountItem(
        data.groupId,
        data.title,
        data.username,
        data.password,
        data.website,
        data.notes
      );
      await get().loadItems();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateItem: async (id, data) => {
    try {
      await api.updateExistingAccountItem(
        id,
        data.title,
        data.username,
        data.password,
        data.website,
        data.notes
      );
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteItem: async (id) => {
    try {
      await api.deleteExistingItem(id);
      if (get().selectedItem?.id === id) {
        set({ selectedItem: null });
      }
      await get().loadItems();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
}));
```

- [ ] **Step 2: Create UI store**

Create `src/stores/ui.ts`:

```typescript
import { create } from 'zustand';

interface UIState {
  selectedGroupId: string | null;
  searchQuery: string;
  sortBy: 'name_asc' | 'name_desc' | 'created' | 'updated';
  isAddItemModalOpen: boolean;
  isAddGroupModalOpen: boolean;
  editingGroupId: string | null;
  editingItemId: string | null;
  deleteConfirmTarget: { type: 'group' | 'item'; id: string; name: string } | null;
  
  setSelectedGroup: (id: string | null) => void;
  setSearchQuery: (query: string) => void;
  setSortBy: (sort: UIState['sortBy']) => void;
  openAddItemModal: () => void;
  closeAddItemModal: () => void;
  openAddGroupModal: () => void;
  closeAddGroupModal: () => void;
  openEditGroupModal: (id: string) => void;
  closeEditGroupModal: () => void;
  openEditItemModal: (id: string) => void;
  closeEditItemModal: () => void;
  openDeleteConfirm: (type: 'group' | 'item', id: string, name: string) => void;
  closeDeleteConfirm: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  selectedGroupId: null,
  searchQuery: '',
  sortBy: 'name_asc',
  isAddItemModalOpen: false,
  isAddGroupModalOpen: false,
  editingGroupId: null,
  editingItemId: null,
  deleteConfirmTarget: null,

  setSelectedGroup: (id) => set({ selectedGroupId: id }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setSortBy: (sort) => set({ sortBy: sort }),
  openAddItemModal: () => set({ isAddItemModalOpen: true }),
  closeAddItemModal: () => set({ isAddItemModalOpen: false }),
  openAddGroupModal: () => set({ isAddGroupModalOpen: true }),
  closeAddGroupModal: () => set({ isAddGroupModalOpen: false }),
  openEditGroupModal: (id) => set({ editingGroupId: id }),
  closeEditGroupModal: () => set({ editingGroupId: null }),
  openEditItemModal: (id) => set({ editingItemId: id }),
  closeEditItemModal: () => set({ editingItemId: null }),
  openDeleteConfirm: (type, id, name) => set({ deleteConfirmTarget: { type, id, name } }),
  closeDeleteConfirm: () => set({ deleteConfirmTarget: null }),
}));
```

- [ ] **Step 3: Commit stores**

```bash
git add -A
git commit -m "feat: add Zustand stores for vault and UI state"
```

### Task 5.4: Create Activity Tracker Hook

**Files:**
- Create: `src/hooks/useActivityTracker.ts`

- [ ] **Step 1: Create activity tracker hook**

Create `src/hooks/useActivityTracker.ts`:

```typescript
import { useEffect, useRef } from 'react';
import { useVaultStore } from '@/stores/vault';

const INACTIVITY_TIMEOUT = 5 * 60 * 1000; // 5 minutes

export function useActivityTracker() {
  const timerRef = useRef<NodeJS.Timeout | null>(null);
  const isUnlocked = useVaultStore((s) => s.isUnlocked);
  const lock = useVaultStore((s) => s.lock);

  useEffect(() => {
    if (!isUnlocked) return;

    const resetTimer = () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      timerRef.current = setTimeout(() => {
        lock();
      }, INACTIVITY_TIMEOUT);
    };

    const events = ['mousedown', 'keydown', 'touchstart'];
    events.forEach((event) => {
      window.addEventListener(event, resetTimer);
    });

    resetTimer();

    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
      events.forEach((event) => {
        window.removeEventListener(event, resetTimer);
      });
    };
  }, [isUnlocked, lock]);
}
```

- [ ] **Step 2: Commit activity tracker**

```bash
git add -A
git commit -m "feat: add activity tracker hook for auto-lock"
```

---

## Chunk 6: React Frontend - UI Components

### Task 6.1: Create Base UI Components (Shadcn-style)

**Files:**
- Create: `src/components/ui/button.tsx`
- Create: `src/components/ui/input.tsx`
- Create: `src/components/ui/card.tsx`
- Create: `src/components/ui/dialog.tsx`

- [ ] **Step 1: Create button component**

Create `src/components/ui/button.tsx`:

```typescript
import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'default' | 'destructive' | 'outline' | 'ghost';
  size?: 'default' | 'sm' | 'lg';
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'default', size = 'default', ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          'inline-flex items-center justify-center rounded-md font-medium transition-colors',
          'focus-visible:outline-none focus-visible:ring-2',
          'disabled:pointer-events-none disabled:opacity-50',
          {
            'bg-primary-600 text-white hover:bg-primary-700': variant === 'default',
            'bg-red-600 text-white hover:bg-red-700': variant === 'destructive',
            'border border-gray-300 bg-white hover:bg-gray-50': variant === 'outline',
            'hover:bg-gray-100': variant === 'ghost',
            'h-9 px-4 text-sm': size === 'default',
            'h-8 px-3 text-xs': size === 'sm',
            'h-10 px-6 text-base': size === 'lg',
          },
          className
        )}
        {...props}
      />
    );
  }
);
```

- [ ] **Step 2: Create input component**

Create `src/components/ui/input.tsx`:

```typescript
import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          'flex h-9 w-full rounded-md border border-gray-300 bg-white px-3 py-1 text-sm',
          'shadow-sm transition-colors',
          'file:border-0 file:bg-transparent file:text-sm file:font-medium',
          'placeholder:text-gray-400',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500',
          'disabled:cursor-not-allowed disabled:opacity-50',
          className
        )}
        ref={ref}
        {...props}
      />
    );
  }
);
```

- [ ] **Step 3: Create card components**

Create `src/components/ui/card.tsx`:

```typescript
import { cn } from '@/lib/utils';

export function Card({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn('rounded-lg border bg-white shadow-sm', className)}
      {...props}
    />
  );
}

export function CardHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('flex flex-col space-y-1.5 p-4', className)} {...props} />;
}

export function CardTitle({ className, ...props }: React.HTMLAttributes<HTMLHeadingElement>) {
  return <h3 className={cn('text-lg font-semibold', className)} {...props} />;
}

export function CardContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('p-4 pt-0', className)} {...props} />;
}
```

- [ ] **Step 4: Create dialog component**

Create `src/components/ui/dialog.tsx`:

```typescript
import { cn } from '@/lib/utils';

interface DialogProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
}

export function Dialog({ open, onClose, title, children }: DialogProps) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="fixed inset-0 bg-black/50" onClick={onClose} />
      <div className="relative z-50 w-full max-w-md rounded-lg bg-white p-6 shadow-lg">
        {title && <h2 className="mb-4 text-lg font-semibold">{title}</h2>}
        {children}
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Commit base UI components**

```bash
git add -A
git commit -m "feat: add base UI components (button, input, card, dialog)"
```

### Task 6.2: Create Unlock Screen

**Files:**
- Create: `src/components/unlock/UnlockScreen.tsx`
- Create: `src/components/unlock/SetupScreen.tsx`

- [ ] **Step 1: Create setup screen for first-run**

Create `src/components/unlock/SetupScreen.tsx`:

```typescript
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useVaultStore } from '@/stores/vault';

export function SetupScreen() {
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [error, setError] = useState('');
  const initialize = useVaultStore((s) => s.initialize);
  const isLoading = useVaultStore((s) => s.isLoading);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }
    if (password !== confirm) {
      setError('Passwords do not match');
      return;
    }

    await initialize(password);
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-100">
      <div className="w-full max-w-sm rounded-lg bg-white p-8 shadow-lg">
        <h1 className="mb-2 text-center text-2xl font-bold">SecKeyBox</h1>
        <p className="mb-6 text-center text-gray-500">Create your master password</p>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="mb-1 block text-sm font-medium">Master Password</label>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter password"
              autoFocus
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium">Confirm Password</label>
            <Input
              type="password"
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              placeholder="Confirm password"
            />
          </div>
          {error && <p className="text-sm text-red-500">{error}</p>}
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? 'Creating...' : 'Create Vault'}
          </Button>
        </form>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Create unlock screen**

Create `src/components/unlock/UnlockScreen.tsx`:

```typescript
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useVaultStore } from '@/stores/vault';

export function UnlockScreen() {
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const unlock = useVaultStore((s) => s.unlock);
  const isLoading = useVaultStore((s) => s.isLoading);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    await unlock(password);
    const storeError = useVaultStore.getState().error;
    if (storeError) {
      setError('Invalid password');
    }
  };

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-100">
      <div className="w-full max-w-sm rounded-lg bg-white p-8 shadow-lg">
        <h1 className="mb-2 text-center text-2xl font-bold">SecKeyBox</h1>
        <p className="mb-6 text-center text-gray-500">Enter your master password</p>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Master password"
              autoFocus
            />
          </div>
          {error && <p className="text-sm text-red-500">{error}</p>}
          <Button type="submit" className="w-full" disabled={isLoading}>
            {isLoading ? 'Unlocking...' : 'Unlock'}
          </Button>
        </form>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Commit unlock screens**

```bash
git add -A
git commit -m "feat: add setup and unlock screens"
```

### Task 6.3: Create Main Layout Components

**Files:**
- Create: `src/components/layout/Sidebar.tsx`
- Create: `src/components/layout/ItemList.tsx`
- Create: `src/components/layout/DetailPanel.tsx`

- [ ] **Step 1: Create sidebar component**

Create `src/components/layout/Sidebar.tsx`:

```typescript
import { Plus, Lock, Settings, MoreVertical } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function Sidebar() {
  const groups = useVaultStore((s) => s.groups);
  const items = useVaultStore((s) => s.items);
  const loadItems = useVaultStore((s) => s.loadItems);
  const lock = useVaultStore((s) => s.lock);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const setSearchQuery = useUIStore((s) => s.setSearchQuery);
  const setSelectedGroup = useUIStore((s) => s.setSelectedGroup);
  const openAddGroupModal = useUIStore((s) => s.openAddGroupModal);
  const openEditGroupModal = useUIStore((s) => s.openEditGroupModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);

  const handleSelectGroup = (id: string | null) => {
    setSelectedGroup(id);
    loadItems(id || undefined);
  };

  const filteredItems = items.filter(
    (item) =>
      searchQuery === '' ||
      item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.subtitle.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="flex h-full w-60 flex-col border-r bg-gray-50">
      <div className="p-4">
        <input
          type="text"
          placeholder="Search..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
        />
      </div>
      
      <div className="flex-1 overflow-y-auto px-2">
        <button
          onClick={() => handleSelectGroup(null)}
          className={`w-full rounded-md px-3 py-2 text-left text-sm ${
            selectedGroupId === null ? 'bg-primary-100 text-primary-700' : 'hover:bg-gray-100'
          }`}
        >
          📁 All Items {searchQuery && `(${filteredItems.length})`}
        </button>
        
        <div className="my-2 border-t" />
        
        {groups.map((group) => {
          const itemCount = group.id.startsWith('built-in-') 
            ? 0 
            : items.filter(i => i.group_id === group.id).length;
          
          return (
            <div key={group.id} className="relative group flex items-center">
              <button
                onClick={() => handleSelectGroup(group.id)}
                className={`flex-1 rounded-md px-3 py-2 text-left text-sm ${
                  selectedGroupId === group.id ? 'bg-primary-100 text-primary-700' : 'hover:bg-gray-100'
                }`}
              >
                {group.icon || '📂'} {group.name}
              </button>
              {!group.id.startsWith('built-in-') && (
                <div className="relative">
                  <button
                    onClick={() => setMenuOpenId(menuOpenId === group.id ? null : group.id)}
                    className="rounded p-1 opacity-0 group-hover:opacity-100 hover:bg-gray-200"
                  >
                    <MoreVertical className="h-4 w-4" />
                  </button>
                  {menuOpenId === group.id && (
                    <div className="absolute right-0 top-6 z-10 w-24 rounded-md border bg-white shadow-lg">
                      <button
                        onClick={() => {
                          openEditGroupModal(group.id);
                          setMenuOpenId(null);
                        }}
                        className="block w-full px-3 py-2 text-left text-sm hover:bg-gray-100"
                      >
                        Rename
                      </button>
                      <button
                        onClick={() => {
                          openDeleteConfirm('group', group.id, group.name);
                          setMenuOpenId(null);
                        }}
                        className="block w-full px-3 py-2 text-left text-sm text-red-500 hover:bg-gray-100"
                      >
                        Delete
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}
        
        <Button
          variant="ghost"
          size="sm"
          className="mt-2 w-full justify-start text-gray-500"
          onClick={openAddGroupModal}
        >
          <Plus className="mr-2 h-4 w-4" /> Add Group
        </Button>
      </div>
      
      <div className="border-t p-2">
        <Button
          variant="ghost"
          size="sm"
          className="w-full justify-start text-gray-500"
          onClick={lock}
        >
          <Lock className="mr-2 h-4 w-4" /> Lock
        </Button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Create item list component**

Create `src/components/layout/ItemList.tsx`:

```typescript
import { Plus } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function ItemList() {
  const items = useVaultStore((s) => s.items);
  const selectItem = useVaultStore((s) => s.selectItem);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const openAddItemModal = useUIStore((s) => s.openAddItemModal);
  const searchQuery = useUIStore((s) => s.searchQuery);
  const sortBy = useUIStore((s) => s.sortBy);
  const setSortBy = useUIStore((s) => s.setSortBy);

  const filteredAndSortedItems = items
    .filter((item) =>
      searchQuery === '' ||
      item.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.subtitle.toLowerCase().includes(searchQuery.toLowerCase())
    )
    .sort((a, b) => {
      switch (sortBy) {
        case 'name_asc':
          return a.title.localeCompare(b.title);
        case 'name_desc':
          return b.title.localeCompare(a.title);
        case 'created':
          return b.created_at - a.created_at;
        case 'updated':
          return b.updated_at - a.updated_at;
        default:
          return 0;
      }
    });

  return (
    <div className="flex h-full w-80 flex-col border-r bg-white">
      <div className="flex items-center justify-between border-b p-4">
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as any)}
          className="rounded-md border border-gray-300 px-2 py-1 text-sm"
        >
          <option value="name_asc">Name (A-Z)</option>
          <option value="name_desc">Name (Z-A)</option>
          <option value="created">Created (newest)</option>
          <option value="updated">Modified (newest)</option>
        </select>
        <Button size="sm" onClick={openAddItemModal}>
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      
      <div className="flex-1 overflow-y-auto">
        {filteredAndSortedItems.length === 0 ? (
          <div className="p-4 text-center text-gray-500">No items</div>
        ) : (
          filteredAndSortedItems.map((item) => (
            <button
              key={item.id}
              onClick={() => selectItem(item.id)}
              className={`w-full border-b p-3 text-left hover:bg-gray-50 ${
                selectedItem?.id === item.id ? 'bg-primary-50' : ''
              }`}
            >
              <div className="font-medium">{item.title}</div>
              <div className="text-sm text-gray-500">{item.subtitle}</div>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Create detail panel component**

Create `src/components/layout/DetailPanel.tsx`:

```typescript
import { Copy, Eye, EyeOff, Edit, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { copyToClipboard } from '@/lib/tauri';

export function DetailPanel() {
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const deleteItem = useVaultStore((s) => s.deleteItem);
  const openEditItemModal = useUIStore((s) => s.openEditItemModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [showPassword, setShowPassword] = useState(false);

  if (!selectedItem) {
    return (
      <div className="flex h-full flex-1 items-center justify-center bg-gray-50">
        <p className="text-gray-400">Select an item to view details</p>
      </div>
    );
  }

  const handleCopy = async (text: string) => {
    await copyToClipboard(text);
    // Auto-clear clipboard after 30 seconds
    setTimeout(async () => {
      await import('@/lib/tauri').then(api => api.clearClipboard());
    }, 30000);
  };

  return (
    <div className="flex h-full flex-1 flex-col bg-white p-6">
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-semibold">{selectedItem.title}</h1>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={() => openEditItemModal(selectedItem.id)}>
            <Edit className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => openDeleteConfirm('item', selectedItem.id, selectedItem.title)}
          >
            <Trash2 className="h-4 w-4 text-red-500" />
          </Button>
        </div>
      </div>

      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Username</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">{selectedItem.username}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(selectedItem.username)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Password</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">
              {showPassword ? selectedItem.password : '••••••••'}
            </span>
            <Button variant="ghost" size="sm" onClick={() => setShowPassword(!showPassword)}>
              {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(selectedItem.password)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {selectedItem.website && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Website</label>
            <a
              href={selectedItem.website}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary-600 hover:underline"
            >
              {selectedItem.website}
            </a>
          </div>
        )}

        {selectedItem.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{selectedItem.notes}</p>
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Commit layout components**

```bash
git add -A
git commit -m "feat: add sidebar, item list, and detail panel components"
```

### Task 6.4: Create Modal Components

**Files:**
- Create: `src/components/modals/AddEditItemModal.tsx`
- Create: `src/components/modals/AddGroupModal.tsx`
- Create: `src/components/modals/DeleteConfirmModal.tsx`

- [ ] **Step 1: Create add/edit item modal**

Create `src/components/modals/AddEditItemModal.tsx`:

```typescript
import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function AddEditItemModal() {
  const isOpen = useUIStore((s) => s.isAddItemModalOpen || s.editingItemId !== null);
  const editingItemId = useUIStore((s) => s.editingItemId);
  const closeAddItemModal = useUIStore((s) => s.closeAddItemModal);
  const closeEditItemModal = useUIStore((s) => s.closeEditItemModal);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const groups = useVaultStore((s) => s.groups);
  const createItem = useVaultStore((s) => s.createItem);
  const updateItem = useVaultStore((s) => s.updateItem);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);

  const [form, setForm] = useState({
    groupId: '',
    title: '',
    username: '',
    password: '',
    website: '',
    notes: '',
  });

  useEffect(() => {
    if (editingItemId && selectedItem) {
      setForm({
        groupId: selectedItem.group_id,
        title: selectedItem.title,
        username: selectedItem.username,
        password: '',
        website: selectedItem.website || '',
        notes: selectedItem.notes || '',
      });
    } else {
      setForm({
        groupId: selectedGroupId || groups[0]?.id || '',
        title: '',
        username: '',
        password: '',
        website: '',
        notes: '',
      });
    }
  }, [editingItemId, selectedItem, selectedGroupId, groups]);

  const handleClose = () => {
    closeAddItemModal();
    closeEditItemModal();
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editingItemId) {
      await updateItem(editingItemId, {
        title: form.title,
        username: form.username,
        password: form.password || undefined,
        website: form.website || undefined,
        notes: form.notes || undefined,
      });
    } else {
      await createItem({
        groupId: form.groupId,
        title: form.title,
        username: form.username,
        password: form.password,
        website: form.website || undefined,
        notes: form.notes || undefined,
      });
    }
    handleClose();
  };

  return (
    <Dialog open={isOpen} onClose={handleClose} title={editingItemId ? 'Edit Item' : 'Add Item'}>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium">Group</label>
          <select
            value={form.groupId}
            onChange={(e) => setForm({ ...form, groupId: e.target.value })}
            className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            disabled={!!editingItemId}
          >
            {groups.map((g) => (
              <option key={g.id} value={g.id}>
                {g.icon} {g.name}
              </option>
            ))}
          </select>
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Title *</label>
          <Input
            value={form.title}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
            required
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Username *</label>
          <Input
            value={form.username}
            onChange={(e) => setForm({ ...form, username: e.target.value })}
            required
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">
            Password {editingItemId ? '(leave blank to keep)' : '*'}
          </label>
          <Input
            type="password"
            value={form.password}
            onChange={(e) => setForm({ ...form, password: e.target.value })}
            required={!editingItemId}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Website</label>
          <Input
            value={form.website}
            onChange={(e) => setForm({ ...form, website: e.target.value })}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Notes</label>
          <textarea
            value={form.notes}
            onChange={(e) => setForm({ ...form, notes: e.target.value })}
            className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            rows={3}
          />
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit">{editingItemId ? 'Save' : 'Add'}</Button>
        </div>
      </form>
    </Dialog>
  );
}
```

- [ ] **Step 2: Create add group modal**

Create `src/components/modals/AddGroupModal.tsx`:

```typescript
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function AddGroupModal() {
  const isOpen = useUIStore((s) => s.isAddGroupModalOpen);
  const close = useUIStore((s) => s.closeAddGroupModal);
  const createGroup = useVaultStore((s) => s.createGroup);
  const [name, setName] = useState('');
  const [icon, setIcon] = useState('📂');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await createGroup(name, icon);
    setName('');
    setIcon('📂');
    close();
  };

  const icons = ['📂', '🔒', '💼', '🏠', '💳', '📧', '🎮', '🛒'];

  return (
    <Dialog open={isOpen} onClose={close} title="Add Group">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium">Name *</label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            maxLength={50}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Icon</label>
          <div className="flex gap-2">
            {icons.map((i) => (
              <button
                key={i}
                type="button"
                onClick={() => setIcon(i)}
                className={`rounded p-2 text-xl ${icon === i ? 'bg-primary-100' : 'hover:bg-gray-100'}`}
              >
                {i}
              </button>
            ))}
          </div>
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={close}>
            Cancel
          </Button>
          <Button type="submit">Add</Button>
        </div>
      </form>
    </Dialog>
  );
}
```

- [ ] **Step 3: Create edit group modal**

Create `src/components/modals/EditGroupModal.tsx`:

```typescript
import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function EditGroupModal() {
  const editingGroupId = useUIStore((s) => s.editingGroupId);
  const close = useUIStore((s) => s.closeEditGroupModal);
  const groups = useVaultStore((s) => s.groups);
  const updateGroup = useVaultStore((s) => s.updateGroup);
  const [name, setName] = useState('');
  const [icon, setIcon] = useState('📂');

  const editingGroup = groups.find((g) => g.id === editingGroupId);

  useEffect(() => {
    if (editingGroup) {
      setName(editingGroup.name);
      setIcon(editingGroup.icon || '📂');
    }
  }, [editingGroup]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (editingGroupId) {
      await updateGroup(editingGroupId, name, icon);
    }
    close();
  };

  const icons = ['📂', '🔒', '💼', '🏠', '💳', '📧', '🎮', '🛒'];

  return (
    <Dialog open={!!editingGroupId} onClose={close} title="Rename Group">
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium">Name *</label>
          <Input
            value={name}
            onChange={(e) => setName(e.target.value)}
            required
            maxLength={50}
          />
        </div>
        <div>
          <label className="mb-1 block text-sm font-medium">Icon</label>
          <div className="flex gap-2">
            {icons.map((i) => (
              <button
                key={i}
                type="button"
                onClick={() => setIcon(i)}
                className={`rounded p-2 text-xl ${icon === i ? 'bg-primary-100' : 'hover:bg-gray-100'}`}
              >
                {i}
              </button>
            ))}
          </div>
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={close}>
            Cancel
          </Button>
          <Button type="submit">Save</Button>
        </div>
      </form>
    </Dialog>
  );
}
```

- [ ] **Step 4: Create delete confirm modal**

Create `src/components/modals/DeleteConfirmModal.tsx`:

```typescript
import { Button } from '@/components/ui/button';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';

export function DeleteConfirmModal() {
  const target = useUIStore((s) => s.deleteConfirmTarget);
  const close = useUIStore((s) => s.closeDeleteConfirm);
  const deleteGroup = useVaultStore((s) => s.deleteGroup);
  const deleteItem = useVaultStore((s) => s.deleteItem);
  const items = useVaultStore((s) => s.items);

  if (!target) return null;

  const itemCount = target.type === 'group' 
    ? items.filter((i) => i.group_id === target.id).length 
    : 0;

  const handleDelete = async () => {
    if (target.type === 'group') {
      await deleteGroup(target.id);
    } else {
      await deleteItem(target.id);
    }
    close();
  };

  return (
    <Dialog open={!!target} onClose={close} title={`Delete ${target.type}?`}>
      <p className="mb-4 text-gray-600">
        Are you sure you want to delete "{target.name}"?
        {itemCount > 0 && (
          <span className="block mt-2 font-medium text-red-600">
            This will permanently delete {itemCount} item{itemCount !== 1 ? 's' : ''} in this group.
          </span>
        )}
      </p>
      <div className="flex justify-end gap-2">
        <Button variant="outline" onClick={close}>
          Cancel
        </Button>
        <Button variant="destructive" onClick={handleDelete}>
          Delete
        </Button>
      </div>
    </Dialog>
  );
}
```

- [ ] **Step 5: Commit modal components**

```bash
git add -A
git commit -m "feat: add modal components (add/edit item, add/edit group, delete confirm)"
```

---

## Chunk 7: Integration & Final Setup

### Task 7.1: Update App.tsx

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Update App.tsx with full integration**

Update `src/App.tsx`:

```typescript
import { useEffect } from 'react';
import { useVaultStore } from '@/stores/vault';
import { useActivityTracker } from '@/hooks/useActivityTracker';
import { SetupScreen } from '@/components/unlock/SetupScreen';
import { UnlockScreen } from '@/components/unlock/UnlockScreen';
import { Sidebar } from '@/components/layout/Sidebar';
import { ItemList } from '@/components/layout/ItemList';
import { DetailPanel } from '@/components/layout/DetailPanel';
import { AddEditItemModal } from '@/components/modals/AddEditItemModal';
import { AddGroupModal } from '@/components/modals/AddGroupModal';
import { EditGroupModal } from '@/components/modals/EditGroupModal';
import { DeleteConfirmModal } from '@/components/modals/DeleteConfirmModal';

function MainApp() {
  useActivityTracker();
  const loadItems = useVaultStore((s) => s.loadItems);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  return (
    <div className="flex h-screen">
      <Sidebar />
      <ItemList />
      <DetailPanel />
      <AddEditItemModal />
      <AddGroupModal />
      <EditGroupModal />
      <DeleteConfirmModal />
    </div>
  );
}

function App() {
  const isInitialized = useVaultStore((s) => s.isInitialized);
  const isUnlocked = useVaultStore((s) => s.isUnlocked);
  const checkInitialized = useVaultStore((s) => s.checkInitialized);

  useEffect(() => {
    checkInitialized();
  }, [checkInitialized]);

  if (isInitialized === null) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-gray-100">
        <p>Loading...</p>
      </div>
    );
  }

  if (!isInitialized) {
    return <SetupScreen />;
  }

  if (!isUnlocked) {
    return <UnlockScreen />;
  }

  return <MainApp />;
}

export default App;
```

- [ ] **Step 2: Commit App.tsx**

```bash
git add -A
git commit -m "feat: integrate all components in App.tsx"
```

### Task 7.2: Build and Test

- [ ] **Step 1: Verify frontend builds**

Run: `npm run build`
Expected: Build completes without errors

- [ ] **Step 2: Verify Tauri app builds**

Run: `npm run tauri build`
Expected: Build completes, produces executable in `src-tauri/target/release/`

- [ ] **Step 3: Test complete flow manually**

1. Run `npm run tauri dev`
2. Verify setup screen appears on first run
3. Create master password
4. Verify main interface loads
5. Add a new group
6. Add a new account item
7. View item details
8. Edit item
9. Delete item
10. Lock vault
11. Unlock vault
12. Verify auto-lock after 5 minutes

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete SecKeyBox Phase 1 MVP implementation"
```

---

## Summary

This implementation plan covers Phase 1 MVP of SecKeyBox, including:

1. **Project Setup** - Tauri 2.x + React 18 + TypeScript + Tailwind CSS
2. **Rust Backend** - Crypto (Argon2id, AES-256-GCM), Database (SQLite), Commands
3. **React Frontend** - Stores, Components, Modals
4. **Integration** - Full app assembly and testing

**Total estimated implementation time:** 4-6 hours for an experienced developer.

---

## Known Limitations for MVP

The following features from the spec are **deferred** to keep the MVP focused:

| Feature | Reason |
|---------|--------|
| Brute-force protection (5 attempts → 30s lockout) | Security enhancement; core unlock works without it |
| Toast notifications for clipboard | UX enhancement; copy still works |
| Password strength indicator | Phase 3 feature |

These can be added in a follow-up iteration after the MVP is functional.