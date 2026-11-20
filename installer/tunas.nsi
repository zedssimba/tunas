; TUNAS installer (NSIS). Per-user install, no administrator rights needed.
; S.A.L.T CI · Owned by S.A.L.T. CI TECHNOLOGY · Distributed by Pemangkin Teknologi
; Build: makensis -DVERSION=0.1.0 -DSTAGE=<staged app folder> -DOUTFILE=<exe> tunas.nsi

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma
RequestExecutionLevel user

!ifndef VERSION
  !define VERSION "0.1.0"
!endif
!ifndef STAGE
  !define STAGE "stage"
!endif
!ifndef OUTFILE
  !define OUTFILE "TUNAS_Setup_${VERSION}.exe"
!endif

!define APPNAME "TUNAS"
!define PUBLISHER "S.A.L.T. CI TECHNOLOGY"
!define REGKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\TUNAS"
!define APPKEY "Software\SALT CI\TUNAS"
!define WEBVIEW2_GUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

Name "${APPNAME}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\TUNAS"
InstallDirRegKey HKCU "${APPKEY}" "InstallDir"
BrandingText "TUNAS ${VERSION} - Powered by S.A.L.T CI"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "TUNAS"
VIAddVersionKey "CompanyName" "${PUBLISHER}"
VIAddVersionKey "FileDescription" "TUNAS - personal AI for your machine"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "S.A.L.T CI and its components are proprietary technology of S.A.L.T. CI TECHNOLOGY, distributed and commercialised by Pemangkin Teknologi."

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

!define MUI_ICON "${STAGE}\TUNAS.ico"
!define MUI_UNICON "${STAGE}\TUNAS.ico"
!define MUI_ABORTWARNING
!define MUI_WELCOMEFINISHPAGE_BITMAP "${__FILEDIR__}\welcome.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP_STRETCH AspectFitHeight
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${__FILEDIR__}\welcome.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP_STRETCH AspectFitHeight
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_RIGHT
!define MUI_HEADERIMAGE_BITMAP "${__FILEDIR__}\header.bmp"
!define MUI_HEADERIMAGE_BITMAP_STRETCH AspectFitHeight
!define MUI_HEADERIMAGE_UNBITMAP "${__FILEDIR__}\header.bmp"
!define MUI_HEADERIMAGE_UNBITMAP_STRETCH AspectFitHeight
!define MUI_WELCOMEPAGE_TITLE "Welcome to TUNAS"
!define MUI_WELCOMEPAGE_TEXT "TUNAS is your own AI, running privately on this machine. It works offline and speaks English and Bahasa Melayu.$\r$\n$\r$\nAfter installing, TUNAS checks your machine and helps you get one or two AI models, from the internet or from a USB stick.$\r$\n$\r$\nNo administrator rights are needed.$\r$\n$\r$\nPowered by S.A.L.T CI"
!define MUI_FINISHPAGE_RUN "$INSTDIR\TUNAS.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Open TUNAS now"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
  ; Only one copy of the installer at a time.
  System::Call 'kernel32::CreateMutex(p 0, i 0, t "TUNAS_Setup_Running") p .r1 ?e'
  Pop $R0
  ${If} $R0 == 183
    Abort
  ${EndIf}
  ; A second start of this same installer right after it finished (some antivirus tools hold the first
  ; start back, then release it again): quit quietly instead of installing twice and closing TUNAS.
  ReadRegStr $2 HKCU "${APPKEY}" "SetupDoneTick"
  ReadRegStr $3 HKCU "${APPKEY}" "Version"
  ${If} $2 != ""
  ${AndIf} $3 == "${VERSION}"
    System::Call 'kernel32::GetTickCount() i .r4'
    IntOp $5 $4 - $2
    ${If} $5 >= 0
    ${AndIf} $5 < 180000
      Quit
    ${EndIf}
  ${EndIf}
  ; Close a running copy so files can be replaced.
  nsExec::Exec 'taskkill /IM TUNAS.exe /F'
  nsExec::Exec 'taskkill /IM llama-server.exe /F'
FunctionEnd

Function CheckWebView2
  ReadRegStr $0 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2_GUID}" "pv"
  ${If} $0 == ""
    ReadRegStr $0 HKCU "Software\Microsoft\EdgeUpdate\Clients\${WEBVIEW2_GUID}" "pv"
  ${EndIf}
  ${If} $0 == ""
  ${OrIf} $0 == "0.0.0.0"
    IfSilent +2
    MessageBox MB_ICONINFORMATION|MB_OK "TUNAS needs Microsoft Edge WebView2, which is part of current Windows 10 and 11.$\r$\n$\r$\nIt was not found on this machine. Please install the 'Evergreen WebView2 Runtime' from Microsoft (or ask your IT department), then open TUNAS."
  ${EndIf}
FunctionEnd

Section "TUNAS" SecMain
  ; Make sure no TUNAS or engine process holds files open, then give Windows a moment to release them.
  nsExec::Exec 'taskkill /IM TUNAS.exe /F'
  nsExec::Exec 'taskkill /IM llama-server.exe /F'
  Sleep 1500
  SetOutPath "$INSTDIR"
  ; Older engine folders are removed when possible; a folder still locked by a stuck process is left
  ; behind harmlessly, because each version uses its own engine folder.
  RMDir /r "$INSTDIR\engine"
  File /r "${STAGE}\*.*"

  WriteUninstaller "$INSTDIR\Uninstall TUNAS.exe"

  ; Shortcuts (TUNAS renames these to the brand name when branding is set).
  ReadRegStr $1 HKCU "${APPKEY}" "ShortcutName"
  ${If} $1 == ""
    StrCpy $1 "TUNAS"
  ${EndIf}
  CreateShortcut "$SMPROGRAMS\$1.lnk" "$INSTDIR\TUNAS.exe" "" "$INSTDIR\TUNAS.exe" 0
  ${IfNot} ${FileExists} "$DESKTOP\$1.lnk"
    CreateShortcut "$DESKTOP\$1.lnk" "$INSTDIR\TUNAS.exe" "" "$INSTDIR\TUNAS.exe" 0
  ${EndIf}

  ; Branding packs open with TUNAS (double-click a .tunasbrand file).
  WriteRegStr HKCU "Software\Classes\.tunasbrand" "" "TUNAS.BrandingPack"
  WriteRegStr HKCU "Software\Classes\TUNAS.BrandingPack" "" "TUNAS branding pack"
  WriteRegStr HKCU "Software\Classes\TUNAS.BrandingPack\DefaultIcon" "" "$INSTDIR\TUNAS.exe,0"
  WriteRegStr HKCU "Software\Classes\TUNAS.BrandingPack\shell\open\command" "" '"$INSTDIR\TUNAS.exe" "%1"'
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'

  ; Apps & features entry.
  WriteRegStr HKCU "${APPKEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${APPKEY}" "Version" "${VERSION}"
  System::Call 'kernel32::GetTickCount() i .r4'
  WriteRegStr HKCU "${APPKEY}" "SetupDoneTick" "$4"
  WriteRegStr HKCU "${REGKEY}" "DisplayName" "TUNAS"
  WriteRegStr HKCU "${REGKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${REGKEY}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKCU "${REGKEY}" "URLInfoAbout" "https://www.salt.ci"
  WriteRegStr HKCU "${REGKEY}" "DisplayIcon" "$INSTDIR\TUNAS.exe,0"
  WriteRegStr HKCU "${REGKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${REGKEY}" "UninstallString" '"$INSTDIR\Uninstall TUNAS.exe"'
  WriteRegStr HKCU "${REGKEY}" "QuietUninstallString" '"$INSTDIR\Uninstall TUNAS.exe" /S'
  WriteRegDWORD HKCU "${REGKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${REGKEY}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $2 $3
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD HKCU "${REGKEY}" "EstimatedSize" "$0"

  Call CheckWebView2
SectionEnd

Function un.onInit
  System::Call 'kernel32::CreateMutex(p 0, i 0, t "TUNAS_Uninstall_Running") p .r1 ?e'
  Pop $R0
  ${If} $R0 == 183
    Abort
  ${EndIf}
FunctionEnd

Section "Uninstall"
  nsExec::Exec 'taskkill /IM TUNAS.exe /F'
  nsExec::Exec 'taskkill /IM llama-server.exe /F'

  ReadRegStr $1 HKCU "${APPKEY}" "ShortcutName"
  ${If} $1 != ""
    Delete "$DESKTOP\$1.lnk"
    Delete "$SMPROGRAMS\$1.lnk"
  ${EndIf}
  Delete "$DESKTOP\TUNAS.lnk"
  Delete "$SMPROGRAMS\TUNAS.lnk"

  RMDir /r "$INSTDIR\engine"
  RMDir /r "$INSTDIR\resources"
  Delete "$INSTDIR\*.exe"
  Delete "$INSTDIR\*.dll"
  Delete "$INSTDIR\*.ico"
  Delete "$INSTDIR\*.txt"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "Software\Classes\.tunasbrand"
  DeleteRegKey HKCU "Software\Classes\TUNAS.BrandingPack"
  DeleteRegKey HKCU "${REGKEY}"

  IfSilent keep
  MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 "Also delete your chats, assistants, branding and downloaded models?$\r$\n$\r$\nChoose No to keep them for a later reinstall." IDNO keep
    RMDir /r "$LOCALAPPDATA\TUNAS"
    RMDir /r "$LOCALAPPDATA\ci.salt.tunas"
  keep:
  DeleteRegKey HKCU "${APPKEY}"
SectionEnd
