import { initResultsView } from './views/resultsView.js'
import { initSettingsView } from './views/settingsView.js'

export const initRenderer = (api, i18n) => {
  initResultsView(api)
  const { syncUpdateAction } = initSettingsView({ api, i18n })

  return { syncUpdateAction }
}
