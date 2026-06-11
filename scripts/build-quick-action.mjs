import fs from 'node:fs'
import path from 'node:path'
import { parseArgs } from 'node:util'
import { fileURLToPath } from 'node:url'

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

const { values } = parseArgs({
  options: {
    dev: { type: 'boolean', default: false },
    binary: { type: 'string' },
  },
})

const escapeBashDoubleQuoted = (value) =>
  value
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\$/g, '\\$')
    .replace(/`/g, '\\`')

const releaseShellScript = `RELEASE="/Applications/DropSlim.app/Contents/MacOS/dropslim"

if [ "$#" -eq 0 ]; then
  exit 0
fi

if [ -x "$RELEASE" ]; then
  exec "$RELEASE" "$@"
fi

APP=$(mdfind "kMDItemCFBundleIdentifier == 'com.onza.dropslim'" 2>/dev/null | head -1)
if [ -n "$APP" ] && [ -x "$APP/Contents/MacOS/dropslim" ]; then
  exec "$APP/Contents/MacOS/dropslim" "$@"
fi

osascript -e 'display alert "DropSlim not found" message "Install DropSlim in Applications and install the Finder Quick Action from DropSlim Settings." as warning'
exit 1
`

const devShellScript = (binary) => {
  if (!binary) {
    console.error('build-quick-action: --binary is required for --dev')
    process.exit(1)
  }

  const devBinary = escapeBashDoubleQuoted(path.resolve(binary))

  return `DEV="${devBinary}"

if [ "$#" -eq 0 ]; then
  exit 0
fi

if [ ! -x "$DEV" ]; then
  osascript -e 'display alert "DropSlim (Dev) not found" message "Run npm run dev first to build the debug binary." as warning'
  exit 1
fi

exec "$DEV" "$@"
`
}

const buildDocumentWflow = (
  shellScript
) => `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>523</string>
	<key>AMApplicationVersion</key>
	<string>2.10</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>Action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.path</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>${shellScript}</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>1</integer>
					<key>shell</key>
					<string>/bin/bash</string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>INPUT-UUID-DROPSLIM</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
					<string>Run</string>
					<string>Unix</string>
				</array>
				<key>OutputUUID</key>
				<string>OUTPUT-UUID-DROPSLIM</string>
				<key>UUID</key>
				<string>ACTION-UUID-DROPSLIM</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
			</dict>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>serviceInputTypeIdentifier</key>
		<string>com.apple.Automator.fileSystemObject</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<integer>0</integer>
		<key>useActionsInput</key>
		<true/>
		<key>workflowTypeIdentifiers</key>
		<array>
			<string>com.apple.Automator.servicesMenu</string>
			<string>com.apple.Automator.WFQuickAction</string>
		</array>
	</dict>
</dict>
</plist>
`

const buildInfoPlist = ({
  bundleName,
  bundleId,
}) => `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>Application Stub</string>
	<key>CFBundleIdentifier</key>
	<string>${bundleId}</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>1.0</string>
	<key>CFBundleName</key>
	<string>${bundleName}</string>
	<key>CFBundlePackageType</key>
	<string>BNDL</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>NSHumanReadableCopyright</key>
	<string></string>
</dict>
</plist>
`

const isDev = values.dev
const workflowName = isDev
  ? 'Optimize with DropSlim (Dev).workflow'
  : 'Optimize with DropSlim.workflow'
const shellScript = isDev ? devShellScript(values.binary) : releaseShellScript
const workflowDir = path.join(rootDir, 'build', workflowName, 'Contents')

fs.mkdirSync(workflowDir, { recursive: true })
fs.writeFileSync(
  path.join(workflowDir, 'document.wflow'),
  buildDocumentWflow(shellScript)
)
fs.writeFileSync(
  path.join(workflowDir, 'Info.plist'),
  buildInfoPlist({
    bundleName: isDev
      ? 'Optimize with DropSlim (Dev)'
      : 'Optimize with DropSlim',
    bundleId: isDev
      ? 'com.onza.dropslim.quickaction.dev'
      : 'com.onza.dropslim.quickaction',
  })
)

console.log(`Generated Finder Quick Action workflow (${workflowName})`)
