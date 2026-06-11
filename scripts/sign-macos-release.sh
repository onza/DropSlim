#!/usr/bin/env bash
# Deep ad-hoc sign (gifsicle binaries) then let Tauri build the installer DMG.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/src-tauri/target"

app="$(find "$target" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1)"

if [[ -z "$app" ]]; then
  echo "sign-macos-release: DropSlim.app not found under $target" >&2
  echo "sign-macos-release: build with: npm run tauri -- build --bundles app" >&2
  exit 1
fi

bash "$root/scripts/adhoc-sign.sh" "$app"

cd "$root"
npm run tauri -- bundle --bundles dmg --no-sign --ci
