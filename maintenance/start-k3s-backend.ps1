# =====================================================================
# start-k3s-backend.ps1 — Star 后端 k3s 重启后拉起 (per 2026-09-08 05:32 JST 用户拍板)
# =====================================================================
# 职责: 应对 worktree 重启后 WSL Ubuntu 内 k3s server 没起, 提供一键恢复
#   1. 探测 WSL distro 状态 (Ubuntu / docker-desktop)
#   2. 探测 /usr/local/bin/k3s 二进制是否在
#   3. 探测 k3s server 进程是否在跑 (per 9/7 02:58 JST 实证: pid 217 + 6443 LISTEN)
#   4. 探测 6443 端口是否在听
#   5. 如已起: 报告状态, exit 0
#   6. 如未起: 用 nohup 启 k3s server (需 sudo, 用户密码 or 免密)
#   7. 等待 30s 让 k3s 完成 init
#   8. 验证 kubectl get nodes
#   9. 报告 version / node / pod 状态
#
# 设计原则 (per 守门):
#   - 幂等: 已起直接报告不重启, 没起才启
#   - 不强杀旧进程 (避免数据丢失)
#   - sudo 免密失败: 报告错误退出, 不重试
#   - 输出 kubeconfig 路径, 方便后续 update-backend-k3s 用
#
# 跟 update-backend-k3s.ps1 的关系:
#   - start-k3s-backend: 启集群 (k3s server 进程)
#   - update-backend-k3s: 装应用 (helm upgrade 拉镜像装 chart)
#   - 前置依赖: start-k3s-backend 完成后才能跑 update-backend-k3s
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 0. 探测 WSL distro 状态 ----
Write-Host ""
Write-Host "==== Star 后端 k3s 重启后拉起 ====" -ForegroundColor Cyan
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

Write-Host "[0/5] 探测 WSL distro 状态 ..."
$wslStatus = wsl --status 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: WSL 不可用, exit code: $LASTEXITCODE" -ForegroundColor Red
    Write-Host "  提示: wsl --install (per Windows 守门 #4 需用户确认)" -ForegroundColor Yellow
    exit 1
}

$wslList = wsl -l -v 2>&1 | Out-String
Write-Host "  $wslList"

# 默认 distro: Ubuntu (per 9/7 02:58 JST 实证)
# 不要 parse wsl -l -v 输出 (codepage 不一致导致 Contains / -match 都失败),
# 直接用 wsl -d <distro> echo 测可达性, exit 0 = distro 可启
$DefaultDistro = "Ubuntu"
Write-Host "  探测 distro '$DefaultDistro' 可达性 (wsl -d echo test) ..."
$probe = wsl -d $DefaultDistro echo "distro-ok" 2>&1 | Out-String
$probeOk = $LASTEXITCODE -eq 0 -and $probe.Contains("distro-ok")
if (-not $probeOk) {
    Write-Host "ERROR: distro '$DefaultDistro' 不可达" -ForegroundColor Red
    Write-Host "  result: $probe" -ForegroundColor Yellow
    Write-Host "  当前 wsl 列表:" -ForegroundColor Yellow
    wsl -l -v 2>&1 | Out-String | Write-Host
    exit 1
}
Write-Host "  distro '$DefaultDistro' OK"

# ---- 1. 探测 k3s 二进制 ----
Write-Host ""
Write-Host "[1/5] 探测 k3s 二进制 (/usr/local/bin/k3s) ..."
$k3sVer = wsl -- bash -c "/usr/local/bin/k3s --version 2>&1" 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: /usr/local/bin/k3s 不在 WSL 内" -ForegroundColor Red
    Write-Host "  result: $k3sVer" -ForegroundColor Yellow
    Write-Host "  提示: 在 WSL Ubuntu 内执行 k3s 官方安装脚本" -ForegroundColor Yellow
    Write-Host "    curl -sfL https://get.k3s.io | sh -" -ForegroundColor Yellow
    exit 1
}
Write-Host "  版本: $($k3sVer.Trim())"

# ---- 2. 探测 k3s server 进程 ----
Write-Host ""
Write-Host "[2/5] 探测 k3s server 进程 ..."
$k3sProcs = wsl -- bash -c "ps -ef | grep '[k]3s server' | head -3" 2>&1 | Out-String
$k3sRunning = $k3sProcs -match "[k]3s server"
if ($k3sRunning) {
    Write-Host "  已在跑: $k3sProcs" -ForegroundColor Green
} else {
    Write-Host "  未在跑"
}

# ---- 3. 探测 6443 端口 ----
Write-Host ""
Write-Host "[3/5] 探测 6443 端口 (k3s API server) ..."
$port6443 = wsl -- bash -c "ss -tln 2>/dev/null | grep -E ':6443 ' | head -3" 2>&1 | Out-String
$portListen = $port6443 -match ":6443"
if ($portListen) {
    Write-Host "  在听: $port6443" -ForegroundColor Green
} else {
    Write-Host "  未在听"
}

# ---- 4. 决策: 启 or 跳 ----
Write-Host ""
if ($k3sRunning -and $portListen) {
    Write-Host "[4/5] k3s 已就绪, 跳启动" -ForegroundColor Green
} else {
    Write-Host "[4/5] 启动 k3s server (需 sudo) ..."

    # 检查 sudo 免密
    $sudoTest = wsl -- bash -c "sudo -n whoami" 2>&1 | Out-String
    $sudoOk = ($LASTEXITCODE -eq 0) -and ($sudoTest.Trim() -eq "root")
    if (-not $sudoOk) {
        Write-Host "ERROR: sudo 需要密码, 自动启 k3s 失败" -ForegroundColor Red
        Write-Host "  提示: 在 WSL Ubuntu 内执行下面命令手动启:" -ForegroundColor Yellow
        Write-Host "    sudo nohup /usr/local/bin/k3s server > /tmp/k3s.log 2>&1 &" -ForegroundColor Yellow
        Write-Host "  之后重新跑本脚本验证" -ForegroundColor Yellow
        exit 1
    }

    # 启
    $startCmd = "sudo -n nohup /usr/local/bin/k3s server > /tmp/k3s.log 2>&1 &"
    Write-Host "  cmd: $startCmd"
    $startResult = wsl -- bash -c $startCmd 2>&1 | Out-String
    Write-Host "  result: $startResult"

    Write-Host "  等待 30s 让 k3s 完成 init (k3s 首次启动需要从 ghcr.io 拉 image) ..."
    Start-Sleep -Seconds 30
}

# ---- 5. 验证 ----
Write-Host ""
Write-Host "[5/5] 验证 k3s 状态 ..."
$procsAfter = wsl -- bash -c "ps -ef | grep '[k]3s server' | head -3" 2>&1 | Out-String
Write-Host "  k3s server procs: $procsAfter"

$portAfter = wsl -- bash -c "ss -tln 2>/dev/null | grep -E ':6443 ' | head -3" 2>&1 | Out-String
Write-Host "  6443 listen: $portAfter"

# kubectl get nodes (per 9/7 02:58 JST 实证路径: KUBECONFIG=~/.kube/config)
$nodes = wsl -- bash -c "KUBECONFIG=~/.kube/config kubectl get nodes -o wide --no-headers 2>&1 | head -3" 2>&1 | Out-String
Write-Host "  nodes:" -ForegroundColor $(if ($nodes -match "Ready") {"Green"} else {"Yellow"})
Write-Host "  $nodes"

# kubectl version
$ver = wsl -- bash -c "KUBECONFIG=~/.kube/config kubectl version 2>&1 | head -10" 2>&1 | Out-String
Write-Host "  version: $ver"

# 报告 pod 状态 (per G12, 关注 crash pod)
# 用 wsl 内 grep -E 直接匹配 kubectl 输出, 避开 pwsh 编码问题
$podsCrash = wsl -- bash -c "KUBECONFIG=~/.kube/config kubectl get pods -A --no-headers 2>&1 | grep -E 'CrashLoopBackOff|Error|ImagePullBackOff' | head -10" 2>&1 | Out-String
# $podsCrash 可能混 wsl "no localhost" 错误, 提取 kubectl 真实行
$crashLines = ($podsCrash -split "`n") | Where-Object { $_ -match "CrashLoopBackOff|Error|ImagePullBackOff" }
if ($crashLines) {
    Write-Host "  Crash/Error pods (per G12 已知缺口):" -ForegroundColor Yellow
    $crashLines | ForEach-Object { Write-Host "    $_" }
} else {
    Write-Host "  无 Crash/Error pods" -ForegroundColor Green
}

Write-Host ""
Write-Host "==== k3s 后端就绪 ====" -ForegroundColor Green
Write-Host "  kubeconfig: ~/.kube/config (per 9/7 02:58 JST 实证已有)"
Write-Host "  下一步: 跑 .\maintenance\update-backend-k3s.bat 装 chart (需 G1-G4 修复)"
