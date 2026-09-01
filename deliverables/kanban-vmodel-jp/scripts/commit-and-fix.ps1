$ErrorActionPreference = 'Stop'
$wtBase = 'D:\Star\.worktrees\kanban-vmodel-jp'
$indDir = 'deliverables/kanban-vmodel-jp/data/industries'

# A. amend already-committed (file content changed)
$amendWts = @('wt-p5-impl','wt-p6-test','wt-p62-it','wt-p63-st')
foreach ($w in $amendWts) {
  $base = Join-Path $wtBase $w
  Set-Location $base
  $changes = git status --short $indDir
  if ($changes) {
    git add $indDir 2>&1 | Out-Null
    git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit --amend --no-edit 2>&1 | Out-Null
    $h = git rev-parse --short HEAD
    Write-Host ("amend {0,-16} -> {1}" -f $w, $h)
  } else {
    Write-Host ("{0,-16}: no changes" -f $w)
  }
}
Write-Host '---'
# B. commit uncommitted
$commitWts = @(
  @{ w='wt-p1-upstream'; msg='feat(kanban-vmodel-jp): P1 4 行业预设任务 (超上流工程 · 金融/公共/EC/組込 · 12 task)' },
  @{ w='wt-p3-basic';    msg='feat(kanban-vmodel-jp): P3 4 行业预设任务 (基本設計 · 金融/公共/EC/組込 · 12 task)' },
  @{ w='wt-p4-detail';   msg='feat(kanban-vmodel-jp): P4 4 行业预设任务 (詳細設計 · 金融/公共/EC/組込 · 12 task)' },
  @{ w='wt-p61-ut';      msg='feat(kanban-vmodel-jp): P6.1 4 行业単体試験预设任务 (金融/公共/EC/組込 · 12 task)' },
  @{ w='wt-p9-close';    msg='feat(kanban-vmodel-jp): P9 4 行业终結预设任务 (金融/公共/EC/組込 · 8 task)' }
)
foreach ($info in $commitWts) {
  $base = Join-Path $wtBase $info.w
  Set-Location $base
  git add $indDir 2>&1 | Out-Null
  git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m $info.msg 2>&1 | Out-Null
  $h = git rev-parse --short HEAD
  Write-Host ("commit {0,-16} -> {1}" -f $info.w, $h)
}
Set-Location D:\Star
Write-Host '---'
Write-Host '=== wt status ==='
Get-ChildItem $wtBase -Directory | ForEach-Object {
  Set-Location $_.FullName
  $branch = git rev-parse --abbrev-ref HEAD
  $head = git rev-parse --short HEAD
  $log = git log -1 --oneline
  Write-Host ("{0,-16} {1,-26} {2,-8} {3}" -f $_.Name, $branch, $head, $log)
}
Set-Location D:\Star
