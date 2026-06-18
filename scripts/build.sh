#!/usr/bin/env bash
set -euo pipefail

# Release builds require a local `.release.env` in the repo root (gitignored):
#   APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#   APPLE_ID="you@example.com"
#   APPLE_TEAM_ID="XXXXXXXXXX"
#   APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"  # app-specific password from appleid.apple.com
#   TAURI_SIGNING_PRIVATE_KEY=".tauri/updater.key"  # updater signing key (keep secret)
#   TAURI_SIGNING_PRIVATE_KEY_PASSWORD="..."       # only if the key was created with a password
# Test credentials first: bash scripts/verify-notarize-credentials.sh

root="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_TARGET_DIR="$root/src-tauri/target"
target="$CARGO_TARGET_DIR"
cd "$root"

# shellcheck source=load-release-env.sh
source "$root/scripts/load-release-env.sh"

export DROPSLIM_RELEASE=1

echo "build: release build (notarization may take 5–15 minutes)"

if [[ -n "${APPLE_SIGNING_IDENTITY:-}" ]]; then
  echo "build: signing identity loaded"
else
  echo "build: warning — APPLE_SIGNING_IDENTITY not set (create .release.env — see header in this script)" >&2
fi

if [[ -z "${APPLE_PASSWORD:-}" ]] || [[ "$APPLE_PASSWORD" == *"REPLACE"* ]] || [[ "$APPLE_PASSWORD" == "xxxx-xxxx-xxxx-xxxx" ]]; then
  echo "build: fix APPLE_PASSWORD in .release.env (see header in this script)" >&2
  echo "build: create an app-specific password at appleid.apple.com" >&2
  exit 1
fi

updater_key="${TAURI_SIGNING_PRIVATE_KEY:-${TAURI_SIGNING_PRIVATE_KEY_PATH:-$root/.tauri/updater.key}}"
if [[ -f "$updater_key" ]]; then
  updater_key="$(cd "$(dirname "$updater_key")" && pwd)/$(basename "$updater_key")"
elif [[ -f "$root/$updater_key" ]]; then
  updater_key="$(cd "$root" && cd "$(dirname "$updater_key")" && pwd)/$(basename "$updater_key")"
fi

if [[ ! -f "$updater_key" ]]; then
  echo "build: missing updater signing key at $updater_key" >&2
  echo "build: run: npm run tauri signer generate -w .tauri/updater.key" >&2
  exit 1
fi

export TAURI_SIGNING_PRIVATE_KEY="$updater_key"

npm run tauri -- build --bundles dmg,app
node "$root/scripts/verify-release.mjs" --bundle
node "$root/scripts/generate-latest-json.mjs"

dmg=$(find "$target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)

if [[ ! -f "$dmg" ]]; then
  echo "build: DMG not found under $target" >&2
  exit 1
fi

mkdir -p "$root/dist"
/bin/cp -f "$dmg" "$root/dist/$(basename "$dmg")"

updater_bundle=$(find "$target" -name '*.app.tar.gz' -path '*/release/bundle/macos/*' 2>/dev/null | head -1)
if [[ -f "$updater_bundle" ]]; then
  /bin/cp -f "$updater_bundle" "$root/dist/$(basename "$updater_bundle")"
  /bin/cp -f "${updater_bundle}.sig" "$root/dist/$(basename "$updater_bundle").sig"
fi

echo "build: ok (dist/$(basename "$dmg"))"
if [[ -f "$root/dist/latest.json" ]]; then
  echo "build: updater manifest (dist/latest.json)"
fi
