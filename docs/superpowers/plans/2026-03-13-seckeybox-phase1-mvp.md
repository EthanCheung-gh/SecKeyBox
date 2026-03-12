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