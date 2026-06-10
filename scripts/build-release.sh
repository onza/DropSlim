#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

npm run tauri:build

app=$(bash "$root/scripts/find-app-bundle.sh")

node scripts/verify-release.mjs "$app"

dmg=$(find "$root/src-tauri/target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)

if [[ ! -f "$dmg" ]]; then
  echo "build-release: DMG not found under src-tauri/target" >&2
  exit 1
fi

DMG="$dmg"

mkdir -p "$root/dist"
cp "$DMG" "$root/dist/$(basename "$DMG")"

echo "build-release: ok (dist/$(basename "$DMG"))"
