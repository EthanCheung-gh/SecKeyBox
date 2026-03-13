# SecKeyBox Phase 2: API Key Management Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add API key management support with encrypted key storage, partial masking, and type-switching UI.

**Architecture:** Extend existing account management pattern. Single modal with type selector, unified get_item_detail endpoint with type dispatch, run_migrations for schema upgrades.

**Tech Stack:** Tauri 2.x, Rust, SQLite, React 18, TypeScript, Zustand

---

## File Structure

```
src-tauri/src/
├── db/
│   ├── schema.rs          # MODIFY: Add run_migrations, bump SCHEMA_VERSION to 2
│   └── operations.rs      # MODIFY: Add ApiKeyItemDetail, ItemDetail enum, CRUD functions
├── commands/
│   └── items.rs           # MODIFY: Add API key commands, modify get_item_detail
└── lib.rs                 # MODIFY: Register new commands

src/
├── types/
│   └── index.ts           # MODIFY: Add ApiKeyItemDetail, update types
├── lib/
│   ├── tauri.ts           # MODIFY: Add API key IPC functions
│   └── utils.ts           # MODIFY: Add maskKeyValue, formatRotationDate
├── stores/
│   ├── vault.ts           # MODIFY: Add API key actions, update selectedItem type
│   └── ui.ts              # MODIFY: Add newItemType state
├── components/
│   ├── modals/
│   │   └── AddEditItemModal.tsx  # MODIFY: Add type selector, API key fields
│   └── layout/
│       └── DetailPanel.tsx       # MODIFY: Add API key rendering branch
```

---

## Chunk 1: Database Migration

### Task 1.1: Add run_migrations to schema.rs

**Files:**
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: Update SCHEMA_VERSION constant**

```rust
const SCHEMA_VERSION: i32 = 2;
```

- [ ] **Step 2: Add run_migrations function after init_schema**

Add after line 72 in `src-tauri/src/db/schema.rs`:

```rust
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version = get_schema_version(conn)?;

    if version < 2 {
        conn.execute_batch(
            "ALTER TABLE api_key_items ADD COLUMN auth_method TEXT;
             ALTER TABLE api_key_items ADD COLUMN rotation_date INTEGER;"
        ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;
    }

    set_schema_version(conn, SCHEMA_VERSION)?;
    Ok(())
}
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 4: Commit database migration changes**

```bash
git add src-tauri/src/db/schema.rs
git commit -m "feat(db): add run_migrations for API key columns"
```

### Task 1.2: Add migration call to unlock_vault

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

- [ ] **Step 1: Import run_migrations**

Change line 5 in `src-tauri/src/commands/vault.rs`:

```rust
use crate::db::{
    get_db_path, get_password_hash, get_password_salt, init_schema, insert_builtin_groups,
    open_connection, run_migrations, set_password_hash, DbConnection,
};
```

- [ ] **Step 2: Add migration call in unlock_vault**

Add after line 70 (after `let conn = ...`) in `src-tauri/src/commands/vault.rs`:

```rust
    run_migrations(&conn)?;
```

The unlock_vault function should now look like:

```rust
#[tauri::command]
pub fn unlock_vault(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    master_password: String,
) -> Result<bool> {
    let conn =
        db.0.lock()
            .map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;

    run_migrations(&conn)?;

    let stored_hash = get_password_hash(&conn)?.ok_or(VaultError::VaultNotInitialized)?;
    // ... rest unchanged
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 4: Commit unlock_vault changes**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "feat(vault): run migrations on unlock for existing vaults"
```

### Task 1.3: Add migration call to initialize_vault

**Files:**
- Modify: `src-tauri/src/commands/vault.rs`

- [ ] **Step 1: Add migration call in initialize_vault**

Add after `init_schema(&conn)?;` (line 41) in `src-tauri/src/commands/vault.rs`:

```rust
    run_migrations(&conn)?;
```

The initialize_vault function should now have:

```rust
    let conn = open_connection(&db_path)?;
    init_schema(&conn)?;
    run_migrations(&conn)?;
    insert_builtin_groups(&conn)?;
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 3: Commit initialize_vault changes**

```bash
git add src-tauri/src/commands/vault.rs
git commit -m "feat(vault): run migrations on initialize for new vaults"
```

---

## Chunk 2: Backend Operations

### Task 2.1: Add ApiKeyItemDetail struct and ItemDetail enum

**Files:**
- Modify: `src-tauri/src/db/operations.rs`

- [ ] **Step 1: Add ItemDetail enum after AccountItemDetail struct**

Add after line 49 in `src-tauri/src/db/operations.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemDetail {
    #[serde(rename = "account")]
    Account(AccountItemDetail),
    #[serde(rename = "api_key")]
    ApiKey(ApiKeyItemDetail),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub key_name: String,
    pub key_value: String,
    pub endpoint: Option<String>,
    pub auth_method: Option<String>,
    pub rotation_date: Option<i64>,
    pub notes: Option<String>,
}
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

### Task 2.2: Add API key CRUD functions

**Files:**
- Modify: `src-tauri/src/db/operations.rs`

- [ ] **Step 1: Add get_api_key_item_detail function**

Add after line 300 (after `get_account_item_detail` function) in `src-tauri/src/db/operations.rs`:

```rust
pub fn get_api_key_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<ApiKeyItemDetail> {
    let item: (String, String, String, Option<String>, bool, i64, i64) = conn
        .query_row(
            "SELECT id, group_id, title, icon, is_favorite, created_at, updated_at 
             FROM items WHERE id = ?1",
            [id],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get::<_, i32>(4)? != 0, row.get(5)?, row.get(6)?
            )),
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let api_key: (String, Vec<u8>, [u8; 12], Option<String>, Option<String>, Option<String>, Option<i64>) = conn
        .query_row(
            "SELECT key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date
             FROM api_key_items WHERE item_id = ?1",
            [id],
            |row| {
                let nonce_bytes: Vec<u8> = row.get(2)?;
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                Ok((
                    row.get(0)?, row.get(1)?, nonce, row.get(3)?,
                    row.get(4)?, row.get(5)?, row.get(6)?
                ))
            },
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let key_value = String::from_utf8(
        crate::crypto::decrypt(key, &api_key.2, &api_key.1)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?
    ).map_err(|_| VaultError::CryptoError("Invalid UTF-8 in key value".to_string()))?;

    Ok(ApiKeyItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        key_name: api_key.0,
        key_value,
        endpoint: api_key.3,
        auth_method: api_key.4,
        notes: api_key.5,
        rotation_date: api_key.6,
    })
}
```

- [ ] **Step 2: Add create_api_key_item function**

Add after `get_api_key_item_detail`:

```rust
pub fn create_api_key_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    key_name: &str,
    key_value: &str,
    endpoint: Option<&str>,
    auth_method: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    let encrypted = crate::crypto::encrypt(key, key_value.as_bytes())
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'api_key', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    conn.execute(
        "INSERT INTO api_key_items (item_id, key_name, key_value_encrypted, key_value_nonce, endpoint, auth_method, notes, rotation_date) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, key_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), endpoint, auth_method, notes, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(id)
}
```

- [ ] **Step 3: Add update_api_key_item function**

Add after `create_api_key_item`:

```rust
pub fn update_api_key_item(
    conn: &Connection,
    id: &str,
    title: &str,
    key_name: &str,
    key_value: Option<&str>,
    endpoint: Option<&str>,
    auth_method: Option<&str>,
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    if let Some(kv) = key_value {
        let encrypted = crate::crypto::encrypt(key, kv.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "UPDATE api_key_items SET key_name = ?1, key_value_encrypted = ?2, key_value_nonce = ?3, endpoint = ?4, auth_method = ?5, notes = ?6, rotation_date = ?7 WHERE item_id = ?8",
            params![key_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), endpoint, auth_method, notes, now, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE api_key_items SET key_name = ?1, endpoint = ?2, auth_method = ?3, notes = ?4 WHERE item_id = ?5",
            params![key_name, endpoint, auth_method, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}
```

- [ ] **Step 4: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

### Task 2.3: Update item list queries to include API key subtitle

**Files:**
- Modify: `src-tauri/src/db/operations.rs`

- [ ] **Step 1: Update get_all_items query**

Replace the `get_all_items` function (lines 191-220) with:

```rust
pub fn get_all_items(conn: &Connection) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, 
                COALESCE(a.username, ak.key_name, '') as subtitle,
                i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at 
             FROM items i 
             LEFT JOIN account_items a ON i.id = a.item_id 
             LEFT JOIN api_key_items ak ON i.id = ak.item_id 
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
```

- [ ] **Step 2: Update get_items_by_group query**

Replace the `get_items_by_group` function (lines 222-252) with:

```rust
pub fn get_items_by_group(conn: &Connection, group_id: &str) -> Result<Vec<ItemSummary>> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.title, 
                COALESCE(a.username, ak.key_name, '') as subtitle,
                i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at 
             FROM items i 
             LEFT JOIN account_items a ON i.id = a.item_id 
             LEFT JOIN api_key_items ak ON i.id = ak.item_id 
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
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 4: Commit backend operations**

```bash
git add src-tauri/src/db/operations.rs
git commit -m "feat(db): add API key structs, CRUD functions, and update item queries"
```

---

## Chunk 3: Backend Commands

### Task 3.1: Update imports and modify get_item_detail

**Files:**
- Modify: `src-tauri/src/commands/items.rs`

- [ ] **Step 1: Update imports**

Replace line 3 with:

```rust
use crate::db::{DbConnection, get_all_items, get_items_by_group, get_account_item_detail, get_api_key_item_detail, create_account_item, create_api_key_item, update_account_item, update_api_key_item, delete_item, toggle_favorite, ItemSummary, AccountItemDetail, ApiKeyItemDetail, ItemDetail};
```

- [ ] **Step 2: Modify get_item_detail to support type dispatch**

Replace the `get_item_detail` function (lines 18-32) with:

```rust
#[tauri::command]
pub fn get_item_detail(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
) -> Result<ItemDetail> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    let item_type: String = conn.query_row(
        "SELECT type FROM items WHERE id = ?1",
        [&id],
        |row| row.get(0),
    ).map_err(|_| VaultError::ItemNotFound)?;
    
    match item_type.as_str() {
        "account" => get_account_item_detail(&conn, &id, key.as_bytes())
            .map(|d| ItemDetail::Account(d)),
        "api_key" => get_api_key_item_detail(&conn, &id, key.as_bytes())
            .map(|d| ItemDetail::ApiKey(d)),
        _ => Err(VaultError::ItemNotFound),
    }
}
```

### Task 3.2: Add API key CRUD commands

**Files:**
- Modify: `src-tauri/src/commands/items.rs`

- [ ] **Step 1: Add create_new_api_key_item command**

Add after `update_existing_account_item` function:

```rust
#[tauri::command]
pub fn create_new_api_key_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    key_name: String,
    key_value: String,
    endpoint: Option<String>,
    auth_method: Option<String>,
    notes: Option<String>,
) -> Result<String> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if key_name.is_empty() || key_name.len() > 100 {
        return Err(VaultError::DatabaseError("Key name must be 1-100 characters".to_string()));
    }
    if key_value.is_empty() || key_value.len() > 50000 {
        return Err(VaultError::DatabaseError("Key value must be 1-50000 characters".to_string()));
    }
    if let Some(ref e) = endpoint {
        if e.len() > 500 {
            return Err(VaultError::DatabaseError("Endpoint must be under 500 characters".to_string()));
        }
    }
    if let Some(ref am) = auth_method {
        if am.len() > 20 {
            return Err(VaultError::DatabaseError("Auth method must be under 20 characters".to_string()));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError("Notes must be under 5000 characters".to_string()));
        }
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    create_api_key_item(
        &conn,
        &group_id,
        &title,
        &key_name,
        &key_value,
        endpoint.as_deref(),
        auth_method.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}
```

- [ ] **Step 2: Add update_existing_api_key_item command**

Add after `create_new_api_key_item`:

```rust
#[tauri::command]
pub fn update_existing_api_key_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    key_name: String,
    key_value: Option<String>,
    endpoint: Option<String>,
    auth_method: Option<String>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if key_name.is_empty() || key_name.len() > 100 {
        return Err(VaultError::DatabaseError("Key name must be 1-100 characters".to_string()));
    }
    if let Some(ref kv) = key_value {
        if kv.is_empty() || kv.len() > 50000 {
            return Err(VaultError::DatabaseError("Key value must be 1-50000 characters".to_string()));
        }
    }
    if let Some(ref e) = endpoint {
        if e.len() > 500 {
            return Err(VaultError::DatabaseError("Endpoint must be under 500 characters".to_string()));
        }
    }
    if let Some(ref am) = auth_method {
        if am.len() > 20 {
            return Err(VaultError::DatabaseError("Auth method must be under 20 characters".to_string()));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError("Notes must be under 5000 characters".to_string()));
        }
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    update_api_key_item(
        &conn,
        &id,
        &title,
        &key_name,
        key_value.as_deref(),
        endpoint.as_deref(),
        auth_method.as_deref(),
        notes.as_deref(),
        key.as_bytes(),
    )
}
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

### Task 3.3: Register new commands in lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Update db imports**

Replace line 10-14 with:

```rust
pub use db::{
    init_schema, insert_builtin_groups, run_migrations,
    Group, ItemSummary, AccountItemDetail, ApiKeyItemDetail, ItemDetail,
    DbConnection, open_connection, get_db_path,
};
```

- [ ] **Step 2: Register new commands in generate_handler!**

Add `create_new_api_key_item` and `update_existing_api_key_item` to the invoke_handler. Replace lines 36-55 with:

```rust
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
            create_new_api_key_item,
            update_existing_api_key_item,
            delete_existing_item,
            toggle_item_favorite,
            copy_to_clipboard,
            clear_clipboard,
        ])
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 4: Commit backend commands**

```bash
git add src-tauri/src/commands/items.rs src-tauri/src/lib.rs
git commit -m "feat(cmd): add API key commands and type dispatch for get_item_detail"
```

---

## Chunk 4: Frontend Types & IPC

### Task 4.1: Update TypeScript types

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Update ItemSummary type**

Replace lines 11-21 with:

```typescript
export interface ItemSummary {
  id: string;
  title: string;
  subtitle: string;
  icon?: string;
  type: 'account' | 'api_key';
  is_favorite: boolean;
  group_id: string;
  created_at: number;
  updated_at: number;
}
```

- [ ] **Step 2: Update ItemDetail to be a union type**

Replace lines 23-36 with:

```typescript
export interface AccountItemDetail {
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

export interface ApiKeyItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'api_key';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  key_name: string;
  key_value: string;
  endpoint?: string;
  auth_method?: string;
  rotation_date?: number;
  notes?: string;
}

export type ItemDetail = AccountItemDetail | ApiKeyItemDetail;
```

- [ ] **Step 3: Add API key input types**

Add after `UpdateAccountItemInput`:

```typescript
export interface CreateApiKeyItemInput {
  group_id: string;
  title: string;
  key_name: string;
  key_value: string;
  endpoint?: string;
  auth_method?: string;
  notes?: string;
}

export interface UpdateApiKeyItemInput {
  id: string;
  title: string;
  key_name: string;
  key_value?: string;
  endpoint?: string;
  auth_method?: string;
  notes?: string;
}
```

- [ ] **Step 4: Verify TypeScript compiles**

Run: `npm run build 2>&1`
Expected: Build succeeds (may have type errors in other files, fix in next tasks)

### Task 4.2: Add API key IPC functions

**Files:**
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Update imports**

Replace line 2 with:

```typescript
import type { Group, ItemSummary, ItemDetail, ApiKeyItemDetail } from '@/types';
```

- [ ] **Step 2: Add API key IPC functions**

Add after `toggleItemFavorite` function (after line 80):

```typescript
export async function createNewApiKeyItem(
  groupId: string,
  title: string,
  keyName: string,
  keyValue: string,
  endpoint?: string,
  authMethod?: string,
  notes?: string
): Promise<string> {
  return invoke<string>('create_new_api_key_item', { 
    group_id: groupId, 
    title, 
    key_name: keyName, 
    key_value: keyValue, 
    endpoint, 
    auth_method: authMethod, 
    notes 
  });
}

export async function updateExistingApiKeyItem(
  id: string,
  title: string,
  keyName: string,
  keyValue?: string,
  endpoint?: string,
  authMethod?: string,
  notes?: string
): Promise<void> {
  return invoke<void>('update_existing_api_key_item', { 
    id, 
    title, 
    key_name: keyName, 
    key_value: keyValue, 
    endpoint, 
    auth_method: authMethod, 
    notes 
  });
}
```

- [ ] **Step 3: Commit frontend types and IPC**

```bash
git add src/types/index.ts src/lib/tauri.ts
git commit -m "feat(frontend): add API key types and IPC functions"
```

---

## Chunk 5: Frontend Store & Utils

### Task 5.1: Add utility functions

**Files:**
- Modify: `src/lib/utils.ts`

- [ ] **Step 1: Add maskKeyValue and formatRotationDate**

Add after `truncate` function:

```typescript
export function maskKeyValue(keyValue: string): string {
  if (keyValue.length <= 4) return '••••';
  const lastFour = keyValue.slice(-4);
  return `••••${lastFour}`;
}

export function formatRotationDate(timestamp: number | undefined): string {
  if (!timestamp) return 'Never rotated';
  const days = Math.floor((Date.now() / 1000 - timestamp) / 86400);
  if (days === 0) return 'Rotated today';
  if (days === 1) return 'Rotated yesterday';
  return `Rotated ${days} days ago`;
}
```

### Task 5.2: Update vault store

**Files:**
- Modify: `src/stores/vault.ts`

- [ ] **Step 1: Update imports**

Replace line 2 with:

```typescript
import type { Group, ItemSummary, ItemDetail, AccountItemDetail, ApiKeyItemDetail } from '@/types';
```

- [ ] **Step 2: Update VaultState interface**

Replace line 10 with:

```typescript
  selectedItem: AccountItemDetail | ApiKeyItemDetail | null;
```

Add after `deleteItem` in the interface (after line 40):

```typescript
  createApiKeyItem: (data: {
    groupId: string;
    title: string;
    keyName: string;
    keyValue: string;
    endpoint?: string;
    authMethod?: string;
    notes?: string;
  }) => Promise<void>;
  updateApiKeyItem: (id: string, data: {
    title: string;
    keyName: string;
    keyValue?: string;
    endpoint?: string;
    authMethod?: string;
    notes?: string;
  }) => Promise<void>;
```

- [ ] **Step 3: Add createApiKeyItem action**

Add after `deleteItem` action (after line 192):

```typescript
  createApiKeyItem: async (data) => {
    try {
      await api.createNewApiKeyItem(
        data.groupId, data.title, data.keyName, data.keyValue,
        data.endpoint, data.authMethod, data.notes
      );
      await get().loadItems();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  updateApiKeyItem: async (id, data) => {
    try {
      await api.updateExistingApiKeyItem(
        id, data.title, data.keyName, data.keyValue,
        data.endpoint, data.authMethod, data.notes
      );
      await get().loadItems();
      if (get().selectedItem?.id === id) {
        await get().selectItem(id);
      }
    } catch (e) {
      set({ error: String(e) });
    }
  },
```

- [ ] **Step 4: Verify TypeScript compiles**

Run: `npm run build 2>&1`
Expected: Build succeeds

- [ ] **Step 5: Commit store and utils**

```bash
git add src/lib/utils.ts src/stores/vault.ts
git commit -m "feat(store): add API key actions and utility functions"
```

### Task 5.3: Update UI store

**Files:**
- Modify: `src/stores/ui.ts`

- [ ] **Step 1: Add newItemType state**

Add after `deleteConfirmTarget` in the interface (after line 11):

```typescript
  newItemType: 'account' | 'api_key';
```

Add after `closeDeleteConfirm` in the interface (after line 25):

```typescript
  setNewItemType: (type: 'account' | 'api_key') => void;
```

- [ ] **Step 2: Add state and setter implementation**

Add after `deleteConfirmTarget: null,` (after line 36):

```typescript
  newItemType: 'account',
```

Add after `closeDeleteConfirm` implementation (after line 50):

```typescript
  setNewItemType: (type) => set({ newItemType: type }),
```

- [ ] **Step 3: Commit UI store**

```bash
git add src/stores/ui.ts
git commit -m "feat(ui): add newItemType state for type selector"
```

---

## Chunk 6: UI Components

### Task 6.1: Update AddEditItemModal with type selector

**Files:**
- Modify: `src/components/modals/AddEditItemModal.tsx`

- [ ] **Step 1: Add imports and state for API key form**

Replace the file content with:

```typescript
import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog } from '@/components/ui/dialog';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import type { ItemDetail } from '@/types';

export function AddEditItemModal() {
  const isOpen = useUIStore((s) => s.isAddItemModalOpen || s.editingItemId !== null);
  const editingItemId = useUIStore((s) => s.editingItemId);
  const closeAddItemModal = useUIStore((s) => s.closeAddItemModal);
  const closeEditItemModal = useUIStore((s) => s.closeEditItemModal);
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const groups = useVaultStore((s) => s.groups);
  const createItem = useVaultStore((s) => s.createItem);
  const updateItem = useVaultStore((s) => s.updateItem);
  const createApiKeyItem = useVaultStore((s) => s.createApiKeyItem);
  const updateApiKeyItem = useVaultStore((s) => s.updateApiKeyItem);
  const selectedGroupId = useUIStore((s) => s.selectedGroupId);
  const newItemType = useUIStore((s) => s.newItemType);
  const setNewItemType = useUIStore((s) => s.setNewItemType);

  const [accountForm, setAccountForm] = useState({
    groupId: '',
    title: '',
    username: '',
    password: '',
    website: '',
    notes: '',
  });

  const [apiKeyForm, setApiKeyForm] = useState({
    groupId: '',
    title: '',
    keyName: '',
    keyValue: '',
    endpoint: '',
    authMethod: '',
    notes: '',
  });

  const editingType = selectedItem?.type || 'account';
  const isEditing = editingItemId !== null;
  const currentType = isEditing ? editingType : newItemType;

  useEffect(() => {
    if (isEditing && selectedItem) {
      if (selectedItem.type === 'account') {
        setAccountForm({
          groupId: selectedItem.group_id,
          title: selectedItem.title,
          username: selectedItem.username,
          password: '',
          website: selectedItem.website || '',
          notes: selectedItem.notes || '',
        });
      } else if (selectedItem.type === 'api_key') {
        setApiKeyForm({
          groupId: selectedItem.group_id,
          title: selectedItem.title,
          keyName: selectedItem.key_name,
          keyValue: '',
          endpoint: selectedItem.endpoint || '',
          authMethod: selectedItem.auth_method || '',
          notes: selectedItem.notes || '',
        });
      }
    } else {
      const defaultGroupId = selectedGroupId || groups[0]?.id || '';
      setAccountForm({
        groupId: defaultGroupId,
        title: '',
        username: '',
        password: '',
        website: '',
        notes: '',
      });
      setApiKeyForm({
        groupId: defaultGroupId,
        title: '',
        keyName: '',
        keyValue: '',
        endpoint: '',
        authMethod: '',
        notes: '',
      });
    }
  }, [isEditing, selectedItem, selectedGroupId, groups]);

  const handleClose = () => {
    closeAddItemModal();
    closeEditItemModal();
    setNewItemType('account');
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (currentType === 'account') {
      if (isEditing) {
        await updateItem(editingItemId!, {
          title: accountForm.title,
          username: accountForm.username,
          password: accountForm.password || undefined,
          website: accountForm.website || undefined,
          notes: accountForm.notes || undefined,
        });
      } else {
        await createItem({
          groupId: accountForm.groupId,
          title: accountForm.title,
          username: accountForm.username,
          password: accountForm.password,
          website: accountForm.website || undefined,
          notes: accountForm.notes || undefined,
        });
      }
    } else if (currentType === 'api_key') {
      if (isEditing) {
        await updateApiKeyItem(editingItemId!, {
          title: apiKeyForm.title,
          keyName: apiKeyForm.keyName,
          keyValue: apiKeyForm.keyValue || undefined,
          endpoint: apiKeyForm.endpoint || undefined,
          authMethod: apiKeyForm.authMethod || undefined,
          notes: apiKeyForm.notes || undefined,
        });
      } else {
        await createApiKeyItem({
          groupId: apiKeyForm.groupId,
          title: apiKeyForm.title,
          keyName: apiKeyForm.keyName,
          keyValue: apiKeyForm.keyValue,
          endpoint: apiKeyForm.endpoint || undefined,
          authMethod: apiKeyForm.authMethod || undefined,
          notes: apiKeyForm.notes || undefined,
        });
      }
    }
    handleClose();
  };

  const renderAccountFields = () => (
    <>
      <div>
        <label className="mb-1 block text-sm font-medium">Username *</label>
        <Input
          value={accountForm.username}
          onChange={(e) => setAccountForm({ ...accountForm, username: e.target.value })}
          required
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">
          Password {isEditing ? '(leave blank to keep)' : '*'}
        </label>
        <Input
          type="password"
          value={accountForm.password}
          onChange={(e) => setAccountForm({ ...accountForm, password: e.target.value })}
          required={!isEditing}
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Website</label>
        <Input
          value={accountForm.website}
          onChange={(e) => setAccountForm({ ...accountForm, website: e.target.value })}
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Notes</label>
        <textarea
          value={accountForm.notes}
          onChange={(e) => setAccountForm({ ...accountForm, notes: e.target.value })}
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
          rows={3}
        />
      </div>
    </>
  );

  const renderApiKeyFields = () => (
    <>
      <div>
        <label className="mb-1 block text-sm font-medium">Key Name *</label>
        <Input
          value={apiKeyForm.keyName}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, keyName: e.target.value })}
          required
          placeholder="e.g., OpenAI API Key"
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">
          Key Value {isEditing ? '(leave blank to keep)' : '*'}
        </label>
        <textarea
          value={apiKeyForm.keyValue}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, keyValue: e.target.value })}
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm font-mono"
          rows={4}
          required={!isEditing}
          placeholder="Paste your API key here..."
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Endpoint URL</label>
        <Input
          value={apiKeyForm.endpoint}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, endpoint: e.target.value })}
          placeholder="https://api.example.com"
        />
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Auth Method</label>
        <select
          value={apiKeyForm.authMethod}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, authMethod: e.target.value })}
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
        >
          <option value="">Select...</option>
          <option value="bearer">Bearer Token</option>
          <option value="api_key">API Key Header</option>
          <option value="basic">Basic Auth</option>
          <option value="custom">Custom</option>
        </select>
      </div>
      <div>
        <label className="mb-1 block text-sm font-medium">Notes</label>
        <textarea
          value={apiKeyForm.notes}
          onChange={(e) => setApiKeyForm({ ...apiKeyForm, notes: e.target.value })}
          className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
          rows={3}
        />
      </div>
    </>
  );

  return (
    <Dialog open={isOpen} onClose={handleClose} title={isEditing ? 'Edit Item' : 'Add Item'}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {!isEditing && (
          <div>
            <label className="mb-1 block text-sm font-medium">Type</label>
            <select
              value={currentType}
              onChange={(e) => setNewItemType(e.target.value as 'account' | 'api_key')}
              className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            >
              <option value="account">Account</option>
              <option value="api_key">API Key</option>
            </select>
          </div>
        )}
        <div>
          <label className="mb-1 block text-sm font-medium">Group</label>
          <select
            value={currentType === 'account' ? accountForm.groupId : apiKeyForm.groupId}
            onChange={(e) => {
              const form = { groupId: e.target.value };
              if (currentType === 'account') {
                setAccountForm({ ...accountForm, ...form });
              } else {
                setApiKeyForm({ ...apiKeyForm, ...form });
              }
            }}
            className="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm"
            disabled={isEditing}
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
            value={currentType === 'account' ? accountForm.title : apiKeyForm.title}
            onChange={(e) => {
              if (currentType === 'account') {
                setAccountForm({ ...accountForm, title: e.target.value });
              } else {
                setApiKeyForm({ ...apiKeyForm, title: e.target.value });
              }
            }}
            required
          />
        </div>
        {currentType === 'account' ? renderAccountFields() : renderApiKeyFields()}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button type="submit">{isEditing ? 'Save' : 'Add'}</Button>
        </div>
      </form>
    </Dialog>
  );
}
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run build 2>&1`
Expected: Build succeeds

### Task 6.2: Update DetailPanel with API key rendering

**Files:**
- Modify: `src/components/layout/DetailPanel.tsx`

- [ ] **Step 1: Replace DetailPanel content**

Replace the file content with:

```typescript
import { Copy, Eye, EyeOff, Edit, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { useVaultStore } from '@/stores/vault';
import { useUIStore } from '@/stores/ui';
import { copyToClipboard } from '@/lib/tauri';
import { maskKeyValue, formatRotationDate } from '@/lib/utils';
import type { AccountItemDetail, ApiKeyItemDetail } from '@/types';

export function DetailPanel() {
  const selectedItem = useVaultStore((s) => s.selectedItem);
  const openEditItemModal = useUIStore((s) => s.openEditItemModal);
  const openDeleteConfirm = useUIStore((s) => s.openDeleteConfirm);
  const [showPassword, setShowPassword] = useState(false);
  const [showKeyValue, setShowKeyValue] = useState(false);

  if (!selectedItem) {
    return (
      <div className="flex h-full flex-1 items-center justify-center bg-gray-50">
        <p className="text-gray-400">Select an item to view details</p>
      </div>
    );
  }

  const handleCopy = async (text: string) => {
    await copyToClipboard(text);
    setTimeout(async () => {
      await import('@/lib/tauri').then(api => api.clearClipboard());
    }, 30000);
  };

  const renderAccountDetail = () => {
    const item = selectedItem as AccountItemDetail;
    return (
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Username</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">{item.username}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.username)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Password</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">
              {showPassword ? item.password : '••••••••'}
            </span>
            <Button variant="ghost" size="sm" onClick={() => setShowPassword(!showPassword)}>
              {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.password)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {item.website && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Website</label>
            <a
              href={item.website}
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary-600 hover:underline"
            >
              {item.website}
            </a>
          </div>
        )}

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  const renderApiKeyDetail = () => {
    const item = selectedItem as ApiKeyItemDetail;
    return (
      <div className="space-y-4">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Key Name</label>
          <div className="flex items-center gap-2">
            <span className="font-medium">{item.key_name}</span>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.key_name)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Key Value</label>
          <div className="flex items-center gap-2">
            <span className="font-medium font-mono text-sm">
              {showKeyValue ? item.key_value : maskKeyValue(item.key_value)}
            </span>
            <Button variant="ghost" size="sm" onClick={() => setShowKeyValue(!showKeyValue)}>
              {showKeyValue ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="sm" onClick={() => handleCopy(item.key_value)}>
              <Copy className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {item.endpoint && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Endpoint</label>
            <div className="flex items-center gap-2">
              <a
                href={item.endpoint}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary-600 hover:underline"
              >
                {item.endpoint}
              </a>
              <Button variant="ghost" size="sm" onClick={() => handleCopy(item.endpoint)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>
        )}

        {item.auth_method && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Auth Method</label>
            <div className="flex items-center gap-2">
              <span className="font-medium capitalize">{item.auth_method.replace('_', ' ')}</span>
              <Button variant="ghost" size="sm" onClick={() => handleCopy(item.auth_method)}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          </div>
        )}

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-500">Last Rotated</label>
          <span className="text-gray-700">{formatRotationDate(item.rotation_date)}</span>
        </div>

        {item.notes && (
          <div>
            <label className="mb-1 block text-sm font-medium text-gray-500">Notes</label>
            <p className="whitespace-pre-wrap text-gray-700">{item.notes}</p>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="flex h-full flex-1 flex-col bg-white p-6">
      <div className="mb-6 flex items-start justify-between">
        <div>
          <h1 className="text-xl font-semibold">{selectedItem.title}</h1>
          <span className="text-sm text-gray-500 capitalize">{selectedItem.type.replace('_', ' ')}</span>
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

      {selectedItem.type === 'account' ? renderAccountDetail() : renderApiKeyDetail()}
    </div>
  );
}
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `npm run build 2>&1`
Expected: Build succeeds

- [ ] **Step 3: Commit UI components**

```bash
git add src/components/modals/AddEditItemModal.tsx src/components/layout/DetailPanel.tsx
git commit -m "feat(ui): add type selector and API key fields to modal and detail panel"
```

---

## Chunk 7: Integration Testing

### Task 7.1: Build and verify

- [ ] **Step 1: Verify frontend builds**

Run: `npm run build 2>&1`
Expected: Build completes without errors

- [ ] **Step 2: Verify Rust builds**

Run: `cd src-tauri && cargo build 2>&1`
Expected: Compilation succeeds

- [ ] **Step 3: Run Tauri dev mode**

Run: `npm run tauri dev 2>&1`
Expected: App opens without errors

### Task 7.2: Manual testing

- [ ] **Step 1: Test migration on existing vault**
  1. Open an existing vault from Phase 1
  2. Verify app loads without errors
  3. Check that `auth_method` and `rotation_date` columns exist

- [ ] **Step 2: Test API key creation**
  1. Click "Add" button
  2. Select "API Key" type
  3. Fill in: Title, Key Name, Key Value
  4. Click "Add"
  5. Verify item appears in list

- [ ] **Step 3: Test API key viewing**
  1. Click on the API key item
  2. Verify key_value shows masked (••••xxxx)
  3. Click eye icon to reveal
  4. Copy key_value and verify clipboard

- [ ] **Step 4: Test API key editing**
  1. Click Edit on an API key
  2. Change title and key name
  3. Leave key value blank
  4. Save and verify rotation_date unchanged

- [ ] **Step 5: Test API key with new key value**
  1. Edit API key
  2. Enter new key value
  3. Save and verify rotation_date updates to "Rotated today"

- [ ] **Step 6: Test API key deletion**
  1. Delete an API key
  2. Verify it's removed from list

- [ ] **Step 7: Final commit**

```bash
git add -A
git commit -m "feat: complete API Key management implementation"
```

---

## Summary

This plan implements API Key management in 7 chunks:

1. **Database Migration** - Schema version 2, run_migrations
2. **Backend Operations** - Structs, CRUD functions, query updates
3. **Backend Commands** - Type dispatch, new commands
4. **Frontend Types & IPC** - TypeScript types, API functions
5. **Frontend Store & Utils** - Zustand actions, utility functions
6. **UI Components** - Type selector, API key fields, detail rendering
7. **Integration Testing** - Build verification, manual tests