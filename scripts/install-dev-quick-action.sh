#!/usr/bin/env bash
# Install the dev Finder Quick Action (points at the debug binary).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
binary="$(cd "$root/src-tauri" && cargo metadata --format-version 1 --no-deps | node -pe "JSON.parse(require('fs').readFileSync(0,'utf8')).target_directory + '/debug/dropslim'")"
services="$HOME/Library/Services"
workflow="Optimize with DropSlim (Dev).workflow"

if [[ ! -x "$binary" ]]; then
  echo "install-dev-quick-action: debug binary not found: $binary" >&2
  exit 1
fi

node "$root/scripts/build-quick-action.mjs" --dev --binary "$binary"
rm -rf "$services/$workflow"
/bin/cp -R "$root/build/$workflow" "$services/"

echo "install-dev-quick-action: ok ($services/$workflow → $binary)"
