import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { GIFSICLE_DLLS } from './windows-native-deps.mjs'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const tauriDir = path.join(root, 'src-tauri')
const target = path.join(tauriDir, 'resources')
const gifsicleBinary =
  process.platform === 'win32' ? 'gifsicle.exe' : 'gifsicle'
const gifsicleSourceDir = path.join(root, 'vendor', 'gifsicle')
const gifsicleTargetDir = path.join(target, 'vendor', 'gifsicle')
const gifsicleTarget = path.join(gifsicleTargetDir, gifsicleBinary)

const gifsicleSource = path.join(gifsicleSourceDir, gifsicleBinary)

const copyGifsicleBundle = () => {
  fs.rmSync(gifsicleTargetDir, { recursive: true, force: true })
  fs.mkdirSync(gifsicleTargetDir, { recursive: true })

  if (!fs.existsSync(gifsicleSource)) {
    console.error(`prepare-resources: gifsicle missing at ${gifsicleSource}`)
    console.error('prepare-resources: run npm ci first')
    process.exit(1)
  }

  for (const entry of fs.readdirSync(gifsicleSourceDir)) {
    const source = path.join(gifsicleSourceDir, entry)
    if (!fs.statSync(source).isFile()) {
      continue
    }

    const dest = path.join(gifsicleTargetDir, entry)
    fs.copyFileSync(source, dest)
    if (process.platform !== 'win32' && entry === gifsicleBinary) {
      fs.chmodSync(dest, 0o755)
    }
  }

  if (process.platform === 'win32') {
    for (const dll of GIFSICLE_DLLS) {
      if (!fs.existsSync(path.join(gifsicleTargetDir, dll))) {
        console.error(`prepare-resources: ${dll} missing next to gifsicle`)
        process.exit(1)
      }
    }
  }
}

const releaseBuild = process.env.DROPSLIM_RELEASE === '1'
const signingIdentity = releaseBuild
  ? process.env.APPLE_SIGNING_IDENTITY || process.env.CODESIGN_IDENTITY || ''
  : '-'

const isReleaseSign = Boolean(signingIdentity && signingIdentity !== '-')

const vcRuntimeDlls = ['vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll']

const copyWindowsNativeDeps = () => {
  if (process.platform !== 'win32') {
    return
  }

  const vcpkgRoot =
    process.env.VCPKG_INSTALLATION_ROOT || process.env.VCPKG_ROOT || ''
  const dav1dSource = vcpkgRoot
    ? path.join(vcpkgRoot, 'installed', 'x64-windows', 'bin', 'dav1d.dll')
    : ''

  if (!dav1dSource || !fs.existsSync(dav1dSource)) {
    console.error('prepare-resources: dav1d.dll missing for Windows bundle')
    console.error(
      'prepare-resources: install with vcpkg install dav1d:x64-windows and set VCPKG_INSTALLATION_ROOT'
    )
    process.exit(1)
  }

  fs.copyFileSync(dav1dSource, path.join(tauriDir, 'dav1d.dll'))
  console.log(`prepare-resources: bundled dav1d.dll from ${dav1dSource}`)

  const vcRuntimeDir = process.env.DROPSLIM_VC_RUNTIME_DIR
  if (
    !vcRuntimeDir ||
    !fs.existsSync(path.join(vcRuntimeDir, 'vcruntime140.dll'))
  ) {
    console.error(
      'prepare-resources: DROPSLIM_VC_RUNTIME_DIR not set or invalid'
    )
    console.error(
      'prepare-resources: run scripts/set-vc-runtime-env.ps1 on Windows CI'
    )
    process.exit(1)
  }

  for (const dll of vcRuntimeDlls) {
    const source = path.join(vcRuntimeDir, dll)
    if (!fs.existsSync(source)) {
      console.error(`prepare-resources: ${dll} missing in ${vcRuntimeDir}`)
      process.exit(1)
    }

    fs.copyFileSync(source, path.join(tauriDir, dll))
    console.log(`prepare-resources: bundled ${dll}`)
  }
}

// .gitkeep satisfies tauri resources/**/* when gifsicle is skipped
// windows still needs dav1d.dll next to the exe — copy-windows-bundle-dlls.mjs runs before nsis
if (process.env.CI_SKIP_GIFSICLE === '1') {
  fs.rmSync(target, { recursive: true, force: true })
  const keep = path.join(target, 'vendor', 'gifsicle', '.gitkeep')
  fs.mkdirSync(path.dirname(keep), { recursive: true })
  fs.writeFileSync(keep, '')
  console.log('prepare-resources: skipped gifsicle (CI_SKIP_GIFSICLE)')
  copyWindowsNativeDeps()
  process.exit(0)
}

const signBinary = (filePath) => {
  const args = ['--force', '--sign', signingIdentity]

  if (isReleaseSign) {
    args.push(
      '--options',
      'runtime',
      '--timestamp=http://timestamp.apple.com/ts01'
    )
  }

  args.push(filePath)

  if (isReleaseSign) {
    console.log(
      'prepare-resources: signing gifsicle (keychain dialog may appear — choose Always Allow)'
    )
  }

  const result = spawnSync('codesign', args, {
    stdio: 'inherit',
    timeout: 120_000,
  })

  if (result.error?.code === 'ETIMEDOUT') {
    console.error(
      'prepare-resources: codesign timed out after 2 minutes (keychain or network)'
    )
    process.exit(1)
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1)
  }
}

fs.rmSync(target, { recursive: true, force: true })
copyGifsicleBundle()

if (process.platform === 'darwin') {
  console.log(
    `prepare-resources: signing gifsicle (${isReleaseSign ? signingIdentity : 'ad-hoc'})`
  )
  signBinary(gifsicleTarget)
} else {
  console.log('prepare-resources: signing skipped (not macOS)')
}

copyWindowsNativeDeps()

console.log(`prepare-resources: ok (${target})`)
