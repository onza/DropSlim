import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const target = path.join(root, 'src-tauri', 'resources')
const gifsicleBinary =
  process.platform === 'win32' ? 'gifsicle.exe' : 'gifsicle'
const gifsicleSource = path.join(root, 'vendor', 'gifsicle', gifsicleBinary)
const gifsicleTarget = path.join(target, 'vendor', 'gifsicle', gifsicleBinary)

const releaseBuild = process.env.DROPSLIM_RELEASE === '1'
const signingIdentity = releaseBuild
  ? process.env.APPLE_SIGNING_IDENTITY || process.env.CODESIGN_IDENTITY || ''
  : '-'

const isReleaseSign = Boolean(signingIdentity && signingIdentity !== '-')

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

console.log(`prepare-resources: ok (${target})`)
