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

const vcRuntimeDlls = ['vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll']

const findVcRuntimeDir = () => {
  const directCandidates = [
    process.env.VCToolsRedistDir &&
      path.join(process.env.VCToolsRedistDir, 'x64', 'Microsoft.VC143.CRT'),
    process.env.VCToolsRedistDir &&
      path.join(process.env.VCToolsRedistDir, 'x64', 'Microsoft.VC142.CRT'),
  ].filter(Boolean)

  for (const candidate of directCandidates) {
    if (fs.existsSync(path.join(candidate, 'vcruntime140.dll'))) {
      return candidate
    }
  }

  const redistRoots = [
    'C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise\\VC\\Redist\\MSVC',
    'C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\VC\\Redist\\MSVC',
    'C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Redist\\MSVC',
    'C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Redist\\MSVC',
  ]

  for (const root of redistRoots) {
    if (!fs.existsSync(root)) {
      continue
    }

    const versions = fs
      .readdirSync(root, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort()
      .reverse()

    for (const version of versions) {
      const archRoot = path.join(root, version, 'x64')
      if (!fs.existsSync(archRoot)) {
        continue
      }

      const crtDir = fs
        .readdirSync(archRoot, { withFileTypes: true })
        .find(
          (entry) =>
            entry.isDirectory() && entry.name.startsWith('Microsoft.VC')
        )?.name

      if (!crtDir) {
        continue
      }

      const candidate = path.join(archRoot, crtDir)
      if (fs.existsSync(path.join(candidate, 'vcruntime140.dll'))) {
        return candidate
      }
    }
  }

  return null
}

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

  const vcRuntimeDir = findVcRuntimeDir()
  if (!vcRuntimeDir) {
    console.error('prepare-resources: Visual C++ runtime DLLs not found')
    console.error(
      'prepare-resources: install Visual Studio Build Tools or set VCToolsRedistDir'
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
    console.log(`prepare-resources: bundled ${dll} from ${source}`)
  }
}

// .gitkeep satisfies tauri resources/**/* when gifsicle is skipped
// windows still needs dav1d.dll — tauri.windows.conf.json lists it as a bundle resource
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

copyWindowsNativeDeps()

console.log(`prepare-resources: ok (${target})`)
