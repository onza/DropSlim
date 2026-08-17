import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  confirmConvertOutputFormatWarning,
  confirmOverwriteDimensionWarning,
} from './confirmDialog.js'
import { t } from './i18n/index.js'
import { formatBackendError } from './i18n/backend.js'
import { shouldWarnConvertOutputFormat } from './utils/convertWarning.js'
import { getOutputFormatLabelKey } from './utils/outputFormat.js'
import { shouldWarnOverwriteWithDimensionLimits } from './utils/overwriteWarning.js'

export const createOptimizationService = ({
  settings,
  onStatus,
  onProcessingChange,
  onClearResults,
}) => {
  let inFlight = false

  const submitPaths = async (paths) => {
    if (!paths.length) {
      return
    }

    await invoke('optimize_paths', {
      paths,
      settings: settings.pickOptimizeSettings(),
    })
  }

  const run = async (paths) => {
    if (!paths?.length) {
      return false
    }

    if (inFlight) {
      onStatus?.(t('status.alreadyCompressing'))
      return false
    }

    const currentSettings = settings.getSync()

    if (shouldWarnOverwriteWithDimensionLimits(currentSettings)) {
      const { proceed, dontAskAgain } = await confirmOverwriteDimensionWarning()
      if (!proceed) {
        return false
      }
      if (dontAskAgain) {
        settings.setSync('skipOverwriteDimensionWarning', true)
      }
    }

    if (shouldWarnConvertOutputFormat(settings.getSync())) {
      const outputFormat = settings.getSync().output_format
      const formatLabel = t(getOutputFormatLabelKey(outputFormat))
      const { proceed, dontAskAgain } =
        await confirmConvertOutputFormatWarning(formatLabel)
      if (!proceed) {
        return false
      }
      if (dontAskAgain) {
        settings.setSync('skipConvertWarning', true)
      }
    }

    inFlight = true
    onClearResults?.()
    onProcessingChange?.(true)

    try {
      await submitPaths(paths)
      return true
    } catch (error) {
      console.error(error)
      onProcessingChange?.(false)
      onStatus?.(formatBackendError(error))
      return false
    } finally {
      inFlight = false
    }
  }

  const setupDropHandling = (zone) => {
    if (!zone) {
      return
    }

    const win = getCurrentWindow()

    win.onDragDropEvent((event) => {
      const { type } = event.payload

      if (type === 'enter' || type === 'over') {
        zone.classList.add('drag-active')
        return
      }

      if (type === 'leave') {
        zone.classList.remove('drag-active')
        return
      }

      if (type !== 'drop') {
        return
      }

      zone.classList.remove('drag-active')

      const paths = event.payload.paths ?? []

      if (paths.length) {
        void run(paths)
      }
    })
  }

  const bindStartupPaths = async () => {
    const startupPaths = await invoke('consume_startup_paths')
    void run(startupPaths)

    listen('startup-paths', (event) => {
      void run(event.payload)
    })
  }

  const pickAndOptimize = async () => {
    if (inFlight) {
      onStatus?.(t('status.alreadyCompressing'))
      return { canceled: true, filePaths: [], busy: true }
    }

    const filePaths = await invoke('pick_paths')

    if (!filePaths.length) {
      return { canceled: true, filePaths: [] }
    }

    const ok = await run(filePaths)

    return { canceled: !ok, filePaths, busy: false }
  }

  return {
    pickAndOptimize,
    cancel: () => invoke('cancel_optimization'),
    setupDropHandling,
    bindStartupPaths,
  }
}

const emitEvent = (name, callback) =>
  listen(name, (event) => {
    callback(
      ...(Array.isArray(event.payload) ? event.payload : [event.payload])
    )
  })

export const createEventSubscriptions = () => ({
  onOpenSettings: (callback) => emitEvent('open-settings', callback),
  onBatchStarted: (callback) =>
    emitEvent('batch-started', (total) => callback(total)),
  onBatchProgress: (callback) =>
    emitEvent('batch-progress', (done, total) => callback(done, total)),
  onOptimized: (callback) =>
    emitEvent('image-optimized', (payload) =>
      callback(
        payload.outputPath,
        payload.summary,
        payload.sourceName,
        payload.resized ?? null
      )
    ),
  onFileProcessing: (callback) =>
    emitEvent('file-processing', (fileName) => callback(fileName)),
  onDropError: (callback) =>
    emitEvent('drop-error', (payload) =>
      callback(payload.fileName, payload.error)
    ),
  onBatchComplete: (callback) =>
    emitEvent('batch-complete', (payload) => callback(payload)),
  onBatchCancelled: (callback) =>
    emitEvent('batch-cancelled', (done, total, succeeded, failed) =>
      callback(done, total, succeeded, failed)
    ),
})
