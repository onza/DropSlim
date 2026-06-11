#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="${CARGO_TARGET_DIR:-$root/src-tauri/target}"
cd "$root"

npm run tauri -- build --bundles dmg
bash "$root/scripts/verify-release-bundle.sh"

dmg=$(find "$target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)

if [[ ! -f "$dmg" ]]; then
  echo "build-release: DMG not found under $target" >&2
  exit 1
fi

mkdir -p "$root/dist"
/bin/cp -f "$dmg" "$root/dist/$(basename "$dmg")"

echo "build-release: ok (dist/$(basename "$dmg"))"
