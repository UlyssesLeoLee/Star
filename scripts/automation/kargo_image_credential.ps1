<#
.SYNOPSIS
    给 Kargo Project 注入容器镜像仓库读凭据 (Secret)。

.DESCRIPTION
    Kargo 不通过 CRD 字段引用凭据，而是**按约定发现**特殊标注的 Secret
    (per Kargo 官方 docs: "Managing Credentials", v1.x):

        kind: Secret
        metadata:
          labels:
            kargo.akuity.io/cred-type: image
        stringData:
          repoURL: <registry repo>
          username: <user>
          password: <PAT>

    Secret 必须放在 **Project 自己的 namespace** (本仓 = `star`), 名字任意 ——
    Kargo 按「标签 + repoURL 匹配」发现, 精确匹配优先, 其后才按正则匹配。

    凭据形态 (本仓实测, 2026-10-05):
      - GHCR 只接受 GitHub **PAT**, 不接受 `gho_` OAuth app token。
        实证: gh keyring 里的 gho_ 与一把伪造的垃圾 Bearer 串对同一公开仓库
        返回**完全相同**的结果 —— 真实凭据与垃圾不可区分 = 没被当作凭据评估。
      - 注册表 API 的认证流程是 `Basic(user:PAT)` 换 registry token, 再作 Bearer;
        **不能**把 PAT 直接当 Bearer 塞进 /v2/<repo>/tags/list。
        (本脚本早期版本犯过这个错, 导致「所有仓库都 403」的假结论;
         对照组 `fluxcd/source-controller` 匿名可拉 OK 才把它揪出来。)

.PARAMETER RepoUrl
    目标仓库全 URL, 形如 `ghcr.io/sayatelier/star-canvas-engine`。
    可传多个; 同一把 PAT 覆盖多个仓库时用 -UseRegex 走 `repoURLIsRegex`。

.PARAMETER Username
    认证用户名。GHCR 下 PAT 场景通常无关紧要, 但仍按 docker login 惯例传入。

.PARAMETER Project
    Kargo Project 名 (决定 Secret 落在哪个 namespace)。默认 `star`。

.PARAMETER UseRegex
    把 RepoUrl 当作正则 (Secret 里同时写 `repoURLIsRegex: "true"`)。
    仅在**一把凭据覆盖多个仓库**时使用; 精确匹配总是优先于正则匹配。

.PARAMETER SkipVerify
    跳过建之前的验权。**默认必须验**; 仅在你明知凭据有效且需要离线排障时用。

.NOTES
    守门 #5: 本脚本**从不打印** PAT —— 只读环境变量, 只在内存中流转,
    构造 base64 时不落盘、不回显。命令行里也不接受明文 PAT 参数
    (会进 shell history)。
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string[]] $RepoUrl,
    [string] $Username = 'UlyssesLeoLee',
    [string] $Project = 'star',
    [switch] $UseRegex,
    [switch] $SkipVerify
)

$ErrorActionPreference = 'Stop'

function Get-Pat {
    foreach ($n in 'GHCR_PAT', 'GHCR_TOKEN', 'GITHUB_TOKEN') {
        foreach ($scope in 'Process', 'User', 'Machine') {
            $v = [Environment]::GetEnvironmentVariable($n, $scope)
            if (-not [string]::IsNullOrWhiteSpace($v)) { return $v }
        }
    }
    throw "未找到 PAT。请先设置环境变量 GHCR_PAT (不要作为命令行参数传入, 会进 shell history)。"
}

# GHCR 注册表 API 的正确流程: Basic(user:PAT) -> token endpoint -> Bearer
function Test-RepoAccess {
    param([string] $Pat, [string] $User, [string] $Repo)
    $basic = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("${User}:${Pat}"))
    $scope = [uri]::EscapeDataString("repository:${Repo}:pull")
    try {
        $tok = Invoke-RestMethod -Uri "https://ghcr.io/token?scope=${scope}&service=ghcr.io" `
            -Headers @{ Authorization = "Basic $basic" } -TimeoutSec 20
        if ([string]::IsNullOrWhiteSpace($tok.token)) { return @{ ok = $false; code = 'no-token' } }
        $r = Invoke-RestMethod -Uri "https://ghcr.io/v2/${Repo}/tags/list" `
            -Headers @{ Authorization = "Bearer $($tok.token)" } -TimeoutSec 20
        $tags = @($r.tags)
        return @{ ok = $true; code = 'ok'; count = $tags.Count; tags = $tags }
    }
    catch {
        $c = $_.Exception.Response.StatusCode.value__
        if (-not $c) { $c = 'no-response' }
        return @{ ok = $false; code = $c }
    }
}

$pat = Get-Pat
Write-Host "[1/3] 已从环境变量取得凭据 (长度 $($pat.Length), 值不回显)"

# ── 建之前的验权 ───────────────────────────────────────────────────────────
# fail closed: 一把读不到目标的凭据建进去, 只会让 Warehouse 继续静默失败 ——
# 那是"配置看起来做了"而非"配置生效", 正是本仓守门 #30 要防的失效模式。
if (-not $SkipVerify) {
    Write-Host "[2/3] 验权 (对照组 + 目标仓库)"
    $ctl = Test-RepoAccess -Pat $pat -User $Username -Repo 'fluxcd/source-controller'
    if (-not $ctl.ok) {
        throw "对照组失败: 凭据连公开仓库 fluxcd/source-controller 都读不了 (HTTP $($ctl.code))。凭据本身无效, 不要继续。"
    }
    Write-Host "      对照组 OK (公开仓库可读) -> 凭据本身有效"

    $unreadable = @()
    foreach ($r in $RepoUrl) {
        $res = Test-RepoAccess -Pat $pat -User $Username -Repo $r
        if ($res.ok) {
            $sem = @($res.tags | Where-Object { $_ -match '^\d+\.\d+\.\d+$' })
            Write-Host "      OK   $r  tags=$($res.count)  semver3=$($sem.Count)"
        }
        else {
            Write-Host "      FAIL $r  HTTP $($res.code)"
            $unreadable += $r
        }
    }
    if ($unreadable.Count -gt 0) {
        Write-Host ""
        Write-Warning "以下仓库用该凭据读不了, 凭据覆盖不全: $($unreadable -join ', ')"
        Write-Warning "Kargo 会为读不了的仓库持续报 UNAUTHORIZED/DENIED 且 apply 仍然 exit 0 (静默失效)。"
        Write-Warning "确认后再决定是否继续: -SkipVerify 可跳过本次阻断。"
        throw "验权未全数通过, 已中止 (未创建 Secret)。"
    }
}
else {
    Write-Host "[2/3] 已跳过验权 (-SkipVerify)"
}

# ── 构造 Secret ────────────────────────────────────────────────────────────
$repoValue = if ($UseRegex) { ($RepoUrl -join '|') } else { $RepoUrl[0] }
$secretName = if ($UseRegex) { 'kargo-image-ghcr-regex' } else { 'kargo-image-ghcr' }

$json = @{
    apiVersion = 'v1'
    kind       = 'Secret'
    metadata   = @{
        name      = $secretName
        namespace = $Project
        labels    = @{ 'kargo.akuity.io/cred-type' = 'image' }
    }
    stringData = @{
        repoURL  = $repoValue
        username = $Username
        password = $pat
    }
}
if ($UseRegex) { $json.stringData['repoURLIsRegex'] = 'true' }

Write-Host "[3/3] 创建 Secret '$secretName' (namespace: $Project, cred-type: image)"
Write-Host "      repoURL = $repoValue"
Write-Host "      注意: 含明文 PAT 的 JSON 在管道中传给 multipass, 不落盘、不进 git。"

$json | & multipass exec $env:KARGO_VM -- bash -c "sudo k3s kubectl apply -f -" 2>&1 | ForEach-Object { "      $_" }

Write-Host ""
Write-Host "完成。验证 Kargo 是否真的用上了它 —— 必须读控制器日志, 不能只看 apply 返回码:"
Write-Host "  multipass exec $($env:KARGO_VM) -- bash -c 'sudo k3s kubectl logs -n kargo deploy/kargo-controller --tail=100 | grep -i warehouse'"
Write-Host "Warehouse .status 出现 phase/freightCount 才算生效; status 全空 = 还没 reconcile 过。"
