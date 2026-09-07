@echo off
REM ==================================================
REM start-frontend.bat - Star 1-key start frontend dev
REM ==================================================
REM Flow:
REM   1. Delegate to pwsh (PowerShell 7+) which runs start-frontend.ps1
REM   2. The .ps1 script: detect node_modules + npm ci + npm run dev
REM   3. Same-window foreground, Ctrl+C stops dev, close .bat
REM
REM Design (same as start-k3s-backend.bat):
REM   - pwsh -NoProfile -NonInteractive: prevents parent powershell eating $vars
REM   - ExecutionPolicy Bypass: allows .ps1 from cmd
REM   - exit /b %ERRORLEVEL% propagates ps1 exit to .bat window

setlocal

echo.
echo ==== Star start-frontend.bat: delegate to pwsh start-frontend.ps1 ====
echo.

REM Prefer pwsh (PowerShell 7+), fallback to Windows PowerShell 5.1
where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-frontend.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-frontend.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== start-frontend.ps1 failed, exit code: %EXITCODE% ====
    echo Press any key to close window ...
    pause >nul
) else (
    echo.
    echo ==== dev server clean exit ====
)

endlocal & exit /b %EXITCODE%