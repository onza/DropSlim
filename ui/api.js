import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { load } from '@tauri-apps/plugin-store'
import {
  checkForUpdates,
  getPendingUpdateVersion,
  hasPendingUpdate,
  installPendingUpdate,
} from './updates.js'

const OPTIMIZE_SETTING_KEYS = [
  'folderswitch',
  'suffix',
  'subfolder',
  'savepath',
]

const defaults = {
  folderswitch: true,
  clearlist: false,
  suffix: true,
  subfolder: false,
  autoCheckUpdates: true,
  autoInstallUpdates: false,
}

let storePromise
let cachedSettings = { ...defaults }
let optimizeInFlight = false

const getStore = () => {
  if (!storePromise) {
    storePromise = load('settings.json', { autoSave: true, defaults })
  }

  return storePromise
}

const dragzone = () => document.getElementById('dragzone')

const dragzoneStatus = () => document.getElementById('dragzoneStatus')

const updateStatusEl = () => document.getElementById('updateStatus')

const pickOptimizeSettings = (settings) =>
  Object.fromEntries(
    OPTIMIZE_SETTING_KEYS.filter((key) => key in settings).map((key) => [
      key,
      settings[key],
    ])
  )

const setDragzoneStatus = (message) => {
  const status = dragzoneStatus()

  if (!status) {
    return
  }

  status.textContent = message
  status.hidden = !message
}

let lastUpdateStatus = ''
let appVersionText = ''

const renderAppVersion = () => {
  const versionEl = document.getElementById('appVersion')

  if (versionEl) {
    versionEl.textContent = appVersionText
  }
}

const setUpdateStatus = (message) => {
  const status = updateStatusEl()
  lastUpdateStatus = message || ''

  if (status) {
    status.textContent = lastUpdateStatus
  }
}

const setAppVersion = (text) => {
  appVersionText = text
  renderAppVersion()
}

const reapplyUpdateStatus = () => {
  if (lastUpdateStatus) {
    setUpdateStatus(lastUpdateStatus)
  }
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

const beginProcessing = () => {
  setProcessing(true)
}

const endProcessing = () => {
  setProcessing(false)
}

const emitEvent = (name, callback) => {
  return listen(name, (event) => {
    callback(
      ...(Array.isArray(event.payload) ? event.payload : [event.payload])
    )
  })
}

const clearResultsIfNeeded = () => {
  if (!cachedSettings.clearlist) {
    return
  }

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

const submitPaths = async (paths) => {
  if (!paths.length) {
    return
  }

  await invoke('optimize_paths', {
    paths,
    settings: pickOptimizeSettings(cachedSettings),
  })
}

const runOptimization = async (paths) => {
  if (!paths?.length) {
    return false
  }

  if (optimizeInFlight) {
    setDragzoneStatus('Already compressing images…')
    return false
  }

  optimizeInFlight = true
  clearResultsIfNeeded()
  beginProcessing()

  try {
    await submitPaths(paths)
    return true
  } catch (error) {
    console.error(error)
    endProcessing()
    setDragzoneStatus(String(error))
    return false
  } finally {
    optimizeInFlight = false
  }
}

const setupDropHandling = () => {
  const zone = dragzone()
  const win = getCurrentWindow()

  win.onDragDropEvent((event) => {
    const { type } = event.payload

    if (type === 'enter' || type === 'over') {
      zone?.classList.add('drag-active')
      return
    }

    if (type === 'leave') {
      zone?.classList.remove('drag-active')
      return
    }

    if (type !== 'drop') {
      return
    }

    zone?.classList.remove('drag-active')

    const paths = event.payload.paths ?? []

    if (!paths.length) {
      return
    }

    runOptimization(paths)
  })
}

export const initApi = async () => {
  const store = await getStore()
  const entries = await store.entries()

  cachedSettings = {
    ...defaults,
    ...Object.fromEntries(entries),
  }

  setupDropHandling()

  const startupPaths = await invoke('consume_startup_paths')
  void runOptimization(startupPaths)

  listen('startup-paths', (event) => {
    runOptimization(event.payload)
  })
}

export const createDropslimApi = () => ({
  settings: {
    getSync: () => cachedSettings,
    setSync: (key, value) => {
      cachedSettings[key] = value
      getStore()
        .then((store) => store.set(key, value))
        .catch((error) => console.error(error))
    },
  },
  setStatus: setDragzoneStatus,
  setUpdateStatus,
  setAppVersion,
  reapplyUpdateStatus,
  setProcessing,
  endProcessing,
  pickAndOptimize: async () => {
    if (optimizeInFlight) {
      setDragzoneStatus('Already compressing images…')
      return { canceled: true, filePaths: [], busy: true }
    }

    const filePaths = await invoke('pick_paths')

    if (!filePaths.length) {
      return { canceled: true, filePaths: [] }
    }

    const ok = await runOptimization(filePaths)

    return { canceled: !ok, filePaths, busy: false }
  },
  pickSaveFolder: async () => {
    const filePaths = await invoke('pick_save_folder')

    if (!filePaths.length) {
      return { canceled: true, filePaths: [] }
    }

    return { canceled: false, filePaths }
  },
  cancelOptimization: () => invoke('cancel_optimization'),
  showItemInFolder: (filePath) => revealItemInDir(filePath),
  onOpenSettings: (callback) => emitEvent('open-settings', callback),
  onBatchStarted: (callback) =>
    emitEvent('batch-started', (total) => callback(total)),
  onBatchProgress: (callback) =>
    emitEvent('batch-progress', (done, total) => callback(done, total)),
  onOptimized: (callback) =>
    emitEvent('image-optimized', (filePath, summary, sourceFileName) =>
      callback(filePath, summary, sourceFileName)
    ),
  onFileProcessing: (callback) =>
    emitEvent('file-processing', (fileName) => callback(fileName)),
  onDropError: (callback) =>
    emitEvent('drop-error', (fileName, message) => callback(fileName, message)),
  onBatchComplete: (callback) =>
    emitEvent('batch-complete', (summary) => callback(summary)),
  onBatchCancelled: (callback) =>
    emitEvent('batch-cancelled', (done, total, succeeded, failed) =>
      callback(done, total, succeeded, failed)
    ),
  checkForUpdates: (options) =>
    checkForUpdates({
      ...options,
      onStatus: options?.onStatus ?? setUpdateStatus,
    }),
  hasPendingUpdate,
  getPendingUpdateVersion,
  installPendingUpdate: (options) =>
    installPendingUpdate({
      ...options,
      onStatus: options?.onStatus ?? setUpdateStatus,
    }),
})
