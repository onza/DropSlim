#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=load-release-env.sh
source "$root/scripts/load-release-env.sh"

for var in APPLE_ID APPLE_TEAM_ID APPLE_PASSWORD; do
  if [[ -z "${!var:-}" ]]; then
    echo "verify-notarize: missing $var in .release.env" >&2
    exit 1
  fi
done

if [[ "$APPLE_PASSWORD" == *"REPLACE"* ]] || [[ "$APPLE_PASSWORD" == "xxxx-xxxx-xxxx-xxxx" ]]; then
  echo "verify-notarize: APPLE_PASSWORD still looks like a placeholder in .release.env" >&2
  exit 1
fi

echo "verify-notarize: testing $APPLE_ID (team $APPLE_TEAM_ID)…"

xcrun notarytool history \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_PASSWORD" \
  | head -5

echo "verify-notarize: ok (credentials accepted by Apple)"
