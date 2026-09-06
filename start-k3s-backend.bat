@echo off
REM =====================================================================
REM start-k3s-backend.bat — 画布游戏 k3s 后端 一键部署 (per 2026-09-07 07:06 JST 用户拍板)
REM =====================================================================
REM 流程:
REM   1. 调 pwsh (PowerShell 7+) 跑 scripts/start-k3s-backend.ps1
REM   2. ps 脚本: 探测 kubectl + k3s cluster + secret + apply yaml + rollout status + port-forward
REM   3. 同窗口前台执行, Ctrl+C 终止监控循环, 关闭 .bat 窗口
REM
REM 设计原则 (跟 start.bat 模式一致):
REM   - 用 pwsh -NoProfile -NonInteractive (per Windows 守门 11:35 JST 拍板)
REM   - ExecutionPolicy Bypass 允许从 cmd 跑 .ps1
REM   - exit /b %ERRORLEVEL% 把 pwsh 的 exit code 透传给 .bat 窗口
REM
REM 不做 (per 缺标比错标):
REM   - 不拉 git (per 11:44 JST 拍板: 只装 + 启)
REM   - 不 build 镜像 (per 11:44 JST 拍板: 一键脚本只部署, 不构建)
REM   - 不另开窗口 (同窗口, Ctrl+C 终止 + 关闭 .bat)
REM =====================================================================

setlocal

echo.
echo ==== Star start-k3s-backend.bat: 转交到 pwsh start-k3s-backend.ps1 ====
echo.

REM 优先 pwsh (PowerShell 7+), 退回 Windows PowerShell 5.1
where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0scripts\start-k3s-backend.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0scripts\start-k3s-backend.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== start-k3s-backend.ps1 异常退出, exit code: %EXITCODE% ====
    echo 按任意键关闭窗口 ...
    pause >nul
) else (
    echo.
    echo ==== k3s 后端已干净退出 ====
)

endlocal & exit /b %EXITCODE%
