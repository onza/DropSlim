import { getVersion } from '@tauri-apps/api/app'
import { type } from '@tauri-apps/plugin-os'
import { createDropslimApi, initApi } from './api.js'
import { initI18n, onLocaleChange } from './i18n/index.js'
import { syncNativeUi } from './native.js'
import { initRenderer } from './renderer.js'
import { maybeCheckForUpdates } from './updates.js'

const boot = async () => {
  const osType = type()

  if (osType === 'macos') {
    document.body.classList.add('platform-mac')
  } else {
    document.body.classList.add('platform-desktop')
    if (osType === 'windows') {
      document.body.classList.add('platform-win')
    }
  }

  const core = await initApi()
  const api = createDropslimApi(core)

  const i18n = await initI18n({
    getPreference: () => api.settings.getSync().locale,
    setPreference: (locale) => api.settings.setSync('locale', locale),
  })

  const { syncUpdateAction } = initRenderer(api, i18n)

  await syncNativeUi()

  onLocaleChange(() => {
    api.renderAppVersion()
    api.syncOutputFormatStatus()
    api.reapplyUpdateStatus()
    syncUpdateAction(api.getPendingUpdateVersion())
    void syncNativeUi()
  })

  try {
    api.setAppVersion(await getVersion())
  } catch (error) {
    console.error(error)
  }

  void maybeCheckForUpdates(
    api.settings,
    (message) => {
      if (message) {
        api.setUpdateStatus(message)
      }
    },
    syncUpdateAction
  ).catch((error) => {
    console.error(error)
  })
}

boot().catch((error) => {
  console.error(error)
})
