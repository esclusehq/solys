; Custom API Key Input Page for Escluse Agent Installer
; Include this header in main NSIS script

!ifndef APIKEYPAGE_NSH_INCLUDED
!define APIKEYPAGE_NSH_INCLUDED

!include "nsDialogs.nsh"
!include "LogicLib.nsh"

; Variables for API key input
Var API_KEY_INPUT
Var API_KEY

;--------------------------------
; Custom Page: API Key Input
;--------------------------------
Function APIKeyPage
  !insertmacro MUI_HEADER_TEXT "API Key Configuration" "Enter your Escluse API Key to connect to the backend"
  
  nsDialogs::Create 1018
  Pop $0
  
  ${If} $0 == error
    Abort
  ${EndIf}
  
  ; Instructions label
  ${NSD_CreateLabel} 0 0 100% 24u "Please enter your Escluse API Key. This key is required to connect your agent to the Escluse backend.$\r$\n$\r$\nYou can find your API Key in your Escluse dashboard under Settings > API Keys."
  
  ; API Key label
  ${NSD_CreateLabel} 0 40u 100% 12u "API Key:"
  
  ; API Key input field (password-style with reveal option)
  ${NSD_CreateText} 0 52u 100% 12u ""
  Pop $API_KEY_INPUT
  
  ; Set focus to input field
  ${NSD_SetFocus} $API_KEY_INPUT
  
  nsDialogs::Show
FunctionEnd

;--------------------------------
; Page Leave: Validate API Key
;--------------------------------
Function APIKeyPageLeave
  ; Get the API key from input field
  ${NSD_GetText} $API_KEY_INPUT $API_KEY
  
  ; Debug: show what was captured (remove in production)
  ; MessageBox MB_OK "Debug: API_KEY=[$API_KEY]"
  
  ; Validate: cannot be empty (check length first)
  StrLen $0 $API_KEY
  ${If} $0 == 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "API Key cannot be empty. Please enter a valid API Key."
    Abort
  ${EndIf}
  
  ; Validate: minimum length check (at least 8 chars)
  ${If} $0 < 8
    MessageBox MB_OK|MB_ICONEXCLAMATION "API Key appears to be too short (minimum 8 characters). Please enter a valid API Key."
    Abort
  ${EndIf}
FunctionEnd

!endif ; APIKEYPAGE_NSH_INCLUDED
