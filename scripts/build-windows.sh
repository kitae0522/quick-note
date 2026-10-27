#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
target="x86_64-pc-windows-gnu"
command -v x86_64-w64-mingw32-gcc >/dev/null || { echo "Install MinGW-w64 first (on macOS: brew install mingw-w64)." >&2; exit 2; }
cargo build --locked --release --target "$target" --bin quick-note
mkdir -p dist/windows/Licenses dist/windows/i18n
cp i18n/*.json dist/windows/i18n/
cp "target/$target/release/quick-note.exe" "dist/windows/Quick Note.exe"
cp assets/licenses/*.txt dist/windows/Licenses/
cp assets/licenses/Third-party.html dist/windows/Licenses/
cp LICENSE dist/windows/Licenses/Quick-Note-LICENSE.txt
cp docs/windows-readme.txt dist/windows/Readme.txt
cd dist/windows
rm -f '../Quick-Note-windows-x86_64.zip'
zip -q -r '../Quick-Note-windows-x86_64.zip' 'Quick Note.exe' Licenses Readme.txt i18n
