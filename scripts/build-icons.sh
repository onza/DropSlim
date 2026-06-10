#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE="$ROOT/assets/icon/icon-1024.png"
MACOS="$ROOT/build/icon-macos.png"
BUILD="$ROOT/build"
ICONSET="$BUILD/icon.iconset"
MASK="$ROOT/scripts/mask-icon.mjs"
SVG="$ROOT/assets/icon/dropslim-icon.svg"

if [[ ! -f "$SOURCE" ]]; then
    if [[ ! -f "$SVG" ]]; then
        echo "build-icons: missing assets/icon/dropslim-icon.svg" >&2
        exit 1
    fi
    node "$ROOT/scripts/rasterize-icon.mjs"
fi

if ! command -v iconutil >/dev/null 2>&1; then
    echo "iconutil not found (macOS required to build .icns)" >&2
    exit 1
fi

mkdir -p "$BUILD"
node "$MASK" "$SOURCE" "$MACOS" 1024

rm -rf "$ICONSET"
mkdir -p "$ICONSET"

sips -z 16 16 "$MACOS" --out "$ICONSET/icon_16x16.png" >/dev/null
sips -z 32 32 "$MACOS" --out "$ICONSET/icon_16x16@2x.png" >/dev/null
sips -z 32 32 "$MACOS" --out "$ICONSET/icon_32x32.png" >/dev/null
sips -z 64 64 "$MACOS" --out "$ICONSET/icon_32x32@2x.png" >/dev/null
sips -z 128 128 "$MACOS" --out "$ICONSET/icon_128x128.png" >/dev/null
sips -z 256 256 "$MACOS" --out "$ICONSET/icon_128x128@2x.png" >/dev/null
sips -z 256 256 "$MACOS" --out "$ICONSET/icon_256x256.png" >/dev/null
sips -z 512 512 "$MACOS" --out "$ICONSET/icon_256x256@2x.png" >/dev/null
sips -z 512 512 "$MACOS" --out "$ICONSET/icon_512x512.png" >/dev/null
sips -z 1024 1024 "$MACOS" --out "$ICONSET/icon_512x512@2x.png" >/dev/null

iconutil -c icns "$ICONSET" -o "$BUILD/icon.icns"
mkdir -p "$ROOT/src-tauri/icons"
/bin/cp -f "$BUILD/icon.icns" "$ROOT/src-tauri/icons/icon.icns"
/bin/cp -f "$MACOS" "$ROOT/src-tauri/icons/icon.png"
rm -rf "$ICONSET"

echo "Icons built (macOS dock grid) → src-tauri/icons/"
