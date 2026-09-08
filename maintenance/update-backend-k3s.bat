@echo off
REM ==================================================
REM update-backend-k3s.bat - Star update k3s backend (helm upgrade)
REM ==================================================
REM Flow:
REM   1. Delegate to pwsh which runs update-backend-k3s.ps1
REM   2. The .ps1 script: smart git pull + ensure helm + kubectl cluster-info
REM   3. If cluster OK: helm upgrade --install (8 services in star-system ns)
REM
REM Pre-requisite:
REM   - WSL k3s running (use start-k3s-backend.bat first)
REM   - helm installed (winget install Helm.Helm or scoop install helm)
REM   - Images in ghcr.io/ulyssesleolee/rustgameserver (see G1 known gap)

setlocal

echo.
echo ==== Star update-backend-k3s.bat: delegate to pwsh update-backend-k3s.ps1 ====
echo.

where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0update-backend-k3s.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0update-backend-k3s.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== update-backend-k3s.ps1 failed, exit code: %EXITCODE% ====
    echo Press any key to close window ...
    pause >nul
) else (
    echo.
    echo ==== backend update done ====
)

endlocal & exit /b %EXITCODE%