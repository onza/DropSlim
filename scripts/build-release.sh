#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

npm run tauri -- build --bundles dmg
bash "$root/scripts/sign-macos-release.sh"
bash "$root/scripts/verify-release-bundle.sh"

dmg=$(find "$root/src-tauri/target" -name '*.dmg' -path '*/release/bundle/dmg/*' 2>/dev/null | head -1)

if [[ ! -f "$dmg" ]]; then
  echo "build-release: DMG not found under src-tauri/target" >&2
  exit 1
fi

mkdir -p "$root/dist"
cp "$dmg" "$root/dist/$(basename "$dmg")"

echo "build-release: ok (dist/$(basename "$dmg"))"
