import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const target = path.join(root, 'src-tauri', 'resources')
const gifsicleSource = path.join(root, 'vendor', 'gifsicle', 'gifsicle')
const gifsicleTarget = path.join(target, 'vendor', 'gifsicle', 'gifsicle')

const releaseBuild = process.env.DROPSLIM_RELEASE === '1'
const signingIdentity = releaseBuild
  ? process.env.APPLE_SIGNING_IDENTITY || process.env.CODESIGN_IDENTITY || ''
  : '-'

const isReleaseSign = Boolean(signingIdentity && signingIdentity !== '-')

const signBinary = (filePath) => {
  const args = ['--force', '--sign', signingIdentity]

  if (isReleaseSign) {
    args.push('--options', 'runtime', '--timestamp')
  }

  args.push(filePath)
  execFileSync('codesign', args, { stdio: 'inherit' })
}

fs.rmSync(target, { recursive: true, force: true })
fs.mkdirSync(path.dirname(gifsicleTarget), { recursive: true })

if (!fs.existsSync(gifsicleSource)) {
  console.error(`prepare-resources: gifsicle missing at ${gifsicleSource}`)
  console.error('prepare-resources: run npm ci first')
  process.exit(1)
}

fs.copyFileSync(gifsicleSource, gifsicleTarget)
fs.chmodSync(gifsicleTarget, 0o755)

if (process.platform === 'darwin') {
  console.log(
    `prepare-resources: signing gifsicle (${isReleaseSign ? signingIdentity : 'ad-hoc'})`
  )
  signBinary(gifsicleTarget)
} else {
  console.log('prepare-resources: signing skipped (not macOS)')
}

console.log(`prepare-resources: ok (${target})`)
