# Builds the Windows release artifacts into dist\:
#   dupe-finder-<version>-windows-x86_64-setup.exe  installer (Start menu entry, uninstaller)
#   dupe-finder-<version>-windows-x86_64.zip        portable exe, no install needed
#
# Needs Rust (MSVC toolchain) and Inno Setup 6 (`choco install innosetup`).
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

$version = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages[0].version
$name = "dupe-finder-$version-windows-x86_64"

cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }

New-Item -ItemType Directory -Force dist | Out-Null

$stage = Join-Path ([IO.Path]::GetTempPath()) $name
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory $stage | Out-Null
Copy-Item target\release\dupe_finder.exe, README.md $stage
Copy-Item LICENSE (Join-Path $stage 'LICENSE.txt')
Compress-Archive -Path $stage -DestinationPath "dist\$name.zip" -Force
Remove-Item -Recurse -Force $stage

$iscc = Get-Command iscc -ErrorAction SilentlyContinue
$isccPath = if ($iscc) { $iscc.Source } else { "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" }
& $isccPath "/DAppVersion=$version" packaging\windows\installer.iss
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup failed' }

Get-ChildItem dist
