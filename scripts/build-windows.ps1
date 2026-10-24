$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
cargo build --locked --release --target x86_64-pc-windows-msvc --bin quick-note
if ($LASTEXITCODE -ne 0) { throw 'Rust build failed.' }
New-Item -ItemType Directory -Force 'dist/windows/Licenses' | Out-Null
Copy-Item 'target/x86_64-pc-windows-msvc/release/quick-note.exe' 'dist/windows/Quick Note.exe'
Copy-Item 'assets/licenses/*.txt' 'dist/windows/Licenses/'
Copy-Item 'docs/windows-readme.txt' 'dist/windows/Readme.txt'
New-Item -ItemType Directory -Force 'dist/windows/i18n' | Out-Null
Copy-Item 'i18n/*.json' 'dist/windows/i18n/'
Compress-Archive -Force -Path 'dist/windows/Quick Note.exe', 'dist/windows/Licenses', 'dist/windows/Readme.txt', 'dist/windows/i18n' -DestinationPath 'dist/Quick-Note-windows-x86_64.zip'
