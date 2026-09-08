@echo off
REM ==================================================
REM update-frontend.bat - Star update frontend (git pull + npm ci)
REM ==================================================
REM Flow:
REM   1. Delegate to pwsh which runs update-frontend.ps1
REM   2. The .ps1 script: check working dir clean + smart git pull + npm ci
REM   3. npm ci fails -> npm install fallback to fix lock + retry

setlocal

echo.
echo ==== Star update-frontend.bat: delegate to pwsh update-frontend.ps1 ====
echo.

where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0update-frontend.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0update-frontend.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== update-frontend.ps1 failed, exit code: %EXITCODE% ====
    echo Press any key to close window ...
    pause >nul
) else (
    echo.
    echo ==== frontend update done ====
)

endlocal & exit /b %EXITCODE%