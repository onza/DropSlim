#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/src-tauri/target"

app=$(find "$target" -type d -name 'DropSlim.app' -path '*/release/bundle/macos/*' 2>/dev/null | head -1)

if [[ -z "$app" ]]; then
  echo "find-app-bundle: DropSlim.app not found under $target" >&2
  echo "find-app-bundle: bundle contents:" >&2
  find "$target" -path '*/release/bundle/*' 2>/dev/null | head -40 >&2 || true
  exit 1
fi

echo "$app"
