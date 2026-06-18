import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import { execFileSync } from 'node:child_process'
import binBuild from 'bin-build'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const rootDir = path.join(__dirname, '..')
const vendorDir = path.join(rootDir, 'vendor', 'gifsicle')
const sourceDir = path.join(rootDir, 'vendor', 'source')
const sourceArchive = path.join(sourceDir, 'gifsicle-1.96.tar.gz')
const sourceUrl = 'https://www.lcdf.org/gifsicle/gifsicle-1.96.tar.gz'
const binaryName = process.platform === 'win32' ? 'gifsicle.exe' : 'gifsicle'
const binaryPath = path.join(vendorDir, binaryName)
const versionPattern = /1\.96/

if (process.env.CI_SKIP_GIFSICLE === '1') {
  console.log('gifsicle: skipped (CI_SKIP_GIFSICLE)')
  process.exit(0)
}

const runVersionCheck = (targetPath = binaryPath) => {
  const output = execFileSync(targetPath, ['--version'], {
    encoding: 'utf8',
  })

  return versionPattern.test(output)
}

const downloadSource = async () => {
  fs.mkdirSync(sourceDir, { recursive: true })

  if (fs.existsSync(sourceArchive)) {
    return
  }

  const response = await fetch(sourceUrl)

  if (!response.ok) {
    throw new Error(`Failed to download gifsicle source (${response.status})`)
  }

  fs.writeFileSync(sourceArchive, Buffer.from(await response.arrayBuffer()))
}

const buildGifsicle = async (destDir, arch) => {
  fs.mkdirSync(destDir, { recursive: true })

  const config = [
    './configure --disable-gifview --disable-gifdiff',
    `--prefix="${destDir}" --bindir="${destDir}"`,
  ].join(' ')

  const buildEnv = { ...process.env }

  if (arch === 'arm64') {
    buildEnv.CC = 'clang -arch arm64'
    buildEnv.CXX = 'clang++ -arch arm64'
    buildEnv.CFLAGS = '-arch arm64'
    buildEnv.LDFLAGS = '-arch arm64'
  } else if (arch === 'x86_64') {
    buildEnv.CC = 'clang -arch x86_64'
    buildEnv.CXX = 'clang++ -arch x86_64'
    buildEnv.CFLAGS = '-arch x86_64'
    buildEnv.LDFLAGS = '-arch x86_64'
  }

  const commands = [config, 'make install']

  if (arch === 'x86_64' && process.platform === 'darwin') {
    await binBuild.file(
      sourceArchive,
      commands.map((command) => {
        const escaped = command.replace(/'/g, "'\\''")
        return `arch -x86_64 /bin/sh -c '${escaped}'`
      }),
      {
        env: buildEnv,
      }
    )
  } else {
    await binBuild.file(sourceArchive, commands, {
      env: buildEnv,
    })
  }

  const builtBinary = path.join(destDir, binaryName)

  if (process.platform !== 'win32') {
    fs.chmodSync(builtBinary, 0o755)
  }

  return builtBinary
}

const buildDarwinUniversal = async () => {
  const armDir = path.join(vendorDir, 'build-arm64')
  const x64Dir = path.join(vendorDir, 'build-x64')
  const armBinary = await buildGifsicle(armDir, 'arm64')

  let x64Binary

  try {
    x64Binary = await buildGifsicle(x64Dir, 'x86_64')

    if (
      execFileSync('lipo', ['-info', x64Binary], { encoding: 'utf8' }).includes(
        'arm64'
      )
    ) {
      throw new Error('x86_64 build produced an arm64 binary')
    }
  } catch (error) {
    console.warn(
      'x86_64 gifsicle build failed, using arm64 binary only:',
      error.message
    )
    fs.mkdirSync(vendorDir, { recursive: true })
    fs.copyFileSync(armBinary, binaryPath)
    fs.chmodSync(binaryPath, 0o755)
    return
  }

  fs.mkdirSync(vendorDir, { recursive: true })
  execFileSync('lipo', ['-create', '-output', binaryPath, armBinary, x64Binary])
  fs.chmodSync(binaryPath, 0o755)
}

const installGifsicle = async () => {
  fs.mkdirSync(vendorDir, { recursive: true })

  if (process.platform === 'darwin') {
    await buildDarwinUniversal()
    return
  }

  await buildGifsicle(vendorDir)
}

try {
  if (fs.existsSync(binaryPath) && runVersionCheck()) {
    console.log('gifsicle 1.96 already installed')
    process.exit(0)
  }

  console.log('Installing gifsicle 1.96 from source…')
  await downloadSource()
  await installGifsicle()

  if (!runVersionCheck()) {
    throw new Error('gifsicle 1.96 build verification failed')
  }

  if (process.platform === 'darwin') {
    try {
      const archInfo = execFileSync('lipo', ['-info', binaryPath], {
        encoding: 'utf8',
      }).trim()
      console.log(`gifsicle 1.96 installed successfully (${archInfo})`)
    } catch {
      console.log('gifsicle 1.96 installed successfully')
    }
  } else {
    console.log(`gifsicle 1.96 installed successfully (${os.arch()})`)
  }
} catch (error) {
  console.error(error)
  process.exit(1)
}
