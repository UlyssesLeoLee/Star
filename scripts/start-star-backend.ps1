param(
    [string]$Context = $env:STAR_K3S_CONTEXT,
    [string]$Kubeconfig = $env:KUBECONFIG,
    [string]$Namespace = 'star-system'
)

$ErrorActionPreference = 'Stop'

$InitialDirectory = (Get-Location).Path
$ScriptDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDirectory
if (-not [string]::IsNullOrWhiteSpace($Kubeconfig) -and -not [IO.Path]::IsPathRooted($Kubeconfig)) {
    $Kubeconfig = Join-Path $InitialDirectory $Kubeconfig
}
Set-Location $RepoRoot

Write-Host ''
Write-Host '==== Deploy Star Canvas backend services ====' -ForegroundColor Cyan
Write-Host "Repo      : $RepoRoot"
Write-Host "Namespace : $Namespace"
Write-Host "Time      : $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host ''

if ([string]::IsNullOrWhiteSpace($Context)) {
    Write-Host 'ERROR: choose the target K3s context explicitly.' -ForegroundColor Red
    Write-Host 'Set STAR_K3S_CONTEXT or pass -Context <kubeconfig-context>.' -ForegroundColor Yellow
    exit 1
}
if ([string]::IsNullOrWhiteSpace($Kubeconfig)) {
    Write-Host 'ERROR: provide the kubeconfig file explicitly with -Kubeconfig or KUBECONFIG.' -ForegroundColor Red
    exit 1
}
if (-not (Test-Path -LiteralPath $Kubeconfig -PathType Leaf)) {
    Write-Host "ERROR: kubeconfig file not found: $Kubeconfig" -ForegroundColor Red
    exit 1
}
$Kubeconfig = (Resolve-Path -LiteralPath $Kubeconfig).Path

$KubectlCommand = Get-Command kubectl -ErrorAction SilentlyContinue
if (-not $KubectlCommand) {
    Write-Host 'ERROR: kubectl was not found in PATH.' -ForegroundColor Red
    exit 1
}
$Kubectl = $KubectlCommand.Source
$KubectlArguments = @('--kubeconfig', $Kubeconfig, '--context', $Context)

Write-Host "kubectl  : $Kubectl"
Write-Host "Kubeconfig: $Kubeconfig"
Write-Host "Context  : $Context"

Write-Host ''
Write-Host '[1/4] Check the selected K3s cluster ...' -ForegroundColor Cyan
$ClusterInfo = & $Kubectl @KubectlArguments cluster-info 2>&1 | Out-String
if ($LASTEXITCODE -ne 0) {
    Write-Host 'ERROR: kubectl could not connect to the selected K3s context.' -ForegroundColor Red
    Write-Host $ClusterInfo -ForegroundColor Red
    exit 1
}
Write-Host (($ClusterInfo -split "`n" | Select-Object -First 1).Trim()) -ForegroundColor Green

Write-Host ''
Write-Host '[2/4] Check namespace and private-image pull secret ...' -ForegroundColor Cyan
& $Kubectl @KubectlArguments get namespace $Namespace --no-headers *> $null
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: namespace '$Namespace' must exist before deployment." -ForegroundColor Red
    exit 1
}
& $Kubectl @KubectlArguments --namespace $Namespace get secret ghcr-pull-secret -o name *> $null
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: pre-create ghcr-pull-secret in namespace '$Namespace'." -ForegroundColor Red
    Write-Host 'The workflow or deployment script never creates or prints registry credentials.' -ForegroundColor Yellow
    exit 1
}
& $Kubectl @KubectlArguments get crd servicemonitors.monitoring.coreos.com -o name *> $null
if ($LASTEXITCODE -ne 0) {
    Write-Host 'ERROR: the Prometheus Operator ServiceMonitor CRD is required by this manifest.' -ForegroundColor Red
    Write-Host 'Install the CRD before applying; no cluster resources were changed by this script.' -ForegroundColor Yellow
    exit 1
}
Write-Host 'Namespace and pull-secret reference are ready; secret contents were not read.' -ForegroundColor Green

$Manifest = Join-Path $RepoRoot 'deploy/star-canvas-services-k3s.yaml'
if (-not (Test-Path -LiteralPath $Manifest)) {
    Write-Host "ERROR: manifest not found: $Manifest" -ForegroundColor Red
    exit 1
}

Write-Host ''
Write-Host '[3/4] Apply the Star Canvas manifest ...' -ForegroundColor Cyan
& $Kubectl @KubectlArguments apply -f $Manifest
if ($LASTEXITCODE -ne 0) {
    Write-Host 'ERROR: kubectl apply failed.' -ForegroundColor Red
    exit 1
}

Write-Host ''
Write-Host '[4/4] Wait for four deployments ...' -ForegroundColor Cyan
$Deployments = @('canvas-engine', 'domain-canvas', 'canvas-realtime', 'envoy-edge')
$RolloutFailed = $false
foreach ($Deployment in $Deployments) {
    Write-Host "  rollout $Deployment ..." -ForegroundColor Cyan -NoNewline
    $RolloutOutput = & $Kubectl @KubectlArguments --namespace $Namespace rollout status "deployment/$Deployment" --timeout=180s 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0) {
        Write-Host ' OK' -ForegroundColor Green
    } else {
        $RolloutFailed = $true
        Write-Host ' FAILED' -ForegroundColor Red
        Write-Host $RolloutOutput -ForegroundColor Red
    }
}
if ($RolloutFailed) {
    Write-Host 'One or more deployments did not become ready.' -ForegroundColor Red
    exit 1
}

$LoadBalancerHost = & $Kubectl @KubectlArguments --namespace $Namespace get service envoy-edge -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>$null
if ($LASTEXITCODE -eq 0 -and $LoadBalancerHost) {
    Write-Host "Envoy LoadBalancer: $LoadBalancerHost" -ForegroundColor Green
} else {
    Write-Host 'Envoy LoadBalancer hostname is not assigned; NodePort 30080 remains configured.' -ForegroundColor Yellow
}

Write-Host ''
Write-Host 'Canvas backend health endpoints:' -ForegroundColor Cyan
Write-Host "  kubectl --kubeconfig `"$Kubeconfig`" --context $Context --namespace $Namespace port-forward service/canvas-engine 18080:8080"
Write-Host "  kubectl --kubeconfig `"$Kubeconfig`" --context $Context --namespace $Namespace port-forward service/domain-canvas 18081:8081"
Write-Host "  kubectl --kubeconfig `"$Kubeconfig`" --context $Context --namespace $Namespace port-forward service/canvas-realtime 18082:8082"
Write-Host ''
Write-Host "Four Service objects are managed by this manifest: three Canvas backends and Envoy edge." -ForegroundColor Green
Write-Host 'star-api-rest remains on its separate deployment path.' -ForegroundColor Gray

Write-Host ''
Write-Host 'Watching backend pod status every 60 seconds. Press Ctrl+C to stop.' -ForegroundColor Cyan
try {
    while ($true) {
        Start-Sleep -Seconds 60
        $Phases = & $Kubectl @KubectlArguments --namespace $Namespace get pods -l 'app in (canvas-engine,domain-canvas,canvas-realtime,envoy-edge)' -o jsonpath='{.items[*].status.phase}' 2>$null
        if ($LASTEXITCODE -ne 0 -or -not $Phases -or $Phases -match 'Pending|Failed|Unknown') {
            Write-Host "[$(Get-Date -Format 'HH:mm:ss')] Warning: one or more Star Canvas pods are not healthy: $Phases" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host ''
    Write-Host 'Stopped watching pod status.' -ForegroundColor Yellow
    exit 0
}
