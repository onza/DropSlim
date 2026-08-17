import { t, tCount } from '../i18n/index.js'
import { formatBackendError, formatBackendSummary } from '../i18n/backend.js'
import { formatResultSummaryLine } from '../utils/resultBadges.js'

const MAX_RESULT_ROWS = 100

export const initResultsView = (api) => {
  const resultBox = document.getElementById('result')
  const batchSummary = document.getElementById('batchSummary')
  const btnCancelOptimization = document.getElementById('btnCancelOptimization')
  const dragzone = document.getElementById('dragzone')
  const settings = api.settings

  let batchActive = false
  let batchDone = 0
  let batchTotal = 0
  let hiddenResultCount = 0

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

  const formatBatchStatus = (fileName) => {
    if (batchTotal > 1) {
      if (fileName) {
        return t('status.batchProgressFile', {
          done: batchDone,
          total: batchTotal,
          fileName,
        })
      }

      return t('status.batchProgress', { done: batchDone, total: batchTotal })
    }

    return fileName ? t('status.compressing', { fileName }) : ''
  }

  const updateBatchStatus = (fileName) => {
    const message = formatBatchStatus(fileName)

    if (message) {
      api.setStatus(message)
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

    const label = tCount('results.truncated', hiddenResultCount, {
      max: MAX_RESULT_ROWS,
    })

    if (!notice) {
      notice = createTextLine('resLine resLine--truncated', label)
      resultBox.appendChild(notice)
    } else {
      notice.querySelector('span').textContent = label
      resultBox.appendChild(notice)
    }
  }

  const removeProcessingLine = (fileName) => {
    resultBox.querySelectorAll('.resLine--processing').forEach((line) => {
      if (!fileName || line.dataset.fileName === fileName) {
        line.remove()
      }
    })
  }

  const addProcessingLine = (fileName) => {
    removeProcessingLine(fileName)

    const line = createTextLine(
      'resLine resLine--processing',
      t('status.compressing', { fileName })
    )
    line.dataset.fileName = fileName
    resultBox.prepend(line)
    trimResultRows()
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
    api.pickAndOptimize().catch((error) => {
      console.error(error)
    })
  }

  if (btnCancelOptimization) {
    btnCancelOptimization.onclick = (event) => {
      event.preventDefault()
      btnCancelOptimization.disabled = true
      api.cancelOptimization().catch((error) => {
        console.error(error)
        btnCancelOptimization.disabled = false
      })
    }
  }

  api.onBatchStarted((total) => {
    batchActive = total > 1
    batchDone = 0
    batchTotal = total

    if (settings.getSync().clearlist) {
      hiddenResultCount = 0
    }

    setBatchSummary('')
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

  api.onOptimized((path, summary, sourceFileName, resized) => {
    finishProcessing(sourceFileName)

    const resContainer = document.createElement('div')
    resContainer.className = 'resLine'

    const summarySpan = document.createElement('span')
    summarySpan.textContent = formatResultSummaryLine(
      formatBackendSummary(summary),
      sourceFileName,
      path,
      resized
    )
    resContainer.appendChild(summarySpan)

    const resElement = document.createElement('a')
    resElement.setAttribute('href', '#')
    resElement.textContent = path
    resElement.onclick = (event) => {
      event.preventDefault()
      api.showItemInFolder(path)
    }

    resContainer.appendChild(resElement)
    resultBox.prepend(resContainer)
    trimResultRows()
  })

  api.onDropError((fileName, error) => {
    finishProcessing(fileName)
    resultBox.prepend(
      createTextLine(
        'resLine',
        t('results.optimizeError', {
          fileName,
          message: formatBackendError(error),
        })
      )
    )
    trimResultRows()
  })

  api.onBatchComplete((summary) => {
    endBatch()
    setBatchSummary(formatBackendSummary(summary))
  })

  api.onBatchCancelled((done, total, succeeded, failed) => {
    endBatch()
    setBatchSummary(t('results.cancelled', { done, total, succeeded, failed }))
  })
}
