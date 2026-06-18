export const initRenderer = (api) => {
  const settings = api.settings
  const MAX_RESULT_ROWS = 100

  const dragzone = document.getElementById('dragzone'),
    batchSummary = document.getElementById('batchSummary'),
    resultBox = document.getElementById('result'),
    btnOpenSettings = document.getElementById('btnOpenSettings'),
    btnCloseSettings = document.getElementById('btnCloseSettings'),
    btnCancelOptimization = document.getElementById('btnCancelOptimization'),
    menuSettings = document.getElementById('menuSettings'),
    wrapper = document.querySelector('.wrapper'),
    switches = menuSettings.querySelectorAll('input[type="checkbox"]'),
    btnSavepath = document.getElementById('btnSavepath'),
    wrapperSavePath = document.getElementById('wrapperSavePath'),
    folderswitch = document.getElementById('folderswitch'),
    clearlist = document.getElementById('clearlist'),
    suffix = document.getElementById('suffix'),
    subfolder = document.getElementById('subfolder'),
    autoCheckUpdates = document.getElementById('autoCheckUpdates'),
    autoInstallUpdates = document.getElementById('autoInstallUpdates'),
    btnCheckUpdates = document.getElementById('btnCheckUpdates')

  let userSetting = settings.getSync()
  let batchActive = false
  let batchDone = 0
  let batchTotal = 0
  let hiddenResultCount = 0

  clearlist.checked = userSetting.clearlist
  suffix.checked = userSetting.suffix
  subfolder.checked = userSetting.subfolder
  autoCheckUpdates.checked = userSetting.autoCheckUpdates
  autoInstallUpdates.checked = userSetting.autoInstallUpdates

  if (userSetting.folderswitch === false) {
    folderswitch.checked = false
    wrapperSavePath.classList.remove('is-hidden')
  } else {
    folderswitch.checked = true
  }

  const cutFolderName = (path, length = 20) => {
    return path.length >= length ? '... ' + path.slice(-length) : path
  }

  if (userSetting.savepath) {
    btnSavepath.innerText = cutFolderName(userSetting.savepath[0], 48)
  }

  const setCancelVisible = (visible) => {
    if (btnCancelOptimization) {
      btnCancelOptimization.hidden = !visible
    }
  }

  const setBatchSummary = (message) => {
    if (!batchSummary) {
      return
    }

    batchSummary.textContent = message
    batchSummary.hidden = !message
  }

  const clearBatchSummary = () => {
    setBatchSummary('')
  }

  const formatBatchStatus = (fileName) => {
    if (batchTotal > 1) {
      const progress = `${batchDone} / ${batchTotal}`
      return fileName ? `${progress} — ${fileName}` : progress
    }

    return fileName ? `Compressing ${fileName}…` : ''
  }

  const updateBatchStatus = (fileName) => {
    const message = formatBatchStatus(fileName)

    if (message) {
      api.setStatus(message)
    }
  }

  const trimResultRows = () => {
    const lines = resultBox.querySelectorAll(
      '.resLine:not(.resLine--processing):not(.resLine--truncated)'
    )

    let removed = 0

    for (let index = lines.length - 1; index >= MAX_RESULT_ROWS; index -= 1) {
      lines[index].remove()
      removed += 1
    }

    if (removed > 0) {
      hiddenResultCount += removed
      updateTruncationNotice()
    }
  }

  const updateTruncationNotice = () => {
    let notice = resultBox.querySelector('.resLine--truncated')

    if (hiddenResultCount <= 0) {
      notice?.remove()
      return
    }

    const label =
      hiddenResultCount === 1
        ? `1 older result not shown — only the latest ${MAX_RESULT_ROWS} are listed.`
        : `${hiddenResultCount} older results not shown — only the latest ${MAX_RESULT_ROWS} are listed.`

    if (!notice) {
      notice = createTextLine('resLine resLine--truncated', label)
      resultBox.appendChild(notice)
    } else {
      notice.querySelector('span').textContent = label
      resultBox.appendChild(notice)
    }
  }

  const createTextLine = (className, text) => {
    const line = document.createElement('div')
    line.className = className
    const span = document.createElement('span')
    span.textContent = text
    line.appendChild(span)
    return line
  }

  const addProcessingLine = (fileName) => {
    removeProcessingLine(fileName)

    const line = createTextLine(
      'resLine resLine--processing',
      `Compressing ${fileName}…`
    )
    line.dataset.fileName = fileName
    resultBox.prepend(line)
    trimResultRows()
  }

  const removeProcessingLine = (fileName) => {
    resultBox.querySelectorAll('.resLine--processing').forEach((line) => {
      if (!fileName || line.dataset.fileName === fileName) {
        line.remove()
      }
    })
  }

  const finishProcessing = (fileName) => {
    removeProcessingLine(fileName)

    if (batchActive) {
      return
    }

    if (!resultBox.querySelector('.resLine--processing')) {
      api.endProcessing()
      setCancelVisible(false)
    }
  }

  const endBatch = () => {
    batchActive = false
    batchDone = 0
    batchTotal = 0
    api.endProcessing()
    setCancelVisible(false)
    removeProcessingLine()
  }

  dragzone.onclick = () => {
    api
      .pickAndOptimize()
      .then((result) => {
        if (result.busy) {
          api.setStatus('Already compressing images…')
        }
      })
      .catch((err) => {
        console.error(err)
      })
  }

  if (btnCancelOptimization) {
    btnCancelOptimization.onclick = (event) => {
      event.preventDefault()
      btnCancelOptimization.disabled = true
      api.cancelOptimization().catch((err) => {
        console.error(err)
        btnCancelOptimization.disabled = false
      })
    }
  }

  btnSavepath.onclick = () => {
    api
      .pickSaveFolder()
      .then((result) => {
        if (result.filePaths) {
          btnSavepath.innerText = cutFolderName(result.filePaths[0], 48)
          settings.setSync('savepath', result.filePaths)
        }
      })
      .catch((err) => {
        console.error(err)
      })
  }

  if (btnCheckUpdates) {
    btnCheckUpdates.onclick = (event) => {
      event.preventDefault()
      btnCheckUpdates.disabled = true
      api
        .checkForUpdates({ autoInstall: false })
        .catch((err) => {
          console.error(err)
        })
        .finally(() => {
          btnCheckUpdates.disabled = false
        })
    }
  }

  Array.from(switches).forEach((switchEl) => {
    switchEl.onchange = (e) => {
      settings.setSync(e.target['name'], e.target['checked'])
      if (e.target['name'] === 'folderswitch') {
        if (!e.target['checked']) {
          wrapperSavePath.classList.remove('is-hidden')
        } else {
          wrapperSavePath.classList.add('is-hidden')
        }
      }
    }
  })

  const openSettings = () => {
    menuSettings.classList.add('is--open')
    wrapper.classList.add('is--settings-open')
  }

  const closeSettings = () => {
    menuSettings.classList.remove('is--open')
    wrapper.classList.remove('is--settings-open')
  }

  btnOpenSettings.onclick = (e) => {
    e.preventDefault()
    openSettings()
  }

  btnCloseSettings.onclick = (e) => {
    e.preventDefault()
    closeSettings()
  }

  api.onOpenSettings(() => {
    openSettings()
  })

  document.onkeyup = (e) => {
    if (e.key === 'Escape') {
      closeSettings()
    }
  }

  api.onBatchStarted((total) => {
    batchActive = total > 1
    batchDone = 0
    batchTotal = total

    if (settings.getSync().clearlist) {
      hiddenResultCount = 0
    }

    clearBatchSummary()
    api.setProcessing(true)

    if (batchActive) {
      setCancelVisible(true)
      if (btnCancelOptimization) {
        btnCancelOptimization.disabled = false
      }
      updateBatchStatus()
    }
  })

  api.onBatchProgress((done, total) => {
    batchDone = done
    batchTotal = total
    updateBatchStatus()
  })

  api.onFileProcessing((fileName) => {
    api.setProcessing(true)
    updateBatchStatus(fileName)
    addProcessingLine(fileName)
  })

  api.onOptimized((path, summary, sourceFileName) => {
    finishProcessing(sourceFileName)

    const resContainer = document.createElement('div')
    resContainer.className = 'resLine'

    const summarySpan = document.createElement('span')
    summarySpan.textContent = summary
    resContainer.appendChild(summarySpan)

    const resElement = document.createElement('a')
    resElement.setAttribute('href', '#')
    resElement.textContent = path

    resElement.onclick = (el) => {
      el.preventDefault()
      api.showItemInFolder(path)
    }

    resContainer.appendChild(resElement)
    resultBox.prepend(resContainer)
    trimResultRows()
  })

  api.onDropError((fileName, message) => {
    finishProcessing(fileName)
    resultBox.prepend(
      createTextLine('resLine', `Could not optimize ${fileName}: ${message}`)
    )
    trimResultRows()
  })

  api.onBatchComplete((summary) => {
    endBatch()
    setBatchSummary(summary)
  })

  api.onBatchCancelled((done, total, succeeded, failed) => {
    const summary = `Cancelled after ${done} / ${total} (${succeeded} optimized, ${failed} failed)`
    endBatch()
    setBatchSummary(summary)
  })
}
