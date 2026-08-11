export const OUTPUT_FORMAT_OPTIONS = [
  { value: 'original', labelKey: 'settings.outputFormatOriginal' },
  { value: 'jpeg', labelKey: 'settings.outputFormatJpeg' },
  { value: 'png', labelKey: 'settings.outputFormatPng' },
  { value: 'webp', labelKey: 'settings.outputFormatWebp' },
  { value: 'avif', labelKey: 'settings.outputFormatAvif' },
]

export const normalizeOutputFormat = (value) =>
  OUTPUT_FORMAT_OPTIONS.some((option) => option.value === value)
    ? value
    : 'original'

export const getOutputFormatLabelKey = (value) =>
  OUTPUT_FORMAT_OPTIONS.find(
    (option) => option.value === normalizeOutputFormat(value)
  )?.labelKey ?? 'settings.outputFormatOriginal'
