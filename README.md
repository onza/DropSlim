# DropSlim

A minimalist drag-and-drop desktop app for macOS to instantly shrink images.

Built with [Tauri](https://tauri.app/).

<br>

## What it does

- **Drop or open** PNG, JPG, GIF, SVG, WebP, and AVIF — including **whole folders** (processed recursively)
- **Compress** with imagequant, mozjpeg, OXVG, gifsicle, WebP, and AVIF encoders (Rust)
- **Review results** in a simple list and reveal outputs in Finder
- **Tune behavior** in Settings — custom save location, `.min` suffix, subfolder, and more
- **Finder Quick Action** (macOS) — optimize selected files or folders from Finder

### Where the optimized file is saved depends on your settings:

- **`.min` suffix on** (default): writes a new file next to the original, e.g. `photo.png` → `photo.min.png`. The source file stays untouched.
- **`.min` suffix off**: replaces the original file in place with the optimized version.
- **`minified` subfolder**: saves into a `minified/` folder (with or without `.min`, depending on the suffix setting).
- **Custom save folder**: turn off **Save optimized files in same folder** in Settings — **Choose folder** appears; click **Open** to pick a destination.

<br>

## Releases

GitHub Releases are built with [`tauri-apps/tauri-action`](https://github.com/tauri-apps/tauri-action) on tag push (`v*`). macOS Apple Silicon is active today; Intel/Linux/Windows matrix entries can be added later.

<br>

## Install (macOS)

Requires macOS 11 (Big Sur) or later and an Apple Silicon Mac.

1. Download **`DropSlim_*.dmg`** from **[GitHub Releases](https://github.com/onza/DropSlim/releases)** (not **Code → Download ZIP**).
2. Open the DMG and drag **DropSlim** to **Applications**.
3. Open DropSlim from Applications.

macOS may show a security notice the first time — DropSlim is not signed with an Apple Developer ID. After drag-install to Applications, either **right-click DropSlim → Open → Open** once, or run:

```bash
xattr -rd com.apple.quarantine /Applications/DropSlim.app && open -a DropSlim
```

**Finder Quick Action (optional):** In DropSlim, use **Settings → Install** or **Install Finder Quick Action** in the app menu.

**Updates:** Download the new DMG and drag DropSlim to Applications again.

<br>

## Development

Do **not** install the Finder Quick Action from a dev build if you keep a release copy in `/Applications`.

<br>

## Credits

Based on the concept of [Image Shrinker](https://github.com/stefansl/image-shrinker) (CC0-1.0) by Stefan Schulz-Lauterbach.

<br>

## License

[MIT](LICENSE.md)

Copyright (C) 2026-present, Martin Farkas.
