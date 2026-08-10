export const willOverwriteInPlace = (settings = {}) =>
  settings.folderswitch !== false &&
  settings.suffix === false &&
  !settings.subfolder

export const hasActiveDimensionLimits = (settings = {}) => {
  if (!settings.limit_dimensions) {
    return false
  }

  const maxWidth = settings.max_width
  const maxHeight = settings.max_height

  return (
    (typeof maxWidth === 'number' && maxWidth > 0) ||
    (typeof maxHeight === 'number' && maxHeight > 0)
  )
}

export const shouldWarnOverwriteWithDimensionLimits = (settings = {}) =>
  !settings.skipOverwriteDimensionWarning &&
  hasActiveDimensionLimits(settings) &&
  willOverwriteInPlace(settings)
