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
#   - npm ci / npm install 进程跟踪 + 退出清理 (Ctrl+C / 异常 / 正常结束 三条路径都收)
#     (避免 husky / postinstall / prepare 钩子残留孤儿进程)
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 子进程跟踪: 记录 npm + 它派生的整棵树 PID, 退出时收 ----
$Script:TrackedPids = [System.Collections.Generic.List[int]]::new()
$Script:CleanupDone = $false

function Stop-TrackedTree {
    # 幂等: 只跑一次, 避免 Ctrl+C 双触发
    if ($Script:CleanupDone) { return }
    $Script:CleanupDone = $true

    if ($Script:TrackedPids.Count -eq 0) { return }

    Write-Host ""
    Write-Host "==== 清理子进程树 ====" -ForegroundColor Cyan
    foreach ($procId in $Script:TrackedPids) {
        try {
            $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$procId" -EA SilentlyContinue
            if ($proc -and $proc.Name -in @('node.exe','npm.cmd','cmd.exe')) {
                Write-Host ("  -> taskkill /F /T /PID {0} ({1})" -f $procId, $proc.Name) -ForegroundColor DarkGray
                # /F 不给机会, 但 Ctrl+C / 异常退出时必须强杀,
                # 否则 husky / postinstall 钩子派生的 node 子进程会残留
                Start-Process -FilePath 'taskkill.exe' -ArgumentList @('/F','/T','/PID',"$procId") -WindowStyle Hidden -Wait -EA SilentlyContinue | Out-Null
            }
        } catch { }
    }
    Write-Host "==== 清理完成 ====" -ForegroundColor Cyan
}

# Ctrl+C / 关闭窗口 -> 走 cleanup 再 exit
# Windows PowerShell 5.1: TreatControlCAsInput 默认 false, 不需要显式设置
try {
    if ([Console]::TreatControlCAsInput -is [bool]) {
        [Console]::TreatControlCAsInput = $false
    }
} catch { }

$Handler = {
    Write-Host ""
    Write-Host "[Ctrl+C] 收到终止信号" -ForegroundColor Yellow
    Stop-TrackedTree
    [System.Environment]::Exit(130)
}

try {
    if ($null -ne [Console]::CancelKeyPress) {
        [Console]::CancelKeyPress.add($Handler)
    }
} catch {
    Write-Host "  (CancelKeyPress handler 注册失败, 仅依赖 trap + finally 清理)" -ForegroundColor DarkGray
}

# 兜底: 任何未捕获异常都触发
trap {
    Write-Host ""
    Write-Host "[trap] 异常: $_" -ForegroundColor Red
    Stop-TrackedTree
    break
}

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

# ---- 2. git 同步 (smart pull) ----
# 替代原 git pull --ff-only, 处理 3 种常见场景:
#   (a) 有 upstream, 落后 origin       -> git pull --ff-only
#   (b) 有 upstream, 领先 origin       -> 报 ahead N, 提示先 push (per 守门 #1 R-05)
#   (c) 无 upstream (worktree 新建)     -> git fetch origin, 比对 main, 提示用户决策
Write-Host "[1/3] git 同步 (smart pull) ..." -ForegroundColor Cyan

$Upstream = git rev-parse --abbrev-ref --symbolic-full-name '@{u}' 2>$null
$HasUpstream = ($LASTEXITCODE -eq 0)

if ($HasUpstream) {
    # (a) 或 (b): 有 upstream
    $LeftRight = (git rev-list --left-right --count "$Upstream...HEAD" 2>$null) -split '\s+'
    $Behind    = [int]$LeftRight[0]
    $Ahead     = [int]$LeftRight[1]
    Write-Host "  upstream : $Upstream"
    Write-Host "  ahead    : $Ahead, behind : $Behind"

    if ($Behind -gt 0 -and $Ahead -eq 0) {
        # (a) 落后, 安全 fast-forward
        Write-Host "  落后 origin $Behind commit, 跑 git pull --ff-only ..." -ForegroundColor Cyan
        git pull --ff-only
        if ($LASTEXITCODE -ne 0) {
            throw "git pull --ff-only 失败, exit code: $LASTEXITCODE"
        }
        Write-Host "  pull 完成" -ForegroundColor Green
    } elseif ($Ahead -gt 0 -and $Behind -eq 0) {
        # (b) 领先, 守门 #1 R-05 不 push
        Write-Host "  本地领先 origin $Ahead commit, 守门 #1 R-05 不自动 push" -ForegroundColor Yellow
        Write-Host "  如确认要同步给同事, 手动: git push" -ForegroundColor Yellow
    } elseif ($Ahead -gt 0 -and $Behind -gt 0) {
        # 分叉
        throw "本地跟 origin 分叉 (ahead $Ahead, behind $Behind), 需要手动 rebase 或 merge"
    } else {
        Write-Host "  跟 origin 一致" -ForegroundColor Green
    }
} else {
    # (c) 无 upstream (worktree 新建分支典型场景)
    Write-Host "  当前分支无 upstream (per 9/6 18:50 JST 实证: worktree 新建分支常见)" -ForegroundColor Yellow
    Write-Host "  拉 origin fetch 拿最新, 报告 ahead/behind main ..." -ForegroundColor Cyan
    git fetch origin 2>&1 | Out-Null
    $FetchOk = $LASTEXITCODE
    if ($FetchOk -ne 0) {
        Write-Host "  git fetch origin 失败 (exit $FetchOk), 跳过 sync 步骤继续" -ForegroundColor Yellow
        Write-Host "  提示: 检查 remote 可达性 (per 守门 #1a 网络错误)" -ForegroundColor Yellow
    } else {
        $MainSha = git rev-parse --verify origin/main 2>$null
        if ($MainSha) {
            $LeftRight = (git rev-list --left-right --count "origin/main...HEAD" 2>$null) -split '\s+'
            $Behind    = [int]$LeftRight[0]
            $Ahead     = [int]$LeftRight[1]
            Write-Host "  比对 origin/main: ahead $Ahead, behind $Behind" -ForegroundColor Cyan
            if ($Behind -gt 0) {
                Write-Host "  origin/main 有 $Behind commit 还没合到当前分支" -ForegroundColor Yellow
                Write-Host "  提示: 手动 rebase / merge, 或 git push -u origin HEAD 建 upstream" -ForegroundColor Yellow
            } else {
                Write-Host "  本地领先 origin/main, 守门 #1 R-05 不自动 push" -ForegroundColor Yellow
            }
        }
    }
}
Write-Host "[1/3] git 同步检查完成" -ForegroundColor Green

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
    # 用 Start-Process 拿 npm PID, 便于退出时 taskkill /T 收整棵树
    # (避免 husky / postinstall 钩子派生的 node 子进程残留孤儿)
    $NpmProc = Start-Process -FilePath 'npm.cmd' `
                              -ArgumentList 'ci' `
                              -NoNewWindow `
                              -PassThru `
                              -Wait `
                              -RedirectStandardOutput "$FrontendDir\ci.out.log" `
                              -RedirectStandardError  "$FrontendDir\ci.err.log"
    $Script:TrackedPids.Add($NpmProc.Id) | Out-Null
    $NpmExit = $NpmProc.ExitCode
    if ($NpmExit -eq 0) {
        $CiOk = $true
    } else {
        # Fallback: npm ci 失败 (典型: package.json 比 lock 新, lock 不同步)
        # 触发自动 npm install 修 lock, 再重试一次
        Write-Host "  npm ci 失败 (exit $NpmExit), 触发 fallback: npm install 修 lock + retry" -ForegroundColor Yellow
        Write-Host "  (per 9/6 17:07 JST 实际验证: Sprint commit ef2bc80 加依赖没重生 lock)" -ForegroundColor Yellow
        $InstallProc = Start-Process -FilePath 'npm.cmd' `
                                     -ArgumentList 'install','--no-audit','--no-fund' `
                                     -NoNewWindow `
                                     -PassThru `
                                     -Wait `
                                     -RedirectStandardOutput "$FrontendDir\ci.out.log" `
                                     -RedirectStandardError  "$FrontendDir\ci.err.log"
        $Script:TrackedPids.Add($InstallProc.Id) | Out-Null
        $InstallExit = $InstallProc.ExitCode
        if ($InstallExit -ne 0) {
            throw "npm install 失败, exit code: $InstallExit (修 lock 重试都失败, 需要手动介入)"
        }
        Write-Host "  npm install 完成, 锁已修复, 重试 npm ci ..." -ForegroundColor Cyan
        $CiProc = Start-Process -FilePath 'npm.cmd' `
                                -ArgumentList 'ci' `
                                -NoNewWindow `
                                -PassThru `
                                -Wait `
                                -RedirectStandardOutput "$FrontendDir\ci.out.log" `
                                -RedirectStandardError  "$FrontendDir\ci.err.log"
        $Script:TrackedPids.Add($CiProc.Id) | Out-Null
        $CiRetryExit = $CiProc.ExitCode
        if ($CiRetryExit -ne 0) {
            throw "npm ci 重试仍失败, exit code: $CiRetryExit"
        }
        $CiOk = $true
    }
} finally {
    Pop-Location
    Stop-TrackedTree
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
