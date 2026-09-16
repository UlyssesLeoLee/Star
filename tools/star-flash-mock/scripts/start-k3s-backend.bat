@echo off
REM =====================================================================
REM start-k3s-backend.bat — Star 后端 k3s 重启后拉起 (per 2026-09-08 05:32 JST 用户拍板)
REM =====================================================================
REM 流程:
REM   1. 调 pwsh (PowerShell 7+) 跑 maintenance/start-k3s-backend.ps1
REM   2. ps 脚本: 探测 WSL + k3s 二进制 + 进程 + 6443 端口 + 启 server + 验证
REM   3. 同窗口前台执行, exit code 透传给 .bat
REM
REM 设计原则 (同 start-frontend.bat):
REM   - 用 pwsh -NoProfile -NonInteractive: pwsh 避免父 powershell 把 $ 变量吃掉
REM   - ExecutionPolicy Bypass 允许从 cmd 跑 .ps1
REM   - exit /b %ERRORLEVEL% 把 pwsh 的 exit code 透传给 .bat 窗口
REM
REM 跟 update-backend-k3s.bat 关系:
REM   - start-k3s-backend.bat: 启 k3s server 进程
REM   - update-backend-k3s.bat: 装 chart (依赖集群已就绪)
REM =====================================================================

setlocal

echo.
echo ==== Star start-k3s-backend.bat: 转交到 pwsh start-k3s-backend.ps1 ====
echo.

REM 优先 pwsh (PowerShell 7+), 退回 Windows PowerShell 5.1
where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-k3s-backend.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0start-k3s-backend.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== start-k3s-backend.ps1 失败, exit code: %EXITCODE% ====
    echo 按任意键关闭窗口 ...
    pause >nul
) else (
    echo.
    echo ==== k3s 后端就绪 ====
)

endlocal & exit /b %EXITCODE%
