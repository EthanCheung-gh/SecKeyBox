<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="SecKeyBox logo">
</p>

<h1 align="center">SecKeyBox</h1>

<p align="center">
  A local-first password manager built for developers.
</p>

<p align="center">
  <a href="https://github.com/EthanCheung-gh/SecKeyBox/releases"><img src="https://img.shields.io/badge/release-v0.6.2-blue?style=flat-square" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square" alt="Platforms">
  <a href="license"><img src="https://img.shields.io/badge/license-Apache--2.0-green?style=flat-square" alt="License"></a>
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a>
</p>

---

## Why SecKeyBox

Developer credentials do not fit the "website + password" model of classic password managers. SecKeyBox ships with eight purpose-built entry types — from database connections and SSH servers to cloud access keys, license keys and SMTP accounts — stores everything in a local encrypted SQLite database, and never talks to a server.

**The installer ships with zero built-in credentials.** First launch walks you through creating a vault with your own master password. No accounts, no cloud, no telemetry.

## Screenshots

| Vault | Security dashboard |
|-------|--------------------|
| ![Main window](docs/screenshots/main.png) | ![Security dashboard](docs/screenshots/security.png) |

## Features

| Area | What you get |
|------|--------------|
| 8 credential types | Account, API Key, Environment Variables, Database Connection, SSH Server, Cloud Credential, License, SMTP Account |
| Purpose-built fields | e.g. Database: type/host/port/DB name/user/password/connection URL; SSH: host/port/user/password/key path/key passphrase |
| Encryption | Per-field AES-256-GCM with a random nonce; key derived from your master password with Argon2id (m=64 MB, t=3, p=4) |
| Master password change | Verifies the current password, derives a fresh key, and re-encrypts **every** stored secret in a single transaction — a failure rolls back and leaves the vault readable |
| Group management | Built-in group per credential type; every group (built-in included) can be renamed or deleted — deleting a group moves its items to the trash first, then removes the group |
| Search | Sidebar search filters title and subtitle across the item list in real time (backend SQL LIKE); `Ctrl+Shift+Space` opens a system-wide quick-search palette — arrows to select, Enter copies the secret |
| Built-in 2FA (TOTP) | Optional base32 secret per account; live 6-digit code with a countdown ring in the detail panel |
| Password generator | Crypto-random, per-class guarantees, length/charset/avoid-ambiguous options, embedded in every password field |
| Secret history | Previous values are kept (encrypted) whenever a password/key/secret is rotated — viewable and copyable from the detail panel |
| Security dashboard | Local audit: weak passwords, reused secrets, stale API-key rotations, accounts without 2FA, expiring licenses |
| Import / Export | JSON backup export; merge or replace import. Bulk import from Chrome / Bitwarden / 1Password / LastPass CSV (folders become groups, TOTP columns honored) and from `.env` files |
| CLI access | `seckeybox-cli` reads secrets from the running app over a local socket (mode 0600): `export DB_PASSWORD=$(seckeybox-cli get "prod database")`. Read-only — it can never unlock or modify the vault |
| Encrypted attachments | Attach SSH private keys, certificates, `.env` bundles or images (up to 5 MB) to any item; contents are AES-256-GCM encrypted, only file metadata stays readable |
| TOTP import | Paste an `otpauth://` URI or scan a QR screenshot; secret, issuer and account are filled in automatically (6-digit / 30-second entries) |
| Custom fields | Add encrypted key/value pairs to any item for whatever the credential template does not cover |
| Tags | Cross-group labels; filter the list by tag from the sidebar; items can carry several tags |
| Local backups | Consistent `VACUUM INTO` snapshots, one per day plus on demand, ten most recent kept; restore verifies integrity and snapshots the current state first |
| Trash & recovery | Deleting an item moves it to a hidden trash group instead of erasing it: restore it to its original group (or a fallback group once that group is gone), purge a single item, or empty the trash. Retention is configurable (default 30 days); expired entries are purged after the daily backup |
| Integrity check | Decrypts every encrypted field — all 8 types, custom fields, secret history and optionally attachments — and runs SQLite integrity plus foreign-key checks; failures are reported per item and field without exposing secrets |
| Duplicate cleanup | Groups active items by type plus identity fields (account username + website host, SSH user@host:port, API key name + endpoint, …), keeps the copy you choose, carries its tags, attachments, custom fields and secret history over, and moves the rest to the trash |
| Tray resident | Closing the window keeps SecKeyBox running in the system tray (Show / Lock / Quit); the global hotkey and idle lock keep working |
| Session safety | Configurable auto-lock (off / 1 / 5 / 15 / 30 min, enforced by the app process even while the window is hidden) and clipboard auto-clear (0 / 15 / 30 / 60 s); keys are zeroized on drop |
| In-app updates | Signed GitHub Release updates from `Settings → Updates`: each update is verified against the embedded minisign public key before it is downloaded, installed and the app restarted (Linux AppImage restarts are handled: the restart command clears the stale AppImage environment and the release AppImage ships a patched `AppRun`) |
| Appearance | Light / dark / follow-system theme |

Notes are stored as plain text by design; all password/key/secret fields are encrypted.

## Command line

The Linux `.deb` installs `/usr/bin/seckeybox-cli` next to the app. The CLI never decrypts anything itself: it talks to the running app over a local socket (mode 0600) and only works while the vault is unlocked.

```bash
seckeybox-cli status                        # locked | unlocked
seckeybox-cli list [QUERY]                  # id <TAB> type <TAB> title
seckeybox-cli get "GitHub"                  # prints the primary secret
seckeybox-cli get "GitHub" --field username
seckeybox-cli get --id <id> --json
seckeybox-cli totp "GitHub"                 # current 6-digit code

export DB_PASSWORD=$(seckeybox-cli get "prod database")
```

Exit codes: `0` success, `1` vault locked, `2` not found / ambiguous / bad usage, `3` app not reachable.

## Security model

```
Master password
   │  Argon2id (m=64MB, t=3, p=4)
   ▼
32-byte key ──► AES-256-GCM, unique nonce per field
   │
   ▼
Sensitive fields in a local SQLite database
```

- The master password never leaves your machine; only its PHC hash and salt are stored (for verification).
- Locking the vault drops the in-memory key; unlocking requires a fresh derivation.
- Changing the master password rotates the key for all data atomically (see [tests](src-tauri/src/db/operations.rs) covering rotation, NULL secrets and rollback).

## Download

Grab an installer from the [latest release](https://github.com/EthanCheung-gh/SecKeyBox/releases/latest):

| Platform | Artifact |
|----------|----------|
| Windows | `.msi` or `.exe` (NSIS setup) |
| macOS (Apple Silicon) | `.dmg` |
| Linux | `.deb` (includes `seckeybox-cli`), `.rpm`, `.AppImage` |

## Build from source

Prerequisites: [Rust](https://rustup.rs) 1.70+, Node.js 18+, and the [Tauri v2 system dependencies](https://tauri.app/start/prerequisites/) for your platform (e.g. `libwebkit2gtk-4.1-dev libgtk-3-dev` on Debian/Ubuntu).

```bash
git clone https://github.com/EthanCheung-gh/SecKeyBox.git
cd SecKeyBox
npm install

npm run tauri dev     # run the desktop app in dev mode
npm run tauri build   # produce installers in src-tauri/target/release/bundle/
                      # (builds the CLI first; the deb bundles /usr/bin/seckeybox-cli)
```

Useful scripts:

```bash
npm run dev           # frontend only in a plain browser (a built-in mock
                      # replaces the Tauri backend, seeded with demo data)
npm run build         # typecheck + production web build
npm run build:cli     # build seckeybox-cli (cli/target/release/seckeybox-cli)
npm test              # frontend unit tests (Vitest)
cd src-tauri && cargo test   # unit tests incl. re-encryption, trash, integrity & duplicates
```

`.github/workflows/ci.yml` runs `tsc`, the frontend tests, the web build and `cargo test` on every push and pull request.

## Project structure

```
SecKeyBox/
├── src/                    # React frontend
│   ├── components/         # layout, modals, unlock screens, security & trash panels
│   ├── stores/             # Zustand stores (vault / ui / settings)
│   ├── lib/                # Tauri API wrappers, utils
│   └── mocks/              # dev-only browser mock of the Tauri backend
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri commands (vault / items / attachments / import-export)
│       ├── crypto/         # Argon2id KDF + AES-256-GCM cipher + TOTP
│       ├── db/             # schema, migrations, data access, trash / integrity / duplicates
│       ├── ipc/            # local socket protocol for the CLI
│       └── state.rs        # in-memory vault state (master key, idle timer)
├── cli/                    # standalone `seckeybox-cli` crate
└── .github/workflows/      # ci.yml (push/PR checks) + release.yml (tag-triggered, 3 platforms)
```

## Roadmap

- [ ] Browser extension
- [ ] Mobile apps
- [ ] End-to-end encrypted cloud sync
- [ ] Recovery key / emergency kit
- [ ] Autocomplete / autofill

## Contributing

Issues and pull requests are welcome. For larger changes, please open an issue first to discuss the design.

## License

[Apache License 2.0](license) © 2026 EthanCheung
