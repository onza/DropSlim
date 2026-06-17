#!/usr/bin/env bash
set -euo pipefail

# Release builds require a local `.release.env` in the repo root (gitignored):
#   APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
#   APPLE_ID="you@example.com"
#   APPLE_TEAM_ID="XXXXXXXXXX"
#   APPLE_PASSWORD="xxxx-xxxx-xxxx-xxxx"  # app-specific password from appleid.apple.com
# Test credentials first: bash scripts/verify-notarize-credentials.sh

root="$(cd "$(dirname "$0")/.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/src-tauri/target}"
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

npm run tauri -- build --bundles dmg
node "$root/scripts/verify-release.mjs" --bundle

dmg=$(find "$target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)

if [[ ! -f "$dmg" ]]; then
  echo "build: DMG not found under $target" >&2
  exit 1
fi

mkdir -p "$root/dist"
/bin/cp -f "$dmg" "$root/dist/$(basename "$dmg")"

echo "build: ok (dist/$(basename "$dmg"))"
