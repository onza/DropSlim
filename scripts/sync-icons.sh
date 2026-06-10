#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE="$ROOT/assets/icon/icon-1024.png"
MACOS="$ROOT/build/icon-macos.png"

if [[ ! -f "$SOURCE" || ! -f "$MACOS" || ! -f "$ROOT/build/icon.icns" || "$SOURCE" -nt "$MACOS" ]]; then
    bash "$ROOT/scripts/build-icons.sh"
else
    mkdir -p "$ROOT/src-tauri/icons"
    /bin/cp -f "$ROOT/build/icon.icns" "$ROOT/src-tauri/icons/icon.icns"
    /bin/cp -f "$MACOS" "$ROOT/src-tauri/icons/icon.png"
fi

echo "sync-icons: ok (macOS dock icon → src-tauri/icons/)"
