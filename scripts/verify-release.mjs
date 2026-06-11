import crypto from 'node:crypto'
import { execFileSync, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const projectRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '..'
)

const appPath = process.argv[2]

if (!appPath) {
  console.error('Usage: node scripts/verify-release.mjs /path/to/DropSlim.app')
  process.exit(1)
}

const fail = (message) => {
  console.error(`verify-release: ${message}`)
  process.exit(1)
}

const isExecutable = (filePath) => {
  try {
    fs.accessSync(filePath, fs.constants.X_OK)
    return true
  } catch {
    return false
  }
}

const executable = path.resolve(appPath, 'Contents/MacOS/dropslim')
const resources = path.join(appPath, 'Contents/Resources/resources')
const gifsicle = path.join(resources, 'vendor', 'gifsicle', 'gifsicle')
const quickAction = path.join(
  resources,
  'build',
  'Optimize with DropSlim.workflow',
  'Contents',
  'document.wflow'
)

if (!fs.existsSync(appPath)) {
  fail(`app bundle not found: ${appPath}`)
}

if (!fs.existsSync(executable)) {
  fail(`main executable missing: ${executable}`)
}

if (!isExecutable(executable)) {
  fail(`main executable is not marked executable: ${executable}`)
}

if (!fs.existsSync(gifsicle)) {
  fail(`gifsicle binary missing: ${gifsicle}`)
}

if (!isExecutable(gifsicle)) {
  fail(`gifsicle is not marked executable: ${gifsicle}`)
}

if (!fs.existsSync(quickAction)) {
  fail(`Quick Action workflow missing: ${quickAction}`)
}

try {
  execFileSync('codesign', ['--verify', '--deep', '--strict', appPath], {
    stdio: 'pipe',
  })
} catch {
  fail(
    'app bundle has an invalid code signature (macOS may report the app as damaged)'
  )
}

const { stderr: codesignInfo } = spawnSync('codesign', ['-dvvv', executable], {
  encoding: 'utf8',
})
if (!/runtime/.test(codesignInfo ?? '')) {
  fail(
    'main executable is missing hardened runtime (app may crash on recent macOS)'
  )
}

const bundledIcon = path.join(appPath, 'Contents/Resources/icon.icns')
const expectedIcon = path.join(projectRoot, 'src-tauri/icons/icon.icns')

if (!fs.existsSync(bundledIcon)) {
  fail(`bundle icon.icns missing: ${bundledIcon}`)
}

if (fs.existsSync(expectedIcon)) {
  const hash = (filePath) =>
    crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex')

  if (hash(expectedIcon) !== hash(bundledIcon)) {
    fail('bundle icon.icns does not match src-tauri/icons/icon.icns')
  }
}

console.log(`verify-release: ok (${appPath})`)
