import { Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { t } from './i18n/index.js'

const checkWithRetry = async ({ attempts = 3 } = {}) => {
  let lastError

  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      const metadata = await invoke('check_for_updates')
      return metadata ? new Update(metadata) : null
    } catch (error) {
      lastError = error
      if (attempt < attempts) {
        await new Promise((resolve) => setTimeout(resolve, 400 * attempt))
      }
    }
  }

  throw lastError
}

let pendingUpdate = null
let lastStatus = null

export const hasPendingUpdate = () => pendingUpdate !== null

export const getPendingUpdateVersion = () => pendingUpdate?.version ?? null

const clearPendingUpdate = () => {
  pendingUpdate = null
}

const formatError = (error) => {
  if (error instanceof Error && error.message) {
    return error.message
  }

  return String(error)
}

const setStatusState = (key, params, onStatus) => {
  lastStatus = { key, params }
  onStatus(t(key, params))
}

export const reportUpdateStatus = (key, params, onStatus) => {
  setStatusState(key, params ?? {}, onStatus)
}

export const reapplyUpdateStatus = (onStatus) => {
  if (lastStatus && onStatus) {
    onStatus(t(lastStatus.key, lastStatus.params))
  }
}

export const createStatusSink = (element) => {
  const apply = (message) => {
    if (element) {
      element.textContent = message || ''
    }
  }

  return {
    setMessage: apply,
    setKey: (key, params) => reportUpdateStatus(key, params ?? {}, apply),
    reapply: () => reapplyUpdateStatus(apply),
  }
}

export const installPendingUpdate = async ({ onStatus = () => {} } = {}) => {
  if (!isTauri()) {
    setStatusState('updates.requiresDesktop', {}, onStatus)
    return { installed: false, skipped: true }
  }

  const update = pendingUpdate

  if (!update) {
    setStatusState('updates.noUpdateReady', {}, onStatus)
    return { installed: false }
  }

  const { version } = update

  try {
    setStatusState('updates.downloading', { version }, onStatus)
    await update.downloadAndInstall((event) => {
      if (event.event === 'Finished') {
        setStatusState('updates.installingUpdate', {}, onStatus)
      }
    })

    clearPendingUpdate()
    await relaunch()
    return { installed: true }
  } catch (error) {
    console.error('update install failed', error)
    setStatusState(
      'updates.couldNotInstall',
      { error: formatError(error) },
      onStatus
    )
    return { installed: false, error }
  }
}

export const checkForUpdates = async ({
  autoInstall = false,
  onStatus = () => {},
  onUpdateAvailable = () => {},
} = {}) => {
  if (!isTauri()) {
    setStatusState('updates.requiresDesktop', {}, onStatus)
    return { available: false, skipped: true }
  }

  try {
    setStatusState('updates.checkingStatus', {}, onStatus)
    const update = await checkWithRetry()

    if (!update) {
      clearPendingUpdate()
      onUpdateAvailable(null)
      setStatusState('updates.upToDate', {}, onStatus)
      return { available: false }
    }

    const { version } = update
    pendingUpdate = update
    onUpdateAvailable(version)
    setStatusState('updates.available', { version }, onStatus)

    if (autoInstall) {
      return installPendingUpdate({ onStatus })
    }

    return { available: true, installed: false, version }
  } catch (error) {
    console.error('update check failed', error)
    setStatusState(
      'updates.couldNotCheckDetail',
      { error: formatError(error) },
      onStatus
    )
    return { available: false, error }
  }
}

export const maybeCheckForUpdates = async (
  settings,
  onStatus,
  onUpdateAvailable
) => {
  if (!settings.getSync().autoCheckUpdates) {
    return
  }

  await checkForUpdates({
    autoInstall: settings.getSync().autoInstallUpdates,
    onStatus,
    onUpdateAvailable,
  })
}
