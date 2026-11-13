Unicode true
ManifestDPIAware true
ManifestDPIAwareness PerMonitorV2
RequestExecutionLevel admin
SetCompressor /SOLID lzma
!include MUI2.nsh
!include x64.nsh
!include LogicLib.nsh
!ifndef VERSION
!define VERSION "0.1.4"
!endif
!ifndef SOURCE_DIR
!error "SOURCE_DIR is required"
!endif
!ifndef OUTPUT_FILE
!error "OUTPUT_FILE is required"
!endif
!define PRODUCT "QuickPick"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\QuickPick"
!define APP_KEY "Software\udstd0038\QuickPick"
Name "${PRODUCT}"
OutFile "${OUTPUT_FILE}"
InstallDir "$PROGRAMFILES64\QuickPick"
InstallDirRegKey HKLM "${APP_KEY}" ""
BrandingText "QuickPick"
!define MUI_ICON "..\assets\icon.ico"
!define MUI_UNICON "..\assets\icon.ico"
!define MUI_LANGDLL_REGISTRY_ROOT "HKCU"
!define MUI_LANGDLL_REGISTRY_KEY "${APP_KEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "Installer Language"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "SimpChinese"
!insertmacro MUI_LANGUAGE "TradChinese"
!insertmacro MUI_LANGUAGE "Korean"
!insertmacro MUI_LANGUAGE "Japanese"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "Spanish"
LangString CloseRequired ${LANG_ENGLISH} "Please exit QuickPick normally before installing or uninstalling."
LangString CloseRequired ${LANG_SIMPCHINESE} "请先正常退出 QuickPick，再安装或卸载。"
LangString CloseRequired ${LANG_TRADCHINESE} "請先正常結束 QuickPick，再安裝或解除安裝。"
LangString CloseRequired ${LANG_KOREAN} "QuickPick을 정상 종료한 후 설치 또는 제거하세요."
LangString CloseRequired ${LANG_JAPANESE} "QuickPickを終了してからインストールまたは削除してください。"
LangString CloseRequired ${LANG_FRENCH} "Quittez QuickPick avant l'installation ou la désinstallation."
LangString CloseRequired ${LANG_GERMAN} "Beenden Sie QuickPick vor der Installation oder Deinstallation."
LangString CloseRequired ${LANG_SPANISH} "Cierra QuickPick antes de instalar o desinstalar."
LangString DesktopShortcut ${LANG_ENGLISH} "Create a desktop shortcut?"
LangString DesktopShortcut ${LANG_SIMPCHINESE} "创建桌面快捷方式？"
LangString DesktopShortcut ${LANG_TRADCHINESE} "建立桌面捷徑？"
LangString DesktopShortcut ${LANG_KOREAN} "바탕 화면 바로 가기를 만드시겠습니까?"
LangString DesktopShortcut ${LANG_JAPANESE} "デスクトップにショートカットを作成しますか？"
LangString DesktopShortcut ${LANG_FRENCH} "Créer un raccourci sur le bureau ?"
LangString DesktopShortcut ${LANG_GERMAN} "Desktop-Verknüpfung erstellen?"
LangString DesktopShortcut ${LANG_SPANISH} "¿Crear un acceso directo en el escritorio?"
!macro RequireStopped
  nsExec::ExecToStack '"$SYSDIR\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -NonInteractive -Command "if (Get-Process -Name quickpick -ErrorAction SilentlyContinue) { exit 1 } else { exit 0 }"'
  Pop $0
  Pop $1
  ${If} $0 != 0
    MessageBox MB_ICONEXCLAMATION "$(CloseRequired)"
    Abort
  ${EndIf}
  ; 新旧程序共享互斥量，不终止用户进程。
  System::Call 'kernel32::OpenMutexW(i 0x00100000, i 0, w "Local\QuickPick.Instance.com.quickpick.desktop") p.r0'
  ${If} $0 != 0
    System::Call 'kernel32::CloseHandle(p r0)'
    MessageBox MB_ICONEXCLAMATION "$(CloseRequired)"
    Abort
  ${EndIf}
  ${If} ${FileExists} "$INSTDIR\quickpick.exe"
    ClearErrors
    FileOpen $0 "$INSTDIR\quickpick.exe" a
    ${If} ${Errors}
      MessageBox MB_ICONEXCLAMATION "$(CloseRequired)"
      Abort
    ${EndIf}
    FileClose $0
  ${EndIf}
!macroend
Function .onInit
  SetRegView 64
  !insertmacro MUI_LANGDLL_DISPLAY
  ${IfNot} ${RunningX64}
    Abort "Windows x64 is required."
  ${EndIf}
FunctionEnd
Function un.onInit
  SetRegView 64
  !insertmacro MUI_UNGETLANGUAGE
FunctionEnd
Section "QuickPick" Main
  !insertmacro RequireStopped
  SetShellVarContext all
  SetOutPath "$INSTDIR"
  File /r "${SOURCE_DIR}\*"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${APP_KEY}" "" "$INSTDIR"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayName" "QuickPick"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "Publisher" "udstd0038"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "MainBinaryName" "quickpick.exe"
  WriteRegStr HKLM "${UNINSTALL_KEY}" "DisplayIcon" '$\"$INSTDIR\quickpick.exe$\"'
  WriteRegStr HKLM "${UNINSTALL_KEY}" "InstallLocation" '$\"$INSTDIR$\"'
  WriteRegStr HKLM "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\uninstall.exe$\"'
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${UNINSTALL_KEY}" "NoRepair" 1
  CreateDirectory "$SMPROGRAMS\QuickPick"
  CreateShortcut "$SMPROGRAMS\QuickPick\QuickPick.lnk" "$INSTDIR\quickpick.exe"
  MessageBox MB_YESNO "$(DesktopShortcut)" IDNO done
    CreateShortcut "$DESKTOP\QuickPick.lnk" "$INSTDIR\quickpick.exe"
  done:
SectionEnd
Section "Uninstall"
  !insertmacro RequireStopped
  SetShellVarContext all
  ; 只删除此安装包拥有的程序文件，保留配置、密钥和用户自行放入的其他文件。
  Delete "$INSTDIR\quickpick.exe"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\THIRD-PARTY-NOTICES.json"
  Delete "$INSTDIR\THIRD-PARTY-SOURCES.json"
  RMDir /r "$INSTDIR\third-party-licenses"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$DESKTOP\QuickPick.lnk"
  Delete "$SMPROGRAMS\QuickPick\QuickPick.lnk"
  RMDir "$SMPROGRAMS\QuickPick"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "${UNINSTALL_KEY}"
  DeleteRegKey HKLM "${APP_KEY}"
  ; 仅移除指向此安装目录的当前用户启动项，不影响便携版或其他安装。
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "QuickPick"
  StrCmp $0 '$\"$INSTDIR\quickpick.exe$\"' 0 keep_autostart
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "QuickPick"
  keep_autostart:
SectionEnd
