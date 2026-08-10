import {
  getLocaleOptions,
  getStoredLocalePreference,
  onLocaleChange,
  t,
} from '../i18n/index.js'
import { confirmDisableMinSuffixWarning } from '../confirmDialog.js'
import { cutFolderName } from '../utils/pathDisplay.js'

const parseDimensionPx = (raw) => {
  const trimmed = String(raw ?? '').trim()
  if (!trimmed) {
    return null
  }

  const value = Number.parseInt(trimmed, 10)
  if (!Number.isFinite(value) || value < 1) {
    return null
  }

  return value
}

export const initSettingsView = ({ api, i18n }) => {
  const menuSettings = document.getElementById('menuSettings')
  const wrapper = document.querySelector('.wrapper')
  const btnOpenSettings = document.getElementById('btnOpenSettings')
  const btnCloseSettings = document.getElementById('btnCloseSettings')
  const btnSavepath = document.getElementById('btnSavepath')
  const wrapperSavePath = document.getElementById('wrapperSavePath')
  const folderswitch = document.getElementById('folderswitch')
  const clearlist = document.getElementById('clearlist')
  const suffix = document.getElementById('suffix')
  const subfolder = document.getElementById('subfolder')
  const limitDimensions = document.getElementById('limit_dimensions')
  const wrapperDimensionLimits = document.getElementById(
    'wrapperDimensionLimits'
  )
  const maxWidth = document.getElementById('max_width')
  const maxHeight = document.getElementById('max_height')
  const autoCheckUpdates = document.getElementById('autoCheckUpdates')
  const autoInstallUpdates = document.getElementById('autoInstallUpdates')
  const btnCheckUpdates = document.getElementById('btnCheckUpdates')
  const localeSelect = document.getElementById('localeSelect')
  const localeValue = document.getElementById('localeValue')
  const switches =
    menuSettings?.querySelectorAll('input[type="checkbox"]') ?? []

  const settings = api.settings
  let userSetting = settings.getSync()

  const syncDimensionLimitsVisibility = (enabled) => {
    wrapperDimensionLimits?.classList.toggle('is-hidden', !enabled)
  }

  const updateLocaleDisplay = () => {
    if (!localeSelect || !localeValue) {
      return
    }

    const selected = localeSelect.options[localeSelect.selectedIndex]
    localeValue.textContent = selected?.textContent ?? ''
  }

  const populateLocaleSelect = () => {
    if (!localeSelect) {
      return
    }

    const current = getStoredLocalePreference()
    localeSelect.innerHTML = ''

    getLocaleOptions().forEach(({ value, labelKey }) => {
      const option = document.createElement('option')
      option.value = value
      option.textContent = t(labelKey)
      option.selected = value === current
      localeSelect.appendChild(option)
    })

    updateLocaleDisplay()
  }

  populateLocaleSelect()

  onLocaleChange(() => {
    populateLocaleSelect()
  })

  if (localeSelect) {
    localeSelect.onchange = async (event) => {
      updateLocaleDisplay()
      await i18n.setPreference(event.target.value)
    }
  }

  clearlist.checked = userSetting.clearlist
  suffix.checked = userSetting.suffix
  subfolder.checked = userSetting.subfolder
  limitDimensions.checked = Boolean(userSetting.limit_dimensions)
  syncDimensionLimitsVisibility(limitDimensions.checked)
  if (maxWidth) {
    maxWidth.value =
      userSetting.max_width == null ? '' : String(userSetting.max_width)
  }
  if (maxHeight) {
    maxHeight.value =
      userSetting.max_height == null ? '' : String(userSetting.max_height)
  }
  autoCheckUpdates.checked = userSetting.autoCheckUpdates
  autoInstallUpdates.checked = userSetting.autoInstallUpdates

  if (userSetting.folderswitch === false) {
    folderswitch.checked = false
    wrapperSavePath?.classList.remove('is-hidden')
  } else {
    folderswitch.checked = true
  }

  if (userSetting.savepath && btnSavepath) {
    btnSavepath.innerText = cutFolderName(userSetting.savepath[0], 48)
  }

  const syncUpdateAction = (version) => {
    if (!btnCheckUpdates) {
      return
    }

    btnCheckUpdates.textContent = version
      ? t('updates.install')
      : t('updates.checkNow')
  }

  const openSettings = () => {
    menuSettings?.classList.add('is--open')
    wrapper?.classList.add('is--settings-open')
    populateLocaleSelect()
    api.reapplyUpdateStatus()
    syncUpdateAction(api.getPendingUpdateVersion())
  }

  const closeSettings = () => {
    menuSettings?.classList.remove('is--open')
    wrapper?.classList.remove('is--settings-open')
  }

  btnOpenSettings.onclick = (event) => {
    event.preventDefault()
    openSettings()
  }

  btnCloseSettings.onclick = (event) => {
    event.preventDefault()
    closeSettings()
  }

  api.onOpenSettings(() => {
    openSettings()
  })

  document.onkeyup = (event) => {
    if (event.key === 'Escape') {
      closeSettings()
    }
  }

  btnSavepath.onclick = () => {
    api
      .pickSaveFolder()
      .then((result) => {
        if (result.filePaths?.length) {
          btnSavepath.innerText = cutFolderName(result.filePaths[0], 48)
          settings.setSync('savepath', result.filePaths)
        }
      })
      .catch((error) => {
        console.error(error)
      })
  }

  Array.from(switches).forEach((switchEl) => {
    switchEl.onchange = (event) => {
      const { name, checked } = event.target

      if (name === 'suffix' && !checked) {
        event.target.checked = true
        void confirmDisableMinSuffixWarning().then(({ proceed }) => {
          if (!proceed) {
            return
          }
          event.target.checked = false
          settings.setSync('suffix', false)
        })
        return
      }

      settings.setSync(name, checked)

      if (name === 'folderswitch' && wrapperSavePath) {
        wrapperSavePath.classList.toggle('is-hidden', checked)
      }

      if (name === 'limit_dimensions') {
        syncDimensionLimitsVisibility(checked)
      }
    }
  })

  const bindDimensionInput = (input) => {
    if (!input) {
      return
    }

    input.onchange = (event) => {
      const value = parseDimensionPx(event.target.value)
      event.target.value = value == null ? '' : String(value)
      settings.setSync(event.target.name, value)
    }
  }

  bindDimensionInput(maxWidth)
  bindDimensionInput(maxHeight)

  if (btnCheckUpdates) {
    btnCheckUpdates.onclick = (event) => {
      event.preventDefault()
      btnCheckUpdates.disabled = true

      if (api.hasPendingUpdate()) {
        btnCheckUpdates.textContent = t('updates.installing')
        api
          .installPendingUpdate()
          .catch((error) => {
            console.error(error)
          })
          .finally(() => {
            btnCheckUpdates.disabled = false
            syncUpdateAction(api.getPendingUpdateVersion())
          })
        return
      }

      btnCheckUpdates.textContent = t('updates.checking')
      api
        .checkForUpdates({
          onUpdateAvailable: syncUpdateAction,
        })
        .catch((error) => {
          console.error(error)
          api.reportUpdateStatus('updates.couldNotCheck')
        })
        .finally(() => {
          btnCheckUpdates.disabled = false
          syncUpdateAction(api.getPendingUpdateVersion())
        })
    }
  }

  return { syncUpdateAction }
}
