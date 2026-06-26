import { execSync } from 'node:child_process'
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import sharp from 'sharp'
import toIco from 'to-ico'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const source = path.join(root, 'assets/icon/icon-1024.png')
const buildDir = path.join(root, 'build')
const macosIcon = path.join(buildDir, 'icon-macos.png')
const iconset = path.join(buildDir, 'icon.iconset')
const icns = path.join(buildDir, 'icon.icns')
const iconSizes = [
  [16, 'icon_16x16.png'],
  [32, 'icon_16x16@2x.png'],
  [32, 'icon_32x32.png'],
  [64, 'icon_32x32@2x.png'],
  [128, 'icon_128x128.png'],
  [256, 'icon_128x128@2x.png'],
  [256, 'icon_256x256.png'],
  [512, 'icon_256x256@2x.png'],
  [512, 'icon_512x512.png'],
  [1024, 'icon_512x512@2x.png'],
]
const icoSizes = [16, 24, 32, 48, 64, 128, 256]

async function maskIcon(inputPath, outputPath, canvas = 1024) {
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
}

async function writeIco(inputPath, outputPath) {
  const buffers = await Promise.all(
    icoSizes.map((size) =>
      sharp(readFileSync(inputPath))
        .resize(size, size, { fit: 'cover' })
        .png()
        .toBuffer()
    )
  )

  writeFileSync(outputPath, await toIco(buffers))
}

if (!existsSync(source)) {
  console.error(`build-icons: missing ${source}`)
  process.exit(1)
}

mkdirSync(buildDir, { recursive: true })
await maskIcon(source, macosIcon, 1024)

const iconsDir = path.join(root, 'src-tauri/icons')
mkdirSync(iconsDir, { recursive: true })
cpSync(macosIcon, path.join(iconsDir, 'icon.png'))
await writeIco(macosIcon, path.join(iconsDir, 'icon.ico'))

if (process.platform === 'darwin') {
  try {
    execSync('command -v iconutil >/dev/null')
  } catch {
    console.warn('build-icons: iconutil not found, skipping .icns generation')
  }

  if (existsSync('/usr/bin/iconutil')) {
    rmSync(iconset, { recursive: true, force: true })
    mkdirSync(iconset, { recursive: true })

    for (const [size, fileName] of iconSizes) {
      const output = path.join(iconset, fileName)
      execSync(`sips -z ${size} ${size} "${macosIcon}" --out "${output}"`, {
        stdio: 'ignore',
      })
    }

    execSync(`iconutil -c icns "${iconset}" -o "${icns}"`, { stdio: 'inherit' })
    cpSync(icns, path.join(iconsDir, 'icon.icns'))
    rmSync(iconset, { recursive: true, force: true })
  }
} else {
  console.log('build-icons: skipping .icns generation (not macOS)')
}

console.log('build-icons: ok (icon-1024.png → src-tauri/icons/)')
