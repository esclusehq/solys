; Escluse Agent Windows Installer
; NSIS Script for professional GUI installer with API key input

!include "MUI2.nsh"
!include "apikeypage.nsh"

;--------------------------------
; General
;--------------------------------
!define PRODUCT_NAME "Escluse Agent"
!define PRODUCT_VERSION "0.1.0"
!define PRODUCT_PUBLISHER "Escluse Team"
!define PRODUCT_WEB_SITE "https://escluse.com"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\escluse-gui.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"

Name "${PRODUCT_NAME}"
OutFile "escluse-agent-setup.exe"
InstallDir "$PROGRAMFILES64\Escluse"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
RequestExecutionLevel admin

;--------------------------------
; Interface Settings
;--------------------------------
!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"

;--------------------------------
; Pages
;--------------------------------
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "license.txt"
Page custom APIKeyPage APIKeyPageLeave
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
; Languages
;--------------------------------
!insertmacro MUI_LANGUAGE "English"

;--------------------------------
; Sections
;--------------------------------
Section "Escluse Agent" SEC_AGENT
  SectionIn RO
  
  SetOutPath "$INSTDIR"
  
  ; Main binaries - escluse-agent.exe (headless service) + escluse-gui.exe (GUI controller)
  File "..\..\release\package\escluse-agent.exe"
  File "..\..\release\package\escluse-service.exe"
  File "..\..\release\package\escluse-gui.exe"
  
  ; Config directory and file
  CreateDirectory "$APPDATA\escluse-agent"
  
  ; Generate config.toml with API key from custom page
  FileOpen $0 "$APPDATA\escluse-agent\config.toml" w
  FileWrite $0 "[server]$\r$\n"
  FileWrite $0 'api_key = "$API_KEY"$\r$\n'
  FileWrite $0 'backend_url = "wss://app.escluse.com/api/ws/node"$\r$\n'
  FileWrite $0 "$\r$\n"
  FileWrite $0 "[agent]$\r$\n"
  FileWrite $0 'agent_name = "windows-agent"$\r$\n'
  FileClose $0
  
  ; Install Windows Service
  ExecWait '"$INSTDIR\escluse-service.exe" --install'
  ExecWait '"$INSTDIR\escluse-service.exe" --start'
  
  ; Create Start Menu shortcuts - escluse-gui.exe is the GUI controller
  CreateDirectory "$SMPROGRAMS\Escluse"
  CreateShortcut "$SMPROGRAMS\Escluse\Escluse Agent Controller.lnk" "$INSTDIR\escluse-gui.exe"
  CreateShortcut "$SMPROGRAMS\Escluse\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  
  ; Create Desktop shortcut
  CreateShortcut "$DESKTOP\Escluse Agent.lnk" "$INSTDIR\escluse-gui.exe"
  
  ; Registry entries
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\escluse-gui.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\escluse-gui.exe"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  
  ; Uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"
  
  ; Launch GUI after installation
  Exec '"$INSTDIR\escluse-gui.exe"'
SectionEnd

;--------------------------------
; Uninstaller
;--------------------------------
Section "Uninstall"
  ; Stop and uninstall Windows Service
  ExecWait '"$INSTDIR\escluse-service.exe" --stop'
  ExecWait '"$INSTDIR\escluse-service.exe" --uninstall'
  
  ; Remove files
  Delete "$INSTDIR\escluse-agent.exe"
  Delete "$INSTDIR\escluse-service.exe"
  Delete "$INSTDIR\uninstall.exe"
  ; Note: Don't delete GUI to preserve user settings
  ; Delete "$INSTDIR\escluse-gui.exe"
  
  ; Remove shortcuts
  Delete "$SMPROGRAMS\Escluse\Escluse Agent Controller.lnk"
  Delete "$SMPROGRAMS\Escluse\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Escluse"
  Delete "$DESKTOP\Escluse Agent.lnk"
  
  ; Remove registry keys
  DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
  DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
  
  ; Remove install directory
  RMDir "$INSTDIR"
  
  ; Note: We don't remove config/data directory to preserve user settings
  ; Config remains at %APPDATA%\escluse-agent\ for reinstallation
SectionEnd
