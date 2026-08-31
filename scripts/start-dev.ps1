# =====================================================================
# start-dev.ps1 — Star 一键启动 dev server (per 2026-08-31 11:44 JST 拍板)
# =====================================================================
# 职责:
#   1. 切到仓库根 (脚本可从任意 cwd 调用)
#   2. 检测 node_modules: 缺则 npm ci (锁文件安装), 有则跳过
#   3. 调 npm run dev (next dev -p 3000, hot reload) — 前台跑
#   4. Ctrl+C 时 npm 进程被 SIGINT 终止, 整个 .bat 窗口干净退出
#
# 不做 (per 缺标比错标):
#   - 不拉 git (origin 不在本地, per R-05 守门 + R-05 反转拍板)
#   - 不 build (dev 模式不需要)
#   - 不开新窗口 (per 11:44 JST 拍板: 同窗口体验)
#
# 设计原则 (per 环境变量安全 2026-08-27 11:06 JST hard ban):
#   - 脚本不打印任何 env var 内容, 不读 .env, 只 invoke
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 0. 切到仓库根 ----
# $PSScriptRoot 是 scripts/ 目录, 上一级是仓库根
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host ""
Write-Host "==== Star 一键启动 dev server ====" -ForegroundColor Cyan
Write-Host "Repo : $RepoRoot"
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

# ---- 1. 探测 node_modules 状态 ----
$NodeModules = Join-Path $RepoRoot 'frontend/node_modules'
$LockFile    = Join-Path $RepoRoot 'frontend/package-lock.json'
$PackageLockInstalled = Join-Path $NodeModules '.package-lock.json'

$NeedInstall = $false
if (-not (Test-Path $NodeModules)) {
    $NeedInstall = $true
    Write-Host "[1/3] frontend/node_modules 缺失, 需要安装" -ForegroundColor Yellow
} elseif (-not (Test-Path $PackageLockInstalled)) {
    $NeedInstall = $true
    Write-Host "[1/3] .package-lock.json 缺失 (node_modules 可能不完整), 需要安装" -ForegroundColor Yellow
} else {
    Write-Host "[1/3] node_modules 已存在, 跳过安装 (若需重装请删 frontend/node_modules)" -ForegroundColor Green
}

# ---- 2. 装依赖 ----
if ($NeedInstall) {
    if (-not (Test-Path $LockFile)) {
        Write-Host "ERROR: $LockFile 不存在, 无法 npm ci" -ForegroundColor Red
        Write-Host "提示: 在 frontend/ 跑 'npm install' 生成 lock 后再试" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    Write-Host "[2/3] 运行 npm ci (锁文件安装) ..." -ForegroundColor Cyan
    Push-Location (Join-Path $RepoRoot 'frontend')
    try {
        npm ci
        if ($LASTEXITCODE -ne 0) {
            throw "npm ci 失败, exit code: $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
    Write-Host "[2/3] npm ci 完成" -ForegroundColor Green
} else {
    Write-Host "[2/3] 跳过 npm ci" -ForegroundColor Green
}

# ---- 3. 启 dev server (前台, Ctrl+C 终止) ----
Write-Host ""
Write-Host "[3/3] 启动 next dev (port 3000) ..." -ForegroundColor Cyan
Write-Host "提示: 浏览器打开 http://localhost:3000" -ForegroundColor Yellow
Write-Host "      Ctrl+C 停止 dev server" -ForegroundColor Yellow
Write-Host ""

Push-Location (Join-Path $RepoRoot 'frontend')
try {
    # 不传 --prefix, 因为已经 Push-Location 到 frontend/
    # npm 收到 SIGINT 会优雅终止 next dev
    npm run dev
    if ($LASTEXITCODE -ne 0) {
        throw "npm run dev 失败, exit code: $LASTEXITCODE"
    }
} finally {
    Pop-Location
    Write-Host ""
    Write-Host "==== dev server 已停止 ====" -ForegroundColor Cyan
}
