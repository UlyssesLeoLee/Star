@echo off
REM =====================================================================
REM start-frontend.bat — Star 一键启动前端 dev server
REM =====================================================================
REM 流程:
REM   1. 调 pwsh (PowerShell 7+) 跑 maintenance/start-frontend.ps1
REM   2. ps 脚本处理 node_modules 检测 + npm ci + npm run dev
REM   3. 同窗口前台执行, Ctrl+C 终止 dev, 关闭 .bat 窗口
REM
REM 设计原则 (per 守门 #6 PowerShell only + Windows 守门 11:35 JST):
REM   - 用 pwsh -NoProfile -NonInteractive: pwsh 避免父 powershell 把 $ 变量吃掉
REM   - ExecutionPolicy Bypass 允许从 cmd 跑 .ps1
REM   - exit /b %ERRORLEVEL% 把 pwsh 的 exit code 透传给 .bat 窗口
REM =====================================================================

setlocal

echo.
echo ==== Star start-frontend.bat: 转交到 pwsh start-frontend.ps1 ====
echo.

REM 优先 pwsh (PowerShell 7+), 退回 Windows PowerShell 5.1
where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-frontend.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-frontend.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== start-frontend.ps1 异常退出, exit code: %EXITCODE% ====
    echo 按任意键关闭窗口 ...
    pause >nul
) else (
    echo.
    echo ==== dev server 已干净退出 ====
)

endlocal & exit /b %EXITCODE%
