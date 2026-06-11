#!/usr/bin/env bash
# Sign the .app and rebuild the DMG so the distributed image contains a valid ad-hoc signature.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/src-tauri/target"
version="$(node -p "require('$root/package.json').version")"

find_app() {
  find "$target" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1
}

find_dmg() {
  find "$target" -name 'DropSlim_*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1
}

app="$(find_app)"
dmg="$(find_dmg)"
staging=""
mount_point=""

cleanup() {
  if [[ -n "$mount_point" && -d "$mount_point" ]]; then
    hdiutil detach "$mount_point" -quiet 2>/dev/null || true
  fi
  if [[ -n "$staging" && -d "$staging" ]]; then
    rm -rf "$staging"
  fi
}

trap cleanup EXIT

if [[ -n "$dmg" ]]; then
  mount_point="$(mktemp -d /tmp/dropslim-sign.XXXXXX)"
  hdiutil attach -readonly -nobrowse -mountpoint "$mount_point" "$dmg" >/dev/null
  mounted_app="$(find "$mount_point" -maxdepth 1 -type d -name 'DropSlim.app' | head -1)"

  if [[ -z "$mounted_app" ]]; then
    echo "sign-macos-release: DropSlim.app missing inside $dmg" >&2
    exit 1
  fi

  staging="$(mktemp -d /tmp/dropslim-app.XXXXXX)"
  cp -R "$mounted_app" "$staging/DropSlim.app"
  app="$staging/DropSlim.app"
  hdiutil detach "$mount_point"
  mount_point=""
elif [[ -z "$app" ]]; then
  echo "sign-macos-release: no DropSlim.app or DMG under $target" >&2
  exit 1
fi

bash "$root/scripts/adhoc-sign.sh" "$app"

dmg_dir="$target/release/bundle/dmg"
mkdir -p "$dmg_dir"
out_dmg="$dmg_dir/DropSlim_${version}_aarch64.dmg"
pack_dir="$(mktemp -d /tmp/dropslim-dmg.XXXXXX)"

cp -R "$app" "$pack_dir/"
rm -f "$dmg_dir"/DropSlim_*.dmg 2>/dev/null || true
hdiutil create -volname "DropSlim" -srcfolder "$pack_dir" -ov -format UDZO "$out_dmg" >/dev/null
rm -rf "$pack_dir"

echo "sign-macos-release: ok ($out_dmg)"
