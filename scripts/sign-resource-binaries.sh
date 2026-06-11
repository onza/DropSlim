#!/usr/bin/env bash
# Ad-hoc sign embedded binaries (gifsicle) before Tauri bundles them into the .app.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
resources="${1:-$root/src-tauri/resources}"

if [[ ! -d "$resources" ]]; then
  echo "sign-resource-binaries: not found: $resources" >&2
  exit 1
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "sign-resource-binaries: skipped (not macOS)"
  exit 0
fi

signed=0
while IFS= read -r -d '' binary; do
  codesign --force --sign - "$binary" >/dev/null
  signed=$((signed + 1))
done < <(
  find "$resources" -type f \( -perm -0100 -o -name gifsicle \) -print0 2>/dev/null
)

echo "sign-resource-binaries: ok ($signed binaries in $resources)"
