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

## Install (macOS)

Requires macOS 11 (Big Sur) or later and an Apple Silicon Mac.

1. Download **`DropSlim_*.dmg`** from **[GitHub Releases](https://github.com/onza/DropSlim/releases)**.
2. Open the DMG and drag **DropSlim** to **Applications**.
3. Open DropSlim from **Applications**.

**Updates:** Download the new DMG and drag DropSlim to Applications again.

<br>

## A Frustrating Side Note

DropSlim is open source and free. Nevertheless, in the Apple ecosystem, it apparently costs money just to download an app, drag it into the “Applications” folder, and open it without any further hassle.

That’s because Apple requires me to join its paid Developer Program, which costs 99 EUR per year. On top of that: identity verification, certificate signing requests, Developer ID certificates, app-specific passwords, authorizations, and a registration process that feels like the administration from “Asterix Conquers Rome” digitized the A38 pass to make breathing fresh air subject to approval.

None of this makes the app any better. It merely serves to satisfy Gatekeeper so that app users aren’t greeted with _“Apple couldn’t verify…”_ and a useless “Done” button, only to then have to click an “Open Anyway” button buried deep in the security settings.

This project covers that fee, so you don’t have to worry about it. If any developer knows a trick to bypass the Developer Program without compromising the installation process, I’d be very happy to hear from you :)

<br>

## Development

```bash
npm run dev
```

Window title: **DropSlim (Dev)**. Installs a separate Finder Quick Action **Optimize with DropSlim (Dev)** alongside the release Quick Action.

<br>

## Credits

Based on the concept of [Image Shrinker](https://github.com/stefansl/image-shrinker) (CC0-1.0) by Stefan Schulz-Lauterbach.

<br>

## License

[MIT](LICENSE.md)

Copyright (C) 2026-present, Martin Farkas.
