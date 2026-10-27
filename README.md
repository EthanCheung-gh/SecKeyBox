<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="SecKeyBox logo">
</p>

<h1 align="center">SecKeyBox</h1>

<p align="center">
  A local-first password manager built for developers.
</p>

<p align="center">
  <a href="https://github.com/EthanCheung-gh/SecKeyBox/releases"><img src="https://img.shields.io/badge/release-v0.5.0-blue?style=flat-square" alt="Release"></a>
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
| Group management | Built-in group per credential type; every group (built-in included) can be renamed or deleted, with cascade delete of contained items |
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
| Tray resident | Closing the window keeps SecKeyBox running in the system tray (Show / Lock / Quit); the global hotkey and idle lock keep working |
| Session safety | Configurable auto-lock (off / 1 / 5 / 15 / 30 min, enforced by the app process even while the window is hidden) and clipboard auto-clear (0 / 15 / 30 / 60 s); keys are zeroized on drop |
| Appearance | Light / dark / follow-system theme |

Notes are stored as plain text by design (searchable); all password/key/secret fields are encrypted.

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
cd src-tauri && cargo test   # unit tests incl. re-encryption, cascade & IPC protocol
```

## Project structure

```
SecKeyBox/
├── src/                    # React frontend
│   ├── components/         # layout, modals, unlock screens, ui primitives
│   ├── stores/             # Zustand stores (vault / ui / settings)
│   ├── lib/                # Tauri API wrappers, utils
│   └── mocks/              # dev-only browser mock of the Tauri backend
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri commands (vault / items / attachments / import-export)
│       ├── crypto/         # Argon2id KDF + AES-256-GCM cipher + TOTP
│       ├── db/             # schema, migrations, data access
│       ├── ipc/            # local socket protocol for the CLI
│       └── state.rs        # in-memory vault state (master key, idle timer)
├── cli/                    # standalone `seckeybox-cli` crate
└── .github/workflows/      # tag-triggered release CI (3 platforms)
```

## Roadmap

- [ ] Browser extension
- [ ] Mobile apps
- [ ] End-to-end encrypted cloud sync
- [ ] Attachments and tags — attachments done in v0.4.0, tags planned
- [ ] Autocomplete / autofill

## Contributing

Issues and pull requests are welcome. For larger changes, please open an issue first to discuss the design.

## License

[Apache License 2.0](license) © 2026 EthanCheung
