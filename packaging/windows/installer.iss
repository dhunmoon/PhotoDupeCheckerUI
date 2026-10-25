; Inno Setup script for the Windows installer. Built by packaging/package-windows.ps1:
;   iscc /DAppVersion=0.1.0 packaging\windows\installer.iss
; Installs per-user by default (no admin prompt); users can choose "all users" instead.

#ifndef AppVersion
  #define AppVersion "0.0.0"
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
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
LicenseFile=..\..\LICENSE
SetupIconFile=app.ico
UninstallDisplayIcon={app}\dupe_finder.exe
OutputDir=..\..\dist
OutputBaseFilename=dupe-finder-{#AppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\dupe_finder.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Dupe Finder"; Filename: "{app}\dupe_finder.exe"
Name: "{autodesktop}\Dupe Finder"; Filename: "{app}\dupe_finder.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\dupe_finder.exe"; Description: "{cm:LaunchProgram,Dupe Finder}"; Flags: nowait postinstall skipifsilent
