@echo off
REM =====================================================================
REM start.bat — Star 根目录薄包装 (per 2026-09-06 13:21 JST 拍板)
REM =====================================================================
REM 用途: 保留根目录 start.bat 作为快捷入口, 转发到 maintenance/start-frontend.bat
REM
REM 完整 3 套启动 / 更新脚本都集中在 maintenance/:
REM   - maintenance\start-frontend.bat       一键启动前端 dev
REM   - maintenance\update-frontend.bat      更新前端 (git pull + npm ci)
REM   - maintenance\update-backend-k3s.bat   更新后端 (k3s 架构, helm upgrade)
REM
REM 设计: 薄包装只调一次 pwsh, 不做任何业务逻辑, 避免双份维护
REM =====================================================================

"%~dp0maintenance\start-frontend.bat"
exit /b %ERRORLEVEL%
