# =====================================================================
# start-k3s-backend.ps1 — 画布游戏 k3s 后端 一键部署 (per 2026-09-07 07:06 JST 用户拍板)
# =====================================================================
# 职责 (per 守门 #1 v25 + 9/5 04:03 JST 拍板推荐项立即执行):
#   1. 切到仓库根 (脚本可从任意 cwd 调用)
#   2. 探测 k3s 状态 (kubectl + k3s cluster)
#   3. 探测 secret 是否就绪 (per 守门 #5 env 安全, 不打印 secret 内容)
#   4. 调 kubectl apply -f deploy/canvas-game-k3s.yaml (4 Deployment + 4 Service + envoy + ...)
#   5. 等待 rollout status 完成 (per Deployment)
#   6. 探测 envoy 边缘 (per 9/1 13:05 JST 拍板 envoy 独立 deployment)
#   7. 打印 port-forward 命令 (用户本地访问)
#   8. Ctrl+C 时 kubectl rollout 进程被 SIGINT 终止, 整个窗口干净退出
#
# 不做 (per 缺标比错标):
#   - 不拉 git (per 11:44 JST 拍板: 只装 + 启, R-05 守门)
#   - 不 build 镜像 (per 11:44 JST 拍板: 一键脚本只部署, 不构建; P0 启动阶段才构建)
#   - 不打印 env var 内容 (per 8/27 11:06 JST hard ban)
#
# 设计原则:
#   - 用 pwsh -NoProfile -NonInteractive (per Windows 守门 11:35 JST 拍板)
#   - exit $LASTEXITCODE 把错误透传给 .bat 窗口
#   - 不重置 / 删 cluster 资源 (per 守门 #12 饱和: 不擅自扩张范围)
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 0. 切到仓库根 ----
$PSScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host ""
Write-Host "==== 画布游戏 k3s 一键部署 ====" -ForegroundColor Cyan
Write-Host "Repo : $RepoRoot"
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host "Spec : dev 本地 + 1 副本 + envoy 边缘 (per 2026-09-07 07:06 JST 用户拍板)"
Write-Host ""

# ---- 1. 探测 kubectl ----
Write-Host "[1/6] 探测 kubectl ..." -ForegroundColor Cyan
$kubectl = Get-Command kubectl -ErrorAction SilentlyContinue
if (-not $kubectl) {
    Write-Host "ERROR: kubectl 未安装或不在 PATH" -ForegroundColor Red
    Write-Host "提示: k3s 安装自带 kubectl, 或参考 https://kubernetes.io/docs/tasks/tools/" -ForegroundColor Yellow
    exit 1
}
Write-Host "  kubectl: $($kubectl.Source)" -ForegroundColor Green
Write-Host "  version: $(kubectl version --client --output=yaml 2>$null | Select-String -Pattern 'gitVersion' | Select-Object -First 1)" -ForegroundColor Green

# ---- 2. 探测 k3s cluster 连接 ----
Write-Host ""
Write-Host "[2/6] 探测 k3s cluster 连接 ..." -ForegroundColor Cyan
try {
    $clusterInfo = kubectl cluster-info 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: kubectl 无法连接 k3s cluster" -ForegroundColor Red
        Write-Host "提示: 'k3s kubectl cluster-info' 或检查 KUBECONFIG 环境变量" -ForegroundColor Yellow
        exit 1
    }
    Write-Host "  cluster: $($clusterInfo -split "`n" | Select-Object -First 1)" -ForegroundColor Green
} catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
    exit 1
}

# ---- 3. 探测 canvas-game-secrets 是否就绪 ----
Write-Host ""
Write-Host "[3/6] 探测 canvas-game-secrets ..." -ForegroundColor Cyan
$SecretExists = kubectl get secret canvas-game-secrets -n star-system 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  canvas-game-secrets 不存在" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  提示 (per 守门 #5 env 安全, 不打印 secret 内容):" -ForegroundColor Yellow
    Write-Host "    创建 secret 命令 (用真实凭据替换占位符):" -ForegroundColor Cyan
    Write-Host ""
    Write-Host '    kubectl create secret generic canvas-game-secrets -n star-system \' -ForegroundColor Gray
    Write-Host '      --from-literal=DATABASE_URL="postgres://star:REAL_PASSWORD@postgres:5432/star_canvas_game" \' -ForegroundColor Gray
    Write-Host '      --from-literal=REDIS_URL="redis://redis:6379/0" \' -ForegroundColor Gray
    Write-Host '      --from-literal=NATS_URL="nats://nats:4222" \' -ForegroundColor Gray
    Write-Host '      --from-literal=S3_BUCKET="star-canvas-game" \' -ForegroundColor Gray
    Write-Host '      --from-literal=S3_ENDPOINT="http://minio:9000" \' -ForegroundColor Gray
    Write-Host '      --from-literal=GHCR_PAT="ghp_REAL_TOKEN"' -ForegroundColor Gray
    Write-Host ""
    $Confirm = Read-Host "  是否现在退出 (Ctrl+C 取消 / Enter 继续部署, secret 后续补)"
    Write-Host "  继续部署, secret 缺失会导致 4 Deployment CrashLoopBackOff" -ForegroundColor Yellow
}

# ---- 4. 应用 deploy/canvas-game-k3s.yaml ----
Write-Host ""
Write-Host "[4/6] 应用 deploy/canvas-game-k3s.yaml (18 个 k3s 资源) ..." -ForegroundColor Cyan
$YamlFile = Join-Path $RepoRoot 'deploy/canvas-game-k3s.yaml'
if (-not (Test-Path $YamlFile)) {
    Write-Host "ERROR: $YamlFile 不存在" -ForegroundColor Red
    exit 1
}
try {
    kubectl apply -f $YamlFile 2>&1 | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: kubectl apply 失败" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
    exit 1
}

# ---- 5. 等待 rollout 完成 (4 Deployment) ----
Write-Host ""
Write-Host "[5/6] 等待 4 Deployment rollout 完成 (dev 本地, 可能 1-3 分钟) ..." -ForegroundColor Cyan
$Deployments = @('canvas-engine', 'domain-canvas', 'canvas-realtime', 'canvas-game', 'envoy-edge')
foreach ($dep in $Deployments) {
    Write-Host "  rollout $dep ..." -ForegroundColor Cyan -NoNewline
    $rolloutOutput = kubectl rollout status deployment/$dep -n star-system --timeout=180s 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host " OK" -ForegroundColor Green
    } else {
        Write-Host " FAILED" -ForegroundColor Red
        Write-Host "  $rolloutOutput" -ForegroundColor Red
        Write-Host "  提示: kubectl describe pod -n star-system -l app=$dep" -ForegroundColor Yellow
    }
}

# ---- 6. 探测 envoy 边缘 + 打印 port-forward ----
Write-Host ""
Write-Host "[6/6] 探测 envoy 边缘 + 打印 port-forward 命令 ..." -ForegroundColor Cyan
$envoyStatus = kubectl get svc envoy-edge -n star-system -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>$null
if ($LASTEXITCODE -eq 0 -and $envoyStatus) {
    Write-Host "  envoy LoadBalancer: $envoyStatus" -ForegroundColor Green
} else {
    Write-Host "  envoy LoadBalancer 尚未就绪 (NodePort 30080 可用)" -ForegroundColor Yellow
}
Write-Host ""
Write-Host "==== 部署完成 ====" -ForegroundColor Green
Write-Host ""
Write-Host "本地访问 (任选其一):" -ForegroundColor Cyan
Write-Host "  1. NodePort:        http://localhost:30080" -ForegroundColor Gray
Write-Host "  2. port-forward:    kubectl port-forward -n star-system svc/envoy-edge 10080:10080" -ForegroundColor Gray
Write-Host "                    然后 http://localhost:10080" -ForegroundColor Gray
Write-Host ""
Write-Host "健康检查:" -ForegroundColor Cyan
Write-Host "  kubectl get pods -n star-system -l app=canvas-game" -ForegroundColor Gray
Write-Host "  kubectl get pods -n star-system -l app=canvas-engine" -ForegroundColor Gray
Write-Host "  kubectl logs -n star-system -l app=canvas-game --tail=50" -ForegroundColor Gray
Write-Host ""
Write-Host "更新镜像 (后续 P0 启动后用):" -ForegroundColor Cyan
Write-Host "  kubectl set image deployment/canvas-game canvas-game=ghcr.io/ulysses-lee-lee/star-canvas-game:v0.1 -n star-system" -ForegroundColor Gray
Write-Host "  kubectl rollout status deployment/canvas-game -n star-system" -ForegroundColor Gray
Write-Host ""
Write-Host "删除部署 (清理):" -ForegroundColor Cyan
Write-Host "  kubectl delete -f deploy/canvas-game-k3s.yaml" -ForegroundColor Gray
Write-Host ""
Write-Host "==== 一键部署完成, 进程保持运行 (Ctrl+C 退出) ====" -ForegroundColor Green

# ---- 7. 持续运行, Ctrl+C 终止 ----
try {
    while ($true) {
        Start-Sleep -Seconds 60
        # 每分钟打印一次 pod 状态 (简版健康检查)
        $status = kubectl get pods -n star-system -l tier=game -o jsonpath='{.items[*].status.phase}' 2>$null
        if ($status -and $status -notmatch 'Running') {
            Write-Host "[$(Get-Date -Format 'HH:mm:ss')] 警告: canvas-game pod 状态异常: $status" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host ""
    Write-Host "==== 收到 Ctrl+C, 退出 ====" -ForegroundColor Yellow
    exit 0
}
