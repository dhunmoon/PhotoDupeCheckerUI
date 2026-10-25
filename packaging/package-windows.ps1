# Builds the Windows release artifacts for one CPU architecture into dist\:
#   DupeFinder-<version>-windows-<arch>-setup.exe    installer (Start menu entry, uninstaller)
#   DupeFinder-<version>-windows-<arch>-portable.zip portable exe, no install needed
#
# Usage: packaging\package-windows.ps1 [-Arch x64|x86|arm64]   (default: x64)
#
# Needs Rust (MSVC toolchain + the matching target) and Inno Setup 6
# (`choco install innosetup`). x86 builds cross-compile fine on an x64 machine; arm64 is
# built on an ARM machine in CI, but cross-compiling works too when the Visual Studio
# "ARM64 build tools" component is installed.
param(
    [ValidateSet('x64', 'x86', 'arm64')]
    [string]$Arch = 'x64'
)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

$target = @{
    x64   = 'x86_64-pc-windows-msvc'
    x86   = 'i686-pc-windows-msvc'
    arm64 = 'aarch64-pc-windows-msvc'
}[$Arch]
$version = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages[0].version
$name = "DupeFinder-$version-windows-$Arch"
$exeDir = "target\$target\release"

cargo build --release --locked --target $target
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }

New-Item -ItemType Directory -Force dist | Out-Null

# Portable zip: a folder with the exe, readme and license.
$stage = Join-Path ([IO.Path]::GetTempPath()) "$name-portable"
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory $stage | Out-Null
Copy-Item "$exeDir\dupe_finder.exe", README.md $stage
Copy-Item LICENSE (Join-Path $stage 'LICENSE.txt')
Compress-Archive -Path $stage -DestinationPath "dist\$name-portable.zip" -Force
Remove-Item -Recurse -Force $stage

$iscc = Get-Command iscc -ErrorAction SilentlyContinue
$isccPath = if ($iscc) { $iscc.Source } else { "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" }
& $isccPath "/DAppVersion=$version" "/DArch=$Arch" "/DExeDir=$exeDir" "/DOutputName=$name-setup" `
    packaging\windows\installer.iss
if ($LASTEXITCODE -ne 0) { throw 'Inno Setup failed' }

Get-ChildItem dist
