#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
configuration="${1:-release}"
if [[ "$configuration" != release && "$configuration" != debug ]]; then
  echo "Usage: scripts/build-app.sh [release|debug]" >&2
  exit 2
fi
if [[ "$(uname -s)" != Darwin ]]; then
  echo "Build the macOS app on a Mac. Use scripts/build-windows.ps1 on Windows." >&2
  exit 2
fi
if [[ "$configuration" == release ]]; then
  cargo build --locked --release --bin quick-note
else
  cargo build --locked --bin quick-note
fi
binary_directory="$(pwd)/target/$configuration"
application="$(pwd)/dist/Quick Note.app"
mkdir -p "$application/Contents/MacOS" "$application/Contents/Resources"
cp "$binary_directory/quick-note" "$application/Contents/MacOS/QuickNote"
cp assets/AppIcon.icns "$application/Contents/Resources/AppIcon.icns"
mkdir -p "$application/Contents/Resources/Licenses"
cp assets/licenses/*.txt "$application/Contents/Resources/Licenses/"
mkdir -p "$application/Contents/Resources/i18n"
cp i18n/*.json "$application/Contents/Resources/i18n/"
cat > "$application/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Quick Note</string>
  <key>CFBundleDisplayName</key><string>Quick Note</string>
  <key>CFBundleIdentifier</key><string>local.quicknote.mac</string>
  <key>CFBundleExecutable</key><string>QuickNote</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleLocalizations</key><array><string>en</string><string>ko</string></array>
  <key>CFBundleShortVersionString</key><string>0.2.0</string>
  <key>CFBundleVersion</key><string>2</string>
  <key>LSMinimumSystemVersion</key><string>14.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
  <key>NSHumanReadableCopyright</key><string>Quick Note</string>
</dict></plist>
PLIST
codesign --force --sign - "$application"
printf '%s\n' "$application"
