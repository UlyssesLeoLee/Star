@echo off
REM =====================================================================
REM verify-k3s-uat-3000.bat — 3000 端口 UAT 验证 pwsh 包装
REM =====================================================================
REM 流程:
REM   1. pwsh (PowerShell 7+) 转 scripts/verify-k3s-uat-3000.ps1
REM   2. 5 步走 守门 v27/v28/v29: 验证 + rollout + enable port-forward + curl
REM   3. 同步前台, exit code 透传 .bat
REM
REM 前置 (Ulysses 必先做):
REM   wsl -d Ubuntu -- bash -lc "sudo systemctl restart k3s; sleep 60"
REM
REM 跟 start-k3s-backend.bat 关系:
REM   - start-k3s-backend.bat: 启 k3s server
REM   - verify-k3s-uat-3000.bat: 验 3000 (本脚本, k3s server 已起)
REM =====================================================================

setlocal

echo.
echo ==== Star verify-k3s-uat-3000.bat: 转 pwsh verify-k3s-uat-3000.ps1 ====
echo.

where pwsh >nul 2>&1
if %ERRORLEVEL% equ 0 (
    pwsh -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0verify-k3s-uat-3000.ps1"
) else (
    powershell -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "%~dp0verify-k3s-uat-3000.ps1"
)

set EXITCODE=%ERRORLEVEL%

if %EXITCODE% neq 0 (
    echo.
    echo ==== verify-k3s-uat-3000.ps1 失败, exit code: %EXITCODE% ====
    echo 等待手动排查 ...
    pause >nul
) else (
    echo.
    echo ==== 3000 端口 UAT 验证 PASSED ====
)

endlocal & exit /b %EXITCODE%
