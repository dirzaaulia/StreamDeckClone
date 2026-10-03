@echo off
setlocal
cd /d "%~dp0"
echo ===================================================
echo     Starting StreamDeck Configurator Desktop GUI
echo ===================================================
if exist "desktop-gui\src-tauri\target\release\streamdeck-gui.exe" (
    echo Launching native desktop application...
    start "" "desktop-gui\src-tauri\target\release\streamdeck-gui.exe"
) else (
    echo Launching development server...
    cd desktop-gui
    npm run tauri dev
)

