# Release

Runs via **`bash scripts/release.sh`** on the Mac — notarize and updater key stay local. Don’t click around in the GitHub UI unless the script is broken.

<br>

## What happens

**Full on `main`:** GitHub latest + `updater/latest.json` on `main` + website (dropslim.app).

**Draft:** stays draft. No latest, no updater sync, no website.

| Branch         | Mode         | Version              | Live? |
| -------------- | ------------ | -------------------- | ----- |
| `main`         | full         | `x.y.z`              | yes   |
| `main`         | draft        | `x.y.z`              | no    |
| Feature branch | always draft | `x.y.z-<branch>.<n>` | no    |

Feature drafts are for pipeline testing only. Never latest.

<br>

## Before

- `git`, `gh`, `jq`, `node`, `npm`
- `gh` as **onza**, origin **onza/DropSlim**
- Working tree clean (stash WIP)
- `.release.env` in the repo root (header in `scripts/build.sh`)
- Test Apple: `bash scripts/verify-notarize-credentials.sh`

Windows signing comes from repo secrets. Apple stays on the Mac.

<br>

## Start

```bash
bash scripts/release.sh
```

Dialog: version, full/draft, Mac yes/no. Then it runs through; type the version for full.

Without dialog:

```bash
bash scripts/release.sh 1.6.2
bash scripts/release.sh 1.6.2 --draft-only
bash scripts/release.sh --continue          # version already in package.json
```

Feature branch: dialog suggests the next `x.y.z-<branch>.<n>` and forces draft.

<br>

### Flags

- `--continue` — no bump, wait for CI on the current commit
- `--draft-only` — leave as draft on `main`
- `--skip-mac` — skip the Mac build (resume if Mac is already uploaded)
- `--yes` — skip the start confirm; still type the version to undraft
- `--skip-ci-wait` — don’t wait for CI (usually don’t)

Undraft without typing:

```bash
DROPSLIM_RELEASE_YES=1 bash scripts/release.sh 1.6.2 --yes
```

<br>

## Flow

```
Bump → CI → Windows draft → Mac → upload/merge → undraft (full only)
```

1. **Bump** — `package.json`, `npm run version` (Cargo.toml), commit, push. Same version as already in the file → script refuses, use `--continue`.
2. **CI** — waits for the CI run. Feature branch without a PR to `main` often has no run; then it continues.
3. **Windows** — `publish.yml` on the current branch. Draft `v…` with NSIS. Draft already exists → resume. Tag already published → abort, new version.
4. **Mac** — `scripts/build.sh`, upload dmg + `DropSlim.app.tar.gz` + `.sig`, merge `latest.json`.
5. **Full** — release `latest.json` must have Mac **and** Windows, type the version, `--draft=false` (notes stay). Then commit `updater/latest.json`.

`--skip-mac` + full only works if the merged manifest is already on the release. Otherwise the draft stays. `--skip-mac` does not touch the updater file.

<br>

## If it blows up

Working tree must be clean again (bump is already on the remote).

- Stopped mid-way, draft exists: `bash scripts/release.sh --continue`
- Mac already up, only undraft left: `--continue --skip-mac` on `main`
- Tag already live: don’t reuse, next version

Dirty: `git stash push -u`, script, `git stash pop`.

`full publish needs darwin-aarch64 and windows-x86_64` → Mac merge missing. Run again without `--skip-mac`, or merge by hand, then `--continue --skip-mac`.

Notarize: `bash scripts/verify-notarize-credentials.sh`.

<br>

## After (full)

- https://github.com/onza/DropSlim/releases/latest
- dropslim.app should pick up the tag
- In-app updater: jsDelivr → raw `updater/latest.json` → GitHub latest asset

Apps only update if `updater/latest.json` on `main` is correct and signed with the production key.

<br>

## Emergency without the script

Same order:

1. Version in `package.json` → `npm run version` → commit/push
2. Wait for CI
3. Actions → publish on that branch → draft
4. `bash scripts/build.sh`
5. Upload dmg + `DropSlim.app.tar.gz` (+ `.sig`)
6. `node scripts/merge-latest-json.mjs dist/latest.json <windows-latest.json> dist/latest.json`  
   then `gh release upload vX.Y.Z dist/latest.json --clobber`
7. Full on main only: `gh release edit vX.Y.Z --draft=false`  
   plus commit `updater/latest.json`
