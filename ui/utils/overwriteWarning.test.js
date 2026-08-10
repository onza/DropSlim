import { describe, expect, it } from 'vitest'
import {
  hasActiveDimensionLimits,
  shouldWarnOverwriteWithDimensionLimits,
  willOverwriteInPlace,
} from './overwriteWarning.js'

describe('willOverwriteInPlace', () => {
  it('is true only for same folder without suffix or subfolder', () => {
    expect(
      willOverwriteInPlace({
        folderswitch: true,
        suffix: false,
        subfolder: false,
      })
    ).toBe(true)
    expect(
      willOverwriteInPlace({
        folderswitch: true,
        suffix: true,
        subfolder: false,
      })
    ).toBe(false)
    expect(
      willOverwriteInPlace({
        folderswitch: true,
        suffix: false,
        subfolder: true,
      })
    ).toBe(false)
    expect(
      willOverwriteInPlace({
        folderswitch: false,
        suffix: false,
        subfolder: false,
      })
    ).toBe(false)
  })
})

describe('hasActiveDimensionLimits', () => {
  it('requires the toggle and at least one positive limit', () => {
    expect(
      hasActiveDimensionLimits({
        limit_dimensions: true,
        max_width: 1200,
        max_height: null,
      })
    ).toBe(true)
    expect(
      hasActiveDimensionLimits({
        limit_dimensions: true,
        max_width: null,
        max_height: null,
      })
    ).toBe(false)
    expect(
      hasActiveDimensionLimits({
        limit_dimensions: false,
        max_width: 1200,
        max_height: 800,
      })
    ).toBe(false)
  })
})

describe('shouldWarnOverwriteWithDimensionLimits', () => {
  it('warns only when limits and in-place overwrite apply', () => {
    expect(
      shouldWarnOverwriteWithDimensionLimits({
        limit_dimensions: true,
        max_width: 1600,
        max_height: null,
        folderswitch: true,
        suffix: false,
        subfolder: false,
        skipOverwriteDimensionWarning: false,
      })
    ).toBe(true)
  })

  it('respects dont-ask-again', () => {
    expect(
      shouldWarnOverwriteWithDimensionLimits({
        limit_dimensions: true,
        max_width: 1600,
        max_height: null,
        folderswitch: true,
        suffix: false,
        subfolder: false,
        skipOverwriteDimensionWarning: true,
      })
    ).toBe(false)
  })
})
