#!/usr/bin/env bash
# Deep ad-hoc sign inside the finished Tauri DMG (gifsicle + main binary).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/src-tauri/target"
mount_point=""

cleanup() {
  if [[ -n "$mount_point" && -d "$mount_point" ]]; then
    hdiutil detach "$mount_point" -quiet 2>/dev/null || true
    rmdir "$mount_point" 2>/dev/null || true
  fi
}

trap cleanup EXIT

dmg="$(find "$target" -name 'DropSlim_*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)"

if [[ -z "$dmg" ]]; then
  echo "sign-macos-release: DMG not found under $target" >&2
  echo "sign-macos-release: build with: npm run tauri -- build --bundles dmg" >&2
  exit 1
fi

mount_point="$(mktemp -d /tmp/dropslim-sign.XXXXXX)"
hdiutil attach -readwrite -nobrowse -mountpoint "$mount_point" "$dmg" >/dev/null

app="$(find "$mount_point" -maxdepth 1 -type d -name 'DropSlim.app' | head -1)"

if [[ -z "$app" || ! -d "$app" ]]; then
  echo "sign-macos-release: DropSlim.app not found in DMG" >&2
  ls -la "$mount_point" >&2 || true
  exit 1
fi

bash "$root/scripts/adhoc-sign.sh" "$app"

sync
hdiutil detach "$mount_point" -quiet
mount_point=""

echo "sign-macos-release: ok ($dmg)"
