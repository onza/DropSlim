import { getVersion } from '@tauri-apps/api/app'
import { createDropslimApi, initApi } from './api.js'
import { initRenderer } from './renderer.js'
import { maybeCheckForUpdates } from './updates.js'

const boot = async () => {
  document.body.classList.add('platform-mac')

  await initApi()
  const api = createDropslimApi()
  initRenderer(api)

  const versionEl = document.getElementById('appVersion')
  if (versionEl) {
    try {
      api.setAppVersion(`Version ${await getVersion()}`)
    } catch (error) {
      console.error(error)
    }
  }

  void maybeCheckForUpdates(api.settings, (message) => {
    if (message) {
      api.setUpdateStatus(message)
    }
  }).catch((error) => {
    console.error(error)
  })
}

boot().catch((error) => {
  console.error(error)
})
