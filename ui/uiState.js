import { t } from './i18n/index.js'
import { isConvertOutputFormatActive } from './utils/convertWarning.js'
import {
  getOutputFormatLabelKey,
  normalizeOutputFormat,
} from './utils/outputFormat.js'

export const createUiState = () => {
  let appVersionNumber = ''

  const dragzone = () => document.getElementById('dragzone')
  const dragzoneStatus = () => document.getElementById('dragzoneStatus')
  const dragzoneOutputFormat = () =>
    document.getElementById('dragzoneOutputFormat')
  const updateStatusEl = () => document.getElementById('updateStatus')
  const appVersionEl = () => document.getElementById('appVersion')

  const syncOutputFormatStatus = (settings = {}) => {
    const outputFormatEl = dragzoneOutputFormat()

    if (!outputFormatEl) {
      return
    }

    const outputFormat = normalizeOutputFormat(settings.output_format)

    if (!isConvertOutputFormatActive({ output_format: outputFormat })) {
      outputFormatEl.textContent = ''
      outputFormatEl.hidden = true
      return
    }

    outputFormatEl.textContent = t('dragzone.outputFormat', {
      format: t(getOutputFormatLabelKey(outputFormat)),
    })
    outputFormatEl.hidden = false
  }

  const setDragzoneStatus = (message) => {
    const status = dragzoneStatus()

    if (!status) {
      return
    }

    status.textContent = message
    status.hidden = !message
  }

  const renderAppVersion = () => {
    const versionEl = appVersionEl()

    if (versionEl) {
      versionEl.textContent = appVersionNumber
        ? t('footer.version', { version: appVersionNumber })
        : ''
    }
  }

  const setAppVersion = (version) => {
    appVersionNumber = version
    renderAppVersion()
  }

  const setProcessing = (active) => {
    const zone = dragzone()

    if (!zone) {
      return
    }

    zone.classList.toggle('is--processing', active)

    if (!active) {
      setDragzoneStatus('')
    }
  }

  const clearResults = () => {
    const resultBox = document.getElementById('result')
    const batchSummary = document.getElementById('batchSummary')

    if (resultBox) {
      resultBox.innerHTML = ''
    }

    if (batchSummary) {
      batchSummary.textContent = ''
      batchSummary.hidden = true
    }
  }

  return {
    dragzone,
    setDragzoneStatus,
    setAppVersion,
    renderAppVersion,
    syncOutputFormatStatus,
    setProcessing,
    endProcessing: () => setProcessing(false),
    updateStatusEl,
    clearResults,
  }
}
