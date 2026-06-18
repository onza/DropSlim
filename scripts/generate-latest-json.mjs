import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const target =
  process.env.CARGO_TARGET_DIR ?? path.join(root, 'src-tauri', 'target')
const bundleRoot = path.join(target, 'release', 'bundle')
const repo = process.env.GITHUB_REPOSITORY ?? 'onza/DropSlim'

const { version } = JSON.parse(
  fs.readFileSync(path.join(root, 'package.json'), 'utf8')
)
const tag = process.env.RELEASE_TAG ?? `v${version}`

const findFile = (dir, pattern) => {
  if (!fs.existsSync(dir)) {
    return null
  }

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name)

    if (entry.isDirectory()) {
      const nested = findFile(fullPath, pattern)
      if (nested) {
        return nested
      }
      continue
    }

    if (pattern.test(entry.name)) {
      return fullPath
    }
  }

  return null
}

const archive = findFile(bundleRoot, /\.app\.tar\.gz$/)
const signaturePath = archive ? `${archive}.sig` : null

if (!archive || !signaturePath || !fs.existsSync(signaturePath)) {
  console.error('generate-latest-json: updater bundle or signature not found')
  console.error(`generate-latest-json: looked under ${bundleRoot}`)
  process.exit(1)
}

const assetName = path.basename(archive)
const url = `https://github.com/${repo}/releases/download/${tag}/${assetName}`
const signature = fs.readFileSync(signaturePath, 'utf8').trim()

const manifest = {
  version,
  notes: `DropSlim ${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    'darwin-aarch64': {
      url,
      signature,
    },
  },
}

const distDir = path.join(root, 'dist')
fs.mkdirSync(distDir, { recursive: true })

const manifestPath = path.join(distDir, 'latest.json')
fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)

console.log(`generate-latest-json: ok (${manifestPath})`)
