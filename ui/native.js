import { invoke, isTauri } from '@tauri-apps/api/core'
import { t } from './i18n/index.js'

export const syncNativeUi = async () => {
  if (!isTauri()) {
    return
  }

  await invoke('update_native_ui', {
    strings: {
      preferences: t('menu.preferences'),
      window: t('menu.window'),
      pickImages: t('dialog.pickImages'),
      pickSaveFolder: t('dialog.pickSaveFolder'),
    },
  })
}
