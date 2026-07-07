# DropSlim

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md) [![CI](https://github.com/onza/DropSlim/actions/workflows/ci.yml/badge.svg)](https://github.com/onza/DropSlim/actions/workflows/ci.yml)

Make large image files **small** — on your computer, with drag & drop. Local, fast, private, free, and open source.

Drop your images. DropSlim crunches the numbers. Done. Everything stays on your machine — no account, no server.

<br>

## Why DropSlim

- **Drag & drop** — drop files or folders on the window; whole folders are processed recursively
- **Small and fast** — lightweight app, fast compression; built with imagequant, oxipng, zenjpeg, OXVG, gifsicle, WebP, and AVIF encoders
- **Runs offline** — no internet required; compression runs entirely on your computer
- **Privacy** — no upload, no tracking; your images never leave your machine
- **Batch processing** — hundreds of files in one go; new file alongside the original, or replace in place
- **Common formats** — PNG, JPEG, GIF, SVG, WebP, and AVIF. **HEIC on macOS only**
- **Open with DropSlim** (macOS) — right-click an image in Finder → **Open With** → DropSlim
- **Review results** in a simple list and reveal outputs in Finder or File Explorer
- **Open source** — [MIT](LICENSE.md)
- **Languages** — English, German, French, Spanish, Italian, Japanese, Brazilian Portuguese

### Where the optimized file is saved depends on your settings

- **`.min` suffix on** (default): writes a new file next to the original, e.g. `photo.png` → `photo.min.png`. The source file stays untouched.
- **`.min` suffix off**: overwrites the original in place when saving in the same folder and **`minified` subfolder** is off. With the subfolder on, the optimized file goes into `minified/` under the original filename and the source stays untouched.
- **`minified` subfolder**: saves into a `minified/` folder (with or without `.min`, depending on the suffix setting).
- **Custom save folder**: turn off **Save optimized files in same folder** in Settings — **Choose folder** appears; click **Open** to pick a destination.

<br>

## Install (macOS)

Requires **`macOS 11 (Big Sur)`** or later and an **Apple Silicon** Mac (M1 or newer).

1. Download **`DropSlim_*.dmg`** from **[GitHub Releases](https://github.com/onza/DropSlim/releases)**.
2. Open the DMG and drag **DropSlim** to **Applications**.
3. Open DropSlim from **Applications**.

**Intel Mac?** Use [Image Shrinker](https://image-shrinker.com/) — same idea, native on Intel Macs.

<br>

## Install (Windows)

Requires **Windows 10 or later** (64-bit).

1. Download **`DropSlim_*_x64-setup.exe`** from **[GitHub Releases](https://github.com/onza/DropSlim/releases)**.
2. If your browser blocks the download, choose **Keep** or **Keep anyway**.
3. Run the installer.
4. If **Microsoft Defender SmartScreen** shows _"Windows protected your PC"_:
   - Click **More info**
   - Click **Run anyway**
5. Follow the setup wizard and start DropSlim from the Start menu or desktop shortcut.

<br>

## Linux (planned)

A Linux version is **planned but not released yet**. Work is happening on the [`feature/linux`](https://github.com/onza/DropSlim/tree/feature/linux) branch, where CI already builds an **AppImage** (x86_64). For now it is **on hold** because of limited testing capacity. If you'd like to help test, please [open an issue](https://github.com/onza/DropSlim/issues).

<br>

## Translations

UI translations were **generated with AI** and may contain errors or awkward wording. **Corrections are welcome** — if you spot a mistake, please [open an issue](https://github.com/onza/DropSlim/issues) or submit a pull request with an updated string in `ui/i18n/locales/`.

<br>

## A Frustrating Side Note

DropSlim is open source and free. Nevertheless, it appears that on both major desktop platforms, it costs developers money to allow users to simply download, install and open an app without any further hassle.

**On macOS**, Apple requires a paid Developer Program (99 EUR per year). On top of that: identity verification with a wait time of 5 days, certificate signing requests, Developer ID certificates, app-specific passwords, authorizations, and a registration process that feels like the administration from "Asterix Conquers Rome" digitized the A38 pass to make breathing fresh air subject to approval. This project covers that fee, so Mac users get a normal install.

**On Windows**, SmartScreen treats unsigned installers as suspicious. A commercial code-signing certificate (roughly 100+ EUR per year, from a certificate authority or cloud signing service) is the paid entry ticket to a smoother path; even then, reputation builds slowly and rarely helps a small open-source project. This project does not pay for Windows signing — use **More info** → **Run anyway** (steps above). The installer is still safe if you download it from this repository or the [project website](https://dropslim.app/).

None of this makes the app any better. It only buys a smoother install experience. If you know a legitimate way around either gate without compromising security, I’d be happy to hear from you :)

<br>

## Credits

Inspired by [Image Shrinker](https://github.com/stefansl/image-shrinker) (CC0-1.0) by Stefan Schulz-Lauterbach.

<br>

## License

[MIT](LICENSE.md)

Copyright (C) 2026-present, Martin Farkas.
