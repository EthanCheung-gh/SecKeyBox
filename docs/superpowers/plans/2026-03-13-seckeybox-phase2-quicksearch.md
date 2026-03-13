# SecKeyBox Phase 2: Global Quick Search Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add system-wide quick search with Ctrl+Shift+Space hotkey for fast credential access.

**Architecture:** New popup window with search input, reuses existing search logic, provides quick-copy functionality.

**Tech Stack:** Tauri 2.x, Rust, React 18, TypeScript, tauri-plugin-global-shortcut

---

## File Structure

```
src-tauri/
├── Cargo.toml              # MODIFY: Add tauri-plugin-global-shortcut
├── tauri.conf.json         # MODIFY: Add quick-search window config
└── src/
    ├── lib.rs              # MODIFY: Register global shortcut, new window
    └── commands/
        └── quick_search.rs # NEW: search_items, toggle_quick_search

src/
├── main.tsx                # MODIFY: Add quick-search route
└── components/
    └── quick-search/
        └── QuickSearchWindow.tsx  # NEW: Search UI component
```

---

## Chunk 1: Backend Setup

### Task 1.1: Add dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add tauri-plugin-global-shortcut**

```toml
tauri-plugin-global-shortcut = "2"
```

- [ ] **Step 2: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`

### Task 1.2: Add quick-search window config

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add window to app.windows array**

```json
{
  "label": "quick-search",
  "title": "Quick Search",
  "width": 400,
  "height": 300,
  "resizable": false,
  "decorations": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "center": true,
  "visible": false
}
```

### Task 1.3: Create quick_search commands

**Files:**
- Create: `src-tauri/src/commands/quick_search.rs`

- [ ] **Step 1: Create search_items command**

```rust
use crate::db::{DbConnection, ItemSummary};
use crate::error::Result;
use crate::state::VaultState;
use tauri::State;

#[tauri::command]
pub fn search_items(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    query: String,
) -> Result<Vec<ItemSummary>> {
    if !vault_state.is_unlocked() {
        return Ok(vec![]);
    }

    let conn = db.0.lock()
        .map_err(|_| crate::error::VaultError::DatabaseError("Lock poisoned".to_string()))?;
    
    let all_items = crate::db::get_all_items(&conn)?;
    
    if query.is_empty() {
        return Ok(all_items);
    }

    let query_lower = query.to_lowercase();
    let filtered: Vec<ItemSummary> = all_items
        .into_iter()
        .filter(|item| {
            item.title.to_lowercase().contains(&query_lower) ||
            item.subtitle.to_lowercase().contains(&query_lower)
        })
        .collect();

    Ok(filtered)
}
```

- [ ] **Step 2: Create commands mod.rs export**

Create `src-tauri/src/commands/mod.rs`:
```rust
pub mod vault;
pub mod items;
pub mod clipboard;
pub mod quick_search;

pub use vault::*;
pub use items::*;
pub use clipboard::*;
pub use quick_search::*;
```

- [ ] **Step 3: Update lib.rs to use commands module**

```rust
mod commands;

pub use commands::*;
```

- [ ] **Step 4: Register search_items command**

Add to invoke_handler in lib.rs.

- [ ] **Step 5: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`

- [ ] **Step 6: Commit backend setup**

```bash
git add src-tauri/
git commit -m "feat(backend): add quick search commands and window config"
```

---

## Chunk 2: Global Shortcut Registration

### Task 2.1: Register global shortcut

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add global shortcut plugin**

```rust
.plugin(tauri_plugin_global_shortcut::Builder::new().build())
```

- [ ] **Step 2: Register Ctrl+Shift+Space shortcut in setup**

```rust
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

.setup(|app| {
    // ... existing setup code ...
    
    // Register global shortcut
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
    let app_handle = app.handle().clone();
    
    app.global_shortcut().register(shortcut, move |_app, _shortcut| {
        if let Some(window) = app_handle.get_webview_window("quick-search") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }).expect("Failed to register global shortcut");
    
    Ok(())
})
```

- [ ] **Step 3: Verify Rust compiles**

Run: `cd src-tauri && cargo check 2>&1`

- [ ] **Step 4: Commit global shortcut**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(shortcut): register Ctrl+Shift+Space global hotkey"
```

---

## Chunk 3: Frontend Quick Search Component

### Task 3.1: Create QuickSearchWindow component

**Files:**
- Create: `src/components/quick-search/QuickSearchWindow.tsx`

- [ ] **Step 1: Create component with search input and results**

```typescript
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { copyToClipboard } from '@/lib/tauri';
import type { ItemSummary, ItemDetail } from '@/types';

export function QuickSearchWindow() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<ItemSummary[]>([]);
  const [selectedItem, setSelectedItem] = useState<ItemDetail | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    
    // Listen for escape key
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        getCurrentWindow().hide();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  useEffect(() => {
    const search = async () => {
      try {
        const items = await invoke<ItemSummary[]>('search_items', { query });
        setResults(items);
      } catch (e) {
        console.error('Search failed:', e);
      }
    };
    
    const debounce = setTimeout(search, 100);
    return () => clearTimeout(debounce);
  }, [query]);

  const handleCopy = async (text: string) => {
    await copyToClipboard(text);
    getCurrentWindow().hide();
  };

  const handleSelectItem = async (id: string) => {
    try {
      const detail = await invoke<ItemDetail>('get_item_detail', { id });
      setSelectedItem(detail);
    } catch (e) {
      console.error('Failed to get item detail:', e);
    }
  };

  const renderCopyButtons = () => {
    if (!selectedItem) return null;
    
    switch (selectedItem.type) {
      case 'account':
        return (
          <div className="space-y-2 p-2">
            <button onClick={() => handleCopy(selectedItem.username)} 
              className="w-full rounded bg-gray-100 p-2 text-left hover:bg-gray-200">
              Copy username: {selectedItem.username}
            </button>
            <button onClick={() => handleCopy(selectedItem.password)}
              className="w-full rounded bg-gray-100 p-2 text-left hover:bg-gray-200">
              Copy password: ••••••••
            </button>
          </div>
        );
      case 'api_key':
        return (
          <div className="space-y-2 p-2">
            <button onClick={() => handleCopy(selectedItem.key_value)}
              className="w-full rounded bg-gray-100 p-2 text-left hover:bg-gray-200">
              Copy API key: ••••{selectedItem.key_value.slice(-4)}
            </button>
          </div>
        );
      case 'env_var':
        return (
          <div className="space-y-2 p-2">
            {selectedItem.variables.map((v, i) => (
              <button key={i} onClick={() => handleCopy(v.value)}
                className="w-full rounded bg-gray-100 p-2 text-left hover:bg-gray-200">
                Copy {v.key}: ••••••••
              </button>
            ))}
          </div>
        );
    }
  };

  return (
    <div className="h-full bg-white p-2">
      <input
        ref={inputRef}
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search items..."
        className="w-full rounded border border-gray-300 px-3 py-2"
      />
      
      {selectedItem ? (
        <div className="mt-2">
          <button 
            onClick={() => setSelectedItem(null)}
            className="text-sm text-gray-500 hover:text-gray-700"
          >
            ← Back to results
          </button>
          {renderCopyButtons()}
        </div>
      ) : (
        <div className="mt-2 max-h-[220px] overflow-y-auto">
          {results.length === 0 ? (
            <p className="p-2 text-center text-gray-400">No items found</p>
          ) : (
            results.map((item) => (
              <button
                key={item.id}
                onClick={() => handleSelectItem(item.id)}
                className="w-full rounded p-2 text-left hover:bg-gray-100"
              >
                <div className="font-medium">{item.title}</div>
                <div className="text-sm text-gray-500">{item.subtitle}</div>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
```

### Task 3.2: Add quick-search route

**Files:**
- Modify: `src/main.tsx` or create `src/quick-search.tsx`

- [ ] **Step 1: Create quick-search entry point**

Create `src/quick-search.tsx`:
```typescript
import React from 'react';
import ReactDOM from 'react-dom/client';
import { QuickSearchWindow } from './components/quick-search/QuickSearchWindow';
import './index.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QuickSearchWindow />
  </React.StrictMode>
);
```

- [ ] **Step 2: Update Vite config for multi-page**

Modify `vite.config.ts`:
```typescript
export default defineConfig({
  // ... existing config
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        'quick-search': resolve(__dirname, 'quick-search.html'),
      },
    },
  },
});
```

- [ ] **Step 3: Create quick-search.html**

Create `quick-search.html` in project root:
```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>Quick Search</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/quick-search.tsx"></script>
  </body>
</html>
```

- [ ] **Step 4: Update tauri.conf.json window URL**

```json
{
  "label": "quick-search",
  "url": "quick-search.html"
  // ... rest of config
}
```

- [ ] **Step 5: Verify frontend builds**

Run: `npm run build 2>&1`

- [ ] **Step 6: Commit frontend component**

```bash
git add src/components/quick-search/ src/quick-search.tsx quick-search.html vite.config.ts
git commit -m "feat(ui): add quick search popup component"
```

---

## Chunk 4: Integration Testing

### Task 4.1: Build and test

- [ ] **Step 1: Build the application**

Run: `npm run tauri build 2>&1`

- [ ] **Step 2: Manual testing**

1. Run the built application
2. Unlock the vault
3. Press Ctrl+Shift+Space
4. Verify popup appears
5. Type search query
6. Click item, verify copy buttons appear
7. Click copy, verify clipboard and window closes
8. Press Escape, verify window closes

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete Global Quick Search implementation"
```

---

## Summary

This plan implements Global Quick Search in 4 chunks:

1. **Backend Setup** - Dependencies, window config, search command
2. **Global Shortcut** - Register Ctrl+Shift+Space hotkey
3. **Frontend Component** - Search UI with copy functionality
4. **Integration Testing** - Build and verify