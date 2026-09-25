#!/usr/bin/env bash
set -euo pipefail

# builds a standalone dropslim cli release tarball (macos + linux x86_64).
# does not touch updater/latest.json or the gui app bundle.
#
# usage:
#   bash scripts/package-cli.sh           # version from package.json
#   bash scripts/package-cli.sh 1.6.3     # override version in the archive name
#
# output:
#   macos: dist/dropslim-cli_<version>_<arch>.tar.gz
#   linux: dist/dropslim-cli_<version>_linux_<arch>.tar.gz
#   layout: dropslim, LICENSE.md, README.md, vendor/gifsicle (optional),
#           vendor/dav1d (linux — libdav1d.so.*)

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

cli_os() {
  case "$(uname -s)" in
    Darwin) printf 'darwin' ;;
    Linux) printf 'linux' ;;
    *) die "unsupported OS for cli packaging: $(uname -s) (macOS or Linux only)" ;;
  esac
}

need_cmd cargo
need_cmd tar
need_cmd node

os="$(cli_os)"
if [[ "$os" == "linux" ]]; then
  case "$(uname -m)" in
    x86_64) ;;
    *) die "linux cli packaging currently supports x86_64 only (got $(uname -m))" ;;
  esac
fi

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(node -p "require('./package.json').version")"
fi
[[ -n "$version" ]] || die "could not resolve version"

arch="$(cli_arch)"
target_dir="${CARGO_TARGET_DIR:-$root/target}"
bin="$target_dir/release/dropslim"
if [[ "$os" == "linux" ]]; then
  asset_stem="dropslim-cli_${version}_linux_${arch}"
else
  asset_stem="dropslim-cli_${version}_${arch}"
fi
stage="$(mktemp -d "${TMPDIR:-/tmp}/dropslim-cli-pack.XXXXXX")"
trap 'rm -rf "$stage"' EXIT

log "building dropslim-cli (release, CARGO_TARGET_DIR=$target_dir)"
CARGO_TARGET_DIR="$target_dir" cargo build -p dropslim-cli --release
[[ -f "$bin" ]] || die "missing binary: $bin"

if command -v strip >/dev/null 2>&1; then
  if [[ "$os" == "darwin" ]]; then
    strip -x "$bin" 2>/dev/null || true
  else
    strip "$bin" 2>/dev/null || true
  fi
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
  log "WARNING: vendor/gifsicle/gifsicle missing — tarball relies on PATH"
fi

# Linux: ship libdav1d next to the binary (AVIF decode) so apt install is not required.
if [[ "$os" == "linux" ]]; then
  need_cmd ldd
  need_cmd patchelf
  mkdir -p "$bundle/vendor/dav1d"
  mapfile -t dav1d_libs < <(ldd "$bundle/dropslim" | awk '/libdav1d\.so/ {print $3}' | sort -u)
  [[ "${#dav1d_libs[@]}" -gt 0 ]] || die "libdav1d not linked — install libdav1d-dev before packaging"
  for lib in "${dav1d_libs[@]}"; do
    [[ -f "$lib" ]] || die "missing linked library: $lib"
    cp -L "$lib" "$bundle/vendor/dav1d/"
    chmod 755 "$bundle/vendor/dav1d/$(basename "$lib")"
    log "bundled vendor/dav1d/$(basename "$lib")"
  done
  patchelf --set-rpath '$ORIGIN/vendor/dav1d' "$bundle/dropslim"
  log "set rpath \$ORIGIN/vendor/dav1d"
fi

mkdir -p "$root/dist"
out="$root/dist/${asset_stem}.tar.gz"
tar -C "$stage" -czf "$out" "$asset_stem"

log "wrote $out"
if [[ "$os" == "darwin" ]] && command -v shasum >/dev/null 2>&1; then
  log "homebrew sha256: $(shasum -a 256 "$out" | awk '{print $1}')"
elif command -v sha256sum >/dev/null 2>&1; then
  log "sha256: $(sha256sum "$out" | awk '{print $1}')"
fi
printf '%s\n' "$out"
