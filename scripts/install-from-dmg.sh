#!/usr/bin/env bash
# Run from the mounted DMG folder (or pass path to DropSlim.app).
set -euo pipefail

src="${1:-$(cd "$(dirname "$0")/.." && pwd)/DropSlim.app}"
dest="/Applications/DropSlim.app"

if [[ ! -d "$src" ]]; then
  echo "install-from-dmg: DropSlim.app not found at: $src" >&2
  echo "Usage: bash scripts/install-from-dmg.sh /Volumes/DropSlim/DropSlim.app" >&2
  exit 1
fi

rm -rf "$dest"
ditto "$src" "$dest"
xattr -cr "$dest"
bash "$(cd "$(dirname "$0")" && pwd)/adhoc-sign.sh" "$dest"
touch "$dest"
killall Dock 2>/dev/null || true
open "$dest"

echo "install-from-dmg: installed to $dest"
