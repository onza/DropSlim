import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const tauriDir = path.join(root, 'src-tauri')
const target = path.join(tauriDir, 'resources')
const gifsicleBinary =
  process.platform === 'win32' ? 'gifsicle.exe' : 'gifsicle'
const gifsicleSource = path.join(root, 'vendor', 'gifsicle', gifsicleBinary)
const gifsicleTarget = path.join(target, 'vendor', 'gifsicle', gifsicleBinary)

const releaseBuild = process.env.DROPSLIM_RELEASE === '1'
const signingIdentity = releaseBuild
  ? process.env.APPLE_SIGNING_IDENTITY || process.env.CODESIGN_IDENTITY || ''
  : '-'

const isReleaseSign = Boolean(signingIdentity && signingIdentity !== '-')

// .gitkeep satisfies tauri resources/**/* when gifsicle is skipped
// todo: copy real gifsicle.exe for windows ci and release
if (process.env.CI_SKIP_GIFSICLE === '1') {
  fs.rmSync(target, { recursive: true, force: true })
  const keep = path.join(target, 'vendor', 'gifsicle', '.gitkeep')
  fs.mkdirSync(path.dirname(keep), { recursive: true })
  fs.writeFileSync(keep, '')
  console.log('prepare-resources: skipped gifsicle (CI_SKIP_GIFSICLE)')
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
fs.mkdirSync(path.dirname(gifsicleTarget), { recursive: true })

if (!fs.existsSync(gifsicleSource)) {
  console.error(`prepare-resources: gifsicle missing at ${gifsicleSource}`)
  console.error('prepare-resources: run npm ci first')
  process.exit(1)
}

fs.copyFileSync(gifsicleSource, gifsicleTarget)

if (process.platform !== 'win32') {
  fs.chmodSync(gifsicleTarget, 0o755)
}

if (process.platform === 'darwin') {
  console.log(
    `prepare-resources: signing gifsicle (${isReleaseSign ? signingIdentity : 'ad-hoc'})`
  )
  signBinary(gifsicleTarget)
} else {
  console.log('prepare-resources: signing skipped (not macOS)')
}

if (process.platform === 'win32') {
  const vcpkgRoot =
    process.env.VCPKG_INSTALLATION_ROOT || process.env.VCPKG_ROOT || ''
  const dav1dSource = vcpkgRoot
    ? path.join(vcpkgRoot, 'installed', 'x64-windows', 'bin', 'dav1d.dll')
    : ''
  const dav1dTarget = path.join(tauriDir, 'dav1d.dll')

  if (!dav1dSource || !fs.existsSync(dav1dSource)) {
    console.error('prepare-resources: dav1d.dll missing for Windows bundle')
    console.error(
      'prepare-resources: install with vcpkg install dav1d:x64-windows and set VCPKG_INSTALLATION_ROOT'
    )
    process.exit(1)
  }

  fs.copyFileSync(dav1dSource, dav1dTarget)
  console.log(`prepare-resources: bundled dav1d.dll from ${dav1dSource}`)
}

console.log(`prepare-resources: ok (${target})`)
