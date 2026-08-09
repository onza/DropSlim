import { t } from './i18n/index.js'

let openResolver = null

const getElements = () => {
  const root = document.getElementById('confirmDialog')
  return {
    root,
    title: document.getElementById('confirmDialogTitle'),
    body: document.getElementById('confirmDialogBody'),
    dontAsk: document.getElementById('confirmDialogDontAsk'),
    dontAskLabel: document.getElementById('confirmDialogDontAskLabel'),
    cancel: document.getElementById('confirmDialogCancel'),
    confirm: document.getElementById('confirmDialogConfirm'),
  }
}

const closeDialog = (proceed) => {
  const { root, dontAsk, cancel, confirm } = getElements()
  if (!root || !openResolver) {
    return
  }

  const dontAskAgain = Boolean(dontAsk?.checked)
  root.classList.add('is-hidden')
  root.setAttribute('aria-hidden', 'true')
  document.removeEventListener('keydown', onKeyDown, true)
  cancel.onclick = null
  confirm.onclick = null

  const resolve = openResolver
  openResolver = null
  resolve({ proceed, dontAskAgain })
}

const onKeyDown = (event) => {
  if (event.key !== 'Escape') {
    return
  }

  event.preventDefault()
  event.stopPropagation()
  closeDialog(false)
}

export const confirmOverwriteDimensionWarning = () => {
  const elements = getElements()
  if (!elements.root || openResolver) {
    return Promise.resolve({ proceed: true, dontAskAgain: false })
  }

  elements.title.textContent = t('dialog.overwriteDimensionWarningTitle')
  elements.body.textContent = t('dialog.overwriteDimensionWarningBody')
  elements.dontAskLabel.textContent = t('dialog.dontAskAgain')
  elements.cancel.textContent = t('footer.cancel')
  elements.confirm.textContent = t('dialog.overwriteDimensionWarningConfirm')
  elements.dontAsk.checked = false

  elements.root.classList.remove('is-hidden')
  elements.root.setAttribute('aria-hidden', 'false')
  document.addEventListener('keydown', onKeyDown, true)

  elements.cancel.onclick = () => closeDialog(false)
  elements.confirm.onclick = () => closeDialog(true)
  elements.confirm.focus()

  return new Promise((resolve) => {
    openResolver = resolve
  })
}
