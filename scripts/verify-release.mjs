import crypto from 'node:crypto'
import { execFileSync, execSync, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const projectRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '..'
)
const targetDir =
  process.env.CARGO_TARGET_DIR ?? path.join(projectRoot, 'src-tauri/target')
const bundleMode = process.argv.includes('--bundle')
const appArg = process.argv.find(
  (arg) =>
    arg !== process.argv[0] && arg !== process.argv[1] && !arg.startsWith('-')
)

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

const findFirst = (pattern) => {
  try {
    return execSync(`find "${targetDir}" ${pattern} 2>/dev/null | head -1`, {
      encoding: 'utf8',
    }).trim()
  } catch {
    return ''
  }
}

const verifyApp = (appPath) => {
  const executable = path.resolve(appPath, 'Contents/MacOS/dropslim')
  const resources = path.join(appPath, 'Contents/Resources/resources')
  const gifsicle = path.join(resources, 'vendor', 'gifsicle', 'gifsicle')

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

  try {
    execFileSync('codesign', ['--verify', '--deep', '--strict', appPath], {
      stdio: 'pipe',
    })
  } catch {
    fail(
      'app bundle has an invalid code signature (macOS may report the app as damaged)'
    )
  }

  const { stderr: codesignInfo } = spawnSync(
    'codesign',
    ['-dvvv', executable],
    {
      encoding: 'utf8',
    }
  )
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
      crypto
        .createHash('sha256')
        .update(fs.readFileSync(filePath))
        .digest('hex')

    if (hash(expectedIcon) !== hash(bundledIcon)) {
      fail('bundle icon.icns does not match src-tauri/icons/icon.icns')
    }
  }

  console.log(`verify-release: ok (${appPath})`)
}

const verifyBundle = () => {
  let app = findFirst(
    '-type d -name DropSlim.app -path "*/release/bundle/macos/*"'
  )
  let mountPoint = ''

  const cleanup = () => {
    if (!mountPoint) {
      return
    }

    try {
      execSync(`hdiutil detach "${mountPoint}" -quiet`, { stdio: 'ignore' })
    } catch {
      // ignore detach errors during cleanup
    }

    fs.rmSync(mountPoint, { recursive: true, force: true })
    mountPoint = ''
  }

  try {
    if (!app) {
      const dmg = findFirst('-name "*.dmg" -path "*/release/bundle/dmg/*"')

      if (!dmg) {
        fail(`DropSlim.app and DMG not found under ${targetDir}`)
      }

      mountPoint = fs.mkdtempSync(path.join(os.tmpdir(), 'dropslim-verify.'))
      execSync(
        `hdiutil attach -readonly -nobrowse -mountpoint "${mountPoint}" "${dmg}"`,
        { stdio: 'ignore' }
      )

      const entries = fs.readdirSync(mountPoint, { withFileTypes: true })
      app = entries
        .filter((entry) => entry.isDirectory() && entry.name === 'DropSlim.app')
        .map((entry) => path.join(mountPoint, entry.name))[0]

      if (!app) {
        fail(`DropSlim.app missing inside ${dmg}`)
      }

      if (!fs.existsSync(path.join(mountPoint, 'Applications'))) {
        fail(`Applications drop link missing inside ${dmg}`)
      }

      console.error(`verify-release: verifying app from ${path.basename(dmg)}`)
    }

    verifyApp(app)
  } finally {
    cleanup()
  }
}

if (bundleMode) {
  verifyBundle()
} else if (!appArg) {
  fail('Usage: node scripts/verify-release.mjs /path/to/DropSlim.app')
} else {
  verifyApp(appArg)
}
