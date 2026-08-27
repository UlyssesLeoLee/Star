param(
    [Parameter(Mandatory=$true)][string]$Path,
    [Parameter(Mandatory=$true)][string]$Branch,
    [string]$MainRef = 'main'
)
$ErrorActionPreference = 'Continue'

function Safe-Git {
    param([Parameter(ValueFromRemainingArguments=$true)][string[]]$GitArgs)
    try {
        $out = & git -C $Path @GitArgs 2>&1
        if ($LASTEXITCODE -ne 0) { return "<err: $LASTEXITCODE>" }
        return ($out -join "`n")
    } catch {
        return "<exception: $_>"
    }
}

# Status (porcelain)
$statusOut = Safe-Git @('status','--porcelain')
$dirtyLines = if ($statusOut -is [string]) { ($statusOut -split "`n" | Where-Object { $_.Trim() -ne '' }).Count } else { 0 }
$untracked = Safe-Git @('ls-files','--others','--exclude-standard')
$untrackedCount = if ($untracked -is [string] -and $untracked.Trim() -ne '') { ($untracked -split "`n" | Where-Object { $_.Trim() -ne '' }).Count } else { 0 }

# ahead/behind main
$ahead = Safe-Git @('rev-list','--count',"$MainRef..$Branch")
$behind = Safe-Git @('rev-list','--count',"$Branch..$MainRef")

# Last commit
$lastSha = Safe-Git @('rev-parse','--short','HEAD')
$lastSubj = Safe-Git @('log','-1','--format=%s')
$lastDate = Safe-Git @('log','-1','--format=%ci')

# Tracking remote (git config returns 1 if unset; treat as no-upstream, not error)
$remote = ''
$mergeRef = ''
$cfgRemote = & git -C $Path config --get "branch.$Branch.remote" 2>&1
if ($LASTEXITCODE -eq 0) { $remote = ($cfgRemote -join "`n").Trim() }
$cfgMerge = & git -C $Path config --get "branch.$Branch.merge" 2>&1
if ($LASTEXITCODE -eq 0) { $mergeRef = ($cfgMerge -join "`n").Trim() }

# Unpushed (ahead of upstream if any)
$unpushed = '<n/a>'
if ($remote -and $mergeRef) {
    $upstream = "$remote/$($mergeRef -replace '^refs/heads/','')"
    $unpushed = Safe-Git @('rev-list','--count',"$upstream..$Branch")
}

# Branch subject summary (commits unique to this branch)
$subjects = Safe-Git @('log',"$MainRef..$Branch",'--format=%s','--no-merges')
$subjList = if ($subjects -is [string]) { ($subjects -split "`n" | Select-Object -First 5) -join ' | ' } else { '' }

# Output as hashtable on stdout (so caller can parse)
[pscustomobject]@{
    Path = $Path
    Branch = $Branch
    LastSHA = $lastSha
    LastSubject = $lastSubj
    LastDate = $lastDate
    AheadOfMain = $ahead
    BehindMain = $behind
    DirtyLines = $dirtyLines
    UntrackedCount = $untrackedCount
    Remote = $remote
    MergeRef = $mergeRef
    UnpushedVsOrigin = $unpushed
    SubjectsUnique = $subjList
} | ConvertTo-Json -Compress
