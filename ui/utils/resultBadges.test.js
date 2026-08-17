import { describe, expect, it } from 'vitest'
import {
  formatConvertBadge,
  formatDimensionBadge,
  formatResultSummaryLine,
} from './resultBadges.js'

describe('formatConvertBadge', () => {
  it('returns null when extensions match', () => {
    expect(formatConvertBadge('photo.png', '/tmp/photo.min.png')).toBe(null)
  })

  it('treats jpeg and jpg as the same codec', () => {
    expect(formatConvertBadge('photo.jpeg', '/tmp/photo.min.jpg')).toBe(null)
  })

  it('shows convert badge for format changes', () => {
    expect(formatConvertBadge('photo.png', '/tmp/photo.min.jpg')).toBe(
      'PNG → JPG'
    )
  })
})

describe('formatDimensionBadge', () => {
  it('returns null without resized data', () => {
    expect(formatDimensionBadge(null)).toBe(null)
    expect(formatDimensionBadge(undefined)).toBe(null)
  })

  it('formats pixel sizes before and after', () => {
    expect(
      formatDimensionBadge({
        fromWidth: 1920,
        fromHeight: 1080,
        toWidth: 1280,
        toHeight: 720,
      })
    ).toBe('1920×1080 → 1280×720')
  })
})

describe('formatResultSummaryLine', () => {
  it('joins summary convert and dimension badges', () => {
    expect(
      formatResultSummaryLine('Saved 40%', 'photo.png', '/tmp/photo.min.jpg', {
        fromWidth: 800,
        fromHeight: 600,
        toWidth: 400,
        toHeight: 300,
      })
    ).toBe('Saved 40%  ·  PNG → JPG  ·  800×600 → 400×300')
  })

  it('omits missing badges', () => {
    expect(
      formatResultSummaryLine(
        'Already optimized',
        'photo.png',
        '/tmp/a.png',
        null
      )
    ).toBe('Already optimized')
  })
})
