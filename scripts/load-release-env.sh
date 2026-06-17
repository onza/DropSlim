#!/usr/bin/env bash

root="$(cd "$(dirname "$0")/.." && pwd)"
env_file="$root/.release.env"

if [[ ! -f "$env_file" ]]; then
  echo "build: no .release.env — see scripts/build.sh for required APPLE_* variables" >&2
  return 0 2>/dev/null || exit 0
fi

set -a
# shellcheck source=/dev/null
source "$env_file"
set +a

echo "build: loaded .release.env"
