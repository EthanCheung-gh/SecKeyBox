#!/usr/bin/env bash
#
# Patch the released Linux AppImage so an in-app restart works.
#
# The first update away from an older AppImage is performed by the OLD client,
# which relaunches the freshly installed AppImage with the previous mount's
# LD_LIBRARY_PATH still exported. The stock linuxdeploy AppRun starts with
# `#!/usr/bin/env bash`, and /usr/bin/env then resolves bash through that stale
# LD_LIBRARY_PATH, loading the old mount's libsystemd and dying with
# "version 'LIBSYSTEMD_254' not found".
#
# This script rewrites AppRun so it clears that environment first, repackages
# and re-signs the AppImage, and updates the signatures in latest.json. Newer
# clients no longer depend on this (their `restart_app` command clears the
# AppImage environment itself), but it is what makes the first hop work.
set -euo pipefail

: "${TAG:?TAG (the release tag) is required}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"
: "${GITHUB_WORKSPACE:?GITHUB_WORKSPACE is required}"

REPO_ROOT="$GITHUB_WORKSPACE"
PATCHED_APPRUN="$REPO_ROOT/.github/appimage/AppRun"

if [ ! -f "$PATCHED_APPRUN" ]; then
  echo "error: patched AppRun not found at $PATCHED_APPRUN" >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

echo "==> Downloading release assets for $TAG"
gh release download "$TAG" \
  --repo "$GITHUB_REPOSITORY" \
  --pattern 'latest.json' \
  --pattern 'SecKeyBox_*_amd64.AppImage'

shopt -s nullglob
appimages=(SecKeyBox_*_amd64.AppImage)
if [ "${#appimages[@]}" -ne 1 ]; then
  echo "error: expected exactly one SecKeyBox_*_amd64.AppImage, found ${#appimages[@]}" >&2
  exit 1
fi
APPIMAGE_NAME="${appimages[0]}"
echo "==> AppImage: $APPIMAGE_NAME"

if [ ! -f latest.json ]; then
  echo "error: latest.json was not downloaded" >&2
  exit 1
fi

echo "==> Extracting AppImage"
chmod +x "$APPIMAGE_NAME"
"./$APPIMAGE_NAME" --appimage-extract >/dev/null

echo "==> Installing patched AppRun"
cp "$PATCHED_APPRUN" squashfs-root/AppRun
chmod +x squashfs-root/AppRun
chmod -R a+rX squashfs-root

echo "==> Fetching appimagetool"
curl -fsSL -o appimagetool-x86_64.AppImage \
  https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

echo "==> Repacking AppImage"
rm -f "$APPIMAGE_NAME"
ARCH=x86_64 ./appimagetool-x86_64.AppImage \
  --appimage-extract-and-run \
  --no-appstream \
  squashfs-root "$APPIMAGE_NAME"

echo "==> Re-signing AppImage"
(cd "$REPO_ROOT" && npx --no-install tauri signer sign "$WORK/$APPIMAGE_NAME")

SIG_FILE="$WORK/$APPIMAGE_NAME.sig"
if [ ! -f "$SIG_FILE" ]; then
  echo "error: signature $SIG_FILE was not produced" >&2
  exit 1
fi

echo "==> Updating latest.json signatures"
jq --rawfile sig "$SIG_FILE" '
  ($sig | rtrimstr("\n")) as $s
  | .platforms["linux-x86_64"].signature = $s
  | .platforms["linux-x86_64-appimage"].signature = $s
' latest.json > latest.json.new
mv latest.json.new latest.json

SIG_CONTENT="$(cat "$SIG_FILE")"
for key in linux-x86_64 linux-x86_64-appimage; do
  MANIFEST_SIG="$(jq -r --arg k "$key" '.platforms[$k].signature // empty' latest.json)"
  if [ -z "$MANIFEST_SIG" ] || [ "$MANIFEST_SIG" != "$SIG_CONTENT" ]; then
    echo "error: latest.json signature for $key does not match the new .sig" >&2
    exit 1
  fi
done
echo "==> latest.json signatures match the re-signed AppImage"

echo "==> Verifying the repacked AppImage carries the patched AppRun"
mkdir -p verify
(cd verify && "$WORK/$APPIMAGE_NAME" --appimage-extract >/dev/null)
VERIFY_APPRUN="verify/squashfs-root/AppRun"
if [ ! -f "$VERIFY_APPRUN" ]; then
  echo "error: repacked AppImage has no AppRun" >&2
  exit 1
fi
if [ "$(head -n 1 "$VERIFY_APPRUN")" != '#!/bin/bash' ]; then
  echo "error: repacked AppRun does not start with #!/bin/bash (still the old env-based shebang)" >&2
  exit 1
fi
if ! grep -q 'unset LD_LIBRARY_PATH LD_PRELOAD' "$VERIFY_APPRUN"; then
  echo "error: repacked AppRun is missing the LD_LIBRARY_PATH cleanup" >&2
  exit 1
fi

echo "==> Uploading patched AppImage and latest.json"
gh release upload "$TAG" \
  "$WORK/$APPIMAGE_NAME" \
  "$WORK/$APPIMAGE_NAME.sig" \
  "$WORK/latest.json" \
  --clobber \
  --repo "$GITHUB_REPOSITORY"

echo "==> Done: $APPIMAGE_NAME patched, re-signed and published"
