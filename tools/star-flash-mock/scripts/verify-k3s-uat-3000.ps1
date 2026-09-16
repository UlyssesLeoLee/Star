# =====================================================================
# verify-k3s-uat-3000.ps1 — 3000 端口 UAT 验证 (per 守门 #1 v27/v28/v29)
# =====================================================================
# 职责: Ulysses 跑完 `sudo systemctl restart k3s` 之后, 走 5 步落地 UAT:
#   1. v27 验证: journalctl -u k3s 无 "Skipping pod sync" 警告
#   2. v28 验证: kubelet Running + crictl ps 非空
#   3. 重启 deployment 让 pod 用新镜像 (daocloud)
#   4. 等 pod Ready (60s 预算)
#   5. v29 启用 port-forward service + curl 3000 验证
#
# 设计原则 (per 守门):
#   - 幂等: 每步可单独 re-run
#   - 失败即停: 任一步 fail 立即退出 (守门 #1 累积规 v27-v29 必走全)
#   - 守门 #5: 全程不打印 env 变量值 (只 invoke)
#   - 守门 #6: PowerShell only
#
# 前置 (Ulysses 必先做):
#   wsl -d Ubuntu -- bash -lc 'sudo systemctl restart k3s; sleep 60'
#   (守门 #1 v28: k3s 拉起后必等 60s 验证)
#
# 用法:
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools\star-flash-mock\scripts\verify-k3s-uat-3000.ps1
# =====================================================================

$ErrorActionPreference = 'Stop'

Write-Host ""
Write-Host "==== Star 3000 端口 UAT 验证 ====" -ForegroundColor Cyan
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

# ---- 0. WSL distro 可达性 ----
Write-Host "[0/5] 探测 WSL distro 'Ubuntu' 可达性 ..." -ForegroundColor Cyan
$probe = wsl -d Ubuntu echo "distro-ok" 2>&1 | Out-String
if ($LASTEXITCODE -ne 0 -or -not $probe.Contains("distro-ok")) {
    Write-Host "  ERROR: distro 'Ubuntu' 不可达, exit code: $LASTEXITCODE" -ForegroundColor Red
    exit 1
}
Write-Host "  distro 'Ubuntu' OK"
Write-Host ""

# ---- 1. v27 验证: journalctl -u k3s 无 "Skipping pod sync" (最近 5 分钟) ----
Write-Host "[1/5] v27 验证: journalctl -u k3s | grep -c 'Skipping pod sync' = 0 (最近 5min) ..." -ForegroundColor Cyan
# 用 journalctl --since "5 min ago" 限定时间窗, 避免 daemon restart 过渡期 noise
$skippingCount = wsl -d Ubuntu -- bash -lc "journalctl -u k3s --no-pager --since '5 min ago' 2>&1 | grep -c 'Skipping pod sync'" 2>&1 | Out-String
$skippingCount = $skippingCount.Trim()
if ($skippingCount -ne "0") {
    Write-Host "  ERROR: journalctl 最近 5 分钟有 $skippingCount 条 'Skipping pod sync', kubelet 跟 containerd 通信还没建立" -ForegroundColor Red
    Write-Host "  建议: 等 v28 sleep 60s 之后再跑, 或重新 'sudo systemctl restart k3s; sleep 90' 后再跑本脚本" -ForegroundColor Yellow
    exit 2
}
Write-Host "  Skipping pod sync (5min) = 0 (kubelet 通信 OK)" -ForegroundColor Green
Write-Host ""

# ---- 2. v28 验证: kubelet Running + crictl ps 非空 (最近 5 分钟) ----
Write-Host "[2/5] v28 验证: kubelet Started + crictl ps 非空 (最近 5min) ..." -ForegroundColor Cyan
# kubelet 启动信息是 "Started kubelet" (k3s 内嵌), 不是字面 "Running"
# 用 --since "5 min ago" 限定时间窗
$kubeletStarted = wsl -d Ubuntu -- bash -lc "journalctl -u k3s --no-pager --since '5 min ago' 2>&1 | grep -c 'Started kubelet'" 2>&1 | Out-String
$crictlPsCount = wsl -d Ubuntu -- bash -lc "sudo -n k3s crictl ps 2>&1 | wc -l" 2>&1 | Out-String
$kubeletStarted = $kubeletStarted.Trim()
$crictlPsCount = $crictlPsCount.Trim()
if ($kubeletStarted -eq "0") {
    Write-Host "  ERROR: 'Started kubelet' 最近 5min 未出现, sleep 60s 不够, 建议 sleep 120s 重试" -ForegroundColor Red
    exit 3
}
if ([int]$crictlPsCount -lt 2) {
    Write-Host "  ERROR: crictl ps 只有 $crictlPsCount 行 (期望 >=2, 包含 header), containerd 还没起 sandbox" -ForegroundColor Red
    exit 3
}
Write-Host "  Started kubelet (5min) = $kubeletStarted 次" -ForegroundColor Green
Write-Host "  crictl ps = $crictlPsCount 行 (非空)" -ForegroundColor Green
Write-Host ""

# ---- 3. 重启 deployment (让 pod 用 daocloud 镜像) ----
Write-Host "[3/5] rollout restart deployment star-mock-envoy (用 daocloud 镜像) ..." -ForegroundColor Cyan
$rolloutResult = wsl -d Ubuntu -- bash -lc "KUBECONFIG=/home/leo19/.kube/config kubectl -n star-mock rollout restart deployment star-mock-envoy 2>&1" 2>&1 | Out-String
Write-Host "  $rolloutResult".Trim()
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ERROR: rollout restart 失败, exit code: $LASTEXITCODE" -ForegroundColor Red
    exit 4
}
Write-Host "  deployment restarted, 等 30s 让新 pod 创建" -ForegroundColor Yellow
Start-Sleep -Seconds 30
Write-Host ""

# ---- 4. 等 pod Ready (60s 预算) ----
Write-Host "[4/5] 等 pod Ready (60s 预算) ..." -ForegroundColor Cyan
$ready = $false
for ($i = 0; $i -lt 30; $i++) {
    $podStatus = wsl -d Ubuntu -- bash -lc "KUBECONFIG=/home/leo19/.kube/config kubectl -n star-mock get pod -l app=star-mock,component=envoy --no-headers 2>&1" 2>&1 | Out-String
    if ($podStatus -match "Running" -and $podStatus -match "1/1") {
        Write-Host "  [$($i*2)s] READY: $podStatus" -ForegroundColor Green
        $ready = $true
        break
    } else {
        Write-Host "  [$($i*2)s] $podStatus".Trim() -ForegroundColor Yellow
        Start-Sleep -Seconds 2
    }
}
if (-not $ready) {
    Write-Host "  TIMEOUT 60s" -ForegroundColor Red
    Write-Host "  建议: kubectl -n star-mock describe pod 看 events 段" -ForegroundColor Yellow
    exit 5
}
Write-Host ""

# ---- 5. v29 启用 port-forward service + curl 3000 验证 ----
Write-Host "[5/5] v29 启用 port-forward service + curl localhost:3000 验证 ..." -ForegroundColor Cyan

# 先 enable (per v29 规则: 之前是 disabled, 现在 pod Ready 后 enable)
$enableResult = wsl -d Ubuntu -- bash -lc "XDG_RUNTIME_DIR=/run/user/\$(id -u) systemctl --user enable --now k3s-portforward.service 2>&1" 2>&1 | Out-String
Write-Host "  enable: $enableResult".Trim()

# 等 5s 让 service listen
Start-Sleep -Seconds 5

# 验证 3000 listen
$portListen = Get-NetTCPConnection -LocalPort 3000 -State Listen -ErrorAction SilentlyContinue
if ($null -eq $portListen) {
    Write-Host "  ERROR: 3000 端口未在主机监听" -ForegroundColor Red
    exit 6
}
Write-Host "  3000 端口 LISTEN" -ForegroundColor Green

# curl 3000
try {
    $r = Invoke-WebRequest -Uri 'http://localhost:3000' -TimeoutSec 5 -UseBasicParsing
    $preview = $r.Content.Substring(0, [Math]::Min(100, $r.Content.Length))
    Write-Host "  status=$($r.StatusCode) len=$($r.Content.Length) preview=[$preview]" -ForegroundColor Green
} catch {
    Write-Host "  ERROR: curl localhost:3000 fail: $($_.Exception.Message)" -ForegroundColor Red
    exit 6
}

Write-Host ""
Write-Host "==== 3000 端口 UAT 验证 PASSED ====" -ForegroundColor Green
Write-Host ""
Write-Host "下一步 (Playwright 截图, 需守门 #22 + #23 路径):" -ForegroundColor Cyan
Write-Host "  python scripts/automation/console_server.py &" -ForegroundColor White
Write-Host "  # 在 frontend dev server 起来后, 走 Next.js /automation-debug 路径截图" -ForegroundColor White
