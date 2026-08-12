import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const [basePath, windowsPath, outPathArg] = process.argv.slice(2)

if (!basePath || !windowsPath) {
  console.error(
    'usage: node scripts/merge-latest-json.mjs <mac-latest.json> <windows-latest.json> [output]'
  )
  process.exit(1)
}

const outPath = outPathArg ?? basePath
const repoRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  '..'
)
const repoUpdaterPath = path.join(repoRoot, 'updater', 'latest.json')

const readJson = (filePath) => {
  if (!fs.existsSync(filePath)) {
    console.error(`merge-latest-json: file not found: ${filePath}`)
    process.exit(1)
  }

  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

const base = readJson(path.resolve(basePath))
const windows = readJson(path.resolve(windowsPath))

if (!base.platforms?.['darwin-aarch64']) {
  console.error('merge-latest-json: mac manifest is missing darwin-aarch64')
  process.exit(1)
}

const windowsEntry =
  windows.platforms?.['windows-x86_64'] ??
  windows.platforms?.['windows-x86_64-nsis']

if (!windowsEntry) {
  console.error('merge-latest-json: windows manifest is missing windows-x86_64')
  process.exit(1)
}

const manifest = {
  version: base.version,
  notes: base.notes ?? `DropSlim ${base.version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    ...base.platforms,
    'windows-x86_64': windowsEntry,
    'windows-x86_64-nsis': windowsEntry,
  },
}

const body = `${JSON.stringify(manifest, null, 2)}\n`

fs.mkdirSync(path.dirname(path.resolve(outPath)), { recursive: true })
fs.writeFileSync(outPath, body)

fs.mkdirSync(path.dirname(repoUpdaterPath), { recursive: true })
fs.writeFileSync(repoUpdaterPath, body)

console.log(`merge-latest-json: ok (${outPath})`)
console.log(`merge-latest-json: mirrored (${repoUpdaterPath})`)
console.log(
  `merge-latest-json: platforms: ${Object.keys(manifest.platforms).join(', ')}`
)
