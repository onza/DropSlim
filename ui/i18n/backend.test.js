import { beforeAll, describe, expect, it } from 'vitest'
import { initI18n } from './index.js'
import {
  formatBackendSummary,
  formatBatchSummaryPayload,
  formatErrorPayload,
  formatSummaryPayload,
} from './backend.js'

beforeAll(async () => {
  await initI18n({
    getPreference: () => 'de',
    setPreference: () => {},
  })
})

describe('formatErrorPayload', () => {
  it('translates known processor errors', () => {
    expect(formatErrorPayload({ code: 'fileNotFound' })).toBe(
      'Datei oder Ordner nicht gefunden.'
    )
    expect(formatErrorPayload({ code: 'saveFolderRequired' })).toBe(
      'Bitte zuerst einen Speicherordner in den Einstellungen wählen.'
    )
  })

  it('translates HEIC errors', () => {
    expect(formatErrorPayload({ code: 'heicReadFailed' })).toBe(
      'HEIC-Bild konnte nicht gelesen werden.'
    )
  })

  it('translates gifsicle exit codes', () => {
    expect(formatErrorPayload({ code: 'gifsicleFailed', detail: '1' })).toBe(
      'GIF-Optimierung fehlgeschlagen (Exit-Code 1).'
    )
  })

  it('shows io detail unchanged', () => {
    expect(formatErrorPayload({ code: 'io', detail: 'disk full' })).toBe(
      'disk full'
    )
  })
})

describe('formatSummaryPayload', () => {
  it('translates first-pass savings', () => {
    expect(
      formatSummaryPayload({
        kind: 'saved',
        percent: 60,
        from: '100 KB',
        to: '40 KB',
      })
    ).toBe('60 % gespart · 100 KB → 40 KB')
  })

  it('translates additional savings', () => {
    expect(
      formatSummaryPayload({
        kind: 'savedMore',
        percent: 20,
        from: '50 KB',
        to: '40 KB',
      })
    ).toBe('Weitere 20 % gespart · 50 KB → 40 KB')
  })
})

describe('formatBatchSummaryPayload', () => {
  it('translates batch summaries', () => {
    expect(
      formatBatchSummaryPayload({
        total: 3,
        succeeded: 3,
        failed: 0,
        bytesBefore: 300_000,
        bytesAfter: 120_000,
      })
    ).toBe('3 Bilder · 180 KB gespart (60 %)')

    expect(
      formatBackendSummary({
        total: 3,
        succeeded: 3,
        failed: 0,
        bytesBefore: 300_000,
        bytesAfter: 120_000,
      })
    ).toBe('3 Bilder · 180 KB gespart (60 %)')
  })
})
