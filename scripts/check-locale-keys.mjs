import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const localesDir = path.join(root, 'ui/i18n/locales')
const masterPath = path.join(localesDir, 'en.json')

const flatten = (value, prefix = '') => {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    return prefix ? { [prefix]: value } : {}
  }

  return Object.entries(value).reduce((keys, [key, nested]) => {
    const next = prefix ? `${prefix}.${key}` : key
    return { ...keys, ...flatten(nested, next) }
  }, {})
}

const placeholders = (template) => {
  const matches = String(template).matchAll(/\{\{(\w+)\}\}/g)
  return [...matches].map((match) => match[1]).sort()
}

const master = JSON.parse(fs.readFileSync(masterPath, 'utf8'))
const masterFlat = flatten(master)
const masterKeys = Object.keys(masterFlat).sort()

const localeFiles = fs
  .readdirSync(localesDir)
  .filter((file) => file.endsWith('.json') && file !== 'en.json')

let failed = false

for (const file of localeFiles) {
  const locale = JSON.parse(
    fs.readFileSync(path.join(localesDir, file), 'utf8')
  )
  const localeFlat = flatten(locale)
  const localeKeys = new Set(Object.keys(localeFlat))

  for (const key of masterKeys) {
    if (!localeKeys.has(key)) {
      console.error(`check-locales: missing key "${key}" in ${file}`)
      failed = true
    }
  }

  for (const key of localeKeys) {
    if (!Object.hasOwn(masterFlat, key)) {
      console.error(`check-locales: extra key "${key}" in ${file}`)
      failed = true
    }
  }

  for (const key of masterKeys) {
    const masterValue = masterFlat[key]
    const localeValue = localeFlat[key]

    if (typeof masterValue !== 'string' || typeof localeValue !== 'string') {
      continue
    }

    const masterPlaceholders = placeholders(masterValue).join(',')
    const localePlaceholders = placeholders(localeValue).join(',')

    if (masterPlaceholders !== localePlaceholders) {
      console.error(
        `check-locales: placeholder mismatch for "${key}" in ${file}`
      )
      failed = true
    }
  }
}

if (failed) {
  process.exit(1)
}

console.log(`check-locales: ok (${localeFiles.length + 1} locales)`)
