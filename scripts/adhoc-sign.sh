#!/usr/bin/env bash
set -euo pipefail

app="${1:-}"
root="$(cd "$(dirname "$0")/.." && pwd)"
entitlements="$root/src-tauri/entitlements.plist"

if [[ -z "$app" || ! -d "$app" ]]; then
  echo "Usage: bash scripts/adhoc-sign.sh /path/to/DropSlim.app" >&2
  exit 1
fi

if [[ ! -f "$entitlements" ]]; then
  echo "adhoc-sign: entitlements missing: $entitlements" >&2
  exit 1
fi

xattr -cr "$app" 2>/dev/null || true

while IFS= read -r -d '' binary; do
  codesign --force --sign - "$binary" >/dev/null
done < <(
  find "$app/Contents" -type f \( -perm -0100 -o -name gifsicle \) \
    ! -path '*/MacOS/*' -print0 2>/dev/null
)

main_bin="$app/Contents/MacOS/dropslim"
if [[ ! -f "$main_bin" ]]; then
  echo "adhoc-sign: main executable missing: $main_bin" >&2
  exit 1
fi

chmod +x "$main_bin"
codesign --force --sign - --options runtime --entitlements "$entitlements" \
  "$main_bin" >/dev/null
codesign --force --sign - --options runtime --entitlements "$entitlements" \
  "$app" >/dev/null

if ! codesign --verify --deep --strict "$app" 2>/dev/null; then
  echo "adhoc-sign: verification failed for $app" >&2
  codesign --verify --deep --strict --verbose=2 "$app" >&2 || true
  exit 1
fi

flags="$(codesign -dvvv "$main_bin" 2>&1 | grep '^CodeDirectory' || true)"
if [[ "$flags" != *runtime* ]]; then
  echo "adhoc-sign: hardened runtime missing on $main_bin" >&2
  echo "$flags" >&2
  exit 1
fi

echo "adhoc-sign: ok ($app)"
