import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { isTauri } from '@tauri-apps/api/core'

export const checkForUpdates = async ({
  autoInstall = false,
  onStatus = () => {},
} = {}) => {
  if (!isTauri()) {
    return { available: false, skipped: true }
  }

  try {
    const update = await check()

    if (!update) {
      onStatus('DropSlim is up to date.')
      return { available: false }
    }

    const { version, body } = update

    if (!autoInstall) {
      const notes = body ? `\n\n${body}` : ''
      const install = window.confirm(
        `DropSlim ${version} is available.${notes}\n\nInstall now?`
      )

      if (!install) {
        onStatus(`Update ${version} available.`)
        return { available: true, installed: false }
      }
    }

    onStatus(`Downloading DropSlim ${version}…`)
    await update.downloadAndInstall((event) => {
      if (event.event === 'Finished') {
        onStatus('Installing update…')
      }
    })

    await relaunch()
    return { available: true, installed: true }
  } catch (error) {
    console.error('update check failed', error)
    onStatus('Could not check for updates.')
    return { available: false, error }
  }
}

export const maybeCheckForUpdates = async (settings, onStatus) => {
  if (!settings.getSync().autoCheckUpdates) {
    return
  }

  await checkForUpdates({
    autoInstall: settings.getSync().autoInstallUpdates,
    onStatus,
  })
}
