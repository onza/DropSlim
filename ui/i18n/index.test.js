import { beforeAll, describe, expect, it } from 'vitest'
import {
  initI18n,
  mapLocaleTag,
  normalizeStoredPreference,
  t,
} from './index.js'

beforeAll(async () => {
  await initI18n({
    getPreference: () => 'de',
    setPreference: () => {},
  })
})

describe('mapLocaleTag', () => {
  it('maps language tags to supported locales', () => {
    expect(mapLocaleTag('de-DE')).toBe('de')
    expect(mapLocaleTag('pt-BR')).toBe('pt-BR')
    expect(mapLocaleTag('pt-PT')).toBe('pt-BR')
    expect(mapLocaleTag('ja-JP')).toBe('ja')
    expect(mapLocaleTag('zh-CN')).toBe('en')
  })
})

describe('normalizeStoredPreference', () => {
  it('normalizes invalid values to system', () => {
    expect(normalizeStoredPreference(undefined)).toBe('system')
    expect(normalizeStoredPreference('zh')).toBe('system')
    expect(normalizeStoredPreference('de-AT')).toBe('de')
    expect(normalizeStoredPreference('fr')).toBe('fr')
  })
})

describe('t', () => {
  it('translates active locale', () => {
    expect(t('footer.settings')).toBe('Einstellungen')
  })
})
