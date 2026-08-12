/**
 * windows native runtime dependencies for dropslim.
 *
 * format audit (other platforms use static rust crates or os frameworks):
 *
 * | format   | read                         | write              | windows extra dlls              |
 * |----------|------------------------------|--------------------|---------------------------------|
 * | jpeg     | image (rust)                 | zenjpeg            | —                               |
 * | png      | image / oxipng               | oxipng             | —                               |
 * | webp     | image / webp                 | webp               | —                               |
 * | avif     | image + dav1d (dynamic)      | ravif / rav1e      | dav1d.dll next to dropslim.exe  |
 * | gif      | image (decode only)          | gifsicle subprocess| mingw dlls next to gifsicle.exe |
 * | svg      | oxvg (rust)                  | oxvg               | —                               |
 * | heic     | imageio (macos only)         | —                  | —                               |
 *
 * msvc runtime dlls are copied beside dropslim.exe when the linker does not
 * statically embed them (typical on ci). gif uses a separate process with
 * current_dir set to the gifsicle folder — see optimize/image.rs.
 *
 * macos: no steps in this module; dav1d comes from homebrew at link time,
 * heic from imageio. beforebundlecommand exists only in tauri.windows.conf.json.
 * nsis picks up dlls via bundle.resources pointing at target/release/* after copy.
 */

import fs from 'node:fs'
import path from 'node:path'

/** dlls that must sit next to dropslim.exe */
export const MAIN_PROCESS_DLLS = [
  'dav1d.dll',
  'vcruntime140.dll',
  'vcruntime140_1.dll',
  'msvcp140.dll',
]

/** dlls bundled next to gifsicle.exe in resources/vendor/gifsicle/. */
export const GIFSICLE_DLLS = ['libwinpthread-1.dll', 'libgcc_s_seh-1.dll']

const fail = (prefix, message) => {
  console.error(`${prefix}: ${message}`)
  process.exit(1)
}

export const verifyGifsicleBundle = (gifsicleDir, prefix = 'verify') => {
  const gifsicle = path.join(gifsicleDir, 'gifsicle.exe')

  if (!fs.existsSync(gifsicle)) {
    fail(prefix, `gifsicle.exe missing at ${gifsicle}`)
  }

  for (const dll of GIFSICLE_DLLS) {
    const dllPath = path.join(gifsicleDir, dll)
    if (!fs.existsSync(dllPath)) {
      fail(prefix, `${dll} missing next to gifsicle at ${gifsicleDir}`)
    }
  }
}

export const verifyMainProcessDlls = (
  releaseDir,
  { exeName = 'dropslim.exe', prefix = 'verify' } = {}
) => {
  const exe = path.join(releaseDir, exeName)

  if (!fs.existsSync(exe)) {
    fail(prefix, `${exeName} missing in ${releaseDir}`)
  }

  for (const dll of MAIN_PROCESS_DLLS) {
    const dllPath = path.join(releaseDir, dll)
    if (!fs.existsSync(dllPath)) {
      fail(
        prefix,
        `${dll} must be next to ${exeName} (required for AVIF decode on Windows)`
      )
    }
  }
}

export const copyMainProcessDlls = (sourceDir, releaseDir, prefix) => {
  for (const dll of MAIN_PROCESS_DLLS) {
    const source = path.join(sourceDir, dll)

    if (!fs.existsSync(source)) {
      fail(
        prefix,
        `${dll} missing in ${sourceDir} — run npm run prepare:tauri first`
      )
    }

    const dest = path.join(releaseDir, dll)
    fs.copyFileSync(source, dest)
    console.log(`${prefix}: ${dll} → target/release/`)
  }
}
