@echo off
REM =====================================================================
REM update-frontend.bat — Star 更新前端 (git pull + npm ci)
REM =====================================================================
REM 流程:
REM   1. 调 pwsh 跑 maintenance/update-frontend.ps1
REM   2. ps 脚本: 检查 git 状态 + git pull --ff-only + npm ci
REM   3. 失败立即退出, 透传 exit code
REM
REM 设计原则 (同 start-frontend.bat):
REM   - pwsh -NoProfile -NonInteractive
REM   - ExecutionPolicy Bypass
REM   - exit /b %ERRORLEVEL% 透传
REM =====================================================================

setlocal

echo.
echo ==== Star update-frontend.bat: 转交到 pwsh update-frontend.ps1 ====
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
    echo ==== update-frontend.ps1 失败, exit code: %EXITCODE% ====
    echo 按任意键关闭窗口 ...
    pause >nul
) else (
    echo.
    echo ==== 前端更新完成 ====
)

endlocal & exit /b %EXITCODE%
