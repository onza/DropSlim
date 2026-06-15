import sharp from 'sharp'
import { readFileSync } from 'fs'

const [inputPath, outputPath, iconSizeArg] = process.argv.slice(2)

if (!inputPath || !outputPath) {
  console.error('Usage: node mask-icon.mjs <input.png> <output.png> [size]')
  process.exit(1)
}

// macOS app icon grid: ~82% art area, corner radius 22% of art.
const canvas = Number(iconSizeArg) || 1024
const art = Math.round(canvas * 0.82)
const radius = art * 0.22
const offset = Math.round((canvas - art) / 2)

const artwork = await sharp(readFileSync(inputPath))
  .resize(art, art, { fit: 'cover' })
  .ensureAlpha()
  .png()
  .toBuffer()

const roundedMask = Buffer.from(
  `<svg width="${art}" height="${art}" xmlns="http://www.w3.org/2000/svg">
    <rect width="${art}" height="${art}" rx="${radius}" ry="${radius}" fill="white"/>
  </svg>`
)

const macIcon = await sharp(artwork)
  .composite([{ input: roundedMask, blend: 'dest-in' }])
  .png()
  .toBuffer()

await sharp({
  create: {
    width: canvas,
    height: canvas,
    channels: 4,
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  },
})
  .composite([{ input: macIcon, left: offset, top: offset }])
  .png()
  .toFile(outputPath)
