#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

npm run tauri:build

app=$(find "$root/src-tauri/target" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1)
if [[ -n "$app" ]]; then
  bash "$root/scripts/adhoc-sign.sh" "$app"
fi

bash "$root/scripts/verify-release-bundle.sh"

dmg=$(find "$root/src-tauri/target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)

if [[ ! -f "$dmg" ]]; then
  echo "build-release: DMG not found under src-tauri/target" >&2
  exit 1
fi

DMG="$dmg"

mkdir -p "$root/dist"
cp "$DMG" "$root/dist/$(basename "$DMG")"

echo "build-release: ok (dist/$(basename "$DMG"))"
