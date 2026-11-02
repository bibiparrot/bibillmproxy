#define AppVersion GetEnv('APP_VERSION')

[Setup]
AppId={{F9A67E84-D1C6-4FE6-A654-0423E8C30A0A}
AppName=BibiLLMProxy
AppVersion={#AppVersion}
AppPublisher=bibiparrot
SourceDir=..
DefaultDirName={localappdata}\Programs\BibiLLMProxy
DefaultGroupName=BibiLLMProxy
PrivilegesRequired=lowest
OutputDir=dist
OutputBaseFilename=bibillmproxy-{#AppVersion}-windows-x64-setup
Compression=lzma
SolidCompression=yes
UninstallDisplayIcon={app}\lite-gui.exe

[Files]
Source: "target\release\lite-gui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\release\lite.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "config.yaml.example"; DestDir: "{app}"; Flags: ignoreversion
Source: "docs\desktop-gui.md"; DestDir: "{app}\docs"; Flags: ignoreversion

[Icons]
Name: "{group}\BibiLLMProxy"; Filename: "{app}\lite-gui.exe"; WorkingDir: "{app}"
Name: "{group}\Uninstall BibiLLMProxy"; Filename: "{uninstallexe}"

[Run]
Filename: "{app}\lite-gui.exe"; Description: "Launch BibiLLMProxy"; Flags: nowait postinstall skipifsilent
