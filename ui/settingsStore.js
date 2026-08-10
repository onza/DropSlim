import { load } from '@tauri-apps/plugin-store'

const OPTIMIZE_SETTING_KEYS = [
  'folderswitch',
  'suffix',
  'subfolder',
  'savepath',
  'limit_dimensions',
  'max_width',
  'max_height',
  'output_format',
]

const SETTINGS_DEFAULTS = {
  folderswitch: true,
  clearlist: false,
  suffix: true,
  subfolder: false,
  limit_dimensions: false,
  max_width: null,
  max_height: null,
  output_format: 'original',
  autoCheckUpdates: true,
  autoInstallUpdates: false,
  skipOverwriteDimensionWarning: false,
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
