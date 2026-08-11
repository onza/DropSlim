import { invoke } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import {
  checkForUpdates,
  createStatusSink,
  getPendingUpdateVersion,
  hasPendingUpdate,
  installPendingUpdate,
} from './updates.js'
import { createSettingsStore } from './settingsStore.js'
import {
  createEventSubscriptions,
  createOptimizationService,
} from './optimizationService.js'
import { createUiState } from './uiState.js'

export const initApi = async () => {
  const settings = await createSettingsStore()
  const ui = createUiState()

  const optimization = createOptimizationService({
    settings,
    onStatus: ui.setDragzoneStatus,
    onProcessingChange: ui.setProcessing,
    onClearResults: () => {
      if (settings.getSync().clearlist) {
        ui.clearResults()
      }
    },
  })

  optimization.setupDropHandling(ui.dragzone())
  await optimization.bindStartupPaths()
  ui.syncOutputFormatStatus(settings.getSync())

  const updateStatus = createStatusSink(ui.updateStatusEl())

  return {
    settings,
    optimization,
    ui,
    updateStatus,
  }
}

export const createDropslimApi = ({
  settings,
  optimization,
  ui,
  updateStatus,
}) => {
  const events = createEventSubscriptions()

  return {
    settings,
    setStatus: ui.setDragzoneStatus,
    setUpdateStatus: updateStatus.setMessage,
    reportUpdateStatus: (key, params) => updateStatus.setKey(key, params ?? {}),
    reapplyUpdateStatus: updateStatus.reapply,
    setAppVersion: ui.setAppVersion,
    renderAppVersion: ui.renderAppVersion,
    syncOutputFormatStatus: () => ui.syncOutputFormatStatus(settings.getSync()),
    setProcessing: ui.setProcessing,
    endProcessing: ui.endProcessing,
    pickAndOptimize: optimization.pickAndOptimize,
    pickSaveFolder: async () => {
      const filePaths = await invoke('pick_save_folder')

      if (!filePaths.length) {
        return { canceled: true, filePaths: [] }
      }

      return { canceled: false, filePaths }
    },
    cancelOptimization: optimization.cancel,
    showItemInFolder: (filePath) => revealItemInDir(filePath),
    ...events,
    checkForUpdates: (options) =>
      checkForUpdates({
        ...options,
        onStatus: options?.onStatus ?? updateStatus.setMessage,
      }),
    hasPendingUpdate,
    getPendingUpdateVersion,
    installPendingUpdate: (options) =>
      installPendingUpdate({
        ...options,
        onStatus: options?.onStatus ?? updateStatus.setMessage,
      }),
  }
}
