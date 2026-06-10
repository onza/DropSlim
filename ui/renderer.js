export const initRenderer = (api) => {
  const settings = api.settings

  const dragzone = document.getElementById('dragzone'),
    dragzoneStatus = document.getElementById('dragzoneStatus'),
    resultBox = document.getElementById('result'),
    btnOpenSettings = document.getElementById('btnOpenSettings'),
    btnCloseSettings = document.getElementById('btnCloseSettings'),
    menuSettings = document.getElementById('menuSettings'),
    wrapper = document.querySelector('.wrapper'),
    switches = menuSettings.querySelectorAll('input[type="checkbox"]'),
    btnSavepath = document.getElementById('btnSavepath'),
    btnInstallQuickAction = document.getElementById('btnInstallQuickAction'),
    wrapperSavePath = document.getElementById('wrapperSavePath'),
    folderswitch = document.getElementById('folderswitch'),
    clearlist = document.getElementById('clearlist'),
    suffix = document.getElementById('suffix'),
    subfolder = document.getElementById('subfolder')

  let userSetting = settings.getSync()
  clearlist.checked = userSetting.clearlist
  suffix.checked = userSetting.suffix
  subfolder.checked = userSetting.subfolder

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

  const setDragzoneStatus = (message) => {
    if (!dragzoneStatus) {
      return
    }

    dragzoneStatus.textContent = message
    dragzoneStatus.hidden = !message
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

    if (!resultBox.querySelector('.resLine--processing')) {
      dragzone.classList.remove('is--processing')
      setDragzoneStatus('')
    }
  }

  dragzone.onclick = () => {
    api
      .pickAndOptimize()
      .then((result) => {
        if (result.busy) {
          setDragzoneStatus('Already compressing images…')
        }
      })
      .catch((err) => {
        console.error(err)
      })
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

  btnInstallQuickAction.onclick = (e) => {
    e.preventDefault()
    api.installQuickAction().catch((err) => {
      console.error(err)
    })
  }

  api.onOpenSettings(() => {
    openSettings()
  })

  document.onkeyup = (e) => {
    if (e.key === 'Escape') {
      closeSettings()
    }
  }

  api.onFileProcessing((fileName) => {
    dragzone.classList.add('is--processing')
    setDragzoneStatus('Compressing ' + fileName + '…')
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
  })

  api.onDropError((fileName, message) => {
    finishProcessing(fileName)
    resultBox.prepend(
      createTextLine('resLine', `Could not optimize ${fileName}: ${message}`)
    )
  })

  api.onBatchComplete((summary) => {
    dragzone.classList.remove('is--processing')
    setDragzoneStatus(summary)
    resultBox.prepend(createTextLine('resLine resLine--summary', summary))
  })
}
