# Homebrew (CLI)

Formula lives here. Users install from **`onza/homebrew-tap`**, not from this repo.

<br>

## Users

```bash
brew tap onza/tap
brew install dropslim
```

<br>

## Tap (once)

1. Create GitHub repo **`onza/homebrew-tap`** (clone dir: `homebrew-tap`).
2. Copy `Formula/dropslim.rb` from here into that repo.
3. Push.

<br>

## Each CLI release

Edit `Formula/dropslim.rb`:

- `url` — asset `dropslim-cli_X.Y.Z_aarch64.tar.gz` on the GitHub release
- `sha256` — `npm run package:cli` prints `homebrew sha256: …` at the end
- `version`

Tarball already includes gifsicle under `vendor/` — no `depends_on "gifsicle"`.

Copy the updated formula to `homebrew-tap` and push.

<br>

## Smoke test (local)

Homebrew only installs from a git tap:

```bash
cp -R homebrew /tmp/homebrew-tap-test
git -C /tmp/homebrew-tap-test init && git -C /tmp/homebrew-tap-test add . && git -C /tmp/homebrew-tap-test commit -m init
brew tap onza/dropslim-test file:///tmp/homebrew-tap-test
brew trust onza/dropslim-test
brew install dropslim && dropslim --version
brew untap onza/dropslim-test && rm -rf /tmp/homebrew-tap-test
```
