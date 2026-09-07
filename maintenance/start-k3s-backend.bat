@echo off
REM ==================================================
REM start-k3s-backend.bat - Star 1-key start WSL k3s server
REM ==================================================
REM Flow:
REM   1. Delegate to pwsh which runs start-k3s-backend.ps1
REM   2. The .ps1 script: probe WSL distro + k3s binary + process + port 6443
REM   3. If not running: nohup k3s server (needs sudo)
REM   4. Verify: kubectl get nodes + version + crash pods
REM
REM Pre-requisite:
REM   - WSL Ubuntu distro installed
REM   - k3s installed in WSL at /usr/local/bin/k3s
REM   - sudo configured in WSL (no-password recommended for one-key)
REM
REM Design (same as start-frontend.bat):
REM   - pwsh -NoProfile -NonInteractive + ExecutionPolicy Bypass
REM   - exit /b %ERRORLEVEL% propagates ps1 exit to .bat

setlocal

echo.
echo ==== Star start-k3s-backend.bat: delegate to pwsh start-k3s-backend.ps1 ====
echo.

where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-k3s-backend.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-k3s-backend.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== start-k3s-backend.ps1 failed, exit code: %EXITCODE% ====
    echo Press any key to close window ...
    pause >nul
) else (
    echo.
    echo ==== k3s backend ready ====
)

endlocal & exit /b %EXITCODE%