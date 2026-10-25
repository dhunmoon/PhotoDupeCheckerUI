; Inno Setup script for the Windows installer. Built by packaging/package-windows.ps1:
;   iscc /DAppVersion=0.1.0 /DArch=x64 /DExeDir=target\x86_64-pc-windows-msvc\release ^
;        /DOutputName=DupeFinder-0.1.0-windows-x64-setup packaging\windows\installer.iss
; Installs per-user by default (no admin prompt); users can choose "all users" instead.
; All architectures share one AppId, so installing another arch replaces the old one.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef ExeDir
  #define ExeDir "target\release"
#endif
#ifndef OutputName
  #define OutputName "DupeFinder-" + AppVersion + "-windows-" + Arch + "-setup"
#endif

#if Arch == "x64"
  ; Also allowed on ARM64 Windows 11, which runs x64 apps under emulation.
  #define AllowedArch "x64compatible"
  #define Arch64Mode "x64compatible"
#elif Arch == "arm64"
  #define AllowedArch "arm64"
  #define Arch64Mode "arm64"
#elif Arch == "x86"
  ; 32-bit build: runs everywhere and installs into "Program Files (x86)" on 64-bit Windows.
  #define AllowedArch "x86compatible"
  #define Arch64Mode ""
#else
  #error Unknown Arch (use x64, x86 or arm64)
#endif

[Setup]
AppId={{DE12ECC1-A338-4432-B5C5-2C1CA8E43FC5}
AppName=Dupe Finder
AppVersion={#AppVersion}
AppVerName=Dupe Finder {#AppVersion}
AppPublisher=dhunmoon
AppPublisherURL=https://github.com/dhunmoon/PhotoDupeCheckerUI
AppSupportURL=https://github.com/dhunmoon/PhotoDupeCheckerUI/issues
DefaultDirName={autopf}\Dupe Finder
DefaultGroupName=Dupe Finder
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed={#AllowedArch}
ArchitecturesInstallIn64BitMode={#Arch64Mode}
LicenseFile=..\..\LICENSE
SetupIconFile=app.ico
UninstallDisplayIcon={app}\dupe_finder.exe
OutputDir=..\..\dist
OutputBaseFilename={#OutputName}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\{#ExeDir}\dupe_finder.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Dupe Finder"; Filename: "{app}\dupe_finder.exe"
Name: "{autodesktop}\Dupe Finder"; Filename: "{app}\dupe_finder.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\dupe_finder.exe"; Description: "{cm:LaunchProgram,Dupe Finder}"; Flags: nowait postinstall skipifsilent
