import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const packageJsonPath = path.join(root, 'package.json')
const cargoTomlPath = path.join(root, 'src-tauri', 'Cargo.toml')
const cargoLockPath = path.join(root, 'src-tauri', 'Cargo.lock')

const { version } = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))

if (!version) {
  console.error('sync-version: package.json is missing "version"')
  process.exit(1)
}

const versionLine = /^version\s*=\s*"([^"]*)"/m
const lockPackage = /\[\[package\]\]\nname = "dropslim"\nversion = "([^"]*)"/

const cargoToml = fs.readFileSync(cargoTomlPath, 'utf8')
const tomlMatch = cargoToml.match(versionLine)

if (!tomlMatch) {
  console.error('sync-version: could not find version in src-tauri/Cargo.toml')
  process.exit(1)
}

const cargoLock = fs.readFileSync(cargoLockPath, 'utf8')
const lockMatch = cargoLock.match(lockPackage)

if (!lockMatch) {
  console.error(
    'sync-version: could not find dropslim package in src-tauri/Cargo.lock'
  )
  process.exit(1)
}

let changed = false

if (tomlMatch[1] !== version) {
  fs.writeFileSync(
    cargoTomlPath,
    cargoToml.replace(versionLine, `version = "${version}"`)
  )
  console.log(`sync-version: updated Cargo.toml → ${version}`)
  changed = true
}

if (lockMatch[1] !== version) {
  fs.writeFileSync(
    cargoLockPath,
    cargoLock.replace(
      lockPackage,
      `[[package]]\nname = "dropslim"\nversion = "${version}"`
    )
  )
  console.log(`sync-version: updated Cargo.lock → ${version}`)
  changed = true
}

if (!changed) {
  console.log(`sync-version: ok (${version})`)
}
