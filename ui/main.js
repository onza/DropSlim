import { createDropslimApi, initApi } from './api.js'
import { initRenderer } from './renderer.js'

const boot = async () => {
  document.body.classList.add('platform-mac')

  await initApi()
  initRenderer(createDropslimApi())
}

boot().catch((error) => {
  console.error(error)
})
