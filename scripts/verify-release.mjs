import fs from 'node:fs'
import path from 'node:path'

const appPath = process.argv[2]

if (!appPath) {
  console.error('Usage: node scripts/verify-release.mjs /path/to/DropSlim.app')
  process.exit(1)
}

const fail = (message) => {
  console.error(`verify-release: ${message}`)
  process.exit(1)
}

const executable = path.resolve(appPath, 'Contents/MacOS/dropslim')
const resources = path.join(appPath, 'Contents/Resources/resources')
const gifsicle = path.join(resources, 'vendor', 'gifsicle', 'gifsicle')
const quickAction = path.join(
  resources,
  'build',
  'Optimize with DropSlim.workflow',
  'Contents',
  'document.wflow'
)

if (!fs.existsSync(appPath)) {
  fail(`app bundle not found: ${appPath}`)
}

if (!fs.existsSync(executable)) {
  fail(`main executable missing: ${executable}`)
}

if ((fs.statSync(executable).mode & 0o111) === 0) {
  fail(`main executable is not marked executable: ${executable}`)
}

if (!fs.existsSync(gifsicle)) {
  fail(`gifsicle binary missing: ${gifsicle}`)
}

if ((fs.statSync(gifsicle).mode & 0o111) === 0) {
  fail(`gifsicle is not marked executable: ${gifsicle}`)
}

if (!fs.existsSync(quickAction)) {
  fail(`Quick Action workflow missing: ${quickAction}`)
}

console.log(`verify-release: ok (${appPath})`)
