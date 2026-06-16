import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const target = path.join(root, 'src-tauri', 'resources')

const copyDir = (from, to) => {
  fs.cpSync(from, to, { recursive: true })
}

const signResources = (resourcesDir) => {
  if (process.platform !== 'darwin') {
    console.log('prepare-resources: signing skipped (not macOS)')
    return
  }

  let signed = 0

  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const fullPath = path.join(dir, entry.name)

      if (entry.isDirectory()) {
        walk(fullPath)
        continue
      }

      const isExecutable = (fs.statSync(fullPath).mode & 0o111) !== 0
      if (!isExecutable && entry.name !== 'gifsicle') {
        continue
      }

      execFileSync('codesign', ['--force', '--sign', '-', fullPath], {
        stdio: 'ignore',
      })
      signed += 1
    }
  }

  walk(resourcesDir)
  console.log(`prepare-resources: signed ${signed} binaries`)
}

fs.rmSync(target, { recursive: true, force: true })
fs.mkdirSync(target, { recursive: true })
copyDir(path.join(root, 'vendor'), path.join(target, 'vendor'))
signResources(target)

console.log(`prepare-resources: ok (${target})`)
