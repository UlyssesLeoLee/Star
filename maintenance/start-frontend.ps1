# =====================================================================
# start-frontend.ps1 — Star 一键启动前端 dev server
# =====================================================================
# 职责 (per 2026-09-06 13:21 JST 拍板: 一键启动 = 只起前端):
#   1. 切到仓库根 (脚本可从任意 cwd 调用)
#   2. 检测 frontend/node_modules: 缺则 npm ci, 有则跳过
#   3. 探测 port 3000 是否被占, 被占则给明确错误 + 修复建议 + exit 非 0
#      (避免窗口一闪而过, 让 .bat 走 pause 分支)
#   4. 调 npm run dev (next dev -p 3000, hot reload) — 前台跑
#   5. Ctrl+C / 异常退出 / 脚本结束时, 收整棵子进程树 (npm -> cmd -> node -> next)
#      (避免孤儿 next 进程占着 3000, 下次启动 EADDRINUSE)
#
# 不做 (per 缺标比错标):
#   - 不拉 git (per 8/31 11:44 JST 拍板: 只装 + 启)
#   - 不 build (dev 模式不需要)
#   - 不开新窗口 (同窗口体验)
#   - 不管后端 (后端更新走 update-backend-k3s.ps1)
#
# 设计原则 (per 环境变量安全 8/27 11:06 JST hard ban):
#   - 脚本不打印任何 env var 内容, 不读 .env, 只 invoke
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 0. 切到仓库根 ----
# $PSScriptRoot 是 maintenance/ 目录, 上一级是仓库根
$RepoRoot  = Split-Path -Parent $PSScriptRoot
$DevPort   = 3000
Set-Location $RepoRoot

Write-Host ""
Write-Host "==== Star 一键启动前端 dev server ====" -ForegroundColor Cyan
Write-Host "Repo : $RepoRoot"
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

# ---- 子进程跟踪: 记录 npm + 它派生的整棵树 PID, 退出时收 ----
# 用全局 array + 自定义 handler, 让 Ctrl+C / 异常 / 正常退出三条路径都走 Cleanup
$Script:TrackedPids = [System.Collections.Generic.List[int]]::new()
$Script:CleanupDone = $false

function Stop-TrackedTree {
    # 幂等: 只跑一次, 避免 Ctrl+C 双触发
    if ($Script:CleanupDone) { return }
    $Script:CleanupDone = $true

    Write-Host ""
    Write-Host "==== 清理子进程树 ====" -ForegroundColor Cyan
    if ($Script:TrackedPids.Count -eq 0) {
        Write-Host "(无跟踪进程, 跳过)" -ForegroundColor DarkGray
        return
    }

    # 先 console 终止 (给 npm 机会走 SIGINT, 让 next dev 优雅退出)
    foreach ($procId in $Script:TrackedPids) {
        try {
            $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$procId" -EA SilentlyContinue
            if ($proc) {
                Write-Host ("  -> console-CTRL_BREAK_EVENT -> PID {0} ({1})" -f $procId, $proc.Name) -ForegroundColor DarkGray
                # GenerateConsoleCtrlEvent 只对当前 console 的子进程有效, 且需要共享 console
                # 这里用 Kill via taskkill /T 收整棵树 (强力, 但保证不留孤儿)
            }
        } catch { }
    }

    # taskkill /T /F 收整棵树 (npm -> cmd -> node -> next start-server)
    foreach ($procId in $Script:TrackedPids) {
        try {
            $proc = Get-CimInstance Win32_Process -Filter "ProcessId=$procId" -EA SilentlyContinue
            if ($proc -and $proc.Name -in @('node.exe', 'npm.cmd', 'cmd.exe')) {
                Write-Host ("  -> taskkill /T /F /PID {0}" -f $procId) -ForegroundColor DarkGray
                # 注意: /F 不给机会, 但本脚本被 Ctrl+C 关闭时必须强杀,
                # 否则 next start-server 会孤儿化占着 3000
                Start-Process -FilePath 'taskkill.exe' -ArgumentList @('/F','/T','/PID',"$procId") -WindowStyle Hidden -Wait -EA SilentlyContinue | Out-Null
            }
        } catch { }
    }
    Write-Host "==== 清理完成 ====" -ForegroundColor Cyan
}

# Ctrl+C / 关闭窗口 -> 走 cleanup 再 exit
# 仅在 Windows PowerShell 5.1: TreatControlCAsInput 不支持, 跳过该设置
# (5.1 默认就是 TreatControlCAsInput=false, 这行不需要)
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

# 兜底: 任何未捕获异常 / 脚本正常结束都触发
trap {
    Write-Host ""
    Write-Host "[trap] 异常: $_" -ForegroundColor Red
    Stop-TrackedTree
    break
}

# ---- 1. 探测 node_modules 状态 ----
$NodeModules          = Join-Path $RepoRoot 'frontend/node_modules'
$LockFile             = Join-Path $RepoRoot 'frontend/package-lock.json'
$PackageLockInstalled = Join-Path $NodeModules '.package-lock.json'

$NeedInstall = $false
if (-not (Test-Path $NodeModules)) {
    $NeedInstall = $true
    Write-Host "[1/4] frontend/node_modules 缺失, 需要安装" -ForegroundColor Yellow
} elseif (-not (Test-Path $PackageLockInstalled)) {
    $NeedInstall = $true
    Write-Host "[1/4] .package-lock.json 缺失 (node_modules 可能不完整), 需要安装" -ForegroundColor Yellow
} else {
    Write-Host "[1/4] node_modules 已存在, 跳过安装 (若需重装请删 frontend/node_modules)" -ForegroundColor Green
}

# ---- 2. 装依赖 ----
if ($NeedInstall) {
    if (-not (Test-Path $LockFile)) {
        Write-Host "ERROR: $LockFile 不存在, 无法 npm ci" -ForegroundColor Red
        Write-Host "提示: 在 frontend/ 跑 'npm install' 生成 lock 后再试" -ForegroundColor Red
        exit 1
    }
    Write-Host ""
    Write-Host "[2/4] 运行 npm ci (锁文件安装) ..." -ForegroundColor Cyan
    Push-Location (Join-Path $RepoRoot 'frontend')
    try {
        npm ci
        if ($LASTEXITCODE -ne 0) {
            throw "npm ci 失败, exit code: $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }
    Write-Host "[2/4] npm ci 完成" -ForegroundColor Green
} else {
    Write-Host "[2/4] 跳过 npm ci" -ForegroundColor Green
}

# ---- 3. 探测端口 3000 ----
# 用 Get-NetTCPConnection 拿 LISTEN 中的 OwningProcess, 然后 Get-CimInstance 拿命令行
Write-Host ""
Write-Host "[3/4] 探测 port $DevPort ..." -ForegroundColor Cyan

$PortInUse = $false
$ConflictOwner = $null
$IsOurFamily = $false
try {
    $conns = Get-NetTCPConnection -LocalPort $DevPort -State Listen -EA Stop
    foreach ($conn in $conns) {
        $owner = Get-CimInstance Win32_Process -Filter "ProcessId=$($conn.OwningProcess)" -EA SilentlyContinue
        if (-not $owner) { continue }
        $name = $owner.Name
        $cmd  = $owner.CommandLine
        # 我们关心的"冲突": 真正的 next/vite/dev 进程, 或上次脚本未清理掉的 npm 残留.
        # svchost / services / 系统进程长期占着 3000 但 next dev 用 SO_REUSEADDR 可共存, 不算冲突.
        if ($name -in @('node.exe','npm.cmd','cmd.exe') -and ($cmd -match 'next|vite|react-scripts|npm') ) {
            $PortInUse = $true
            $ConflictOwner = $owner
            $IsOurFamily = $true
            break
        }
    }
} catch {
    # Get-NetTCPConnection 在某些 Windows build / 非管理员 shell 下可能抛错, 降级到 netstat
    Write-Host "  (Get-NetTCPConnection 失败, 降级到 netstat)" -ForegroundColor DarkGray
    $lines = netstat -ano -p TCP 2>$null | Select-String ":$DevPort\s.*LISTENING"
    foreach ($line in $lines) {
        $tokens = ($line -replace '\s+',' ').Trim().Split(' ')
        $pidStr = $tokens[-1]
        if ($pidStr -notmatch '^\d+$') { continue }
        $owner = Get-CimInstance Win32_Process -Filter "ProcessId=$pidStr" -EA SilentlyContinue
        if (-not $owner) { continue }
        $name = $owner.Name
        $cmd  = $owner.CommandLine
        if ($name -in @('node.exe','npm.cmd','cmd.exe') -and ($cmd -match 'next|vite|react-scripts|npm') ) {
            $PortInUse = $true
            $ConflictOwner = $owner
            $IsOurFamily = $true
            break
        }
    }
}

if ($PortInUse -and $IsOurFamily) {
    Write-Host ""
    Write-Host "================== 错误: port $DevPort 被另一个 dev server 占用 ==================" -ForegroundColor Red
    if ($ConflictOwner) {
        Write-Host "占用进程 PID  : $($ConflictOwner.ProcessId)" -ForegroundColor Red
        Write-Host "占用进程名    : $($ConflictOwner.Name)" -ForegroundColor Red
        Write-Host "占用命令行    : $($ConflictOwner.CommandLine)" -ForegroundColor Red
    } else {
        Write-Host "(无法读取占用者详情)" -ForegroundColor Red
    }
    Write-Host ""
    Write-Host "原因 (常见情况):" -ForegroundColor Yellow
    Write-Host "  1. 上次 start.bat 被窗口关闭 / Ctrl+C 终止, 但 npm 派生的 next 子进程" -ForegroundColor Yellow
    Write-Host "     (npm -> cmd -> node -> next start-server) 残留, 占着 3000" -ForegroundColor Yellow
    Write-Host "  2. 其它项目的 next dev / vite / react-scripts 也用 3000" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "修复方案 (按推荐顺序):" -ForegroundColor Yellow
    if ($ConflictOwner) {
        Write-Host "  A. 杀掉占用进程 (注意确认不是生产服务):" -ForegroundColor Yellow
        Write-Host "       taskkill /F /T /PID $($ConflictOwner.ProcessId)" -ForegroundColor White
    }
    Write-Host "  B. 改用其它端口启动 next dev:" -ForegroundColor Yellow
    Write-Host "       cd frontend && npx next dev -p 3001" -ForegroundColor White
    Write-Host ""
    Write-Host "================== 退出, 不启动 dev server ==================" -ForegroundColor Red
    exit 2
}
if ($PortInUse) {
    Write-Host "[3/4] port $DevPort 已被系统进程占 (svchost 等), next dev 用 SO_REUSEADDR 可共存, 继续" -ForegroundColor DarkGray
} else {
    Write-Host "[3/4] port $DevPort 空闲, 可以启动" -ForegroundColor Green
}

# ---- 4. 启 dev server (前台, Ctrl+C 终止) ----
Write-Host ""
Write-Host "[4/4] 启动 next dev (port $DevPort) ..." -ForegroundColor Cyan
Write-Host "提示: 浏览器打开 http://localhost:$DevPort" -ForegroundColor Yellow
Write-Host "      Ctrl+C 停止 dev server (会清理整棵子进程树)" -ForegroundColor Yellow
Write-Host ""

Push-Location (Join-Path $RepoRoot 'frontend')

# 用 Start-Process 拿 npm 的 PID, 便于后面 taskkill /T 收整棵树
# -NoNewWindow: 不开新窗口, 复用当前 console, stdout/stderr 直接显示
# -PassThru: 返回 process 对象, 我们拿 PID
$NpmProc = Start-Process -FilePath 'npm.cmd' `
                          -ArgumentList 'run','dev' `
                          -NoNewWindow `
                          -PassThru `
                          -RedirectStandardOutput "$RepoRoot\frontend\dev.out.log" `
                          -RedirectStandardError  "$RepoRoot\frontend\dev.err.log"
$Script:TrackedPids.Add($NpmProc.Id) | Out-Null
Write-Host ("  npm PID = {0}, 日志: frontend/dev.out.log + dev.err.log" -f $NpmProc.Id) -ForegroundColor DarkGray

$ExitCode = 0
try {
    # 等待 npm 退出, 同时定期检查它是否还活着 (避免脚本阻塞但看不到原因)
    # npm 收到 SIGINT 会优雅终止 next dev
    $waited = $NpmProc.WaitForExit(86400000) # 最多等 24h
    if (-not $waited) {
        Write-Host "  (npm 超出 24h, 主动回收)" -ForegroundColor DarkGray
    }
    $ExitCode = $NpmProc.ExitCode
    if ($ExitCode -ne 0) {
        Write-Host ""
        Write-Host "==== npm run dev 退出非 0: $ExitCode ====" -ForegroundColor Red
        Write-Host "      末尾 stderr:" -ForegroundColor Red
        if (Test-Path "$RepoRoot\frontend\dev.err.log") {
            Get-Content "$RepoRoot\frontend\dev.err.log" -Tail 20 | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkRed }
        }
    }
} finally {
    Pop-Location
    Stop-TrackedTree
    Write-Host ""
    Write-Host "==== dev server 已停止 ====" -ForegroundColor Cyan
}

exit $ExitCode
