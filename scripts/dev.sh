#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${DROPSLIM_DEV_PORT:-1420}"

if pids=$(lsof -ti:"$PORT" 2>/dev/null); then
  echo "dev: freeing port $PORT (pids: $pids)"
  kill $pids 2>/dev/null || true
  sleep 0.3
fi

npm run prepare:tauri

# Re-embed icons when they change (generate_context! + include_bytes!)
cargo build --manifest-path "$ROOT/src-tauri/Cargo.toml"

exec npx tauri dev
