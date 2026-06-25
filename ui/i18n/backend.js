import { t } from './index.js'

const formatsParam = () => ({ formats: t('formats.supported') })

const formatBytes = (bytes) => {
  const unit = 1000

  if (bytes < unit) {
    return `${bytes} B`
  }

  if (bytes < unit * unit) {
    return `${Math.round(bytes / unit)} KB`
  }

  const value = bytes / (unit * unit)
  return `${value.toFixed(1)} MB`
}

export const formatErrorPayload = (payload) => {
  if (!payload?.code) {
    return ''
  }

  const { code, count, max, detail } = payload

  if (code === 'io' && detail) {
    return detail
  }

  const params = {}

  if (code === 'noSupportedImages' || code === 'unsupportedFormat') {
    Object.assign(params, formatsParam())
  }

  if (count != null) {
    params.count = count
  }

  if (max != null) {
    params.max = max
  }

  if (code === 'gifsicleFailed' && detail) {
    params.code = detail
  }

  return t(`errors.${code}`, params)
}

const normalizeErrorPayload = (error) => {
  if (error && typeof error === 'object' && error.code) {
    return error
  }

  if (typeof error === 'string') {
    try {
      const parsed = JSON.parse(error)

      if (parsed?.code) {
        return parsed
      }
    } catch {
      // ignore invalid JSON
    }
  }

  return null
}

export const formatBackendError = (error) => {
  const payload = normalizeErrorPayload(error)

  if (payload) {
    return formatErrorPayload(payload)
  }

  return typeof error === 'string' ? error : String(error)
}

export const formatSummaryPayload = (summary) => {
  if (!summary?.kind) {
    return ''
  }

  switch (summary.kind) {
    case 'alreadyOptimized':
      return t('summary.alreadyOptimized', { size: summary.size })
    case 'saved':
      return t('summary.saved', {
        percent: summary.percent,
        from: summary.from,
        to: summary.to,
      })
    case 'savedMore':
      return t('summary.savedMore', {
        percent: summary.percent,
        from: summary.from,
        to: summary.to,
      })
    default:
      return ''
  }
}

export const formatBatchSummaryPayload = (payload) => {
  if (!payload) {
    return ''
  }

  const { total, succeeded, failed, bytesBefore, bytesAfter } = payload
  const parts = []

  parts.push(
    total === 1
      ? t('summary.batch.image_one')
      : t('summary.batch.image_other', { count: total })
  )

  if (failed > 0) {
    parts.push(t('summary.batch.failed', { count: failed }))
  }

  if (succeeded > 0) {
    const saved = Math.max(0, bytesBefore - bytesAfter)

    if (saved > 0) {
      const percent = Math.round((100 / bytesBefore) * saved)
      parts.push(
        t('summary.batch.saved', {
          size: formatBytes(saved),
          percent,
        })
      )
    } else if (failed === 0) {
      parts.push(t('summary.batch.alreadyOptimized'))
    }
  }

  return parts.join(` ${t('summary.batch.separator')} `)
}

export const formatBackendSummary = (summary) => {
  if (summary?.kind) {
    return formatSummaryPayload(summary)
  }

  if (summary?.total != null) {
    return formatBatchSummaryPayload(summary)
  }

  return ''
}
