# =====================================================================
# wipecluster-and-recover.ps1 — WipeCluster 全流程 (per 2026-09-08 13:26 JST UAT 闭环)
# =====================================================================
# 目的: 修复 k3s cluster 内部网络层损坏 (v30 (c) 症状: kubectl port-forward
#       / NodePort 100% 不通), 重建 k3s 后再 apply star-mock envoy pod
#       走 kubectl port-forward 验证 (envoy 8080 静态文本 "not found")
#
# 设计原则 (per 守门):
#   - 幂等: 每步可单独 re-run
#   - 失败即停: 任一步 fail 立即退出
#   - 守门 #5: 全程不打印 env 变量值 (只 invoke)
#   - 守门 #6: PowerShell only
#
# 前置 (Ulysses 必先做):
#   - 此脚本需要 admin PowerShell 跑 (sudo 不可代理)
#   - 或者拆成 2 步: admin PowerShell 跑 WipeCluster (Step 1-3), 非 admin 跑
#     后续 apply (Step 4+)
#
# 跟 start-k3s-backend.ps1 关系:
#   - start-k3s-backend.ps1: 启 k3s server (WipeCluster 后必跑)
#   - wipecluster-and-recover.ps1: 删 k3s 重建 (Mavis 推荐 A 路径)
#
# 用法 (admin PowerShell):
#   1. wsl -d Ubuntu  # 打开 wsl 终端
#   2. sudo /usr/local/bin/k3s-uninstall.sh
#   3. exit  # 退 wsl
#   4. wsl -d Ubuntu -- sudo /usr/local/bin/k3s install
#   5. sleep 60
#   6. wsl -d Ubuntu -- sudo systemctl status k3s
#   7. (Mavis 跑) pwsh -File tools\star-flash-mock\scripts\verify-k3s-uat-3000.ps1
#   8. (Mavis 跑) pkill kubectl port-forward (if needed)
#   9. (Mavis 跑) curl -i http://localhost:3000  (期望 200 + "not found")
#
# 关键事实 (per 2026-09-08 K3S-v2 实战):
#   - k3s containerd 不需要重启 (k3s install 自带 containerd 启动)
#   - k3s uninstall 不会删 /var/lib/rancher/k3s (默认会删 server 数据)
#   - 重建后必等 60s 让 systemd 拉起 + kubelet 跟 containerd 同步 (守门 v28)
#   - WipeCluster 必重 apply tools\star-flash-mock\k3s\*.yaml (deployment + CM 都没了)
# =====================================================================

$ErrorActionPreference = 'Stop'

Write-Host ""
Write-Host "==== WipeCluster + 重建 k3s 全流程 (per 2026-09-08 13:26 JST) ====" -ForegroundColor Cyan
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

# ---- 1. 探测 WSL distro 可达 ----
Write-Host "[1/9] 探测 WSL distro 'Ubuntu' 可达 ..." -ForegroundColor Cyan
$probe = wsl -d Ubuntu echo "distro-ok" 2>&1 | Out-String
if ($LASTEXITCODE -ne 0 -or -not $probe.Contains("distro-ok")) {
    Write-Host "  ERROR: distro 'Ubuntu' 不可达, exit code: $LASTEXITCODE" -ForegroundColor Red
    exit 1
}
Write-Host "  distro 'Ubuntu' OK"
Write-Host ""

# ---- 2. WipeCluster (Mavis 不能代理 sudo, 必 Ulysses admin PowerShell 跑) ----
Write-Host "[2/9] ⚠️  必手动: WipeCluster 重建 k3s (Mavis 不能代理 sudo)" -ForegroundColor Yellow
Write-Host ""
Write-Host "  Ulysses 必在 admin PowerShell 跑以下命令:" -ForegroundColor Yellow
Write-Host "    wsl -d Ubuntu" -ForegroundColor Yellow
Write-Host "    sudo /usr/local/bin/k3s-uninstall.sh  # 卸载 k3s (会清空 /var/lib/rancher/k3s)" -ForegroundColor Yellow
Write-Host "    exit  # 退 wsl 回到 PowerShell" -ForegroundColor Yellow
Write-Host "    wsl -d Ubuntu -- sudo /usr/local/bin/k3s install  # 重新装 k3s" -ForegroundColor Yellow
Write-Host ""
Write-Host "  跑完按任意键继续 (Ulysses 跑完后请按 y 确认):" -ForegroundColor Yellow
$confirm = Read-Host "  Confirm (y/n)"
if ($confirm -ne 'y') {
    Write-Host "  Aborted by user" -ForegroundColor Yellow
    exit 2
}
Write-Host ""

# ---- 3. 等 60s + 验证 k3s 拉起 ----
Write-Host "[3/9] 等 60s 让 k3s systemd 自动拉起 + kubelet 跟 containerd 同步 (守门 v28) ..." -ForegroundColor Cyan
Start-Sleep -Seconds 60
Write-Host "  60s 已过"
Write-Host ""

# ---- 4. v27 验证: journalctl -u k3s 无 "Skipping pod sync" ----
Write-Host "[4/9] v27 验证: journalctl -u k3s --since '5 min ago' | grep -c 'Skipping pod sync' = 0 ..." -ForegroundColor Cyan
$skippingCount = wsl -d Ubuntu -- bash -lc "journalctl -u k3s --no-pager --since '5 min ago' 2>&1 | grep -c 'Skipping pod sync'" 2>&1 | Out-String
$skippingCount = $skippingCount.Trim()
if ($skippingCount -ne "0") {
    Write-Host "  ERROR: 5min 内有 $skippingCount 条 'Skipping pod sync', 等久一点" -ForegroundColor Red
    Write-Host "  建议: sleep 60s 再跑" -ForegroundColor Yellow
    exit 2
}
Write-Host "  v27 OK (5min 0 条 skip)"
Write-Host ""

# ---- 5. 6443 + node Ready 验证 ----
Write-Host "[5/9] 6443 LISTEN + node Ready 验证 ..." -ForegroundColor Cyan
$portListen = Get-NetTCPConnection -LocalPort 6443 -State Listen -ErrorAction SilentlyContinue
if ($null -eq $portListen) {
    Write-Host "  ERROR: 6443 端口未在主机监听 (wslrelay 没转发?)" -ForegroundColor Red
    exit 3
}
Write-Host "  6443 LISTEN OK (PID $($portListen[0].OwningProcess))"

$nodeReady = wsl -d Ubuntu -- bash -lc "KUBECONFIG=/home/leo19/.kube/config kubectl get nodes --no-headers 2>&1 | head -1" 2>&1 | Out-String
if (-not $nodeReady.Contains("Ready")) {
    Write-Host "  ERROR: node not Ready: $nodeReady" -ForegroundColor Red
    exit 3
}
Write-Host "  node Ready: $nodeReady".Trim()
Write-Host ""

# ---- 6. 重 apply star-mock envoy (k3s 重建后所有资源都没了) ----
Write-Host "[6/9] 重 apply star-mock envoy (deployment + 2 CM) ..." -ForegroundColor Cyan
$applyResult = wsl -d Ubuntu -- bash -lc "KUBECONFIG=/home/leo19/.kube/config kubectl apply -f /mnt/d/Star/.worktrees/feat-auto-20260908-204a1a91/tools/star-flash-mock/k3s/ 2>&1" 2>&1 | Out-String
Write-Host "  $applyResult".Trim()
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ERROR: kubectl apply 失败" -ForegroundColor Red
    exit 4
}
Write-Host ""

# ---- 7. 等 pod Ready (60s 预算) ----
Write-Host "[7/9] 等 envoy pod 1/1 Running (60s 预算) ..." -ForegroundColor Cyan
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
    exit 5
}
Write-Host ""

# ---- 8. enable port-forward service (v29 守门) ----
Write-Host "[8/9] enable port-forward service (v29 必先 enable, pod Ready 后再 start) ..." -ForegroundColor Cyan
$enableResult = wsl -d Ubuntu -- bash -lc "XDG_RUNTIME_DIR=/run/user/\$(id -u) systemctl --user enable --now k3s-portforward.service 2>&1" 2>&1 | Out-String
Write-Host "  enable: $enableResult".Trim()
Start-Sleep -Seconds 5

# ---- 9. curl 3000 验证 (envoy "not found" 文本) ----
Write-Host "[9/9] curl 3000 验证 (期望 200 + 'not found' envoy direct_response) ..." -ForegroundColor Cyan
try {
    $r = Invoke-WebRequest -Uri 'http://localhost:3000' -TimeoutSec 5 -UseBasicParsing
    $preview = $r.Content.Substring(0, [Math]::Min(200, $r.Content.Length))
    if ($r.StatusCode -eq 200 -and $r.Content -match "not found") {
        Write-Host "  status=$($r.StatusCode) len=$($r.Content.Length) preview=$preview" -ForegroundColor Green
        Write-Host ""
        Write-Host "==== UAT 闭环完全成功 (envoy 8080 渲染) ====" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "  status=$($r.StatusCode) 但 body 不是 envoy 'not found', 是: $preview" -ForegroundColor Yellow
        Write-Host "  可能是 kubectl proxy 模式 (v1.1 退化) 或 spdy tunnel 仍断" -ForegroundColor Yellow
        exit 6
    }
} catch {
    Write-Host "  ERROR: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "  链路仍断, 需进一步诊断 (kubectl port-forward spdy tunnel / NodePort kube-proxy 都可能不工作)" -ForegroundColor Red
    exit 6
}
