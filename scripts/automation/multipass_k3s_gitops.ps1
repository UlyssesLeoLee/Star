#!/usr/bin/env pwsh
#requires -Version 7.0
<#
.SYNOPSIS
  Multipass (hcs 驱动) 上建 k3s + Argo CD 三件套本机集群的自动化脚本。

.DESCRIPTION
  宿主约束 (2026-10-05 实测):
    - Windows 11 家庭版 (EditionID=CoreCountrySpecific), 完整 Hyper-V 角色未安装
      (vmms 服务不存在)。因此 Multipass 必须用 1.17 的 hcs 驱动 —— Canonical
      release notes 明确 "works on all editions of Windows, including Windows Home"。
    - 本脚本假定 Multipass 已安装 (1.17.0-rc1+) 且 local.driver=hcs。
    - 磁盘余量须 >= VM 磁盘大小, 本机 C: 实测 35.9 GB 空闲。

  脚本做四件事 (可分阶段跑, 幂等):
    Stage 0  预检: multipass 存在 / driver / 磁盘余量 / cloud-init 文件可读
    Stage 1  创建 VM (Ubuntu 24.04 LTS, 4C/8G/28G) 并注入 cloud-init
    Stage 2  mount 仓库到 VM 的 /home/ubuntu/star
    Stage 3  取回 kubeconfig 到 Windows 侧, 使 kubectl / Argo CD CLI 可直连

  守门 #19 v19: 本脚本落 scripts/automation/, commit message 须引用相对路径。
  守门 #11 缺标比错标: 每个阶段输出 DONE/FAIL 标记, 不以"命令返回 0"当作成功。
  守门 #5 v2: 本脚本不含任何 secret。kubeconfig 属凭据 —— 写到仓库外目录,
             路径经 -KubeconfigOut 参数显式指定, 默认落 $env:LOCALAPPDATA\Star\kubeconfig。

.PARAMETER Stage
  要执行的阶段: 0(预检) / 1(建 VM) / all(0→3)

.PARAMETER InstanceName
  VM 名称。默认 star-k3s。

.PARAMETER Cpus
  vCPU 数。默认 4。

.PARAMETER Memory
  内存。默认 8G。本机 31.8 GB 物理内存, 8G 给 k3s + 三件套绰绰有余。

.PARAMETER Disk
  VM 磁盘。默认 28G —— 刻意低于本机 35.9 GB 余量, 留 7.9 GB 给 Windows 自身。
  守门 #3 显式权衡: k3s 本体 ~1.5G + 三件套镜像 ~4G + 业务镜像 (star-api-rest
  为 Rust debug/release 二进制, ~200-500M) + containerd 双份解压余量。

.PARAMETER KubeconfigOut
  kubeconfig 落地路径。默认 $env:LOCALAPPDATA\Star\kubeconfig (仓库外)。

.EXAMPLE
  pwsh -File scripts/automation/multipass_k3s_gitops.ps1 -Stage 0
  pwsh -File scripts/automation/multipass_k3s_gitops.ps1 -Stage all
#>
[CmdletBinding()]
param(
  [ValidateSet('0', '1', 'all')]
  [string]$Stage = '0',
  [string]$InstanceName = 'star-k3s',
  [int]$Cpus = 4,
  [string]$Memory = '8G',
  [string]$Disk = '28G',
  [string]$KubeconfigOut = (Join-Path $env:LOCALAPPDATA 'Star\kubeconfig')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# ---------- 仓库根 (脚本在 <repo>/scripts/automation/) ----------
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$CloudInit = Join-Path $RepoRoot 'deploy\multipass\cloud-init-k3s-gitops.yaml'

# ---------- 期望值 (钉死, 供 Stage 0 比对; 变更须同步改 ADR-0054) ----------
$ExpectedDriver = 'hcs'
$ExpectedMinVersion = '1.17.0'
$LogFile = '/var/log/star-cloud-init.log'
$CompleteFlag = '/var/log/star-cloud-init.complete'

function Write-Stage { param([string]$Msg) Write-Host "==> $Msg" }
function Write-Done  { param([string]$Msg) Write-Host "    DONE: $Msg" -ForegroundColor Green }
function Write-Fail  { param([string]$Msg) Write-Host "    FAIL: $Msg" -ForegroundColor Red }
function Write-Info  { param([string]$Msg) Write-Host "    $Msg" -ForegroundColor DarkGray }

# ============================================================================
# Stage 0 — 预检
# ============================================================================
function Invoke-Preflight {
  Write-Stage 'Stage 0: preflight'

  # 0.1 multipass CLI 存在?
  $mp = Get-Command multipass -ErrorAction SilentlyContinue
  if (-not $mp) {
    Write-Fail 'multipass not on PATH.'
    Write-Info  'Install Multipass 1.17.0-rc1+ first (see deploy/multipass/README.md for the msiexec line).'
    return $false
  }
  Write-Done "multipass found: $($mp.Source)"

  # 0.2 daemon 可达?  (multipass wait-ready 是 1.17 新增, 正合脚本化场景)
  if (Get-Command multipass -ErrorAction SilentlyContinue) {
    & multipass wait-ready 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
      Write-Fail 'multipass daemon not ready (wait-ready failed).'
      Write-Info  'Start the "Multipass" service: Start-Service Multipass (elevated).'
      return $false
    }
  }
  Write-Done 'multipass daemon ready'

  # 0.3 版本 >= 1.17.0  (hcs 驱动是 1.17 才有的)
  $verRaw = (& multipass version 2>&1) -join ' '
  Write-Info "version: $verRaw"
  $verMatch = [regex]::Match($verRaw, '(\d+\.\d+\.\d+)')
  if (-not $verMatch.Success) {
    Write-Fail "cannot parse multipass version from: $verRaw"
    return $false
  }
  $ver = [version]$verMatch.Groups[1].Value
  if ($ver -lt [version]$ExpectedMinVersion) {
    Write-Fail "multipass $ver < $ExpectedMinVersion — the hcs driver requires 1.17+."
    return $false
  }
  Write-Done "version $ver >= $ExpectedMinVersion"

  # 0.4 driver == hcs  (家庭版无 Hyper-V 角色, hyperv driver 必然失败)
  $driver = (& multipass get local.driver 2>&1) -join ''.Trim()
  if ($driver -ne $ExpectedDriver) {
    Write-Fail "local.driver = '$driver', expected '$ExpectedDriver'."
    Write-Info  "This host is Windows 11 Home (no Hyper-V role). Run: multipass set local.driver=$ExpectedDriver"
    return $false
  }
  Write-Done "driver = $driver"

  # 0.5 磁盘余量够 VM 磁盘 + 缓冲?
  $diskGB = [int]($Disk -replace '[^0-9]', '')
  $freeGB = [math]::Round((Get-PSDrive C).Free / 1GB, 1)
  # Multipass 镜像默认落 %LOCALAPPDATA%\Multipass (C: 盘), 故查 C:
  if ($freeGB -lt ($diskGB + 5)) {
    Write-Fail "C: free $freeGB GB < VM disk $diskGB GB + 5 GB buffer."
    return $false
  }
  Write-Done "C: free $freeGB GB >= $diskGB GB + 5 GB"

  # 0.6 cloud-init 文件可读且是合法 YAML?
  if (-not (Test-Path $CloudInit)) {
    Write-Fail "cloud-init not found: $CloudInit"
    return $false
  }
  $ciLen = (Get-Item $CloudInit).Length
  if ($ciLen -lt 100) {
    Write-Fail "cloud-init suspiciously small ($ciLen bytes) — truncated?"
    return $false
  }
  Write-Done "cloud-init readable ($ciLen bytes)"

  Write-Host ''
  Write-Host 'Preflight PASSED.' -ForegroundColor Green
  return $true
}

# ============================================================================
# Stage 1 — 建 VM
# ============================================================================
function Invoke-CreateVm {
  Write-Stage "Stage 1: create instance '$InstanceName'"

  # 幂等: 已存在则跳过创建, 直接报出状态
  $existing = (& multipass list 2>&1) -join "`n"
  if ($existing -match [regex]::Escape($InstanceName)) {
    Write-Info "instance '$InstanceName' already exists — skipping launch (idempotent)"
  } else {
    $args = @(
      'launch', '24.04',
      '--name',    $InstanceName,
      '--cpus',    "$Cpus",
      '--memory',  $Memory,
      '--disk',    $Disk,
      '--cloud-init', $CloudInit
    )
    Write-Info "multipass $($args -join ' ')"
    & multipass @args 2>&1 | ForEach-Object { Write-Info $_ }
    if ($LASTEXITCODE -ne 0) {
      Write-Fail "multipass launch failed (exit $LASTEXITCODE)"
      return $false
    }
  }
  Write-Done 'instance exists'

  # cloud-init 引导是长任务 (拉三件套镜像通常 3-8 分钟)。轮询完成标记,
  # 不用"launch 返回 0"当成功 —— 那只代表 VM 起来了, 不代表 k3s 就绪。
  Write-Stage 'waiting for cloud-init bootstrap (k3s + cert-manager + 3 components)'
  $deadline = (Get-Date).AddMinutes(25)
  $lastReport = Get-Date
  while ((Get-Date) -lt $deadline) {
    & multipass exec $InstanceName -- test -f $CompleteFlag *> $null
    if ($LASTEXITCODE -eq 0) {
      Write-Done 'cloud-init reported completion'
      break
    }
    if (((Get-Date) - $lastReport).TotalSeconds -ge 60) {
      $lastReport = Get-Date
      # 打印日志尾部, 避免"卡住"变成黑箱
      $tail = (& multipass exec $InstanceName -- sudo tail -n 5 $LogFile 2>&1) -join ' | '
      Write-Info "still bootstrapping... last log: $tail"
    }
    Start-Sleep -Seconds 10
  }
  if ((Get-Date) -ge $deadline) {
    Write-Fail 'cloud-init did not complete within 25 minutes.'
    Write-Info  "Inspect: multipass exec $InstanceName -- sudo cat $LogFile"
    return $false
  }

  # 逐段核对 DONE / FAIL 标记 —— cloud-init 全部阶段"尝试过"不等于全部成功。
  # per 守门 #11: 缺标比错标, 这里显式统计 FAIL 而不是假设。
  Write-Stage 'verifying per-stage markers in cloud-init log'
  $log = (& multipass exec $InstanceName -- sudo cat $LogFile 2>&1) -join "`n"
  $done = ([regex]::Matches($log, 'DONE:([a-z0-9-]+)')).Groups | Where-Object { $_.Name -eq '1' } | ForEach-Object { $_.Value }
  $fail = ([regex]::Matches($log, 'FAIL:([a-z0-9-]+)')).Groups | Where-Object { $_.Name -eq '1' } | ForEach-Object { $_.Value }
  $skip = ([regex]::Matches($log, 'SKIP:([a-z0-9-]+)')).Groups | Where-Object { $_.Name -eq '1' } | ForEach-Object { $_.Value }

  Write-Info "DONE: $($done -join ', ')"
  if ($skip) { Write-Info "SKIP: $($skip -join ', ')" }
  if ($fail) {
    Write-Fail "failed stages: $($fail -join ', ')"
    Write-Info  "Full log: multipass exec $InstanceName -- sudo cat $LogFile"
    return $false
  }
  Write-Done 'no FAIL markers'

  return $true
}

# ============================================================================
# Stage 2 — mount 仓库
# ============================================================================
function Invoke-MountRepo {
  # ${InstanceName} 必须用花括号: 裸 $InstanceName: 会被 PowerShell 解析成
  # 变量名 "InstanceName:" (含冒号) 而报"变量引用无效"。
  # per 2026-10-05 实测: 该 bug 由 [Parser]::ParseFile 的 AST 校验抓出,
  # 语法校验不做就实跑必炸。
  Write-Stage "Stage 2: mount repo -> ${InstanceName}:/home/ubuntu/star"
  # Windows 上 mount 默认关闭, 首次需显式开启 (需管理员一次)。
  # 失败不阻断: cloud-init 阶段 8 会 SKIP 仓库 manifest, 后续可手动 kubectl apply。
  & multipass set local.privileged-mounts=true 2>&1 | Out-Null
  & multipass mount $RepoRoot "${InstanceName}:/home/ubuntu/star" 2>&1 | ForEach-Object { Write-Info $_ }
  if ($LASTEXITCODE -ne 0) {
    Write-Fail "mount failed. Run elevated: multipass set local.privileged-mounts=true"
    return $false
  }
  Write-Done 'repo mounted'
  return $true
}

# ============================================================================
# Stage 3 — 取回 kubeconfig
# ============================================================================
function Invoke-ExportKubeconfig {
  Write-Stage 'Stage 3: export kubeconfig'
  # k3s 用了 --write-kubeconfig-mode 644, 故 VM 内非 root 也可读。
  $yaml = (& multipass exec $InstanceName -- cat /etc/rancher/k3s/k3s.yaml 2>&1) -join "`n"
  if ($yaml -notmatch 'apiVersion:') {
    Write-Fail 'kubeconfig read failed (no apiVersion in output).'
    return $false
  }
  # server 指向 VM 内网 IP, Windows 侧靠 Multipass 的 NAT 转发可达;
  # 但 Multipass 的 hcs 驱动不做端口转发, 需确认 6443 可达。
  $outDir = Split-Path -Parent $KubeconfigOut
  if (-not (Test-Path $outDir)) { New-Item -ItemType Directory -Force -Path $outDir | Out-Null }
  # 凭据落仓库外目录 (守门 #5 v2)
  [System.IO.File]::WriteAllText($KubeconfigOut, $yaml)
  Write-Done "kubeconfig -> $KubeconfigOut"
  Write-Info "use it with: kubectl --kubeconfig `"$KubeconfigOut`" get nodes"
  return $true
}

# ============================================================================
# 主流程
# ============================================================================
Write-Host '=== Multipass k3s + GitOps bootstrap ===' -ForegroundColor Cyan
Write-Info "repo root : $RepoRoot"
Write-Info "cloud-init: $CloudInit"
Write-Info "target    : ${Cpus}C / ${Memory} / ${Disk} on $InstanceName"
Write-Host ''

if (-not (Invoke-Preflight)) {
  Write-Host ''
  Write-Host 'PREFLIGHT FAILED — see FAIL lines above.' -ForegroundColor Red
  exit 2
}

if ($Stage -eq '0') {
  Write-Host ''
  Write-Host 'Stage 0 only (--Stage 0). Nothing changed.' -ForegroundColor Yellow
  exit 0
}

$ok = $true
if (-not (Invoke-CreateVm))   { $ok = $false }
if ($ok -and -not (Invoke-MountRepo))      { Write-Info 'continuing despite mount failure'; }
if ($ok -and -not (Invoke-ExportKubeconfig)) { $ok = $false }

Write-Host ''
if ($ok) {
  Write-Host '=== BOOTSTRAP COMPLETE ===' -ForegroundColor Green
  exit 0
} else {
  Write-Host '=== BOOTSTRAP FAILED (see FAIL lines) ===' -ForegroundColor Red
  exit 1
}
