import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  GIFSICLE_DLLS,
  MAIN_PROCESS_DLLS,
  verifyGifsicleBundle,
  verifyMainProcessDlls,
} from './windows-native-deps.mjs'

const prefix = 'verify-windows-bundle'
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const tauriDir = path.join(root, 'src-tauri')
const releaseDir = path.join(tauriDir, 'target', 'release')
const gifsicleDir = path.join(tauriDir, 'resources', 'vendor', 'gifsicle')

if (process.platform !== 'win32') {
  console.log(`${prefix}: skipped (not Windows)`)
  process.exit(0)
}

verifyMainProcessDlls(releaseDir, { prefix })

if (process.env.CI_SKIP_GIFSICLE !== '1') {
  verifyGifsicleBundle(gifsicleDir, prefix)
} else {
  console.log(`${prefix}: gifsicle check skipped (CI_SKIP_GIFSICLE)`)
}

const nsisDir = path.join(releaseDir, 'bundle', 'nsis')
if (fs.existsSync(nsisDir)) {
  const installers = fs
    .readdirSync(nsisDir)
    .filter((name) => name.endsWith('.exe') && name.includes('setup'))
  if (installers.length === 0) {
    console.error(`${prefix}: no NSIS setup exe under ${nsisDir}`)
    process.exit(1)
  }
  console.log(`${prefix}: found ${installers.join(', ')}`)
}

console.log(
  `${prefix}: ok (main: ${MAIN_PROCESS_DLLS.join(', ')}; gifsicle: ${GIFSICLE_DLLS.join(', ')})`
)
