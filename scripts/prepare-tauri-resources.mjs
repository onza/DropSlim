import fs from 'node:fs'
import path from 'node:path'
import { execSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const target = path.join(root, 'src-tauri', 'resources')

const copyDir = (from, to) => {
  fs.cpSync(from, to, { recursive: true })
}

fs.rmSync(target, { recursive: true, force: true })
fs.mkdirSync(target, { recursive: true })

copyDir(path.join(root, 'vendor'), path.join(target, 'vendor'))

execSync('node scripts/build-quick-action.mjs', {
  cwd: root,
  stdio: 'inherit',
})

copyDir(
  path.join(root, 'build', 'Optimize with DropSlim.workflow'),
  path.join(target, 'build', 'Optimize with DropSlim.workflow')
)

console.log(`prepare-tauri-resources: ok (${target})`)
