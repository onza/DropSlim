import { t } from './i18n/index.js'

let openResolver = null

const getElements = () => {
  const root = document.getElementById('confirmDialog')
  const dontAsk = document.getElementById('confirmDialogDontAsk')
  return {
    root,
    title: document.getElementById('confirmDialogTitle'),
    body: document.getElementById('confirmDialogBody'),
    dontAsk,
    dontAskRow: dontAsk?.closest('label') ?? null,
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

export const showConfirmDialog = ({
  title,
  body,
  confirmLabel,
  cancelLabel,
  dontAskAgainLabel,
  showDontAskAgain = true,
}) => {
  const elements = getElements()
  if (!elements.root || openResolver) {
    return Promise.resolve({ proceed: true, dontAskAgain: false })
  }

  elements.title.textContent = title
  elements.body.textContent = body
  elements.cancel.textContent = cancelLabel
  elements.confirm.textContent = confirmLabel
  elements.dontAsk.checked = false

  if (showDontAskAgain) {
    elements.dontAskLabel.textContent = dontAskAgainLabel
    elements.dontAskRow?.classList.remove('is-hidden')
  } else {
    elements.dontAskRow?.classList.add('is-hidden')
  }

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

export const confirmOverwriteDimensionWarning = () =>
  showConfirmDialog({
    title: t('dialog.overwriteDimensionWarningTitle'),
    body: t('dialog.overwriteDimensionWarningBody'),
    confirmLabel: t('dialog.overwriteDimensionWarningConfirm'),
    cancelLabel: t('footer.cancel'),
    dontAskAgainLabel: t('dialog.dontAskAgain'),
  })

export const confirmConvertOutputFormatWarning = (formatLabel) =>
  showConfirmDialog({
    title: t('dialog.convertOutputFormatWarningTitle'),
    body: t('dialog.convertOutputFormatWarningBody', { format: formatLabel }),
    confirmLabel: t('dialog.convertOutputFormatWarningConfirm'),
    cancelLabel: t('footer.cancel'),
    dontAskAgainLabel: t('dialog.dontAskAgain'),
  })

export const confirmDisableMinSuffixWarning = () =>
  showConfirmDialog({
    title: t('dialog.disableMinSuffixWarningTitle'),
    body: t('dialog.disableMinSuffixWarningBody'),
    confirmLabel: t('dialog.disableMinSuffixWarningConfirm'),
    cancelLabel: t('footer.cancel'),
    showDontAskAgain: false,
  })
