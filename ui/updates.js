import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { isTauri } from '@tauri-apps/api/core'

let pendingUpdate = null

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

export const installPendingUpdate = async ({ onStatus = () => {} } = {}) => {
  if (!isTauri()) {
    onStatus('Update checks require the desktop app.')
    return { installed: false, skipped: true }
  }

  const update = pendingUpdate

  if (!update) {
    onStatus('No update ready to install.')
    return { installed: false }
  }

  const { version } = update

  try {
    onStatus(`Downloading DropSlim ${version}…`)
    await update.downloadAndInstall((event) => {
      if (event.event === 'Finished') {
        onStatus('Installing update…')
      }
    })

    clearPendingUpdate()
    await relaunch()
    return { installed: true }
  } catch (error) {
    console.error('update install failed', error)
    onStatus(`Could not install update: ${formatError(error)}`)
    return { installed: false, error }
  }
}

export const checkForUpdates = async ({
  autoInstall = false,
  onStatus = () => {},
  onUpdateAvailable = () => {},
} = {}) => {
  if (!isTauri()) {
    onStatus('Update checks require the desktop app.')
    return { available: false, skipped: true }
  }

  try {
    onStatus('Checking for updates…')
    const update = await check()

    if (!update) {
      clearPendingUpdate()
      onUpdateAvailable(null)
      onStatus('DropSlim is up to date.')
      return { available: false }
    }

    const { version } = update
    pendingUpdate = update
    onUpdateAvailable(version)
    onStatus(`Update ${version} available.`)

    if (autoInstall) {
      return installPendingUpdate({ onStatus })
    }

    return { available: true, installed: false, version }
  } catch (error) {
    console.error('update check failed', error)
    onStatus(`Could not check for updates: ${formatError(error)}`)
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
