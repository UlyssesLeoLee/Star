# =====================================================================
# update-backend-k3s.ps1 — Star 更新后端 (k3s 架构)
# =====================================================================
# 职责 (per 2026-09-06 13:21 JST 拍板: k3s 架构里的所有后端更新):
#   1. 切到仓库根
#   2. git pull --ff-only (拉代码)
#   3. 探测 k3s / kubectl 工具链
#   4. 同步镜像 tag (写回 deploy/helm/star/values.yaml 的 global.imageTag)
#   5. helm upgrade --install 触发滚动更新 (Phase I 8 服务, NS=star-system)
#   6. 验证 rollout 状态
#
# 适用场景:
#   - 本机连 k3s 集群 (KUBECONFIG 已 export 或 ~/.kube/config 已存在)
#   - 镜像已推送到 ghcr.io/ulysses-lee-lee/<service>:<tag> (本脚本不负责 build & push)
#
# 已知缺口 (per 缺标比错标, DDD Review 必查):
#   - G1: 仓里没有 Dockerfile + build 脚本, "8 服务镜像" 怎么 build / push 还没落地
#         -> 现状: 假设外部 CI (GH Actions) 已经把镜像推到 ghcr.io
#         -> 待补: deploy/build/<service>.Dockerfile + deploy/build/build-and-push.yml
#   - G2: values.yaml 里 global.imageTag: latest, helm upgrade 不会触发拉新镜像
#         (K8s 看到 tag 没变就不拉). 需要把 imageTag 改成 digest 或版本号 (e.g. v0.2.0)
#         -> 本脚本默认把 imageTag 从 latest 改成 git rev-parse --short HEAD, 触发重新拉
#   - G3: KUBECONFIG 路径没在本机固化, 默认读 $env:KUBECONFIG 或 ~/.kube/config
#   - G4: 8 服务 helm template 里 templates 目录只有 _helpers.tpl / NOTES.txt / secret.yaml,
#         缺 deployment.yaml / service.yaml / configmap.yaml / hpa.yaml, helm upgrade
#         实际上装不完整 — 现状 chart 是 stub, 完整 deployment yaml 待补
#   - G5: ingress className 配的 nginx (per 9/1 13:03 JST 拍板应换 envoy),
#         待 5 域 Lead 拍板迁移 (per 守门 #25 v2 招聘文档)
#
# 不做 (per 缺标比错标):
#   - 不 build / push 镜像 (G1 缺口, 外部 CI 负责)
#   - 不动前端 (前端更新走 update-frontend.ps1)
#   - 不打印任何 env var 内容 (per 8/27 11:06 JST hard ban)
#   - 不改 image registry (values.yaml global.imageRegistry, 需手动维护)
# =====================================================================

$ErrorActionPreference = 'Stop'

# ---- 0. 切到仓库根 ----
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host ""
Write-Host "==== Star 更新后端 (k3s) ====" -ForegroundColor Cyan
Write-Host "Repo : $RepoRoot"
Write-Host "Time : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ""

# ---- 1. git pull --ff-only ----
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

Write-Host "[1/5] git pull --ff-only ..." -ForegroundColor Cyan
git pull --ff-only
if ($LASTEXITCODE -ne 0) {
    throw "git pull 失败, exit code: $LASTEXITCODE"
}
Write-Host "[1/5] git pull 完成" -ForegroundColor Green

# ---- 2. 探测工具链 ----
Write-Host ""
Write-Host "[2/5] 探测 k3s / kubectl / helm 工具链 ..." -ForegroundColor Cyan

function Test-Cmd($name) {
    $cmd = Get-Command $name -ErrorAction SilentlyContinue
    if (-not $cmd) {
        Write-Host "ERROR: $name 不在 PATH, 请先装 (k3s.io / kubernetes.io / helm.sh)" -ForegroundColor Red
        exit 1
    }
    return $cmd.Source
}

$kubectl = Test-Cmd 'kubectl'
$helm    = Test-Cmd 'helm'
Write-Host "  kubectl : $kubectl" -ForegroundColor DarkGray
Write-Host "  helm    : $helm" -ForegroundColor DarkGray

# 探测 KUBECONFIG
$KubeConfig = $env:KUBECONFIG
if (-not $KubeConfig) {
    $DefaultKubeConfig = Join-Path $env:USERPROFILE '.kube\config'
    if (Test-Path $DefaultKubeConfig) {
        $KubeConfig = $DefaultKubeConfig
    } else {
        Write-Host "ERROR: KUBECONFIG 未设置, 且 $DefaultKubeConfig 不存在" -ForegroundColor Red
        Write-Host "提示: 把 k3s config 拷到 ~/.kube/config 或 export KUBECONFIG" -ForegroundColor Red
        exit 1
    }
}
Write-Host "  KUBECONFIG : $KubeConfig" -ForegroundColor DarkGray

# 探测 cluster 可达
kubectl --kubeconfig "$KubeConfig" cluster-info | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "kubectl cluster-info 失败, exit code: $LASTEXITCODE (KUBECONFIG 不对 / 集群不可达)"
}

# ---- 3. 决定 imageTag ----
Write-Host ""
Write-Host "[3/5] 决定 imageTag (per 已知缺口 G2: 不能一直用 latest) ..." -ForegroundColor Cyan

$ChartDir   = Join-Path $RepoRoot 'deploy\helm\star'
$ValuesFile = Join-Path $ChartDir 'values.yaml'
if (-not (Test-Path $ValuesFile)) {
    Write-Host "ERROR: $ValuesFile 不存在, 仓内 helm chart 缺失" -ForegroundColor Red
    exit 1
}

# 默认 tag = git short SHA, 触发 K8s 拉新镜像
$GitShort = (git rev-parse --short HEAD).Trim()
$NewTag   = "git-$GitShort"
Write-Host "  当前 HEAD : $GitShort" -ForegroundColor DarkGray
Write-Host "  新 tag    : $NewTag" -ForegroundColor DarkGray

# 改 values.yaml 的 global.imageTag
$ValuesContent = Get-Content $ValuesFile -Raw -Encoding UTF8
$OldTagPattern = '(?m)^(\s*imageTag:\s*).+$'
if ($ValuesContent -notmatch $OldTagPattern) {
    Write-Host "ERROR: values.yaml 找不到 imageTag 行, 手动检查 chart 结构" -ForegroundColor Red
    exit 1
}
$ValuesContentNew = [regex]::Replace($ValuesContent, $OldTagPattern, "`$1$NewTag")
if ($ValuesContentNew -eq $ValuesContent) {
    Write-Host "  values.yaml imageTag 已经是 $NewTag, 无需改" -ForegroundColor DarkGray
} else {
    Set-Content -Path $ValuesFile -Value $ValuesContentNew -Encoding UTF8 -NoNewline
    Write-Host "  values.yaml imageTag -> $NewTag 已落盘" -ForegroundColor Green
}

# ---- 4. helm upgrade ----
Write-Host ""
Write-Host "[4/5] helm upgrade --install star $ChartDir --namespace star-system ..." -ForegroundColor Cyan
Write-Host "  (per 已知缺口 G4: 仓内 helm chart 模板不完整, 8 服务 deployment 待补)" -ForegroundColor Yellow
Write-Host "  (per 已知缺口 G5: ingress className 还是 nginx, 待 envoy 迁移)" -ForegroundColor Yellow

# 确保 namespace 存在
kubectl --kubeconfig "$KubeConfig" get namespace star-system | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "  star-system namespace 不存在, 创建 ..." -ForegroundColor Yellow
    kubectl --kubeconfig "$KubeConfig" create namespace star-system
}

helm upgrade --install star $ChartDir `
    --namespace star-system `
    --kubeconfig "$KubeConfig" `
    --wait --timeout 5m
if ($LASTEXITCODE -ne 0) {
    throw "helm upgrade 失败, exit code: $LASTEXITCODE (大概率是 G1/G4 缺口, 镜像或 chart 模板不全)"
}
Write-Host "[4/5] helm upgrade 完成" -ForegroundColor Green

# ---- 5. 验证 rollout ----
Write-Host ""
Write-Host "[5/5] 验证 rollout 状态 (8 服务: cli/mcp/context/sa/sse/webhook/cache/saga) ..." -ForegroundColor Cyan
$Services = @('cli','mcp','context','sa','sse','webhook','cache','saga')
$RolloutOk = $true
foreach ($svc in $Services) {
    $DeployName = "star-$svc"
    $Status = kubectl --kubeconfig "$KubeConfig" rollout status deployment/$DeployName -n star-system --timeout 10s 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  $DeployName : OK" -ForegroundColor Green
    } else {
        Write-Host "  $DeployName : NOT READY ($Status)" -ForegroundColor Yellow
        $RolloutOk = $false
    }
}

Write-Host ""
if ($RolloutOk) {
    Write-Host "==== 后端更新完成 (8/8 rollout OK, tag=$NewTag) ====" -ForegroundColor Green
} else {
    Write-Host "==== 后端更新完成, 但部分服务未就绪 (tag=$NewTag), 建议查 kubectl describe ====" -ForegroundColor Yellow
    exit 2
}
