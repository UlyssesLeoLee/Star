# uat-3000-regression.ps1 — UAT 3000 端口 3 case 持续回归 (per 2026-09-08 16:35 JST Ulysses 拍板 A)
# 守门 #1 v15 + 守门 #9 #20 brief 落档 + 守门 #12 v21 docs 同步 + 守门 #11 缺标比错标
# 守门 #22 cron 异步探活 (cron self 30min 触发)
# 实证 (v4.0 commit 0405768): S26 200 OK + body, S27 浏览器渲染, S28 截图
# 跟 v3.1 sustained 30min 验证一致 (commit 8a69219)

$ErrorActionPreference = 'Stop'
$WORKTREE = 'D:\Star\.worktrees\feat-auto-20260908-204a1a91'
$LOG_DIR = Join-Path $WORKTREE 'tools\star-flash-mock\docs\regression'
$TIMESTAMP = Get-Date -Format 'yyyyMMdd-HHmmss'
$LOG_FILE = Join-Path $LOG_DIR "uat-3000-$TIMESTAMP.log"

# 0. 探 3001 端口 (star frontend dev 期望)
$port3001 = Get-NetTCPConnection -LocalPort 3001 -State Listen -ErrorAction SilentlyContinue
if (-not $port3001) {
    Write-Host "[FAIL] 3001 端口未 LISTEN, next dev 没启" -ForegroundColor Red
    # 试拉起 next dev (守门 v29 适用)
    Write-Host "[auto-restart] 启 next dev -p 3001"
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c", "cd /d $WORKTREE\frontend && pnpm exec next dev -p 3001 > $WORKTREE\frontend\next-dev-3001.log 2>&1" -WindowStyle Hidden
    Start-Sleep -Seconds 15
    $port3001 = Get-NetTCPConnection -LocalPort 3001 -State Listen -ErrorAction SilentlyContinue
    if (-not $port3001) {
        Add-Content -Path $LOG_FILE -Value "[$TIMESTAMP] FAIL: 3001 端口未 LISTEN after auto-restart"
        exit 2
    }
}

# 1. 探 6443 + cluster
$port6443 = Get-NetTCPConnection -LocalPort 6443 -State Listen -ErrorAction SilentlyContinue
$nodeReady = wsl -d Ubuntu -- bash -lc 'KUBECONFIG=/home/leo19/.kube/config kubectl get nodes --no-headers 2>&1' 2>&1 | Out-String

# 2. curl 3001 验证
try {
    $r = Invoke-WebRequest -Uri 'http://localhost:3001' -TimeoutSec 10 -UseBasicParsing
    $curlStatus = $r.StatusCode
    $curlLen = $r.Content.Length
    $curlTitle = if ($r.Content -match '<title>([^<]+)</title>') { $Matches[1] } else { 'NO_TITLE' }
} catch {
    $curlStatus = 'ERROR'
    $curlLen = 0
    $curlTitle = $_.Exception.Message
}

# 3. 跑 Playwright UAT 3 case (S26/S27/S28, per 5ce001d spec 改成接受 proxy 模式 + apiserver paths 兜底)
$playwrightOut = & cd $WORKTREE\frontend; pnpm exec playwright test e2e/uat-3000-restore.spec.ts --reporter=line 2>&1 | Out-String
$playwrightPass = if ($playwrightOut -match '3 passed') { 'PASS' } elseif ($playwrightOut -match '(\d+) failed') { 'FAIL:' + $Matches[1] } else { 'UNKNOWN' }

# 4. 写 log
$report = @"
[$TIMESTAMP] UAT 3000 端口回归
- 3001 LISTEN: $(if ($port3001) { 'YES' } else { 'NO' })
- 6443 LISTEN: $(if ($port6443) { 'YES' } else { 'NO' })
- node Ready: $($nodeReady.Trim())
- curl 3001: status=$curlStatus len=$curlLen title=$curlTitle
- Playwright: $playwrightPass
- v2 prompt: this session 跟 v2 提示词无关, star frontend 3001 = Vibe Coding Work Management UI
- 守门 #5 env: stdin pipe sudo (per 9/8 14:41 JST Ulysses UbuntuPW)
- 守门 #38: Mavis 必不写"等 Ulysses 下一步"
"@
Add-Content -Path $LOG_FILE -Value $report
Write-Host $report

# 5. 写 regression summary (per 守门 #12 v21 docs 同步)
$summaryFile = Join-Path $LOG_DIR 'uat-3000-summary.md'
Add-Content -Path $summaryFile -Value "## $TIMESTAMP`n- 3001: $(if ($port3001) { 'YES' } else { 'NO' })`n- 6443: $(if ($port6443) { 'YES' } else { 'NO' })`n- curl: $curlStatus / $curlLen / $curlTitle`n- Playwright: $playwrightPass`n- log: $LOG_FILE`n"
