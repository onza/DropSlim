import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const packageJsonPath = path.join(root, 'package.json')
const cargoTomlPaths = [
  path.join(root, 'src-tauri', 'Cargo.toml'),
  path.join(root, 'crates', 'dropslim-core', 'Cargo.toml'),
  path.join(root, 'crates', 'dropslim-cli', 'Cargo.toml'),
]
const cargoLockPath = path.join(root, 'Cargo.lock')

const { version } = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))

if (!version) {
  console.error('sync-version: package.json is missing "version"')
  process.exit(1)
}

const versionLine = /^version\s*=\s*"([^"]*)"/m
const lockPackages = [
  /^name = "dropslim"\r?\nversion = "([^"]*)"/m,
  /^name = "dropslim-core"\r?\nversion = "([^"]*)"/m,
  /^name = "dropslim-cli"\r?\nversion = "([^"]*)"/m,
]

let changed = false

for (const cargoTomlPath of cargoTomlPaths) {
  const cargoToml = fs.readFileSync(cargoTomlPath, 'utf8')
  const tomlMatch = cargoToml.match(versionLine)
  const label = path.relative(root, cargoTomlPath)

  if (!tomlMatch) {
    console.error(`sync-version: could not find version in ${label}`)
    process.exit(1)
  }

  if (tomlMatch[1] !== version) {
    fs.writeFileSync(
      cargoTomlPath,
      cargoToml.replace(versionLine, `version = "${version}"`)
    )
    console.log(`sync-version: updated ${label} → ${version}`)
    changed = true
  }
}

if (!fs.existsSync(cargoLockPath)) {
  if (!changed) {
    console.log(
      `sync-version: ok (${version}; Cargo.lock missing until cargo runs)`
    )
  }
  process.exit(0)
}

let cargoLock = fs.readFileSync(cargoLockPath, 'utf8')
let lockChanged = false

for (const lockPackage of lockPackages) {
  const lockMatch = cargoLock.match(lockPackage)
  if (!lockMatch) {
    continue
  }

  if (lockMatch[1] !== version) {
    const nl = lockMatch[0].includes('\r\n') ? '\r\n' : '\n'
    const nameLine = lockMatch[0].split(/\r?\n/)[0]
    cargoLock = cargoLock.replace(
      lockPackage,
      `${nameLine}${nl}version = "${version}"`
    )
    console.log(
      `sync-version: updated Cargo.lock ${nameLine.replace('name = ', '')} → ${version}`
    )
    lockChanged = true
    changed = true
  }
}

if (lockChanged) {
  fs.writeFileSync(cargoLockPath, cargoLock)
}

if (!changed) {
  console.log(`sync-version: ok (${version})`)
}
