# SecKeyBox - Windows Password Manager

## Overview

SecKeyBox is a secure password vault for Windows that manages environment variables, API keys, and account credentials with custom grouping support.

## Technology Stack

| Layer | Technology |
|-------|------------|
| Frontend | React 18 + TypeScript + Tailwind CSS |
| State Management | Zustand |
| Desktop Framework | Tauri 2.x |
| Backend | Rust |
| Database | SQLite + AES-256-GCM encryption |
| Cryptography | Argon2id (KDF) + AES-256-GCM (cipher) |
| UI Components | Shadcn/ui style |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      SecKeyBox                              │
├─────────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript + Tailwind)                   │
│  ├── Pages: Unlock, Main, Settings                          │
│  ├── Components: Sidebar, ItemList, DetailPanel, Search    │
│  └── State: Zustand                                         │
├─────────────────────────────────────────────────────────────┤
│  Tauri Bridge (IPC)                                         │
│  ├── tauri-plugin-sql (SQLite)                              │
│  ├── tauri-plugin-biometric (Windows Hello)                  │
│  ├── tauri-plugin-clipboard (Secure clipboard)              │
│  └── tauri-plugin-global-shortcut (Quick search)            │
├─────────────────────────────────────────────────────────────┤
│  Backend (Rust)                                             │
│  ├── Crypto: AES-256-GCM, Argon2id                          │
│  └── Database: SQLite                                       │
└─────────────────────────────────────────────────────────────┘
```

## Data Model

### Group

```typescript
interface Group {
  id: string;           // UUID
  name: string;
  icon?: string;        // emoji or icon identifier
  parent_id?: string;   // nested groups support
  order: number;        // sort weight
  created_at: number;
  updated_at: number;
}
```

### Item (Base)

```typescript
interface Item {
  id: string;
  group_id: string;
  type: 'account' | 'api_key' | 'env_var';
  title: string;
  icon?: string;
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
}
```

### Account Item

```typescript
interface AccountItem extends Item {
  type: 'account';
  username: string;
  password: string;      // encrypted
  website?: string;
  notes?: string;
}
```

### API Key Item

```typescript
interface ApiKeyItem extends Item {
  type: 'api_key';
  key_name: string;
  key_value: string;     // encrypted
  endpoint?: string;
  notes?: string;
}
```

### Environment Variable Item

```typescript
interface EnvVarItem extends Item {
  type: 'env_var';
  variables: {
    key: string;
    value: string;       // encrypted
  }[];
  notes?: string;
}
```

## SQLite Schema

```sql
-- Groups table
CREATE TABLE groups (
    id TEXT PRIMARY KEY,           -- UUID
    name TEXT NOT NULL,
    icon TEXT,
    parent_id TEXT REFERENCES groups(id),
    sort_order INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Items table (polymorphic base)
CREATE TABLE items (
    id TEXT PRIMARY KEY,           -- UUID
    group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK(type IN ('account', 'api_key', 'env_var')),
    title TEXT NOT NULL,
    icon TEXT,
    is_favorite INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Account items
CREATE TABLE account_items (
    item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    username TEXT NOT NULL,
    password_encrypted BLOB NOT NULL,  -- AES-256-GCM encrypted
    password_nonce BLOB NOT NULL,      -- 12 bytes
    website TEXT,
    notes TEXT
);

-- API Key items
CREATE TABLE api_key_items (
    item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    key_name TEXT NOT NULL,
    key_value_encrypted BLOB NOT NULL,
    key_value_nonce BLOB NOT NULL,
    endpoint TEXT,
    notes TEXT
);

-- Environment variable items
CREATE TABLE env_var_items (
    item_id TEXT REFERENCES items(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value_encrypted BLOB NOT NULL,
    value_nonce BLOB NOT NULL,
    sort_order INTEGER DEFAULT 0,
    PRIMARY KEY (item_id, key)
);

-- Master password hash (single row)
CREATE TABLE vault_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Stores:
-- 'password_hash' = Argon2id hash
-- 'password_salt' = random salt
-- 'failed_attempts' = current failed attempt count (reset on success)
-- 'lockout_until' = timestamp when lockout expires (null if not locked)

-- Indexes
CREATE INDEX idx_items_group ON items(group_id);
CREATE INDEX idx_items_type ON items(type);
CREATE INDEX idx_items_favorite ON items(is_favorite);
```

### Built-in Groups

Built-in categories (Accounts, API Keys, Environment Variables) are **seed data** inserted on first-run:

```sql
INSERT INTO groups (id, name, icon, sort_order) VALUES
    ('built-in-accounts', 'Accounts', '🔑', 0),
    ('built-in-api-keys', 'API Keys', '🔧', 1),
    ('built-in-env-vars', 'Environment Variables', '📦', 2);
```

- These groups can be renamed but **cannot be deleted**
- Custom groups are inserted after built-in groups (sort_order > 2)
- Frontend checks `id LIKE 'built-in-%'` to disable delete button

### Field Validation Limits

| Field | Max Length | Notes |
|-------|------------|-------|
| Group name | 50 chars | |
| Item title | 100 chars | |
| Username | 100 chars | |
| Password | 1000 chars | Supports long passphrases |
| Website URL | 500 chars | |
| Notes | 5000 chars | |
| API Key value | 10000 chars | Supports long tokens |
| Env var key | 100 chars | |
| Env var value | 5000 chars | |

## Tauri IPC Interface

### Vault Commands

```rust
// Check if vault is initialized (first-run detection)
#[tauri::command]
fn is_vault_initialized() -> Result<bool, String>;

// Initialize vault with new master password
#[tauri::command]
fn initialize_vault(master_password: String) -> Result<(), String>;

// Unlock vault with master password
#[tauri::command]
fn unlock_vault(master_password: String) -> Result<bool, String>;

// Lock vault (clear in-memory keys)
#[tauri::command]
fn lock_vault() -> Result<(), String>;

// Check if vault is unlocked
#[tauri::command]
fn is_vault_unlocked() -> bool;
```

### Group Commands

```rust
#[tauri::command]
fn get_all_groups() -> Result<Vec<Group>, String>;

#[tauri::command]
fn create_group(
    name: String, 
    icon: Option<String>, 
    parent_id: Option<String>,
    sort_order: Option<i32>
) -> Result<Group, String>;

#[tauri::command]
fn update_group(
    id: String, 
    name: String, 
    icon: Option<String>,
    sort_order: Option<i32>
) -> Result<(), String>;

#[tauri::command]
fn delete_group(id: String) -> Result<(), String>;  // Cascades to items
```

### Item Commands

```rust
#[tauri::command]
fn get_items_by_group(group_id: String) -> Result<Vec<ItemSummary>, String>;

#[tauri::command]
fn get_all_items() -> Result<Vec<ItemSummary>, String>;

#[tauri::command]
fn get_item_detail(id: String) -> Result<ItemDetail, String>;

#[tauri::command]
fn create_account_item(
    group_id: String,
    title: String,
    username: String,
    password: String,
    website: Option<String>,
    notes: Option<String>
) -> Result<String, String>;  // Returns new item ID

#[tauri::command]
fn update_account_item(
    id: String,
    title: String,
    username: String,
    password: Option<String>,  // None = don't change
    website: Option<String>,
    notes: Option<String>
) -> Result<(), String>;

#[tauri::command]
fn delete_item(id: String) -> Result<(), String>;

#[tauri::command]
fn toggle_favorite(id: String) -> Result<(), String>;
```

### Item Commands (Phase 2)

```rust
// API Key items - implemented in Phase 2
#[tauri::command]
fn create_api_key_item(
    group_id: String,
    title: String,
    key_name: String,
    key_value: String,
    endpoint: Option<String>,
    notes: Option<String>
) -> Result<String, String>;

#[tauri::command]
fn update_api_key_item(
    id: String,
    title: String,
    key_name: String,
    key_value: Option<String>,
    endpoint: Option<String>,
    notes: Option<String>
) -> Result<(), String>;

// Environment variable items - implemented in Phase 2
#[tauri::command]
fn create_env_var_item(
    group_id: String,
    title: String,
    variables: Vec<{ key: String, value: String }>,
    notes: Option<String>
) -> Result<String, String>;

#[tauri::command]
fn update_env_var_item(
    id: String,
    title: String,
    variables: Vec<{ key: String, value: String }>,
    notes: Option<String>
) -> Result<(), String>;
```

### Clipboard Commands

```rust
#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String>;
// Frontend starts 30s timer, then calls clear_clipboard

#[tauri::command]
fn clear_clipboard() -> Result<(), String>;
```

## First-Run Setup Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    First-Run Detection                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  App starts                                                 │
│      │                                                      │
│      ▼                                                      │
│  is_vault_initialized()?                                    │
│      │                                                      │
│      ├── false ──► Show Setup Screen                        │
│      │               │                                      │
│      │               ▼                                      │
│      │           ┌─────────────────────┐                    │
│      │           │  Welcome to SecKeyBox  │                 │
│      │           │                       │                  │
│      │           │  Create Master Password │                │
│      │           │  [________________]     │                │
│      │           │                       │                  │
│      │           │  Confirm Password       │                │
│      │           │  [________________]     │                │
│      │           │                       │                  │
│      │           │  [Create Vault]         │                │
│      │           └─────────────────────┘                    │
│      │               │                                      │
│      │               ▼                                      │
│      │           initialize_vault(password)                 │
│      │               │                                      │
│      │               ▼                                      │
│      │           Auto-unlock ──► Main Interface             │
│      │                                                      │
│      └── true ───► Show Unlock Screen                       │
│                      │                                      │
│                      ▼                                      │
│                  unlock_vault(password)                     │
│                      │                                      │
│                      ▼                                      │
│                  Main Interface                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Group Deletion Behavior

| Scenario | Behavior |
|----------|----------|
| Delete custom group | **Cascade delete** - all items in group are deleted with confirmation dialog |
| Delete built-in category (Accounts, API Keys, Environment Variables) | **Not allowed** - these are system groups |
| Delete group with subgroups | **Cascade delete** - subgroups and their items all deleted |

**Confirmation Dialog:**
```
Delete "Work"?
This will permanently delete 12 items in this group.
[Cancel] [Delete]
```

## Security Design

### Decryption Flow

```
User enters master password
        │
        ▼
┌─────────────┐
│  Argon2id   │  Memory: 64MB, Iterations: 3, Parallelism: 4
│  Key Deriv. │
└─────────────┘
        │
        ▼
┌─────────────┐
│ Master Key  │  (32 bytes)
└─────────────┘
        │
        ▼
┌─────────────┐
│ Decrypt DB  │  AES-256-GCM
│ Fields      │
└─────────────┘
```

### Security Features

| Feature | Implementation |
|---------|---------------|
| Master Password Storage | Argon2id hash only (no plaintext) |
| Sensitive Data Encryption | AES-256-GCM with unique nonce per field |
| Clipboard Security | Auto-clear after 30 seconds |
| Auto-lock | Lock after 5 minutes inactivity |
| Brute-force Protection | 5 failed attempts → 30 second lockout |

### Master Password Requirements

- Minimum length: 8 characters
- Recommended: 12+ characters with mixed case, numbers, symbols
- No maximum length limit
- Validated on both setup and unlock

### Auto-Lock Implementation

**Inactivity Detection:**
- Frontend tracks user activity: mouse movement, keyboard input, clicks
- Timer resets on any activity
- When 5 minutes pass with no activity → call `lock_vault()`

**Lock Triggers:**
1. Manual: User clicks "Lock" button or uses keyboard shortcut
2. Inactivity: 5 minutes no user input
3. System: App window loses focus for 60+ seconds (configurable)

### Brute-Force Protection

**Implementation:**
- Attempt counter stored in `vault_config` table: `failed_attempts` key
- Counter incremented on each failed unlock
- Counter reset on successful unlock
- After 5 failed attempts:
  - Show lockout message with countdown
  - Disable unlock button for 30 seconds
  - Clear counter after lockout expires

### Windows Hello Integration (Phase 2)

- Master password stored once, protected by Windows Hello
- Hello verification auto-unlocks without re-entering password

## Error Handling Strategy

### Error Categories

| Category | Handling |
|----------|----------|
| Crypto failure | Show error, force re-lock, log to debug file |
| Database error | Show error, attempt recovery, suggest reinstall |
| Corrupted data | Show error with item ID, skip item on load |
| Invalid password | Show "Invalid password", increment attempt counter |
| IPC timeout | Retry 3 times, then show error |

### User-Facing Error Messages

```typescript
enum VaultError {
  InvalidPassword = "The password you entered is incorrect",
  VaultLocked = "Please unlock the vault first",
  ItemNotFound = "This item no longer exists",
  GroupNotEmpty = "This group contains items. Delete them first or confirm cascade delete",
  CryptoError = "Failed to decrypt data. The vault may be corrupted",
  DatabaseError = "A database error occurred. Please restart the application",
}
```

### Clipboard Implementation

```typescript
// Frontend implementation
async function copyWithAutoClear(text: string) {
  await invoke('copy_to_clipboard', { text });
  
  // Schedule auto-clear after 30 seconds
  setTimeout(async () => {
    await invoke('clear_clipboard');
  }, 30000);
  
  showToast('Copied to clipboard (clears in 30s)');
}
```

## UI Design

### Visual Style

- **Material**: Windows 11 Mica effect for main window
- **Colors**: Dark/Light mode support, deep blue or brand purple accent
- **Corners**: 4-8px radius for cards, buttons, inputs
- **Font**: Segoe UI Variable (Windows default)

### Layout: Three-Column Structure

```
┌──────────┬─────────────────┬────────────────────────────┐
│ Left Nav │   Item List     │       Detail Panel         │
│   20%    │      30%        │          50%               │
├──────────┼─────────────────┼────────────────────────────┤
│ 🔍 Search│ ┌─────────────┐ │  ┌──────────────────────┐  │
│          │ │ Search Box  │ │  │  🔒 Item Title       │  │
│ 📁 All   │ ├─────────────┤ │  │  subtitle            │  │
│ ⭐ Fav   │ │ Sort ▼      │ │  ├──────────────────────┤  │
│          │ ├─────────────┤ │  │ Field: value    📋   │  │
│ ───────  │ │ [Card 1]    │ │  │ Password: •••• 👁 📋  │  │
│ 🔑 Acc   │ │  Google     │ │  ├──────────────────────┤  │
│ 🔧 API   │ │  user@g...  │ │  │ Notes:               │  │
│ 📦 Env   │ ├─────────────┤ │  │ ...                  │  │
│          │ │ [Card 2]    │ │  └──────────────────────┘  │
│ ───────  │ │  GitHub     │ │                            │
│ 📂 Work  │ └─────────────┘ │                            │
│ 📂 Personal│               │                            │
│ 📂 Bank  │                 │                            │
│          │                 │                            │
│ ───────  │                 │                            │
│ ⚙️ Settings│               │                            │
└──────────┴─────────────────┴────────────────────────────┘
```

### Left Navigation

- Search box
- "All Items" entry
- "Favorites" entry
- Separator
- Built-in categories: Accounts, API Keys, Environment Variables
- Separator
- Custom groups (user-defined)
- Settings button at bottom

### Center List

- Search/filter input
- Sort dropdown
- Item cards showing: icon, title, subtitle preview
- Click to select, keyboard navigation support

### Right Detail Panel

- Large icon + title
- All fields with copy buttons
- Password field with visibility toggle
- Notes text area
- Edit/Delete actions

### Unlock Screen

- Centered layout
- App logo + welcome message
- Master password input
- Windows Hello button (Phase 2)
- Minimal, focused design

### Search and Sort

**Search Scope:**
- Searches across: title, username, website, key_name
- Does NOT search encrypted fields (password, key_value) for security
- Case-insensitive, partial match

**Sort Options:**
- Name (A-Z)
- Name (Z-A)
- Date Created (newest first)
- Date Modified (newest first)

## Project Structure

```
SecKeyBox/
├── src/                          # React frontend
│   ├── components/
│   │   ├── ui/                   # Shadcn/ui base components
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx       # Left navigation
│   │   │   ├── ItemList.tsx      # Center list
│   │   │   └── DetailPanel.tsx   # Right detail panel
│   │   ├── unlock/
│   │   │   └── UnlockScreen.tsx  # Unlock page
│   │   └── modals/
│   │       ├── AddItemModal.tsx  # Add new item (handles both add/edit)
│   │       └── EditGroupModal.tsx
│   ├── pages/
│   │   └── Main.tsx
│   ├── stores/                   # Zustand state
│   │   ├── vault.ts              # Vault state (locked/unlocked, master key)
│   │   └── ui.ts                 # UI state (selected item, modals)
│   ├── hooks/
│   │   └── useActivityTracker.ts # Auto-lock inactivity detection
│   ├── lib/
│   │   ├── tauri.ts              # Tauri IPC wrapper
│   │   └── utils.ts              # Utility functions
│   └── App.tsx
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   ├── kdf.rs            # Argon2id
│   │   │   └── cipher.rs         # AES-256-GCM
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── operations.rs
│   │   └── commands/
│   │       └── vault.rs
│   └── Cargo.toml
├── package.json
└── tauri.conf.json
```

**Note:** `settings.ts` store and Settings page moved to Phase 3 (import/export, theme toggle).

## Development Phases

### Phase 1: Core MVP

- First-run setup with master password creation
- Unlock screen with master password
- Three-column layout structure
- Account/password CRUD operations
- Basic group management (create, rename, delete with cascade)
- SQLite encrypted storage
- Clipboard with 30s auto-clear
- Search and sort functionality

### Phase 2: Enhanced Features

- Environment variable management (Key-Value pairs)
- API Key management (long text support)
- Global quick search popup (Ctrl+Shift+Space)
- Windows Hello integration for quick unlock

### Phase 3: Advanced Features

- Favorites
- Password strength indicator
- Import/Export
- Dark/Light theme toggle

## Key Dependencies

### Frontend (package.json)

- react, react-dom
- typescript
- tailwindcss
- zustand
- @tauri-apps/api
- lucide-react (icons)

### Backend (Cargo.toml)

- tauri (2.x)
- aes-gcm
- argon2
- rusqlite
- uuid
- serde, serde_json