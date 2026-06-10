import sharp from 'sharp'
import { existsSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const root = join(dirname(fileURLToPath(import.meta.url)), '..')
const svgPath = join(root, 'assets/icon/dropslim-icon.svg')
const pngPath = join(root, 'assets/icon/icon-1024.png')

if (!existsSync(svgPath)) {
  console.error('Missing assets/icon/dropslim-icon.svg')
  process.exit(1)
}

await sharp(svgPath, { density: 300 }).resize(1024, 1024).png().toFile(pngPath)

console.log(`Rasterized ${svgPath} → ${pngPath}`)
