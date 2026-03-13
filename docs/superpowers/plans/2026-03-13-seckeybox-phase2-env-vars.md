# SecKeyBox Phase 2: Environment Variables Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add environment variable management support with encrypted key-value pairs.

**Architecture:** Extend existing item pattern. Multiple key-value pairs per item, each encrypted separately.

**Tech Stack:** Tauri 2.x, Rust, SQLite, React 18, TypeScript, Zustand

---

## File Structure

```
src-tauri/src/
├── db/
│   ├── schema.rs          # MODIFY: Bump SCHEMA_VERSION to 3, add notes column migration
│   └── operations.rs      # MODIFY: Add EnvVarItemDetail, CRUD functions
├── commands/
│   └── items.rs           # MODIFY: Add env var commands, update get_item_detail
└── lib.rs                 # MODIFY: Register new commands

src/
├── types/
│   └── index.ts           # MODIFY: Add EnvVarItemDetail, input types
├── lib/
│   └── tauri.ts           # MODIFY: Add env var IPC functions
├── stores/
│   └── vault.ts           # MODIFY: Add env var actions
├── components/
│   ├── modals/
│   │   └── AddEditItemModal.tsx  # MODIFY: Add env var form fields
│   └── layout/
│       └── DetailPanel.tsx       # MODIFY: Add env var rendering
```

---

## Chunk 1: Database Migration

### Task 1.1: Add notes column migration

**Files:**
- Modify: `src-tauri/src/db/schema.rs`

- [ ] **Step 1: Update SCHEMA_VERSION**

```rust
const SCHEMA_VERSION: i32 = 3;
```

- [ ] **Step 2: Add migration for notes column**

```rust
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version = get_schema_version(conn)?;

    if version < 2 {
        conn.execute_batch(
            "ALTER TABLE api_key_items ADD COLUMN auth_method TEXT;
             ALTER TABLE api_key_items ADD COLUMN rotation_date INTEGER;",
        ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;
    }

    if version < 3 {
        conn.execute_batch(
            "ALTER TABLE env_var_items ADD COLUMN notes TEXT;",
        ).map_err(|e| crate::error::VaultError::DatabaseError(e.to_string()))?;
    }

    set_schema_version(conn, SCHEMA_VERSION)?;
    Ok(())
}
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 4: Commit migration**

```bash
git add src-tauri/src/db/schema.rs
git commit -m "feat(db): add notes column migration for env_var_items"
```

---

## Chunk 2: Backend Operations

### Task 2.1: Add EnvVarItemDetail struct

**Files:**
- Modify: `src-tauri/src/db/operations.rs`

- [ ] **Step 1: Add EnvVarItemDetail struct after ApiKeyItemDetail**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarItemDetail {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub icon: Option<String>,
    pub is_favorite: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub variables: Vec<EnvVarPair>,
    pub notes: Option<String>,
}
```

- [ ] **Step 2: Update ItemDetail enum**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ItemDetail {
    #[serde(rename = "account")]
    Account(AccountItemDetail),
    #[serde(rename = "api_key")]
    ApiKey(ApiKeyItemDetail),
    #[serde(rename = "env_var")]
    EnvVar(EnvVarItemDetail),
}
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

### Task 2.2: Add env var CRUD functions

**Files:**
- Modify: `src-tauri/src/db/operations.rs`

- [ ] **Step 1: Add get_env_var_item_detail function**

```rust
pub fn get_env_var_item_detail(
    conn: &Connection,
    id: &str,
    key: &[u8; 32],
) -> Result<EnvVarItemDetail> {
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

    let notes: Option<String> = conn
        .query_row(
            "SELECT notes FROM env_var_items WHERE item_id = ?1 LIMIT 1",
            [id],
            |row| row.get(0),
        )
        .unwrap_or(None);

    let mut stmt = conn
        .prepare(
            "SELECT key, value_encrypted, value_nonce FROM env_var_items 
             WHERE item_id = ?1 ORDER BY sort_order",
        )
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    let variables = stmt
        .query_map([id], |row| {
            let key: String = row.get(0)?;
            let value_encrypted: Vec<u8> = row.get(1)?;
            let nonce_bytes: Vec<u8> = row.get(2)?;
            let mut nonce = [0u8; 12];
            nonce.copy_from_slice(&nonce_bytes);
            
            let value = crate::crypto::decrypt(key, &nonce, &value_encrypted)
                .map_err(|_| rusqlite::Error::InvalidQuery)?;
            
            Ok(EnvVarPair {
                key,
                value: String::from_utf8(value).map_err(|_| rusqlite::Error::InvalidQuery)?,
            })
        })
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    Ok(EnvVarItemDetail {
        id: item.0,
        group_id: item.1,
        title: item.2,
        icon: item.3,
        is_favorite: item.4,
        created_at: item.5,
        updated_at: item.6,
        variables,
        notes,
    })
}
```

- [ ] **Step 2: Add create_env_var_item function**

```rust
pub fn create_env_var_item(
    conn: &Connection,
    group_id: &str,
    title: &str,
    variables: &[(String, String)],
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono_timestamp();

    conn.execute(
        "INSERT INTO items (id, group_id, type, title, icon, is_favorite, created_at, updated_at) 
         VALUES (?1, ?2, 'env_var', ?3, NULL, 0, ?4, ?5)",
        params![id, group_id, title, now, now],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    for (i, (var_key, var_value)) in variables.iter().enumerate() {
        let encrypted = crate::crypto::encrypt(key, var_value.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "INSERT INTO env_var_items (item_id, key, value_encrypted, value_nonce, sort_order, notes) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, var_key, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), i as i32, notes],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(id)
}
```

- [ ] **Step 3: Add update_env_var_item function**

```rust
pub fn update_env_var_item(
    conn: &Connection,
    id: &str,
    title: &str,
    variables: &[(String, String)],
    notes: Option<&str>,
    key: &[u8; 32],
) -> Result<()> {
    let now = chrono_timestamp();

    conn.execute(
        "UPDATE items SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, now, id],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    // Delete existing variables
    conn.execute(
        "DELETE FROM env_var_items WHERE item_id = ?1",
        params![id],
    ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;

    // Insert new variables
    for (i, (var_key, var_value)) in variables.iter().enumerate() {
        let encrypted = crate::crypto::encrypt(key, var_value.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "INSERT INTO env_var_items (item_id, key, value_encrypted, value_nonce, sort_order, notes) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, var_key, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), i as i32, notes],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}
```

- [ ] **Step 4: Update get_all_items and get_items_by_group queries**

Add LEFT JOIN for env_var_items to show first key as subtitle:

```rust
// In get_all_items
"SELECT i.id, i.title, 
    COALESCE(a.username, ak.key_name, ev.key, '') as subtitle,
    i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at 
 FROM items i 
 LEFT JOIN account_items a ON i.id = a.item_id 
 LEFT JOIN api_key_items ak ON i.id = ak.item_id 
 LEFT JOIN env_var_items ev ON i.id = ev.item_id AND ev.sort_order = 0
 ORDER BY i.title"

// In get_items_by_group  
"SELECT i.id, i.title, 
    COALESCE(a.username, ak.key_name, ev.key, '') as subtitle,
    i.icon, i.type, i.is_favorite, i.group_id, i.created_at, i.updated_at 
 FROM items i 
 LEFT JOIN account_items a ON i.id = a.item_id 
 LEFT JOIN api_key_items ak ON i.id = ak.item_id 
 LEFT JOIN env_var_items ev ON i.id = ev.item_id AND ev.sort_order = 0
 WHERE i.group_id = ?1
 ORDER BY i.title"
```

- [ ] **Step 5: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 6: Commit backend operations**

```bash
git add src-tauri/src/db/operations.rs
git commit -m "feat(db): add env var structs, CRUD functions, and update item queries"
```

---

## Chunk 3: Backend Commands

### Task 3.1: Update get_item_detail for env var dispatch

**Files:**
- Modify: `src-tauri/src/commands/items.rs`

- [ ] **Step 1: Update imports**

Add `get_env_var_item_detail, create_env_var_item, update_env_var_item, EnvVarItemDetail` to imports.

- [ ] **Step 2: Update get_item_detail match arm**

```rust
match item_type.as_str() {
    "account" => get_account_item_detail(&conn, &id, key.as_bytes())
        .map(|d| ItemDetail::Account(d)),
    "api_key" => get_api_key_item_detail(&conn, &id, key.as_bytes())
        .map(|d| ItemDetail::ApiKey(d)),
    "env_var" => get_env_var_item_detail(&conn, &id, key.as_bytes())
        .map(|d| ItemDetail::EnvVar(d)),
    _ => Err(VaultError::ItemNotFound),
}
```

### Task 3.2: Add env var CRUD commands

**Files:**
- Modify: `src-tauri/src/commands/items.rs`

- [ ] **Step 1: Add EnvVarPair struct for IPC**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarPairInput {
    pub key: String,
    pub value: String,
}
```

- [ ] **Step 2: Add create_new_env_var_item command**

```rust
#[tauri::command]
pub fn create_new_env_var_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    group_id: String,
    title: String,
    variables: Vec<EnvVarPairInput>,
    notes: Option<String>,
) -> Result<String> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if variables.is_empty() {
        return Err(VaultError::DatabaseError("At least one variable is required".to_string()));
    }
    for var in &variables {
        if var.key.is_empty() || var.key.len() > 100 {
            return Err(VaultError::DatabaseError("Variable key must be 1-100 characters".to_string()));
        }
        if var.value.is_empty() || var.value.len() > 5000 {
            return Err(VaultError::DatabaseError("Variable value must be 1-5000 characters".to_string()));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError("Notes must be under 5000 characters".to_string()));
        }
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    let vars: Vec<(String, String)> = variables.iter().map(|v| (v.key.clone(), v.value.clone())).collect();
    
    create_env_var_item(
        &conn,
        &group_id,
        &title,
        &vars,
        notes.as_deref(),
        key.as_bytes(),
    )
}
```

- [ ] **Step 3: Add update_existing_env_var_item command**

```rust
#[tauri::command]
pub fn update_existing_env_var_item(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    id: String,
    title: String,
    variables: Vec<EnvVarPairInput>,
    notes: Option<String>,
) -> Result<()> {
    if !vault_state.is_unlocked() {
        return Err(VaultError::VaultLocked);
    }

    if title.is_empty() || title.len() > 100 {
        return Err(VaultError::DatabaseError("Title must be 1-100 characters".to_string()));
    }
    if variables.is_empty() {
        return Err(VaultError::DatabaseError("At least one variable is required".to_string()));
    }
    for var in &variables {
        if var.key.is_empty() || var.key.len() > 100 {
            return Err(VaultError::DatabaseError("Variable key must be 1-100 characters".to_string()));
        }
        if var.value.is_empty() || var.value.len() > 5000 {
            return Err(VaultError::DatabaseError("Variable value must be 1-5000 characters".to_string()));
        }
    }
    if let Some(ref n) = notes {
        if n.len() > 5000 {
            return Err(VaultError::DatabaseError("Notes must be under 5000 characters".to_string()));
        }
    }

    let key = vault_state.get_master_key().ok_or(VaultError::VaultLocked)?;
    let conn = db.0.lock().map_err(|_| VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    let vars: Vec<(String, String)> = variables.iter().map(|v| (v.key.clone(), v.value.clone())).collect();
    
    update_env_var_item(
        &conn,
        &id,
        &title,
        &vars,
        notes.as_deref(),
        key.as_bytes(),
    )
}
```

- [ ] **Step 4: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

### Task 3.3: Register new commands

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add to invoke_handler**

Add `create_new_env_var_item` and `update_existing_env_var_item` to the invoke_handler list.

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`
Expected: Compilation succeeds

- [ ] **Step 3: Commit backend commands**

```bash
git add src-tauri/src/commands/items.rs src-tauri/src/lib.rs
git commit -m "feat(cmd): add env var commands and type dispatch"
```

---

## Chunk 4: Frontend Types & IPC

### Task 4.1: Update TypeScript types

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Add EnvVarItemDetail**

```typescript
export interface EnvVarPair {
  key: string;
  value: string;
}

export interface EnvVarItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'env_var';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;
  variables: EnvVarPair[];
  notes?: string;
}

export type ItemDetail = AccountItemDetail | ApiKeyItemDetail | EnvVarItemDetail;
```

- [ ] **Step 2: Update ItemSummary type field**

```typescript
export interface ItemSummary {
  // ... existing fields
  type: 'account' | 'api_key' | 'env_var';
  // ... rest
}
```

- [ ] **Step 3: Add env var input types**

```typescript
export interface CreateEnvVarItemInput {
  group_id: string;
  title: string;
  variables: EnvVarPair[];
  notes?: string;
}

export interface UpdateEnvVarItemInput {
  id: string;
  title: string;
  variables: EnvVarPair[];
  notes?: string;
}
```

### Task 4.2: Add env var IPC functions

**Files:**
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Add IPC functions**

```typescript
export async function createNewEnvVarItem(
  groupId: string,
  title: string,
  variables: { key: string; value: string }[],
  notes?: string
): Promise<string> {
  return invoke<string>('create_new_env_var_item', {
    group_id: groupId,
    title,
    variables,
    notes
  });
}

export async function updateExistingEnvVarItem(
  id: string,
  title: string,
  variables: { key: string; value: string }[],
  notes?: string
): Promise<void> {
  return invoke<void>('update_existing_env_var_item', {
    id,
    title,
    variables,
    notes
  });
}
```

- [ ] **Step 2: Commit frontend types and IPC**

```bash
git add src/types/index.ts src/lib/tauri.ts
git commit -m "feat(frontend): add env var types and IPC functions"
```

---

## Chunk 5: Frontend Store & UI

### Task 5.1: Update vault store

**Files:**
- Modify: `src/stores/vault.ts`

- [ ] **Step 1: Add env var actions**

Add `createEnvVarItem` and `updateEnvVarItem` actions similar to API key actions.

### Task 5.2: Update AddEditItemModal

**Files:**
- Modify: `src/components/modals/AddEditItemModal.tsx`

- [ ] **Step 1: Add env var form state**

```typescript
const [envVarForm, setEnvVarForm] = useState({
  groupId: '',
  title: '',
  variables: [{ key: '', value: '' }],
  notes: '',
});
```

- [ ] **Step 2: Add env var fields rendering**

Key-value pair editor with add/remove buttons.

### Task 5.3: Update DetailPanel

**Files:**
- Modify: `src/components/layout/DetailPanel.tsx`

- [ ] **Step 1: Add env var rendering**

Table of key-value pairs with copy buttons.

- [ ] **Step 2: Commit UI components**

```bash
git add src/stores/vault.ts src/components/modals/AddEditItemModal.tsx src/components/layout/DetailPanel.tsx
git commit -m "feat(ui): add env var form fields and detail rendering"
```

---

## Chunk 6: Build & Verify

### Task 6.1: Build and test

- [ ] **Step 1: Verify frontend builds**

Run: `npm run build 2>&1`
Expected: Build succeeds

- [ ] **Step 2: Verify Rust builds**

Run: `cd src-tauri && cargo build 2>&1`
Expected: Compilation succeeds

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete Environment Variables implementation"
```

---

## Summary

This plan implements Environment Variables in 6 chunks:

1. **Database Migration** - Schema version 3, notes column
2. **Backend Operations** - Structs, CRUD functions, query updates
3. **Backend Commands** - Type dispatch, new commands
4. **Frontend Types & IPC** - TypeScript types, API functions
5. **Frontend Store & UI** - Zustand actions, form fields, detail rendering
6. **Integration Testing** - Build verification