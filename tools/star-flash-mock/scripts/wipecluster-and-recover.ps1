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
#   - 默认模式 (Mavis 不可代理): Ulysses 必在 admin PowerShell 跑 WipeCluster
#     Step 2 (sudo k3s-uninstall.sh + k3s install), 跑完按 y 确认
#   - Mavis 自动化模式 (Ulysses 主动给 $env:UbuntuPW 授权时): 设
#     $env:MAVIS_AUTO_WIPE = "1" 后跑此脚本, 脚本自动用 $env:UbuntuPW pipe stdin 跑
#     sudo (守门 #5 形式: 密码不打印, 只 invoke)
#
# 跟 start-k3s-backend.ps1 关系:
#   - start-k3s-backend.ps1: 启 k3s server (WipeCluster 后必跑)
#   - wipecluster-and-recover.ps1: 删 k3s 重建 (Mavis 推荐 A 路径)
#
# 用法 1 (默认手动, admin PowerShell):
#   1. pwsh -File tools\star-flash-mock\scripts\wipecluster-and-recover.ps1
#   2. 按提示手动跑:
#        wsl -d Ubuntu
#        sudo /usr/local/bin/k3s-uninstall.sh
#        exit
#        wsl -d Ubuntu -- sudo /usr/local/bin/k3s install
#   3. 按 y 继续, 脚本自动跑 Step 3-9
#
# 用法 2 (Mavis 自动化, Ulysses 已授权 $env:UbuntuPW):
#   1. $env:MAVIS_AUTO_WIPE = "1"
#   2. pwsh -File tools\star-flash-mock\scripts\wipecluster-and-recover.ps1
#   3. 脚本自动跑 WipeCluster (Step 2 用 $env:UbuntuPW pipe stdin 跑 sudo) + 重建验证
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

# ---- 2. WipeCluster (默认手动, $env:MAVIS_AUTO_WIPE=1 时自动用 $env:UbuntuPW pipe sudo) ----
Write-Host "[2/9] WipeCluster 重建 k3s (uninstall + install) ..." -ForegroundColor Cyan

if ($env:MAVIS_AUTO_WIPE -eq "1") {
    # 自动化模式: $env:UbuntuPW | wsl -d Ubuntu -- sudo -S ...
    # 密码完全不上任何命令行 (ps / wsl 命令行都不出现),
    # 只走 stdin pipe 到 wsl 内 sudo -S, 守门 #5 严守
    # (实测: 嵌 bash -lc 复杂 quoting 在 WSL host 半死时触发 NAT 子组件 EOF 错,
    #  改 sudo -S 直连 (no bash -lc) 0 err, 2026-09-08 13:33 JST 实证)
    if ([string]::IsNullOrEmpty($env:UbuntuPW)) {
        Write-Host "  ERROR: \$env:MAVIS_AUTO_WIPE=1 但 \$env:UbuntuPW 未设, 退回手动模式" -ForegroundColor Red
        $env:MAVIS_AUTO_WIPE = ""
    } else {
        Write-Host "  模式: MAVIS_AUTO_WIPE (stdin pipe + sudo -S 直连, 守门 #5 严守)" -ForegroundColor Green

        # 0: 探 k3s 是否已装 (幂等: 已装走 uninstall+install, 未装直接 install)
        $whichK3s = wsl -d Ubuntu -- bash -lc "command -v k3s" 2>&1 | Out-String
        $k3sInstalled = $whichK3s.Trim().EndsWith("/k3s")
        Write-Host "  0/2: k3s installed: $k3sInstalled ($($whichK3s.Trim()))"

        if ($k3sInstalled) {
            Write-Host "  2a/2: 卸载 k3s (k3s-uninstall.sh) ..."

            $env:UbuntuPW | wsl -d Ubuntu -- sudo -S /usr/local/bin/k3s-uninstall.sh 2>&1 | Tee-Object -Variable uninstOut | Out-Null
            Write-Host "  uninstall output (last 5 行):"
            @($uninstOut | Select-Object -Last 5) | ForEach-Object { Write-Host "    $_" }
            if ($LASTEXITCODE -ne 0) {
                Write-Host "  ERROR: k3s-uninstall 失败 exit=$LASTEXITCODE" -ForegroundColor Red
                exit 2
            }
            Write-Host "  uninstall OK (exit=$LASTEXITCODE)"
            Write-Host ""
        } else {
            Write-Host "  2a/2: 跳过 (k3s 未装, 直接 install)"
            Write-Host ""
        }

        Write-Host "  2b/2: 重装 k3s (get.k3s.io) ..."
        Write-Host "    (k3s uninstall 会把 /usr/local/bin/k3s binary 也删, install 改用 get.k3s.io 重装)"

        # 2b-1: 下载 install 脚本 (leo19 跑, 不需 sudo)
        $curlOut = wsl -d Ubuntu -- curl -sfL https://get.k3s.io -o /tmp/k3s-install.sh 2>&1 | Out-String
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  ERROR: get.k3s.io 下载失败 exit=$LASTEXITCODE" -ForegroundColor Red
            Write-Host "  $curlOut" -ForegroundColor Red
            exit 2
        }
        Write-Host "    downloaded /tmp/k3s-install.sh"

        # 2b-2: sudo 跑 install (stdin pipe 密码, 守门 #5)
        $env:UbuntuPW | wsl -d Ubuntu -- sudo -S bash /tmp/k3s-install.sh 2>&1 | Tee-Object -Variable instOut | Out-Null
        Write-Host "  install output (last 8 行):"
        @($instOut | Select-Object -Last 8) | ForEach-Object { Write-Host "    $_" }
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  ERROR: k3s install 失败 exit=$LASTEXITCODE" -ForegroundColor Red
            exit 2
        }
        Write-Host "  install OK (exit=$LASTEXITCODE)"
        Write-Host ""
    }
}

if ($env:MAVIS_AUTO_WIPE -ne "1") {
    Write-Host "  ⚠️  必手动: WipeCluster 重建 k3s (Mavis 不能代理 sudo)" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Ulysses 必在 admin PowerShell 跑以下命令:" -ForegroundColor Yellow
    Write-Host "    wsl -d Ubuntu" -ForegroundColor Yellow
    Write-Host "    sudo /usr/local/bin/k3s-uninstall.sh  # 卸载 k3s" -ForegroundColor Yellow
    Write-Host "    exit  # 退 wsl" -ForegroundColor Yellow
    Write-Host "    wsl -d Ubuntu -- sudo /usr/local/bin/k3s install  # 重新装 k3s" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  或设 \$env:MAVIS_AUTO_WIPE=1 走自动模式 (需 \$env:UbuntuPW 已设)" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  跑完按任意键继续 (Ulysses 跑完后请按 y 确认):" -ForegroundColor Yellow
    $confirm = Read-Host "  Confirm (y/n)"
    if ($confirm -ne 'y') {
        Write-Host "  Aborted by user" -ForegroundColor Yellow
        exit 2
    }
    Write-Host ""
}

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
