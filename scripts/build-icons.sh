#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOURCE="$ROOT/assets/icon/icon-1024.png"
BUILD="$ROOT/build"
ICONSET="$BUILD/icon.iconset"
ICNS="$BUILD/icon.icns"

if [[ ! -f "$SOURCE" ]]; then
  echo "build-icons: missing $SOURCE (this is the only icon source)" >&2
  exit 1
fi

if ! command -v iconutil >/dev/null 2>&1; then
  echo "iconutil not found (macOS required to build .icns)" >&2
  exit 1
fi

mkdir -p "$BUILD"
rm -rf "$ICONSET"
mkdir -p "$ICONSET"

sips -z 16 16 "$SOURCE" --out "$ICONSET/icon_16x16.png" >/dev/null
sips -z 32 32 "$SOURCE" --out "$ICONSET/icon_16x16@2x.png" >/dev/null
sips -z 32 32 "$SOURCE" --out "$ICONSET/icon_32x32.png" >/dev/null
sips -z 64 64 "$SOURCE" --out "$ICONSET/icon_32x32@2x.png" >/dev/null
sips -z 128 128 "$SOURCE" --out "$ICONSET/icon_128x128.png" >/dev/null
sips -z 256 256 "$SOURCE" --out "$ICONSET/icon_128x128@2x.png" >/dev/null
sips -z 256 256 "$SOURCE" --out "$ICONSET/icon_256x256.png" >/dev/null
sips -z 512 512 "$SOURCE" --out "$ICONSET/icon_256x256@2x.png" >/dev/null
sips -z 512 512 "$SOURCE" --out "$ICONSET/icon_512x512.png" >/dev/null
sips -z 1024 1024 "$SOURCE" --out "$ICONSET/icon_512x512@2x.png" >/dev/null

iconutil -c icns "$ICONSET" -o "$ICNS"
mkdir -p "$ROOT/src-tauri/icons"
/bin/cp -f "$ICNS" "$ROOT/src-tauri/icons/icon.icns"
/bin/cp -f "$SOURCE" "$ROOT/src-tauri/icons/icon.png"
rm -rf "$ICONSET"

echo "build-icons: ok (icon-1024.png → src-tauri/icons/)"
