import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  createStatusSink,
  reapplyUpdateStatus,
  reportUpdateStatus,
} from './updates.js'

vi.mock('@tauri-apps/api/core', () => ({
  isTauri: () => false,
}))

vi.mock('./i18n/index.js', () => ({
  t: (key, params = {}) =>
    `${key}:${Object.entries(params)
      .map(([name, value]) => `${name}=${value}`)
      .join(',')}`,
}))

describe('createStatusSink', () => {
  let element

  beforeEach(() => {
    element = { textContent: '' }
  })

  it('writes translated status messages', () => {
    const sink = createStatusSink(element)

    sink.setKey('updates.upToDate')

    expect(element.textContent).toBe('updates.upToDate:')
  })

  it('reapplies the last translated status', () => {
    const sink = createStatusSink(element)

    reportUpdateStatus(
      'updates.available',
      { version: '2.0.0' },
      sink.setMessage
    )
    element.textContent = ''
    reapplyUpdateStatus(sink.setMessage)

    expect(element.textContent).toBe('updates.available:version=2.0.0')
  })
})
