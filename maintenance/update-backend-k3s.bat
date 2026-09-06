@echo off
REM =====================================================================
REM update-backend-k3s.bat — Star 更新后端 (k3s 架构)
REM =====================================================================
REM 流程:
REM   1. 调 pwsh 跑 maintenance/update-backend-k3s.ps1
REM   2. ps 脚本: git pull + 探测工具链 + 改 imageTag + helm upgrade + 验证 rollout
REM   3. 失败立即退出, 透传 exit code
REM
REM 前置条件 (per update-backend-k3s.ps1 已知缺口):
REM   - 本机已装 kubectl / helm
REM   - KUBECONFIG 已 export 或 ~/.kube/config 存在
REM   - 镜像已推送到 ghcr.io/ulysses-lee-lee/<service>:<tag> (本脚本不负责 build)
REM
REM 设计原则 (同 start-frontend.bat):
REM   - pwsh -NoProfile -NonInteractive
REM   - ExecutionPolicy Bypass
REM   - exit /b %ERRORLEVEL% 透传
REM =====================================================================

setlocal

echo.
echo ==== Star update-backend-k3s.bat: 转交到 pwsh update-backend-k3s.ps1 ====
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
    echo ==== update-backend-k3s.ps1 失败, exit code: %EXITCODE% ====
    echo 按任意键关闭窗口 ...
    pause >nul
) else (
    echo.
    echo ==== 后端更新完成 ====
)

endlocal & exit /b %EXITCODE%
