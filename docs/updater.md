# In-app updates

SecKeyBox can check for, download and install signed updates from GitHub
Releases. The client side is fully wired (`Settings → Updates`) and the
release pipeline is configured to sign its output, so a `v*` tag build
publishes installable updates. This document describes the setup that makes
that work.

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

Both secrets are configured at the repository level
(**Settings → Secrets and variables → Actions**), and the release workflow
passes them to `tauri-action`:

| Secret | Value |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | contents of `~/.tauri/seckeybox-updater.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | contents of `~/.tauri/seckeybox-updater.key.password` |

```yaml
- uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```

Without them the release build cannot produce `.sig` files and `latest.json`
— a tag build would fail instead of silently shipping an unsigned release.

To set them again (or on a fork), pipe the files through stdin so the values
never land in your shell history or in `argv`:

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo EthanCheung-gh/SecKeyBox < ~/.tauri/seckeybox-updater.key
tr -d '\n' < ~/.tauri/seckeybox-updater.key.password | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo EthanCheung-gh/SecKeyBox
gh secret list --repo EthanCheung-gh/SecKeyBox   # both names should be listed
```

## 3. Updater artifacts

`src-tauri/tauri.conf.json` has updater artifacts enabled:

```json
"bundle": {
  "createUpdaterArtifacts": true
}
```

Tag builds therefore emit a `.sig` signature for every bundle together with
the `latest.json` manifest that the updater endpoint points at. Turning this
back to `false` would publish releases the updater can never install.

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
