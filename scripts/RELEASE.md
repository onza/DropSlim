# Release checklist

## Order matters

```
push version > Windows Actions > Mac build > upload Mac > merge latest.json > publish
```

Mac build can run while Windows Actions is running. **Upload** only after the draft release exists.

## 1. Version bump

```bash
# Edit package.json > "version": "x.y.z"
npm run version
git add package.json src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "release: bump version to x.y.z"
git push origin main
```

## 2. Windows build (GitHub Actions)

1. GitHub > **Actions** > **publish** > **Run workflow** (branch `main`)
2. Wait until green
3. Check draft release exists:

```bash
gh release view vX.Y.Z --repo onza/DropSlim
```

Expected assets: `DropSlim_X.Y.Z_x64-setup.exe`, `.sig`, `latest.json`.

## 3. macOS build (local)

Requires `.release.env` (see header in `scripts/build.sh`).

```bash
git pull origin main
bash scripts/build.sh
```

Output in `dist/`:

- `DropSlim_X.Y.Z_aarch64.dmg`
- `DropSlim.app.tar.gz` + `.sig`
- `latest.json` (macOS only)

## 4. Upload macOS assets

```bash
TAG=vX.Y.Z
gh release upload "$TAG" dist/DropSlim_*_aarch64.dmg
gh release upload "$TAG" dist/DropSlim.app.tar.gz dist/DropSlim.app.tar.gz.sig
```

## 5. Merge updater manifest

```bash
TAG=vX.Y.Z
gh release download "$TAG" -p latest.json -D /tmp/dropslim-win
node scripts/merge-latest-json.mjs dist/latest.json /tmp/dropslim-win/latest.json
gh release upload "$TAG" dist/latest.json --clobber
```

The merge script also writes `updater/latest.json` (used by jsDelivr / raw.githubusercontent endpoints). Commit and push it:

```bash
git add updater/latest.json
git commit -m "chore: sync updater latest.json for x.y.z"
git push origin main
```

Verify both platforms:

```bash
curl -fsSL "https://github.com/onza/DropSlim/releases/download/$TAG/latest.json" | jq '.platforms | keys'
# > ["darwin-aarch64", "windows-x86_64", "windows-x86_64-nsis"]
curl -fsSL "https://raw.githubusercontent.com/onza/DropSlim/main/updater/latest.json" | jq '.version'
```

## 6. Publish release

```bash
gh release edit "$TAG" --draft=false
```

Optional release notes:

```bash
gh release edit "$TAG" --notes "…"
```

## 7. Checklist after publish

Go through these after step 6 (`gh release edit --draft=false`):

- Website redeploys automatically (`trigger-website.yml` > `DropSlim_Website`)
- Check [dropslim.app](https://dropslim.app/) download links and version
- Optional: update fallback in `DropSlim_Website` > `src/_data/site.js`
- macOS DMG and Windows installer
- In-app updater on both platforms (should show up to date on fresh install)
