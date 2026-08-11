import { describe, expect, it } from 'vitest'
import {
  isConvertOutputFormatActive,
  shouldWarnConvertOutputFormat,
} from './convertWarning.js'

describe('isConvertOutputFormatActive', () => {
  it('is true only for non-original raster targets', () => {
    expect(isConvertOutputFormatActive({ output_format: 'jpeg' })).toBe(true)
    expect(isConvertOutputFormatActive({ output_format: 'webp' })).toBe(true)
    expect(isConvertOutputFormatActive({ output_format: 'original' })).toBe(
      false
    )
    expect(isConvertOutputFormatActive({ output_format: 'gif' })).toBe(false)
  })
})

describe('shouldWarnConvertOutputFormat', () => {
  it('warns when convert is active and not skipped', () => {
    expect(
      shouldWarnConvertOutputFormat({
        output_format: 'jpeg',
        skipConvertWarning: false,
      })
    ).toBe(true)
  })

  it('respects dont-ask-again', () => {
    expect(
      shouldWarnConvertOutputFormat({
        output_format: 'jpeg',
        skipConvertWarning: true,
      })
    ).toBe(false)
  })

  it('does not warn for original format', () => {
    expect(
      shouldWarnConvertOutputFormat({
        output_format: 'original',
        skipConvertWarning: false,
      })
    ).toBe(false)
  })
})
