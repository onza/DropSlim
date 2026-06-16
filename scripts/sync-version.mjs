import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const packageJsonPath = path.join(root, 'package.json')
const cargoTomlPath = path.join(root, 'src-tauri', 'Cargo.toml')

const { version } = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))

if (!version) {
  console.error('sync-version: package.json is missing "version"')
  process.exit(1)
}

const cargoToml = fs.readFileSync(cargoTomlPath, 'utf8')
const versionLine = /^version\s*=\s*"([^"]*)"/m
const match = cargoToml.match(versionLine)

if (!match) {
  console.error('sync-version: could not find version in src-tauri/Cargo.toml')
  process.exit(1)
}

if (match[1] === version) {
  console.log(`sync-version: ok (${version})`)
  process.exit(0)
}

const updatedCargoToml = cargoToml.replace(
  versionLine,
  `version = "${version}"`
)

fs.writeFileSync(cargoTomlPath, updatedCargoToml)
console.log(`sync-version: updated Cargo.toml → ${version}`)
