# In-app updates

SecKeyBox can check for, download and install signed updates from GitHub
Releases. The client side is fully wired (`Settings → Updates`), but the
release pipeline only produces update artifacts once a signing key is
configured. This document lists the one-time setup.

## 1. Signing key

A key pair was generated for this repository at:

```
~/.tauri/seckeybox-updater.key            # private key (mode 600, never commit)
~/.tauri/seckeybox-updater.key.password   # its password   (mode 600)
~/.tauri/seckeybox-updater.key.pub        # public key (already in tauri.conf.json)
```

The public key is stored in `src-tauri/tauri.conf.json` under
`plugins.updater.pubkey`. If you rotate the key, replace that value.

To generate a fresh pair yourself:

```bash
npx tauri signer generate -w ~/.tauri/seckeybox-updater.key
```

## 2. GitHub Actions secrets

Add both secrets in **Settings → Secrets and variables → Actions**:

| Secret | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | contents of `~/.tauri/seckeybox-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | contents of `~/.tauri/seckeybox-updater.key.password` |

Without them the release build cannot produce `.sig` files and `latest.json`.

## 3. Enable updater artifacts

In `src-tauri/tauri.conf.json`:

```json
"bundle": {
  "createUpdaterArtifacts": true
}
```

It is currently `false` so that tag builds stay green until the two
secrets above exist. Flip it to `true` after adding them.

## 4. How updates flow

1. Push a tag (`v*`) → CI builds all bundles and, with
   `createUpdaterArtifacts`, uploads `latest.json` plus `.sig` files to the
   GitHub Release.
2. The installed app checks
   `https://github.com/EthanCheung-gh/SecKeyBox/releases/latest/download/latest.json`
   (configured in `plugins.updater.endpoints`).
3. On confirmation it downloads the bundle, verifies the signature against
   the embedded public key, installs it and offers a restart.

## Notes

- Updates are refused if the signature does not match the embedded public
  key — a tampered release cannot be installed.
- Nothing is sent anywhere during a check beyond the plain HTTPS request for
  `latest.json`; the app has no telemetry.
- Loss of the private key or its password means no further updates can be
  signed; a new key would have to ship in a manual release.
