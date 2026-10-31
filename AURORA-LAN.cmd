@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -STA -File "%~dp0scripts\lan_assistant.ps1"
if errorlevel 1 (
  echo Impossible de lancer l'assistant AURORA. Verifier le reseau prive et le paquet.
  pause
)
