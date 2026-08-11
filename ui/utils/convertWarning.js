const CONVERT_OUTPUT_FORMATS = new Set(['jpeg', 'png', 'webp', 'avif'])

export const isConvertOutputFormatActive = (settings = {}) =>
  CONVERT_OUTPUT_FORMATS.has(settings.output_format)

export const shouldWarnConvertOutputFormat = (settings = {}) =>
  !settings.skipConvertWarning && isConvertOutputFormatActive(settings)
