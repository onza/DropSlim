import { load } from '@tauri-apps/plugin-store'

const OPTIMIZE_SETTING_KEYS = [
  'folderswitch',
  'suffix',
  'subfolder',
  'savepath',
]

const SETTINGS_DEFAULTS = {
  folderswitch: true,
  clearlist: false,
  suffix: true,
  subfolder: false,
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  locale: 'system',
}

export const createSettingsStore = async () => {
  const store = await load('settings.json', {
    autoSave: true,
    defaults: SETTINGS_DEFAULTS,
  })
  const entries = await store.entries()
  let cachedSettings = {
    ...SETTINGS_DEFAULTS,
    ...Object.fromEntries(entries),
  }

  return {
    getSync: () => cachedSettings,
    setSync: (key, value) => {
      cachedSettings[key] = value
      store.set(key, value).catch((error) => console.error(error))
    },
    pickOptimizeSettings: (settings = cachedSettings) =>
      Object.fromEntries(
        OPTIMIZE_SETTING_KEYS.filter((key) => key in settings).map((key) => [
          key,
          settings[key],
        ])
      ),
  }
}
