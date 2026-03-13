# SecKeyBox Phase 2: API Key Management Design

## Overview

Add support for managing API keys and long-form secrets (OAuth tokens, JWT keys, private keys). This extends Phase 1's account management pattern to a new item type with specialized fields and UI considerations.

## Requirements

| Requirement | Details |
|-------------|---------|
| Key length support | Up to 50,000 characters |
| Display masking | Partial - show last 4 chars (e.g., "sk-...xyz") |
| Metadata fields | Endpoint URL, Auth method, Rotation date |
| Notes | Optional, up to 5,000 characters |

## Data Model

### TypeScript Types

```typescript
// Updated ItemSummary type for type discrimination
interface ItemSummary {
  id: string;
  title: string;
  subtitle: string;
  icon?: string;
  type: 'account' | 'api_key';  // Now supports multiple types
  is_favorite: boolean;
  group_id: string;
  created_at: number;
  updated_at: number;
}

// Union type for detail views
type ItemDetail = AccountItemDetail | ApiKeyItemDetail;

interface AccountItemDetail {
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

interface ApiKeyItemDetail {
  id: string;
  group_id: string;
  title: string;
  icon?: string;
  type: 'api_key';
  is_favorite: boolean;
  created_at: number;
  updated_at: number;

  // API Key specific fields
  key_name: string;           // Display name, e.g., "OpenAI API Key"
  key_value: string;          // The actual secret (decrypted)
  endpoint?: string;          // API base URL
  auth_method?: string;       // "bearer" | "api_key" | "basic" | "custom"
  rotation_date?: number;     // Unix timestamp
  notes?: string;
}

interface CreateApiKeyItemInput {
  group_id: string;
  title: string;
  key_name: string;
  key_value: string;
  endpoint?: string;
  auth_method?: string;
  notes?: string;
}

interface UpdateApiKeyItemInput {
  id: string;
  title: string;
  key_name: string;
  key_value?: string;         // Optional - if omitted, keep existing
  endpoint?: string;
  auth_method?: string;
  notes?: string;
}
```

### Database Schema

The `api_key_items` table already exists from Phase 1:

```sql
CREATE TABLE api_key_items (
    item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    key_name TEXT NOT NULL,
    key_value_encrypted BLOB NOT NULL,
    key_value_nonce BLOB NOT NULL,
    endpoint TEXT,
    notes TEXT
);
```

**Migration Required:** Add `auth_method` and `rotation_date` columns:

```sql
ALTER TABLE api_key_items ADD COLUMN auth_method TEXT;
ALTER TABLE api_key_items ADD COLUMN rotation_date INTEGER;
```

### Migration Strategy

**Schema Version:** Bump from `1` to `2`

In `src-tauri/src/db/schema.rs`:

```rust
const SCHEMA_VERSION: i32 = 2;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    let version = get_schema_version(conn)?;
    
    // Version 0: No schema - create everything
    if version < 1 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS groups (...);
            CREATE TABLE IF NOT EXISTS items (...);
            CREATE TABLE IF NOT EXISTS account_items (...);
            CREATE TABLE IF NOT EXISTS api_key_items (...);
            CREATE TABLE IF NOT EXISTS vault_config (...);
            -- indexes etc.
            "#
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    
    // Version 1 -> 2: Add API key columns
    if version < 2 {
        conn.execute_batch(
            "ALTER TABLE api_key_items ADD COLUMN auth_method TEXT;
             ALTER TABLE api_key_items ADD COLUMN rotation_date INTEGER;"
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }
    
    set_schema_version(conn, SCHEMA_VERSION)?;
    Ok(())
}

// Remove init_schema - replaced by run_migrations
pub fn init_schema(conn: &Connection) -> Result<()> {
    run_migrations(conn)
}
```

**Migration Invocation Points:**

| Location | When | Why |
|----------|------|-----|
| `initialize_vault` | First-run setup | Creates all tables |
| `unlock_vault` | Every unlock | Runs any pending migrations |

This ensures existing vaults get the new columns when they unlock after updating the app.

**Caller Changes Required:** Existing `init_schema` callers remain unchanged. Add a new call to `run_migrations` in `unlock_vault` (after opening the connection, before any queries).

### Field Validation Limits

| Field | Max Length | Required |
|-------|------------|----------|
| key_name | 100 chars | Yes |
| key_value | 50,000 chars | Yes |
| endpoint | 500 chars | No |
| auth_method | 20 chars | No |
| notes | 5,000 chars | No |

## Backend Changes

### New Rust Structs

```rust
// In src-tauri/src/db/operations.rs

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
    #[serde(rename = "type")]
    pub item_type: String,  // Always "api_key"
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

### Rust Commands

**Decision: Unified get_item_detail endpoint with type dispatch.** This matches the existing pattern and simplifies the frontend (single call regardless of type).

```rust
// Modified existing command in src-tauri/src/commands/items.rs

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
    
    // First get the item type
    let item_type: String = conn.query_row(
        "SELECT type FROM items WHERE id = ?1",
        [&id],
        |row| row.get(0),
    ).map_err(|_| VaultError::ItemNotFound)?;
    
    // key is SecureKey which has as_bytes() -> &[u8; 32]
    match item_type.as_str() {
        "account" => get_account_item_detail(&conn, &id, key.as_bytes())
            .map(|d| ItemDetail::Account(d)),
        "api_key" => get_api_key_item_detail(&conn, &id, key.as_bytes())
            .map(|d| ItemDetail::ApiKey(d)),
        _ => Err(VaultError::ItemNotFound),
    }
}

// New commands for API key CRUD

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

    // Validation
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

    // Validation
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

### Database Operations

```rust
// In src-tauri/src/db/operations.rs

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
        item_type: "api_key".to_string(),
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
        // Key value changed - encrypt and update rotation_date
        let encrypted = crate::crypto::encrypt(key, kv.as_bytes())
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        conn.execute(
            "UPDATE api_key_items SET key_name = ?1, key_value_encrypted = ?2, key_value_nonce = ?3, endpoint = ?4, auth_method = ?5, notes = ?6, rotation_date = ?7 WHERE item_id = ?8",
            params![key_name, encrypted.ciphertext.to_vec(), encrypted.nonce.to_vec(), endpoint, auth_method, notes, now, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    } else {
        // Key value unchanged - don't update rotation_date
        conn.execute(
            "UPDATE api_key_items SET key_name = ?1, endpoint = ?2, auth_method = ?3, notes = ?4 WHERE item_id = ?5",
            params![key_name, endpoint, auth_method, notes, id],
        ).map_err(|e| VaultError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}
```

### Item List Query Update

Modify both `get_all_items` and `get_items_by_group` to include API key info:

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
    // ... rest unchanged
}

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
    // ... rest unchanged, bind [group_id] instead of []
}
```

**Subtitle Clarification:** 
- For accounts: shows `username`
- For API keys: shows `key_name` (e.g., "OpenAI Production")
- Partial masking applies only to `key_value` display in DetailPanel, not subtitle

### Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/src/db/schema.rs` | Add `run_migrations`, bump `SCHEMA_VERSION` to 2 |
| `src-tauri/src/db/operations.rs` | Add `ApiKeyItemDetail`, `ItemDetail` enum, CRUD functions |
| `src-tauri/src/commands/items.rs` | Modify `get_item_detail`, add API key commands |
| `src-tauri/src/lib.rs` | Register new commands in `generate_handler!` |

**Note on Delete:** The existing `delete_existing_item` command handles API key deletion via CASCADE. No new delete command needed.

## Frontend Changes

### TypeScript Types Update

```typescript
// src/types/index.ts

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

export type ItemDetail = AccountItemDetail | ApiKeyItemDetail;

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
```

### Frontend IPC Wrapper

```typescript
// src/lib/tauri.ts additions

export async function createNewApiKeyItem(
  groupId: string,
  title: string,
  keyName: string,
  keyValue: string,
  endpoint?: string,
  authMethod?: string,
  notes?: string
) {
  // Note: Tauri expects snake_case parameter names
  return invoke('create_new_api_key_item', { 
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
) {
  // Note: Tauri expects snake_case parameter names
  return invoke('update_existing_api_key_item', { 
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

### Zustand Store Updates

```typescript
// src/stores/vault.ts additions

// Update selectedItem type
selectedItem: AccountItemDetail | ApiKeyItemDetail | null;

// New actions
createApiKeyItem: async (data: {
  groupId: string;
  title: string;
  keyName: string;
  keyValue: string;
  endpoint?: string;
  authMethod?: string;
  notes?: string;
}) => {
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

updateApiKeyItem: async (id: string, data: {
  title: string;
  keyName: string;
  keyValue?: string;
  endpoint?: string;
  authMethod?: string;
  notes?: string;
}) => {
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

### UI State for Type Selector

```typescript
// src/stores/ui.ts addition

interface UIState {
  // ... existing fields
  newItemType: 'account' | 'api_key';
  
  setNewItemType: (type: 'account' | 'api_key') => void;
}

// In implementation:
newItemType: 'account',
setNewItemType: (type) => set({ newItemType: type }),
```

### New Files

None. The existing `AddEditItemModal.tsx` will be extended with a type selector.

### Modified Files

| File | Changes |
|------|---------|
| `src/types/index.ts` | Add `ApiKeyItemDetail`, update `ItemDetail` union |
| `src/lib/tauri.ts` | Add API key IPC functions |
| `src/stores/vault.ts` | Add `createApiKeyItem`, `updateApiKeyItem`, update `selectedItem` type |
| `src/stores/ui.ts` | Add `newItemType` state and setter |
| `src/components/modals/AddEditItemModal.tsx` | Add type selector, API key fields (conditional rendering) |
| `src/components/layout/DetailPanel.tsx` | Add API key rendering branch |

**Modal Design Decision:** Single `AddEditItemModal` with type selector dropdown. When creating a new item, the user selects "Account" or "API Key" and the form shows appropriate fields. When editing an existing item, the type is locked (derived from `selectedItem.type`).

### UI Components

#### Type Selector in Add Modal

When adding a new item, show type selector at top:
- Account (default)
- API Key

Switching type resets form fields.

#### API Key Form Fields

```
┌─────────────────────────────────────────┐
│ Type: [API Key ▼]                       │
├─────────────────────────────────────────┤
│ Title *                                 │
│ [________________]                      │
│                                         │
│ Group                                   │
│ [API Keys ▼]                            │
│                                         │
│ Key Name *                              │
│ [________________]                      │
│                                         │
│ Key Value *                             │
│ ┌─────────────────────────────────────┐ │
│ │                                     │ │
│ │  (textarea, 6 rows)                 │ │
│ │                                     │ │
│ └─────────────────────────────────────┘ │
│ [👁 Show]                               │
│                                         │
│ Endpoint URL                            │
│ [________________]                      │
│                                         │
│ Auth Method                             │
│ [Bearer ▼]                              │
│                                         │
│ Notes                                   │
│ ┌─────────────────────────────────────┐ │
│ │                                     │ │
│ └─────────────────────────────────────┘ │
│                                         │
│           [Cancel]  [Add]               │
└─────────────────────────────────────────┘
```

#### Auth Method Options

- **Bearer** - Standard Bearer token in Authorization header
- **API Key Header** - Custom header like `X-API-Key`
- **Basic** - HTTP Basic Authentication
- **Custom** - User-defined method (shown in notes)

#### Detail Panel - API Key View

```
┌────────────────────────────────────────────┐
│ 🔧 OpenAI Production          [Edit] [Del] │
├────────────────────────────────────────────┤
│ Key Name                                   │
│ OpenAI Production                    [📋]  │
│                                            │
│ Key Value                                  │
│ ••••3xyz                            [👁][📋]│
│                                            │
│ Endpoint                                   │
│ https://api.openai.com/v1           [📋]  │
│                                            │
│ Auth Method                                │
│ Bearer                              [📋]  │
│                                            │
│ Last Rotated                               │
│ 14 days ago                                │
│                                            │
│ Notes                                      │
│ Used for GPT-4 production calls            │
└────────────────────────────────────────────┘
```

### Partial Masking Logic

**Strategy:** Simple masking that shows only the last 4 characters with bullet prefix. Does NOT preserve any prefix pattern.

```typescript
// src/lib/utils.ts addition

export function maskKeyValue(keyValue: string): string {
  if (keyValue.length <= 4) return '••••';
  const lastFour = keyValue.slice(-4);
  return `••••${lastFour}`;
  // Example: "sk-proj-abc123xyz" → "••••3xyz"
}

export function formatRotationDate(timestamp: number | undefined): string {
  if (!timestamp) return 'Never rotated';
  const days = Math.floor((Date.now() / 1000 - timestamp) / 86400);
  if (days === 0) return 'Rotated today';
  if (days === 1) return 'Rotated yesterday';
  return `Rotated ${days} days ago`;
}
```

Applied in:
- Detail panel key_value display (masked by default, toggle to reveal)
- Never applied to subtitle (subtitle shows `key_name`)

## Implementation Order

1. **Database Migration** - Add `run_migrations`, update `init_schema` to wrap it, bump schema version to 2
2. **Backend Operations** - Add Rust structs and CRUD functions
3. **Backend Commands** - Modify `get_item_detail`, add API key commands
4. **Frontend Types & IPC** - Add TypeScript types and API wrapper
5. **Frontend Store** - Add Zustand actions, update selectedItem type
6. **UI State** - Add newItemType state
7. **AddEditItemModal** - Add type selector and conditional API key fields
8. **Detail Panel** - Add API key rendering branch
9. **Integration** - Wire everything together, test

## Testing Checklist

- [ ] Run migration on existing database (schema version 1 → 2)
- [ ] Create API key with all fields
- [ ] Create API key with minimum fields (title, key_name, key_value only)
- [ ] Update API key without changing key_value (rotation_date unchanged)
- [ ] Update API key with new key_value (rotation_date updates)
- [ ] Copy key_value to clipboard
- [ ] Toggle key_value visibility
- [ ] Delete API key
- [ ] Verify subtitle shows key_name (not masked)
- [ ] Verify partial masking in detail panel
- [ ] Verify 50,000 character key_value works
- [ ] Verify endpoint URL is clickable link
- [ ] Verify lock clears sensitive data
- [ ] Verify type selector switches between Account and API Key
- [ ] Verify `get_item_detail` returns correct type for both item types