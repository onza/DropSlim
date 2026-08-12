import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  copyMainProcessDlls,
  verifyGifsicleBundle,
  verifyMainProcessDlls,
} from './windows-native-deps.mjs'

const prefix = 'copy-windows-bundle-dlls'
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const tauriDir = path.join(root, 'src-tauri')
const releaseDir = path.join(tauriDir, 'target', 'release')
const gifsicleDir = path.join(tauriDir, 'resources', 'vendor', 'gifsicle')

// no-op on mac - beforebundlecommand is only in tauri.windows.conf.json.
if (process.platform !== 'win32') {
  process.exit(0)
}

if (!fs.existsSync(releaseDir)) {
  console.error(`${prefix}: release dir missing: ${releaseDir}`)
  process.exit(1)
}

copyMainProcessDlls(tauriDir, releaseDir, prefix)
verifyMainProcessDlls(releaseDir, { prefix })

if (process.env.CI_SKIP_GIFSICLE !== '1') {
  verifyGifsicleBundle(gifsicleDir, prefix)
} else {
  console.log(`${prefix}: gifsicle check skipped (CI_SKIP_GIFSICLE)`)
}

console.log(`${prefix}: ok`)
