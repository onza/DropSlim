import { execSync } from 'node:child_process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const run = (script) => {
  execSync(`node ${path.join(root, 'scripts', script)}`, {
    cwd: root,
    stdio: 'inherit',
  })
}

run('sync-version.mjs')
run('build-icons.mjs')
run('prepare-resources.mjs')

console.log('prepare-tauri: ok')
