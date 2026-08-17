#!/usr/bin/env bash
set -euo pipefail

# automates scripts/RELEASE.md.
# secrets stay in .release.env (gitignored). requires: gh, jq, node, npm, git.
#
# usage:
#   bash scripts/release.sh                 # interactive
#   bash scripts/release.sh 1.6.2
#   bash scripts/release.sh 1.6.2 --draft-only
#   bash scripts/release.sh --continue
#   npm run release -- 1.6.2
#
# rules:
#   main + full  → draft, mac, merge, undraft (latest) + sync updater/latest.json
#   main + draft → draft, mac, merge, stay draft
#   other branch → always draft; version x.y.z-<branch>.<n>
#
# flags:
#   --continue         skip bump; use package.json version
#   --skip-mac         skip local mac build/upload
#   --draft-only       leave github release as draft
#   --yes              skip start confirms
#   --skip-ci-wait     do not wait for ci
#
# env:
#   DROPSLIM_RELEASE_YES=1   with --yes, also skip full undraft confirm
#   DROPSLIM_RELEASE_USER    expected gh login (default: repo owner)

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

REPO="${DROPSLIM_REPO:-onza/DropSlim}"
BUMP_VERSION=""
CONTINUE=0
SKIP_MAC=0
CLI_YES=0
SKIP_CI_WAIT=0
MODE="full" # full | draft
EXPECTED_USER="${DROPSLIM_RELEASE_USER:-${REPO%%/*}}"
branch=""

usage() {
  sed -n '/^# usage:/,/^[^#]/p' "$0" | sed '$d; s/^# \{0,1\}//'
  exit "${1:-0}"
}

log() { printf 'release: %s\n' "$*"; }
die() { printf 'release: error: %s\n' "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing command: $1"
}

lower() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

confirm() {
  local prompt="$1"
  if [[ "$CLI_YES" -eq 1 ]]; then
    return 0
  fi
  read -r -p "release: $prompt [y/N] " answer
  [[ "$answer" == "y" || "$answer" == "Y" ]]
}

# full/latest undraft: --yes is not enough; also set DROPSLIM_RELEASE_YES=1
confirm_full_publish() {
  local version="$1"
  local typed
  if [[ "$CLI_YES" -eq 1 && "${DROPSLIM_RELEASE_YES:-}" == "1" ]]; then
    return 0
  fi
  printf '\n' >&2
  log "WARNING: undraft marks v${version} as GitHub latest and can update updater/latest.json"
  typed="$(ask "type ${version} to publish to latest" "")"
  [[ "$typed" == "$version" ]]
}

origin_repo() {
  local url
  url="$(git remote get-url origin 2>/dev/null)" || die "no git remote 'origin'"
  url="${url%.git}"
  url="${url%/}"
  if [[ "$url" =~ github\.com/([^/]+)/([^/]+)$ ]] ||
     [[ "$url" =~ github\.com:([^/]+)/([^/]+)$ ]]; then
    printf '%s/%s' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}"
  else
    die "origin is not a github url: $url"
  fi
}

assert_origin_repo() {
  local origin
  origin="$(origin_repo)"
  [[ "$(lower "$origin")" == "$(lower "$REPO")" ]] ||
    die "origin is $origin — expected $REPO (wrong clone/fork?)"
}

assert_gh_user() {
  local login
  gh auth status >/dev/null 2>&1 || die "gh is not authenticated (gh auth login)"
  login="$(gh api user --jq .login)"
  [[ "$(lower "$login")" == "$(lower "$EXPECTED_USER")" ]] ||
    die "gh is logged in as $login — expected $EXPECTED_USER"
}

ask() {
  local prompt="$1"
  local default="${2:-}"
  local answer
  if [[ -n "$default" ]]; then
    read -r -p "release: $prompt [$default]: " answer
    printf '%s' "${answer:-$default}"
  else
    read -r -p "release: $prompt: " answer
    printf '%s' "$answer"
  fi
}

is_valid_version() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]
}

read_version() {
  node -e "console.log(JSON.parse(require('fs').readFileSync('package.json','utf8')).version)"
}

# feature/foo-bar → feature-foo-bar
branch_slug() {
  local name
  name="$(lower "$1")"
  name="$(printf '%s' "$name" | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//; s/-{2,}/-/g')"
  printf '%s' "$name"
}

# 1.6.2-fix.1 → 1.6.2
base_semver() {
  printf '%s' "${1%%-*}"
}

# 1.6.1 → 1.6.2 (feature drafts you install should sort after the live version)
increment_patch() {
  local major minor patch
  IFS=. read -r major minor patch <<< "$1"
  printf '%s.%s.%s' "$major" "$minor" "$((patch + 1))"
}

# base 1.6.2 + slug fix-updater → next free 1.6.2-fix-updater.N
next_branch_version() {
  local base="$1"
  local slug="$2"
  local max=0
  local n tag

  while IFS= read -r tag; do
    [[ -z "$tag" ]] && continue
    if [[ "$tag" =~ ^v?${base}-${slug}\.([0-9]+)$ ]]; then
      n="${BASH_REMATCH[1]}"
      if (( n > max )); then
        max=$n
      fi
    fi
  done <<<"$(
    {
      git ls-remote --tags --refs origin 2>/dev/null | awk '{print $2}' | sed 's#refs/tags/##'
      git tag -l "v${base}-${slug}.*" "${base}-${slug}.*" 2>/dev/null
    } | sort -u
  )"

  printf '%s-%s.%s' "$base" "$slug" "$((max + 1))"
}

wait_for_workflow() {
  local name="$1"
  local jq_filter="$2"
  local sha="$3"
  local run_id="" i

  log "waiting for $name on $sha"
  for i in $(seq 1 60); do
    run_id="$(
      gh run list --repo "$REPO" --workflow "$name" --limit 30 \
        --json databaseId,headSha,event \
        --jq "$jq_filter"
    )"
    [[ -n "$run_id" ]] && break
    sleep 5
  done
  [[ -n "$run_id" ]] || return 1

  log "$name run: https://github.com/$REPO/actions/runs/$run_id"
  gh run watch "$run_id" --repo "$REPO" --exit-status
  log "$name green"
}

wait_for_ci() {
  local sha="$1"

  if [[ "$SKIP_CI_WAIT" -eq 1 ]]; then
    log "skipping ci wait (--skip-ci-wait)"
    return 0
  fi

  if wait_for_workflow CI "map(select(.headSha == \"$sha\")) | .[0].databaseId // empty" "$sha"; then
    return 0
  fi
  if [[ "$branch" != "main" ]]; then
    log "no ci run for $sha (feature branches only get ci via pr) — continuing"
    return 0
  fi
  die "no ci run found for $sha"
}

wait_for_publish_workflow() {
  local sha="$1"
  wait_for_workflow publish.yml \
    "map(select(.headSha == \"$sha\" and .event == \"workflow_dispatch\")) | .[0].databaseId // empty" \
    "$sha" || die "no publish run found for $sha"
}

wait_for_draft_release() {
  local tag="$1" i
  for i in $(seq 1 60); do
    gh release view "$tag" --repo "$REPO" >/dev/null 2>&1 && return 0
    sleep 5
  done
  die "draft release $tag did not appear"
}

release_is_draft() {
  local tag="$1"
  [[ "$(gh release view "$tag" --repo "$REPO" --json isDraft -q .isDraft)" == "true" ]]
}

push_current_branch() {
  git push -u origin "HEAD:refs/heads/${branch}"
}

bump_version() {
  local next="$1"
  local current
  current="$(read_version)"
  [[ "$current" != "$next" ]] || die "already at $next — use --continue"
  log "bump $current → $next"
  node -e "
    const fs = require('fs');
    const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
    pkg.version = process.argv[1];
    fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
  " "$next"
  npm run version
  git add package.json src-tauri/Cargo.toml
  if [[ -n "$(git status --porcelain -- src-tauri/Cargo.lock)" ]]; then
    git add src-tauri/Cargo.lock
  fi
  git commit -m "$(printf 'release: bump version to %s\n' "$next")"
  push_current_branch
  wait_for_ci "$(git rev-parse HEAD)"
}

sync_updater_manifest() {
  local version="$1"
  if [[ ! -f dist/latest.json ]]; then
    log "WARNING: dist/latest.json missing — updater/latest.json not synced"
    return 0
  fi
  mkdir -p updater
  cp dist/latest.json updater/latest.json
  if [[ -n "$(git status --porcelain -- updater/latest.json)" ]]; then
    git add updater/latest.json
    git commit -m "$(printf 'chore: sync updater latest.json for %s\n' "$version")"
    push_current_branch
    log "pushed updater/latest.json"
  fi
}

manifest_has_required_platforms() {
  local file="$1"
  jq -e '.platforms["darwin-aarch64"] and .platforms["windows-x86_64"]' "$file" >/dev/null
}

# full publish: github latest.json must have mac + windows (source of truth)
assert_github_manifest_complete() {
  local tag="$1"
  local dir i
  dir="$(mktemp -d "${TMPDIR:-/tmp}/dropslim-manifest.XXXXXX")"
  trap 'rm -rf "$dir"' EXIT
  for i in 1 2 3 4 5; do
    gh release download "$tag" -p latest.json -D "$dir" --repo "$REPO" --clobber ||
      die "could not download latest.json from $tag"
    log "release manifest platforms: $(jq -c '.platforms | keys' "$dir/latest.json")"
    if manifest_has_required_platforms "$dir/latest.json"; then
      rm -rf "$dir"
      trap - EXIT
      return 0
    fi
    sleep 2
  done
  die "full publish needs darwin-aarch64 and windows-x86_64 in $tag latest.json — draft remains"
}

upload_and_merge_mac() {
  local tag="$1"
  local version="$2"
  local dmg win_dir platforms archive
  local dmgs=()

  dmg="dist/DropSlim_${version}_aarch64.dmg"
  if [[ ! -f "$dmg" ]]; then
    shopt -s nullglob
    dmgs=(dist/DropSlim_*_aarch64.dmg)
    shopt -u nullglob
    dmg="${dmgs[0]:-}"
  fi
  [[ -n "$dmg" && -f "$dmg" ]] || die "missing mac dmg in dist/"
  [[ -f dist/latest.json ]] || die "missing dist/latest.json"

  archive="$(jq -r '.platforms["darwin-aarch64"].url // empty' dist/latest.json)"
  archive="$(basename "$archive")"
  [[ -n "$archive" && -f "dist/$archive" ]] ||
    die "missing updater archive dist/$archive (from latest.json)"
  [[ -f "dist/${archive}.sig" ]] || die "missing dist/${archive}.sig"

  log "uploading macOS assets"
  gh release upload "$tag" "$dmg" --repo "$REPO" --clobber
  gh release upload "$tag" "dist/$archive" "dist/${archive}.sig" --repo "$REPO" --clobber

  log "merging latest.json"
  win_dir="$(mktemp -d "${TMPDIR:-/tmp}/dropslim-win.XXXXXX")"
  trap 'rm -rf "$win_dir"' EXIT
  gh release download "$tag" -p latest.json -D "$win_dir" --repo "$REPO" --clobber
  node scripts/merge-latest-json.mjs dist/latest.json "$win_dir/latest.json" dist/latest.json
  gh release upload "$tag" dist/latest.json --clobber --repo "$REPO"
  rm -rf "$win_dir"
  trap - EXIT

  platforms="$(jq -c '.platforms | keys' dist/latest.json)"
  log "merged platforms: $platforms"
  manifest_has_required_platforms dist/latest.json ||
    die "merged latest.json missing required platforms"
}

run_interactive() {
  local current base slug suggested mode_answer version_answer summary_mode skip_mac_answer
  current="$(read_version)"
  branch="$(git rev-parse --abbrev-ref HEAD)"
  [[ "$branch" != "HEAD" ]] || die "detached HEAD is not supported"
  [[ -z "$(git status --porcelain)" ]] ||
    die "working tree is dirty — stash or commit wip first (e.g. git stash push -u)"

  log "on branch $branch"
  log "Enter accepts the value in [brackets]"

  if [[ "$branch" == "main" ]]; then
    version_answer="$(ask "version (enter = continue with ${current})" "$current")"
    is_valid_version "$version_answer" || die "invalid version: $version_answer"
    if [[ "$version_answer" == "$current" ]]; then
      CONTINUE=1
      BUMP_VERSION=""
    else
      CONTINUE=0
      BUMP_VERSION="$version_answer"
    fi

    mode_answer="$(ask "mode: full (latest) / draft" "full")"
    case "$mode_answer" in
      full | f) MODE="full" ;;
      draft | d) MODE="draft" ;;
      *) die "unknown mode: $mode_answer (use full or draft)" ;;
    esac
    summary_mode="$MODE"
  else
    base="$(base_semver "$current")"
    if [[ "$current" != *-* ]]; then
      base="$(increment_patch "$base")"
    fi
    base="$(ask "base version for test release" "$base")"
    [[ "$base" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "base version must be x.y.z (got: $base)"
    slug="$(branch_slug "$branch")"
    [[ -n "$slug" ]] || die "could not derive branch slug from: $branch"
    suggested="$(next_branch_version "$base" "$slug")"
    version_answer="$(ask "test version (always draft)" "$suggested")"
    is_valid_version "$version_answer" || die "invalid version: $version_answer"
    BUMP_VERSION="$version_answer"
    CONTINUE=0
    MODE="draft"
    summary_mode="draft (feature branch — never latest)"
  fi

  skip_mac_answer="$(ask "skip mac build? [y/N]" "N")"
  if [[ "$skip_mac_answer" == "y" || "$skip_mac_answer" == "Y" ]]; then
    SKIP_MAC=1
  fi

  printf '\n'
  log "summary"
  log "  branch:  $branch"
  if [[ -n "$BUMP_VERSION" ]]; then
    log "  version: $BUMP_VERSION (bump from $current)"
  else
    log "  version: $current (no bump)"
  fi
  log "  mode:    $summary_mode"
  log "  mac:     $([[ "$SKIP_MAC" -eq 1 ]] && echo skip || echo build+upload)"
  printf '\n'

  if [[ "$MODE" == "full" ]]; then
    log "this is a FULL release (GitHub latest + in-app updater)"
    confirm "are you sure — start full release?" || die "aborted"
  else
    confirm "start draft release?" || die "aborted"
  fi
}

# --- args -------------------------------------------------------------------

if [[ $# -eq 0 ]]; then
  need_cmd git
  need_cmd node
  need_cmd gh
  run_interactive
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --continue)
      CONTINUE=1
      shift
      ;;
    --skip-mac)
      SKIP_MAC=1
      shift
      ;;
    --draft-only)
      MODE="draft"
      shift
      ;;
    --yes)
      CLI_YES=1
      shift
      ;;
    --skip-ci-wait)
      SKIP_CI_WAIT=1
      shift
      ;;
    -h | --help)
      usage 0
      ;;
    -*)
      die "unknown argument: $1 (see --help)"
      ;;
    *)
      [[ -z "$BUMP_VERSION" ]] || die "version already set ($BUMP_VERSION); unexpected: $1"
      is_valid_version "$1" || die "invalid version: $1 (expected e.g. 1.6.2)"
      BUMP_VERSION="$1"
      shift
      ;;
  esac
done

[[ -n "$BUMP_VERSION" && "$CONTINUE" -eq 1 ]] &&
  die "use either a version or --continue, not both"
[[ -n "$BUMP_VERSION" || "$CONTINUE" -eq 1 ]] ||
  die "pass a version (e.g. 1.6.2), --continue, or run without args for interactive"

need_cmd git
need_cmd gh
need_cmd jq
need_cmd npm
need_cmd node

[[ "$(uname -s)" == "Darwin" ]] || [[ "$SKIP_MAC" -eq 1 ]] ||
  die "macOS build requires Darwin (use --skip-mac on other hosts)"

# --- guards -----------------------------------------------------------------

assert_origin_repo
assert_gh_user

[[ -z "$(git status --porcelain)" ]] ||
  die "working tree is dirty — stash or commit wip first (e.g. git stash push -u)"

branch="$(git rev-parse --abbrev-ref HEAD)"
[[ "$branch" != "HEAD" ]] || die "detached HEAD is not supported"

if [[ "$branch" != "main" ]]; then
  MODE="draft"
  log "non-main branch ($branch) — forcing draft-only test release"
fi

git fetch origin "$branch" 2>/dev/null || git fetch origin
local_sha="$(git rev-parse HEAD)"
if ! git rev-parse --verify "origin/$branch" >/dev/null 2>&1; then
  die "origin/$branch missing — push the branch first: git push -u origin HEAD"
fi
[[ "$local_sha" == "$(git rev-parse "origin/$branch")" ]] ||
  die "$branch is not in sync with origin/$branch (pull/push first)"

# --- 1. version bump --------------------------------------------------------

if [[ -n "$BUMP_VERSION" ]]; then
  bump_version "$BUMP_VERSION"
else
  log "continue with package.json version $(read_version)"
  wait_for_ci "$(git rev-parse HEAD)"
fi

VERSION="$(read_version)"
TAG="v${VERSION}"
log "releasing $TAG (mode=$MODE, branch=$branch)"

# --- 2. windows publish workflow --------------------------------------------

if gh release view "$TAG" --repo "$REPO" >/dev/null 2>&1; then
  release_is_draft "$TAG" ||
    die "$TAG is already published — bump the version or delete the github release first"
  log "draft $TAG already exists — skipping workflow_dispatch"
else
  log "starting publish workflow on ref $branch"
  gh workflow run publish.yml --repo "$REPO" --ref "$branch"
  wait_for_publish_workflow "$(git rev-parse HEAD)"
fi

wait_for_draft_release "$TAG"
if [[ "$branch" != "main" || "$VERSION" == *-* ]]; then
  gh release edit "$TAG" --repo "$REPO" --prerelease >/dev/null
  log "marked $TAG as prerelease"
fi
log "draft release ready: $(gh release view "$TAG" --repo "$REPO" --json url -q .url)"

# --- 3-5. mac build, upload, merge ------------------------------------------

if [[ "$SKIP_MAC" -eq 1 ]]; then
  log "skipping macOS build/upload (--skip-mac)"
else
  log "macOS release build (notarization may take several minutes)"
  bash "$root/scripts/build.sh"
  upload_and_merge_mac "$TAG" "$VERSION"
fi

# --- 6. publish or keep draft -----------------------------------------------

if [[ "$MODE" == "draft" ]]; then
  log "leaving $TAG as draft"
  log "done"
  exit 0
fi

assert_github_manifest_complete "$TAG"

if [[ "$SKIP_MAC" -eq 1 ]]; then
  log "WARNING: skip-mac + full will not update updater/latest.json"
fi

confirm_full_publish "$VERSION" || die "aborted before publish — draft remains"

gh release edit "$TAG" --repo "$REPO" --draft=false
if [[ "$SKIP_MAC" -eq 0 ]]; then
  sync_updater_manifest "$VERSION"
fi

log "published: https://github.com/$REPO/releases/tag/$TAG"
log "done — check dropslim.app downloads and in-app updater"
