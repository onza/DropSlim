#!/usr/bin/env bash
# Run once after installing DropSlim from a GitHub DMG (no Apple Developer ID).
set -euo pipefail

app="/Applications/DropSlim.app"

if [[ ! -d "$app" ]]; then
  echo "allow-dropslim: install DropSlim to /Applications first" >&2
  exit 1
fi

xattr -rd com.apple.quarantine "$app" 2>/dev/null || true
bash "$(cd "$(dirname "$0")" && pwd)/adhoc-sign.sh" "$app"
touch "$app"
killall Dock 2>/dev/null || true
open "$app"

echo "allow-dropslim: ok"
