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

## 5. Linux AppImage restart

Restarting into a freshly installed AppImage used to fail with

```
/usr/bin/env: /tmp/.mount_.../usr/lib/libsystemd.so.0: version 'LIBSYSTEMD_254' not found
```

The old AppImage mount leaks `LD_LIBRARY_PATH` (and `APPDIR`/`APPIMAGE`/
`ARGV0`) into the relaunched process. The stock linuxdeploy `AppRun` starts
with `#!/usr/bin/env bash`, so `/usr/bin/env` resolves `bash` through the old
mount's library path, loads the old mount's `libsystemd`, and the new AppImage
never starts. The fix has two layers because the *old* client performs the
relaunch — a fix that only lives in the new binary cannot help the first hop.

**Layer 1 — `restart_app` clears the environment (onwards).**

`Settings → Updates → Restart now` no longer calls the process plugin's
`relaunch`. It invokes the Rust command `restart_app`
(`src-tauri/src/commands/app.rs`), which — when `APPIMAGE` is set — unsets
`LD_LIBRARY_PATH`, `LD_PRELOAD`, `APPDIR`, `APPIMAGE` and `ARGV0` and then
calls `app.restart()`. The new AppImage runtime exports its own values on
start-up, so every update from a client that has this command works without
any release-side patching.

**Layer 2 — the release AppImage's `AppRun` is patched and re-signed (the
first hop).**

When you are running an older AppImage (one that still calls `relaunch`), that
old client does the restart, so v0.6.2's `restart_app` never runs. The v0.6.2
AppImage itself must therefore be launchable from the leaking environment:

- `.github/appimage/AppRun` is a patched copy of the linuxdeploy `AppRun`
  script. It uses `#!/bin/bash` (no `/usr/bin/env` indirection), unsets
  `LD_LIBRARY_PATH`/`LD_PRELOAD`, sources the GTK hook and then
  `exec`s `AppRun.wrapped`.
- The `patch-appimage` job in `.github/workflows/release.yml` runs after the
  whole build matrix: it downloads the published `SecKeyBox_*_amd64.AppImage`,
  `--appimage-extract`s it, drops in the patched `AppRun`, repacks with
  `appimagetool`, re-signs with `tauri signer sign`, rewrites the
  `linux-x86_64` and `linux-x86_64-appimage` signatures in `latest.json`
  (deb/rpm/Windows/macOS entries are untouched), asserts the manifest
  signatures equal the new `.sig`, re-extracts the result to confirm the
  first line of `AppRun` is `#!/bin/bash`, and finally
  `gh release upload --clobber`s the AppImage, its `.sig` and `latest.json`.

Because the patched AppRun strips the stale environment before bash starts,
the old client's relaunch succeeds; from then on `restart_app` keeps it
correct without touching releases.

If you rebuild the AppImage by hand, keep the patched `AppRun` executable
inside the AppDir and re-sign afterwards — otherwise the updater refuses the
artifact (signature) or the restart keeps failing (AppRun).

## Notes

- Updates are refused if the signature does not match the embedded public
  key — a tampered release cannot be installed.
- Nothing is sent anywhere during a check beyond the plain HTTPS request for
  `latest.json`; the app has no telemetry.
- Loss of the private key or its password means no further updates can be
  signed; a new key would have to ship in a manual release.
