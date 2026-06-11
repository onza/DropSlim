#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SVG="$ROOT/assets/icon/dropslim-icon.svg"

if [[ ! -f "$SVG" ]]; then
  echo "sync-icons: missing $SVG" >&2
  exit 1
fi

bash "$ROOT/scripts/build-icons.sh"

echo "sync-icons: ok (built from dropslim-icon.svg → src-tauri/icons/)"
