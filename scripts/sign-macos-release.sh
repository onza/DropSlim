#!/usr/bin/env bash
# Ad-hoc sign the app and repackage the DMG (read-only mount — works on GitHub Actions).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/src-tauri/target"
version="$(node -p "require('$root/package.json').version")"
mount_point=""
pack_dir=""

cleanup() {
  if [[ -n "$mount_point" && -d "$mount_point" ]]; then
    hdiutil detach "$mount_point" -quiet 2>/dev/null || true
    rmdir "$mount_point" 2>/dev/null || true
  fi
  if [[ -n "$pack_dir" && -d "$pack_dir" ]]; then
    rm -rf "$pack_dir"
  fi
}

trap cleanup EXIT

dmg="$(find "$target" -name 'DropSlim_*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)"
bundle_script="$(find "$target" -name 'bundle_dmg.sh' -path '*/bundle/dmg/*' 2>/dev/null | head -1)"

if [[ -z "$dmg" ]]; then
  echo "sign-macos-release: DMG not found under $target" >&2
  echo "sign-macos-release: build with: npm run tauri -- build --bundles dmg" >&2
  exit 1
fi

if [[ ! -x "$bundle_script" ]]; then
  echo "sign-macos-release: bundle_dmg.sh not found under $target" >&2
  exit 1
fi

dmg_dir="$(dirname "$dmg")"
out_dmg="$dmg_dir/DropSlim_${version}_aarch64.dmg"
icon_icns="$dmg_dir/icon.icns"

mount_point="$(mktemp -d /tmp/dropslim-sign.XXXXXX)"
hdiutil attach -readonly -nobrowse -mountpoint "$mount_point" "$dmg" >/dev/null

app="$(find "$mount_point" -maxdepth 1 -type d -name 'DropSlim.app' | head -1)"
if [[ -z "$app" || ! -d "$app" ]]; then
  echo "sign-macos-release: DropSlim.app not found in DMG" >&2
  ls -la "$mount_point" >&2 || true
  exit 1
fi

pack_dir="$(mktemp -d /tmp/dropslim-pack.XXXXXX)"
ditto "$app" "$pack_dir/DropSlim.app"

hdiutil detach "$mount_point" -quiet
mount_point=""

bash "$root/scripts/adhoc-sign.sh" "$pack_dir/DropSlim.app"

args=(
  --volname "DropSlim"
  --window-size 660 400
  --icon-size 128
  --icon "DropSlim.app" 168 240
  --hide-extension "DropSlim.app"
  --app-drop-link 372 240
  --no-internet-enable
  --skip-jenkins
)

if [[ -f "$icon_icns" ]]; then
  args+=(--volicon "$icon_icns")
fi

rm -f "$dmg_dir"/DropSlim_*.dmg 2>/dev/null || true
"$bundle_script" "${args[@]}" "$out_dmg" "$pack_dir"

echo "sign-macos-release: ok ($out_dmg)"
