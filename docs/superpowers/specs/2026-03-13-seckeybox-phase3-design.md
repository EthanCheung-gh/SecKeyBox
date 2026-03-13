# SecKeyBox Phase 3: Advanced Features Design

## Overview

Phase 3 adds four advanced features:
1. **Favorites** - Mark items as favorite with dedicated filter view
2. **Dark/Light Theme** - Theme toggle with system preference detection
3. **Password Strength** - Visual indicator when creating/editing passwords
4. **Import/Export** - Vault backup and restore functionality

## Feature 1: Favorites

### Current State
- `is_favorite` column exists in `items` table ✅
- `toggle_favorite` command exists ✅
- `toggle_item_favorite` IPC function exists ✅
- **Missing:** UI for favorites filter and toggle button

### Implementation

**UI Changes:**
1. Add ⭐ "Favorites" entry in Sidebar (below "All Items")
2. Add star toggle button in DetailPanel header
3. Filter items when "Favorites" is selected

**Backend:** No changes needed - all infrastructure exists.

### User Flow
```
Sidebar
├── 📁 All Items
├── ⭐ Favorites        ← NEW
├── ─────────
├── 🔑 Accounts
├── 🔧 API Keys
├── 📦 Environment Variables
```

---

## Feature 2: Dark/Light Theme

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Theme System                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Settings Store (Zustand)                                   │
│  ├── theme: 'light' | 'dark' | 'system'                    │
│  └── setTheme(theme)                                        │
│                                                             │
│  Tailwind CSS                                               │
│  ├── dark: variant for dark mode                            │
│  └── CSS variables for theming                              │
│                                                             │
│  Persistence                                                │
│  └── localStorage: 'seckeybox-theme'                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Implementation

**Files to create/modify:**
1. Create `src/stores/settings.ts` - Theme state
2. Modify `tailwind.config.js` - Dark mode config
3. Modify `src/index.css` - CSS variables
4. Create `src/components/ui/theme-toggle.tsx` - Toggle component
5. Add settings button to Sidebar

### Color Tokens

**Light Theme:**
```css
--bg-primary: #ffffff
--bg-secondary: #f9fafb
--text-primary: #111827
--text-secondary: #6b7280
--border: #e5e7eb
```

**Dark Theme:**
```css
--bg-primary: #1f2937
--bg-secondary: #111827
--text-primary: #f9fafb
--text-secondary: #9ca3af
--border: #374151
```

---

## Feature 3: Password Strength

### Algorithm

Use zxcvbn-lite or custom scoring:
- **Weak:** < 40 points (red)
- **Fair:** 40-60 points (orange)
- **Good:** 60-80 points (yellow)
- **Strong:** > 80 points (green)

### Scoring Criteria
```typescript
function calculateStrength(password: string): number {
  let score = 0;
  
  // Length
  score += Math.min(password.length * 4, 32);
  
  // Character variety
  if (/[a-z]/.test(password)) score += 5;
  if (/[A-Z]/.test(password)) score += 5;
  if (/[0-9]/.test(password)) score += 5;
  if (/[^a-zA-Z0-9]/.test(password)) score += 10;
  
  // Penalty for common patterns
  if /(.)\1{2,}/.test(password)) score -= 10; // Repeated chars
  if /^[a-z]+$/.test(password)) score -= 10;   // Only lowercase
  if /^[0-9]+$/.test(password)) score -= 15;   // Only numbers
  
  return Math.max(0, Math.min(100, score));
}
```

### UI Component
```
┌─────────────────────────────────────────┐
│ Password: [________________]            │
│ Strength: ████████░░░░ Good             │
│          ████████░░░░                   │
│          ↑ Colored bar                  │
└─────────────────────────────────────────┘
```

---

## Feature 4: Import/Export

### Export Format

JSON with encrypted data:
```json
{
  "version": "1.0",
  "exported_at": "2026-03-13T12:00:00Z",
  "app": "SecKeyBox",
  "data": {
    "groups": [...],
    "items": [
      {
        "id": "...",
        "type": "account",
        "title": "...",
        "username": "...",
        "password_encrypted": "base64...",
        "password_nonce": "base64..."
      }
    ]
  }
}
```

**Key Points:**
- Data remains encrypted with master password
- Import requires same master password to decrypt
- Groups and items exported together

### Import Flow

```
User clicks Import
       │
       ▼
Select file (.json)
       │
       ▼
Prompt: "Enter master password used for this export"
       │
       ▼
Decrypt and validate structure
       │
       ├── Invalid password → Error
       │
       └── Valid
           │
           ▼
       Preview: "Found X items, Y groups. Merge or Replace?"
           │
           ├── Merge → Add new, skip duplicates
           │
           └── Replace → Clear existing, import all
```

### Implementation

**Backend Commands:**
```rust
#[tauri::command]
fn export_vault(db: State<DbConnection>, vault_state: State<VaultState>) -> Result<String>;

#[tauri::command]
fn import_vault(
    db: State<DbConnection>,
    vault_state: State<VaultState>,
    data: String,
    mode: "merge" | "replace"
) -> Result<ImportResult>;
```

**Frontend:**
- File picker dialog
- Progress indicator
- Import summary

---

## Implementation Order

| Feature | Priority | Effort | Dependencies |
|---------|----------|--------|--------------|
| Favorites | 1 | Low | None |
| Dark/Light Theme | 2 | Medium | None |
| Password Strength | 3 | Low | None |
| Import/Export | 4 | High | None |

---

## Technical Notes

### Favorites
- No backend changes required
- Filter happens in frontend (ItemList component)

### Theme
- Use Tailwind's `dark:` variant
- Store preference in localStorage
- Detect system preference with `window.matchMedia('(prefers-color-scheme: dark)')`

### Password Strength
- Pure frontend calculation
- No network calls needed
- Real-time feedback on input

### Import/Export
- Use Tauri's file dialog API
- Large exports should stream (for future scalability)
- Validate JSON structure before processing