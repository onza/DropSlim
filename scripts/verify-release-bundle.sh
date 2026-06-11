#!/usr/bin/env bash
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

find_app() {
  find "$target" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1
}

find_dmg() {
  find "$target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1
}

app="$(find_app)"

if [[ -z "$app" ]]; then
  dmg="$(find_dmg)"

  if [[ -z "$dmg" ]]; then
    echo "verify-release-bundle: DropSlim.app and DMG not found under $target" >&2
    find "$target" -path '*/release/bundle/*' 2>/dev/null | head -40 >&2 || true
    exit 1
  fi

  mount_point="$(mktemp -d /tmp/dropslim-verify.XXXXXX)"
  hdiutil attach -readonly -nobrowse -mountpoint "$mount_point" "$dmg" >/dev/null
  app="$(find "$mount_point" -maxdepth 1 -type d -name 'DropSlim.app' | head -1)"

  if [[ -z "$app" ]]; then
    echo "verify-release-bundle: DropSlim.app missing inside $dmg" >&2
    ls -la "$mount_point" >&2 || true
    exit 1
  fi

  if [[ ! -e "$mount_point/Applications" ]]; then
    echo "verify-release-bundle: Applications drop link missing inside $dmg" >&2
    ls -la "$mount_point" >&2 || true
    exit 1
  fi

  echo "verify-release-bundle: verifying app from $(basename "$dmg")" >&2
fi

node "$root/scripts/verify-release.mjs" "$app"
