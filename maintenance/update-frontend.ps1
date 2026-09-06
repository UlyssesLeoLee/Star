# =====================================================================
# update-frontend.ps1 — Star 更新前端 (per 2026-09-06 13:21 JST 拍板)
# =====================================================================
# 职责: 只拉前端依赖
#   1. 切到仓库根
#   2. git pull --ff-only (拉代码, 失败不回滚本地改动)
#   3. frontend/ 跑 npm ci (锁文件安装, 同步 node_modules)
#   4. 可选: npm run build (默认跳过, dev 模式不需要)
#
# 不做 (per 缺标比错标):
#   - 不 rm -rf node_modules (npm ci 本身就是从锁文件重装, 删了反而会丢掉 npm 缓存)
#   - 不动后端 (后端更新走 update-backend-k3s.ps1)
#   - 不动 helm / k3s
#   - 不动 env var 内容 (per 8/27 11:06 JST hard ban)
#
# 设计原则:
#   - git pull --ff-only: 本地有未推送 commit / 分支会失败, 不偷偷 merge
#   - 失败立即退出 (ErrorActionPreference Stop), 留给调用方决定是否回滚
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 0. 切到仓库根 ----
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host ""
Write-Host "==== Star 更新前端 ====" -ForegroundColor Cyan
Write-Host "Repo : $RepoRoot"
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

# ---- 1. 检查 git 状态 ----
if (-not (Test-Path (Join-Path $RepoRoot '.git'))) {
    Write-Host "ERROR: $RepoRoot 不是 git 仓库, 退出" -ForegroundColor Red
    exit 1
}

$GitStatus = git status --porcelain
if ($GitStatus) {
    Write-Host "ERROR: 工作区有未提交改动, 请先 commit / stash 再 update" -ForegroundColor Red
    Write-Host $GitStatus
    exit 1
}

# ---- 2. git pull --ff-only ----
Write-Host "[1/3] git pull --ff-only ..." -ForegroundColor Cyan
git pull --ff-only
if ($LASTEXITCODE -ne 0) {
    throw "git pull 失败, exit code: $LASTEXITCODE (本地有未推送 commit / remote 不可达都会触发)"
}
Write-Host "[1/3] git pull 完成" -ForegroundColor Green

# ---- 3. npm ci (with fallback) ----
$FrontendDir = Join-Path $RepoRoot 'frontend'
$LockFile    = Join-Path $FrontendDir 'package-lock.json'
if (-not (Test-Path $LockFile)) {
    Write-Host "ERROR: $LockFile 不存在, 无法 npm ci" -ForegroundColor Red
    Write-Host "提示: 在 frontend/ 跑 'npm install' 生成 lock 后再试" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "[2/3] frontend/ npm ci (锁文件同步) ..." -ForegroundColor Cyan
Push-Location $FrontendDir
try {
    $CiOk = $false
    npm ci 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        $CiOk = $true
    } else {
        # Fallback: npm ci 失败 (典型: package.json 比 lock 新, lock 不同步)
        # 触发自动 npm install 修 lock, 再重试一次
        Write-Host "  npm ci 失败 (exit $LASTEXITCODE), 触发 fallback: npm install 修 lock + retry" -ForegroundColor Yellow
        Write-Host "  (per 9/6 17:07 JST 实际验证: Sprint commit ef2bc80 加依赖没重生 lock)" -ForegroundColor Yellow
        npm install --no-audit --no-fund
        if ($LASTEXITCODE -ne 0) {
            throw "npm install 失败, exit code: $LASTEXITCODE (修 lock 重试都失败, 需要手动介入)"
        }
        Write-Host "  npm install 完成, 锁已修复, 重试 npm ci ..." -ForegroundColor Cyan
        npm ci
        if ($LASTEXITCODE -ne 0) {
            throw "npm ci 重试仍失败, exit code: $LASTEXITCODE"
        }
        $CiOk = $true
    }
} finally {
    Pop-Location
}
if ($CiOk) {
    Write-Host "[2/3] npm ci 完成" -ForegroundColor Green
}

# ---- 4. 提示下一步 ----
Write-Host ""
Write-Host "[3/3] 前端已更新, 下一步可:" -ForegroundColor Cyan
Write-Host "      - 调 .\maintenance\start-frontend.bat 启 dev server" -ForegroundColor Yellow
Write-Host "      - 调 .\maintenance\update-backend-k3s.bat 更新后端" -ForegroundColor Yellow
Write-Host ""
Write-Host "==== 前端更新完成 ====" -ForegroundColor Green
