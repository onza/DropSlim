#!/usr/bin/env bash
# Deep ad-hoc sign the .app, then build the installer DMG.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

app="$(find "$root" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1)"

if [[ -z "$app" ]]; then
  echo "sign-macos-release: DropSlim.app not found" >&2
  echo "sign-macos-release: run: npm run tauri -- build --bundles app" >&2
  exit 1
fi

bash "$root/scripts/adhoc-sign.sh" "$app"
bash "$root/scripts/bundle-macos-dmg.sh" "$app"
