#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

npm run tauri:build

app=$(find "$root/src-tauri/target" -name 'DropSlim.app' -path '*/release/bundle/macos/*' | head -1)

if [[ ! -d "$app" ]]; then
  echo "build-release: DropSlim.app not found under src-tauri/target" >&2
  exit 1
fi

node scripts/verify-release.mjs "$app"

dmg_dir="$root/src-tauri/target/release/bundle/dmg"
DMG=$(ls "$dmg_dir"/*.dmg 2>/dev/null | head -1)

if [[ ! -f "$DMG" ]]; then
  echo "build-release: DMG not found in $dmg_dir" >&2
  exit 1
fi

mkdir -p "$root/dist"
cp "$DMG" "$root/dist/$(basename "$DMG")"

echo "build-release: ok (dist/$(basename "$DMG"))"
