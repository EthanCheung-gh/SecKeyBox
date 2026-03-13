# SecKeyBox Phase 2: Global Quick Search & Windows Hello

## Overview

Two remaining Phase 2 features:
1. **Global Quick Search** - System-wide hotkey (Ctrl+Shift+Space) to quickly search and copy credentials
2. **Windows Hello** - Biometric authentication for quick unlock

## Feature 1: Global Quick Search

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Windows Desktop                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Any Application                          │   │
│  │                                                       │   │
│  │         User presses Ctrl+Shift+Space                │   │
│  └───────────────────────┬─────────────────────────────┘   │
│                          │                                  │
│                          ▼                                  │
│  ┌─────────────────────────────────────────┐               │
│  │        Quick Search Popup (400x300)      │               │
│  │  ┌─────────────────────────────────────┐│               │
│  │  │ 🔍 Search items...                  ││               │
│  │  └─────────────────────────────────────┘│               │
│  │  ┌─────────────────────────────────────┐│               │
│  │  │ 🔑 Google        user@gmail.com  📋 ││               │
│  │  │ 🔧 OpenAI API    sk-xxx...       📋 ││               │
│  │  │ 📦 .env.local    DATABASE_URL... 📋 ││               │
│  │  └─────────────────────────────────────┘│               │
│  └─────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────┘
```

### Components

**Backend (Rust):**
- `tauri-plugin-global-shortcut` - Register system-wide hotkey
- New window: `quick-search` - Small popup, always-on-top
- Command: `search_items` - Fast search across all items

**Frontend (React):**
- `QuickSearchWindow.tsx` - Search input + results list
- Copy buttons for quick credential access
- Keyboard navigation (arrows, enter, escape)

### User Flow

1. User presses **Ctrl+Shift+Space** anywhere in Windows
2. Quick Search popup appears (centered, focused)
3. User types to filter items (searches title, subtitle, username)
4. Results show: icon, title, subtitle, copy button
5. Click item or press Enter → expand to show copyable fields
6. Click copy button → copies to clipboard → window closes
7. Press **Escape** or click outside → window closes

### Data Flow

```
User types query
       │
       ▼
search_items command (Rust)
       │
       ├── Queries SQLite (title, subtitle LIKE %query%)
       │
       ▼
Returns ItemSummary[] (no decryption needed)
       │
       ▼
Frontend renders list
       │
       ▼
User clicks "Copy password"
       │
       ▼
get_item_detail (decrypts)
       │
       ▼
copy_to_clipboard
       │
       ▼
Window closes
```

### Technical Details

**Window Configuration:**
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
  "center": true
}
```

**Global Shortcut:**
```rust
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

// Register Ctrl+Shift+Space
let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
app.global_shortcut().register(shortcut, |app, _shortcut| {
    // Toggle quick search window
});
```

---

## Feature 2: Windows Hello Integration

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Unlock Flow                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  User opens SecKeyBox                                       │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────────┐                                        │
│  │  Unlock Screen  │                                        │
│  │                 │                                        │
│  │  [Password...]  │                                        │
│  │                 │                                        │
│  │  🔐 Windows Hello │  ←─ New button                       │
│  │                 │                                        │
│  └────────┬────────┘                                        │
│           │                                                 │
│           ▼                                                 │
│  ┌─────────────────────────────────────┐                   │
│  │     Windows Hello Prompt            │                   │
│  │                                     │                   │
│  │  "Verify your identity to unlock    │                   │
│  │   SecKeyBox"                        │                   │
│  │                                     │                   │
│  │  [PIN] [Fingerprint] [Face]         │                   │
│  └────────────────┬────────────────────┘                   │
│                   │                                         │
│                   ▼                                         │
│           Decrypt stored master password                    │
│                   │                                         │
│                   ▼                                         │
│           Auto-unlock vault                                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Setup Flow

```
After first successful unlock:
       │
       ▼
Prompt: "Enable Windows Hello for quick unlock?"
       │
       ├── No → Skip, can enable later in Settings
       │
       └── Yes
           │
           ▼
     Windows Hello prompt
           │
           ▼
     Encrypt master password with biometric key
           │
           ▼
     Store in Windows Credential Manager
           │
           ▼
     "Windows Hello enabled!"
```

### Components

**Backend (Rust):**
- `tauri-plugin-biometric` - Windows Hello authentication
- Command: `setup_biometric_unlock` - Store encrypted password
- Command: `unlock_with_biometric` - Authenticate and retrieve

**Frontend (React):**
- Modify `UnlockScreen.tsx` - Add Windows Hello button
- Settings option to enable/disable

### Security Model

**Storage:**
- Master password is encrypted with a key derived from Windows Hello
- Stored in Windows Credential Manager (protected by OS)
- App cannot access without successful biometric auth

**Threat Model:**
- **Protected:** Password cannot be extracted without biometric
- **Protected:** Memory is cleared on lock
- **Risk:** If Windows account is compromised, attacker could use Hello

### Technical Details

**Biometric Prompt:**
```rust
use tauri_plugin_biometric::Biometric;

let biometric = Biometric::new(app_handle);
let result = biometric.authenticate("Unlock SecKeyBox").await?;
```

**Credential Storage:**
```rust
// Encrypt master password with biometric-derived key
let encrypted = encrypt_with_biometric_key(master_password);

// Store in credential manager
keyring::Entry::new("SecKeyBox", "biometric_key").set_password(&encrypted)?;
```

---

## Implementation Order

1. **Global Quick Search** - ✅ COMPLETE - Works on all platforms
2. **Windows Hello** - ⏸️ DEFERRED - tauri-plugin-biometric only supports Android/iOS, not Windows

## Dependencies

**Cargo.toml additions:**
```toml
tauri-plugin-global-shortcut = "2"
```

**Note:** Windows Hello would require direct Win32 API integration via the `windows` crate, which is deferred for future implementation.

## UI/UX Considerations

**Quick Search:**
- Fast: Results appear within 100ms
- Keyboard-first: Tab, arrows, enter work naturally
- Minimal: Only essential info shown
- Auto-close: Disappears after copy or 30s inactivity

**Windows Hello:**
- Visible: Button always shown when available
- Fallback: Password field always available
- Clear status: "Windows Hello not set up" vs "Click to unlock"

## Error Handling

| Scenario | Handling |
|----------|----------|
| Biometric not available | Hide Windows Hello button |
| Biometric setup fails | Show error, fall back to password |
| Quick search finds nothing | Show "No items found" message |
| Copy fails | Show toast notification |