; The Windows setup (Inno Setup 6), built by .github/workflows/release.yml:
;   iscc /DVersion=1.9.2 packaging\marea.iss
; It installs for the current user only (no admin prompt), into
; %LOCALAPPDATA%\Programs\Marea, so the app can still update itself in
; place. Progress lives in %APPDATA%\Marea (src/store.rs) and is kept
; unless the person asks the uninstaller to delete it. A copy installed
; before the rename (as Lingua Onde) keeps its folder: the setup reuses it,
; puts marea.exe there and removes the old exe and shortcuts.

#ifndef Version
  #define Version "0.0.0"
#endif

[Setup]
; Never change AppId: Windows finds the installed copy by it. src/update.rs
; uses it too, to show the new version in Settings, Apps after an update.
AppId={{4AC3DCD0-819C-4F78-A97A-0F9342519D3B}
AppName=Marea
AppVersion={#Version}
UninstallDisplayName=Marea
AppPublisherURL=https://github.com/Matteo404404/marea
AppSupportURL=https://github.com/Matteo404404/marea/issues
AppUpdatesURL=https://github.com/Matteo404404/marea/releases/latest
PrivilegesRequired=lowest
DefaultDirName={autopf}\Marea
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile=marea.ico
UninstallDisplayIcon={app}\marea.exe
Compression=lzma2/max
SolidCompression=yes
OutputDir=..
OutputBaseFilename=marea-setup-windows-x86_64

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\marea-windows-x86_64.exe"; DestDir: "{app}"; DestName: "marea.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Marea"; Filename: "{app}\marea.exe"
Name: "{autodesktop}\Marea"; Filename: "{app}\marea.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\marea.exe"; Description: "{cm:LaunchProgram,Marea}"; Flags: nowait postinstall skipifsilent

[InstallDelete]
; the app and shortcuts from before the rename to Marea
Type: files; Name: "{app}\lingua-onde.exe"
Type: files; Name: "{app}\lingua-onde.exe.old"
Type: files; Name: "{autoprograms}\Lingua Onde.lnk"
Type: files; Name: "{autodesktop}\Lingua Onde.lnk"

[UninstallDelete]
; what an update leaves beside the app (src/update.rs)
Type: files; Name: "{app}\marea.exe.old"
Type: files; Name: "{app}\.marea-windows-x86_64.exe.part"

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Data: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    Data := ExpandConstant('{userappdata}\Marea');
    // a copy from before the rename that never ran as Marea
    if not DirExists(Data) then Data := ExpandConstant('{userappdata}\LinguaOnde');
    if DirExists(Data) and
       (SuppressibleMsgBox('Also delete your progress, courses and downloaded voices?' + #13#10#13#10 +
         'Choose No to keep them for a later install.', mbConfirmation, MB_YESNO or MB_DEFBUTTON2, IDNO) = IDYES) then
      DelTree(Data, True, True, True);
  end;
end;
