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
│  ├── tauri-plugin-biometric (Windows Hello - Phase 2)       │
│  ├── tauri-plugin-clipboard (Secure clipboard)              │
│  └── tauri-plugin-global-shortcut (Quick search - Phase 2) │
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
| Brute-force Protection | Argon2id slow hash + attempt limiting |

### Windows Hello Integration (Phase 2)

- Master password stored once, protected by Windows Hello
- Hello verification auto-unlocks without re-entering password

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
│   │       ├── AddItemModal.tsx
│   │       └── EditGroupModal.tsx
│   ├── pages/
│   │   └── Main.tsx
│   ├── stores/                   # Zustand state
│   │   ├── vault.ts              # Vault state
│   │   ├── settings.ts           # Settings state
│   │   └── ui.ts                 # UI state
│   ├── lib/
│   │   ├── tauri.ts              # Tauri IPC wrapper
│   │   └── crypto.ts             # Frontend crypto helpers
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

## Development Phases

### Phase 1: Core MVP

- Unlock screen with master password
- Three-column layout structure
- Account/password CRUD operations
- Basic group management
- SQLite encrypted storage

### Phase 2: Enhanced Features

- Environment variable management (Key-Value pairs)
- API Key management (long text support)
- Global quick search popup (Ctrl+Shift+Space)
- Windows Hello integration

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