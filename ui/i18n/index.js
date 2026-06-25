import { isTauri } from '@tauri-apps/api/core'
import { locale as osLocale } from '@tauri-apps/plugin-os'

const localeModules = import.meta.glob('./locales/*.json', { eager: true })

const localeIdFromPath = (path) => path.match(/\/([^/]+)\.json$/)?.[1]

const localeIdToLabelKey = (locale) => {
  const suffix = locale
    .split('-')
    .map((part, index) =>
      index === 0
        ? part.toLowerCase()
        : part.charAt(0).toUpperCase() + part.slice(1).toLowerCase()
    )
    .join('')

  return `settings.language${suffix.charAt(0).toUpperCase()}${suffix.slice(1)}`
}

const MESSAGES = Object.fromEntries(
  Object.entries(localeModules).map(([path, module]) => [
    localeIdFromPath(path),
    module.default,
  ])
)

const SUPPORTED_LOCALES = Object.keys(MESSAGES).sort((a, b) =>
  a.localeCompare(b, undefined, { sensitivity: 'base' })
)

const STORED_PREFERENCES = ['system', ...SUPPORTED_LOCALES]

const LOCALE_OPTION_KEYS = {
  system: 'settings.languageSystem',
  ...Object.fromEntries(
    SUPPORTED_LOCALES.map((locale) => [locale, localeIdToLabelKey(locale)])
  ),
}

let activeLocale = 'en'
let storedPreference = 'system'
const listeners = new Set()

const resolve = (messages, key) =>
  key.split('.').reduce((value, part) => value?.[part], messages)

const interpolate = (template, params = {}) =>
  template.replace(/\{\{(\w+)\}\}/g, (_, name) =>
    params[name] !== undefined ? String(params[name]) : ''
  )

export const mapLocaleTag = (tag) => {
  if (!tag || tag === 'system') {
    return null
  }

  if (SUPPORTED_LOCALES.includes(tag)) {
    return tag
  }

  const normalized = String(tag).toLowerCase().replaceAll('_', '-')
  const primary = normalized.split('-')[0]

  if (primary === 'pt' && SUPPORTED_LOCALES.includes('pt-BR')) {
    return 'pt-BR'
  }

  const match = SUPPORTED_LOCALES.find(
    (locale) => locale.split('-')[0].toLowerCase() === primary
  )

  return match ?? 'en'
}

export const normalizeStoredPreference = (preference) => {
  if (!preference || preference === 'system') {
    return 'system'
  }

  if (STORED_PREFERENCES.includes(preference)) {
    return preference
  }

  const normalized = String(preference)

  if (!normalized.includes('-') && !normalized.includes('_')) {
    return 'system'
  }

  const mapped = mapLocaleTag(preference)

  if (mapped && SUPPORTED_LOCALES.includes(mapped)) {
    return mapped
  }

  return 'system'
}

const detectBrowserLocale = () => {
  const tags = navigator.languages?.length
    ? navigator.languages
    : [navigator.language || 'en']

  for (const tag of tags) {
    const mapped = mapLocaleTag(tag)

    if (mapped) {
      return mapped
    }
  }

  return 'en'
}

const detectSystemLocale = async () => {
  if (isTauri()) {
    try {
      const tag = await osLocale()

      if (tag) {
        const mapped = mapLocaleTag(tag)

        if (mapped) {
          return mapped
        }
      }
    } catch (error) {
      console.error('locale detection failed', error)
    }
  }

  return detectBrowserLocale()
}

export const getStoredLocalePreference = () => storedPreference

export const getLocaleOptions = () => [
  { value: 'system', labelKey: LOCALE_OPTION_KEYS.system },
  ...SUPPORTED_LOCALES.map((locale) => ({
    value: locale,
    labelKey: LOCALE_OPTION_KEYS[locale],
  })),
]

export const t = (key, params) => {
  let template = resolve(MESSAGES[activeLocale], key)

  if (typeof template !== 'string' && activeLocale !== 'en') {
    template = resolve(MESSAGES.en, key)
  }

  if (typeof template !== 'string') {
    return key
  }

  return interpolate(template, params)
}

export const tCount = (keyBase, count, params = {}) => {
  const suffix = count === 1 ? '_one' : '_other'
  return t(`${keyBase}${suffix}`, { count, ...params })
}

const htmlLangFor = (locale) => (locale === 'pt-BR' ? 'pt-BR' : locale)

const notify = () => {
  listeners.forEach((listener) => listener(activeLocale))
}

export const onLocaleChange = (listener) => {
  listeners.add(listener)

  return () => listeners.delete(listener)
}

const applyDomTranslations = () => {
  if (typeof document === 'undefined') {
    return
  }

  document.querySelectorAll('[data-i18n]').forEach((element) => {
    element.textContent = t(element.dataset.i18n)
  })

  document.title = t('app.title')
  document.documentElement.lang = htmlLangFor(activeLocale)
}

const setActiveLocale = (locale) => {
  activeLocale = SUPPORTED_LOCALES.includes(locale) ? locale : 'en'
  applyDomTranslations()
  notify()
}

const resolvePreference = async (preference) => {
  if (preference === 'system') {
    return detectSystemLocale()
  }

  return mapLocaleTag(preference) || 'en'
}

export const initI18n = async ({ getPreference, setPreference }) => {
  const rawPreference = getPreference?.()
  const normalized = normalizeStoredPreference(rawPreference)

  if (normalized !== rawPreference) {
    setPreference?.(normalized)
  }

  storedPreference = normalized
  setActiveLocale(await resolvePreference(storedPreference))

  return {
    setPreference: async (preference) => {
      const nextPreference = normalizeStoredPreference(preference)
      storedPreference = nextPreference
      setPreference?.(nextPreference)
      setActiveLocale(await resolvePreference(nextPreference))
    },
  }
}
