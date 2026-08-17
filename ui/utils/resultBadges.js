export const normalizeExt = (ext) => (ext === 'jpeg' ? 'jpg' : ext)

export const formatConvertBadge = (sourceFileName, outputPath) => {
  const sourceExt = normalizeExt(
    (sourceFileName ?? '').split('.').pop()?.toLowerCase()
  )
  const outputExt = normalizeExt(
    (outputPath ?? '').split('.').pop()?.toLowerCase()
  )

  if (!sourceExt || !outputExt || sourceExt === outputExt) {
    return null
  }

  return `${sourceExt.toUpperCase()} → ${outputExt.toUpperCase()}`
}

export const formatDimensionBadge = (resized) => {
  if (
    !resized ||
    resized.fromWidth == null ||
    resized.fromHeight == null ||
    resized.toWidth == null ||
    resized.toHeight == null
  ) {
    return null
  }

  return `${resized.fromWidth}×${resized.fromHeight} → ${resized.toWidth}×${resized.toHeight}`
}

export const formatResultSummaryLine = (
  summaryText,
  sourceFileName,
  path,
  resized
) => {
  const parts = [summaryText]
  const convertBadge = formatConvertBadge(sourceFileName, path)
  const dimensionBadge = formatDimensionBadge(resized)

  if (convertBadge) {
    parts.push(convertBadge)
  }
  if (dimensionBadge) {
    parts.push(dimensionBadge)
  }

  return parts.join('  ·  ')
}
