#!/usr/bin/env bash
# Package a signed DropSlim.app into a DMG (Tauri's bundle_dmg.sh, no --skip-jenkins).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(node -p "require('$root/package.json').version")"
app="${1:-}"

if [[ -z "$app" || ! -d "$app" ]]; then
  app="$(find "$root" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1)"
fi

target="$(cd "$(dirname "$app")/../../.." && pwd)"

if [[ -z "$app" || ! -d "$app" ]]; then
  echo "bundle-macos-dmg: DropSlim.app not found" >&2
  exit 1
fi

bundle_script="$(find "$target" -name 'bundle_dmg.sh' -path '*/bundle/dmg/*' 2>/dev/null | head -1)"
if [[ ! -x "$bundle_script" ]]; then
  echo "bundle-macos-dmg: bundle_dmg.sh not found — run: npm run tauri -- build --bundles app" >&2
  exit 1
fi

dmg_dir="$target/release/bundle/dmg"
mkdir -p "$dmg_dir"
out_dmg="$dmg_dir/DropSlim_${version}_aarch64.dmg"
icon_icns="$dmg_dir/icon.icns"
pack_dir="$(mktemp -d /tmp/dropslim-dmg.XXXXXX)"

cp -R "$app" "$pack_dir/DropSlim.app"

args=(
  --volname "DropSlim"
  --window-size 660 400
  --icon-size 128
  --icon "DropSlim.app" 168 240
  --hide-extension "DropSlim.app"
  --app-drop-link 372 240
  --no-internet-enable
)

if [[ -f "$icon_icns" ]]; then
  args+=(--volicon "$icon_icns")
fi

rm -f "$dmg_dir"/DropSlim_*.dmg 2>/dev/null || true
"$bundle_script" "${args[@]}" "$out_dmg" "$pack_dir"
rm -rf "$pack_dir"

echo "bundle-macos-dmg: ok ($out_dmg)"
