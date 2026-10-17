; Windows installer for tubevibe (Inno Setup 6). Built by the release workflow:
;   iscc /DAppVersion=0.5.0 /DExe=path\to\tubevibe.exe /DOutDir=dist packaging\windows\tubevibe.iss
; Installs for the current user only (no administrator), adds tubevibe to the
; user's PATH and the Start menu, and can install mpv and yt-dlp with winget.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef Exe
  #define Exe "..\..\target\release\tubevibe.exe"
#endif
#ifndef OutDir
  #define OutDir "..\..\dist"
#endif

[Setup]
AppId={{B0AF9F4D-AF4F-4D9F-A3F8-3620150B8170}
AppName=tubevibe
AppVersion={#AppVersion}
AppPublisher=mortbrew
AppPublisherURL=https://github.com/mortbrew/tubevibe
AppSupportURL=https://github.com/mortbrew/tubevibe/issues
DefaultDirName={autopf}\tubevibe
DefaultGroupName=tubevibe
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ChangesEnvironment=yes
OutputDir={#OutDir}
OutputBaseFilename=tubevibe-x86_64-windows-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName=tubevibe

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
es.Deps=Instalar mpv y yt-dlp con winget (necesarios para ver videos)
en.Deps=Install mpv and yt-dlp with winget (needed to play videos)
es.Installing=Instalando %1 con winget…
en.Installing=Installing %1 with winget…

[Tasks]
Name: "deps"; Description: "{cm:Deps}"

[Files]
Source: "{#Exe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
Name: "{autoprograms}\tubevibe"; Filename: "{app}\tubevibe.exe"; WorkingDir: "{userdocs}"

[Run]
Filename: "winget"; Parameters: "install --id mpv-player.mpv-CI.MSVC -e --accept-source-agreements --accept-package-agreements"; StatusMsg: "{cm:Installing,mpv}"; Tasks: deps; Flags: waituntilterminated
Filename: "winget"; Parameters: "install --id yt-dlp.yt-dlp -e --accept-source-agreements --accept-package-agreements"; StatusMsg: "{cm:Installing,yt-dlp}"; Tasks: deps; Flags: waituntilterminated

[Registry]
; The install folder goes on the user's PATH (only once).
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}'))

[Code]
function NeedsAddPath(Dir: string): Boolean;
var
  Path: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Path) + ';') = 0;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Path, Dir: string;
  P: Integer;
begin
  if CurUninstallStep <> usPostUninstall then
    exit;
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path) then
    exit;
  Dir := ExpandConstant('{app}');
  P := Pos(';' + Uppercase(Dir), Uppercase(Path));
  if P > 0 then
  begin
    Delete(Path, P, Length(Dir) + 1);
    RegWriteExpandStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Path);
  end;
end;
