; The Windows setup (Inno Setup 6), built by .github/workflows/release.yml:
;   iscc /DVersion=1.9.2 packaging\lingua-onde.iss
; It installs for the current user only (no admin prompt), into
; %LOCALAPPDATA%\Programs\Lingua Onde, so the app can still update itself in
; place. Progress lives in %APPDATA%\LinguaOnde (src/store.rs) and is kept
; unless the person asks the uninstaller to delete it.

#ifndef Version
  #define Version "0.0.0"
#endif

[Setup]
; Never change AppId: Windows finds the installed copy by it. src/update.rs
; uses it too, to show the new version in Settings, Apps after an update.
AppId={{4AC3DCD0-819C-4F78-A97A-0F9342519D3B}
AppName=Lingua Onde
AppVersion={#Version}
UninstallDisplayName=Lingua Onde
AppPublisherURL=https://github.com/Matteo404404/lingua-onde
AppSupportURL=https://github.com/Matteo404404/lingua-onde/issues
AppUpdatesURL=https://github.com/Matteo404404/lingua-onde/releases/latest
PrivilegesRequired=lowest
DefaultDirName={autopf}\Lingua Onde
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
WizardStyle=modern
SetupIconFile=lingua-onde.ico
UninstallDisplayIcon={app}\lingua-onde.exe
Compression=lzma2/max
SolidCompression=yes
OutputDir=..
OutputBaseFilename=lingua-onde-setup-windows-x86_64

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\lingua-onde-windows-x86_64.exe"; DestDir: "{app}"; DestName: "lingua-onde.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Lingua Onde"; Filename: "{app}\lingua-onde.exe"
Name: "{autodesktop}\Lingua Onde"; Filename: "{app}\lingua-onde.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\lingua-onde.exe"; Description: "{cm:LaunchProgram,Lingua Onde}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; what an update leaves beside the app (src/update.rs)
Type: files; Name: "{app}\lingua-onde.exe.old"
Type: files; Name: "{app}\.lingua-onde-windows-x86_64.exe.part"

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Data: String;
begin
  if CurUninstallStep = usPostUninstall then
  begin
    Data := ExpandConstant('{userappdata}\LinguaOnde');
    if DirExists(Data) and
       (SuppressibleMsgBox('Also delete your progress, courses and downloaded voices?' + #13#10#13#10 +
         'Choose No to keep them for a later install.', mbConfirmation, MB_YESNO or MB_DEFBUTTON2, IDNO) = IDYES) then
      DelTree(Data, True, True, True);
  end;
end;
