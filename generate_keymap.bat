@echo off
echo Generating keymap visualization locally...
echo.

REM Parse the ZMK keymap to YAML
echo [1/2] Parsing ZMK keymap...
python -m keymap_drawer parse -z config/eyelash_corne.keymap -o config/eyelash_corne.yaml
if %ERRORLEVEL% neq 0 (
    echo Error: Failed to parse keymap
    exit /b 1
)

REM Generate the SVG with enhanced styling
echo [2/2] Generating SVG with enhanced styling...
python -m keymap_drawer -c keymap_drawer.config.yaml draw config/eyelash_corne.yaml -o keymap-drawer/eyelash_corne.svg
if %ERRORLEVEL% neq 0 (
    echo Error: Failed to generate SVG
    exit /b 1
)

echo.
echo ✅ Keymap generated successfully!
echo    📁 Location: keymap-drawer/eyelash_corne.svg
echo    🎨 Features: Enhanced styling, layer tinting, semantic combo coloring
echo.
echo Opening SVG in browser...
start "" "keymap-drawer/eyelash_corne.svg"
