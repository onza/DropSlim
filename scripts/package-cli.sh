#!/usr/bin/env bash
set -euo pipefail

# builds a standalone dropslim cli release tarball (macos first).
# does not touch updater/latest.json or the gui app bundle.
#
# usage:
#   bash scripts/package-cli.sh           # version from package.json
#   bash scripts/package-cli.sh 1.6.3     # override version in the archive name
#
# output:
#   dist/dropslim-cli_<version>_<arch>.tar.gz
#   layout: dropslim, LICENSE.md, README.md, optional vendor/gifsicle/gifsicle

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

log() { printf 'package-cli: %s\n' "$*"; }
die() { printf 'package-cli: error: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing command: $1"
}

cli_arch() {
  case "$(uname -m)" in
    arm64 | aarch64) printf 'aarch64' ;;
    x86_64) printf 'x86_64' ;;
    *) die "unsupported architecture: $(uname -m)" ;;
  esac
}

need_cmd cargo
need_cmd tar
need_cmd node

[[ "$(uname -s)" == "Darwin" ]] || die "cli packaging currently supports macOS only"

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(node -p "require('./package.json').version")"
fi
[[ -n "$version" ]] || die "could not resolve version"

arch="$(cli_arch)"
target_dir="${CARGO_TARGET_DIR:-$root/target}"
bin="$target_dir/release/dropslim"
asset_stem="dropslim-cli_${version}_${arch}"
stage="$(mktemp -d "${TMPDIR:-/tmp}/dropslim-cli-pack.XXXXXX")"
trap 'rm -rf "$stage"' EXIT

log "building dropslim-cli (release, CARGO_TARGET_DIR=$target_dir)"
CARGO_TARGET_DIR="$target_dir" cargo build -p dropslim-cli --release
[[ -f "$bin" ]] || die "missing binary: $bin"

if command -v strip >/dev/null 2>&1; then
  strip -x "$bin" 2>/dev/null || true
fi

bundle="$stage/$asset_stem"
mkdir -p "$bundle"
cp "$bin" "$bundle/dropslim"
chmod 755 "$bundle/dropslim"
cp "$root/LICENSE.md" "$bundle/LICENSE.md"
cp "$root/README.md" "$bundle/README.md"

gifsicle_src="$root/vendor/gifsicle/gifsicle"
if [[ -f "$gifsicle_src" ]]; then
  mkdir -p "$bundle/vendor/gifsicle"
  cp "$gifsicle_src" "$bundle/vendor/gifsicle/gifsicle"
  chmod 755 "$bundle/vendor/gifsicle/gifsicle"
  log "bundled vendor/gifsicle/gifsicle"
else
  log "WARNING: vendor/gifsicle/gifsicle missing — tarball relies on PATH / Homebrew"
fi

mkdir -p "$root/dist"
out="$root/dist/${asset_stem}.tar.gz"
tar -C "$stage" -czf "$out" "$asset_stem"

log "wrote $out"
printf '%s\n' "$out"
