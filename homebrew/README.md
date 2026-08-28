# Homebrew (CLI)

Formula source in this repo. Published tap: **https://github.com/onza/homebrew-tap**

<br>

## Users

```bash
brew tap onza/tap
brew install dropslim
```

<br>

## Each CLI release

Edit `Formula/dropslim.rb` here, then copy to `onza/homebrew-tap` and push:

- `url` — asset `dropslim-cli_X.Y.Z_aarch64.tar.gz`
- `sha256` — `npm run package:cli` prints `homebrew sha256: …`
- `version`

Tarball includes gifsicle under `vendor/` — no `depends_on "gifsicle"`.
