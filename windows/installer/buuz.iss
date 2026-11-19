; SPDX-License-Identifier: MIT OR Apache-2.0
; Inno Setup script for Buuz 2.0. Build one installer for each CPU:
;   ISCC /DArch=x64   /DSourceDir=<folder with the x64 files>   buuz.iss
;   ISCC /DArch=arm64 /DSourceDir=<folder with the ARM64 files> buuz.iss
; Add /DArm64x=1 when SourceDir holds the ARM64X forwarder (windows/arm64x).
; SourceDir holds buuz_tsf.dll, buuz_settings.exe and mn.txt for that CPU.

#ifndef Arch
  #error Pass /DArch=x64 or /DArch=arm64
#endif
#ifndef SourceDir
  #error Pass /DSourceDir=<folder>
#endif
#ifndef Version
  #define Version "2.0.0"
#endif

[Setup]
; Never change AppId: Windows uses it to find the old version on upgrade.
AppId={{C0D4E3B1-7A52-4F18-9B6E-2D8A5F1E7C30}
AppName=Buuz 2.0
AppVersion={#Version}
AppPublisher=Buuz
AppPublisherURL=https://buuz.thetengis.com
DefaultDirName={autopf}\Buuz
DefaultGroupName=Buuz
DisableProgramGroupPage=yes
OutputBaseFilename=Buuz-{#Version}-windows-{#Arch}
OutputDir=output
Compression=lzma2
SolidCompression=yes
; The text service goes into HKLM, so the installer needs an administrator.
PrivilegesRequired=admin
#if Arch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif
WizardStyle=modern
UninstallDisplayName=Buuz 2.0
; Asks the user to close apps that use the DLL, or schedules a replace at the next restart.
CloseApplications=no
RestartIfNeededByRun=no


[Files]
; `regserver` runs DllRegisterServer at install and DllUnregisterServer at uninstall.
; `restartreplace` replaces a DLL that an open app still holds at the next restart.
Source: "{#SourceDir}\buuz_tsf.dll"; DestDir: "{app}"; Flags: regserver restartreplace uninsrestartdelete 64bit
Source: "{#SourceDir}\buuz_settings.exe"; DestDir: "{app}"; Flags: ignoreversion
; The word list sits next to the DLL. The text service loads it on the first key.
Source: "{#SourceDir}\mn.txt"; DestDir: "{app}"; Flags: ignoreversion
#ifdef Arm64x
; ARM64X: buuz_tsf.dll above is a forwarder. It loads one of these two DLLs.
Source: "{#SourceDir}\buuz_tsf_arm64.dll"; DestDir: "{app}"; Flags: restartreplace uninsrestartdelete 64bit
Source: "{#SourceDir}\buuz_tsf_x64.dll"; DestDir: "{app}"; Flags: restartreplace uninsrestartdelete 64bit
#endif

[Icons]
Name: "{autoprograms}\Buuz 2.0"; Filename: "{app}\buuz_settings.exe"

[Run]
Filename: "{app}\buuz_settings.exe"; Description: "Buuz 2.0"; Flags: nowait postinstall skipifsilent
