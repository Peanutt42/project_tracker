@echo off

:: Check if the script is running with Administrator privileges
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo This script requires Administrator privileges.
    echo Attempting to relaunch as Administrator...
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

set "scriptDir=%~dp0"
set "projectRoot=%~dp0..\"

echo %projectRoot%
cd %projectRoot%
cargo b --release

set "installationFolder=%LocalAppData%\Programs\Project Tracker"
mkdir "%installationFolder%"
set "programFilepath=%installationFolder%\project_tracker.exe"
echo F|xcopy /y "%projectRoot%\target\release\project_tracker.exe" "%programFilepath%"
set "iconFilepath=%installationFolder%\ProjectTracker.ico"
echo F|xcopy /y "%projectRoot%\assets\icon.ico" "%iconFilepath%"

set "startMenuPath=%ProgramData%\Microsoft\Windows\Start Menu\Programs"
set "shorcutFilepath=%startMenuPath%\Project Tracker.lnk"

:: Create the shortcut using PowerShell commands
powershell.exe -Command ^
    "$shell = New-Object -ComObject WScript.Shell; " ^
    "$shortcut = $shell.CreateShortcut('%shorcutFilepath%'); " ^
    "$shortcut.TargetPath = '%programFilepath%'; " ^
    "$shortcut.IconLocation = '%iconFilepath%'; " ^
    "$shortcut.Save();"