$ErrorActionPreference = 'Stop'
Set-Location D:\Star

$outCsv = 'D:\Star\_wt_audit\wt_audit.csv'
$report = @()

$wtLines = & git worktree list --porcelain
$i = 0
$current = @{ Path=''; Branch='' }
foreach ($line in $wtLines) {
    if ($line -match '^worktree (.+)$') {
        if ($i -gt 0) {
            $report += [pscustomobject]@{
                Idx = $i
                Path = $current.Path
                Branch = $current.Branch
            }
        }
        $i++
        $current = @{ Path = $Matches[1].Trim(); Branch = '' }
    } elseif ($line -match '^branch (.+)$') {
        $current.Branch = $Matches[1].Trim() -replace '^refs/heads/',''
    }
}
if ($i -gt 0) { $report += [pscustomobject]@{ Idx = $i; Path = $current.Path; Branch = $current.Branch } }

# Inspect each via JSON output
$final = @()
foreach ($r in $report) {
    $rPath = $r.Path
    $rBranch = $r.Branch
    if ([string]::IsNullOrEmpty($rBranch)) {
        $final += [pscustomobject]@{
            Idx = $r.Idx; Path = $rPath; Branch = '<detached>'
            LastSHA=''; LastSubject=''; LastDate=''
            AheadOfMain=''; BehindMain=''
            DirtyLines=''; UntrackedCount=''
            Remote=''; MergeRef=''; UnpushedVsOrigin=''
            SubjectsUnique=''
        }
        continue
    }
    $json = & pwsh -NoProfile -NonInteractive -File D:\Star\_wt_audit\inspect_wt.ps1 -Path $rPath -Branch $rBranch 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($json)) {
        $final += [pscustomobject]@{
            Idx = $r.Idx; Path = $rPath; Branch = $rBranch
            LastSHA='<error>'; LastSubject=''; LastDate=''
            AheadOfMain=''; BehindMain=''
            DirtyLines=''; UntrackedCount=''
            Remote=''; MergeRef=''; UnpushedVsOrigin=''
            SubjectsUnique=''
        }
        continue
    }
    try {
        $obj = $json | ConvertFrom-Json
    } catch {
        $final += [pscustomobject]@{
            Idx = $r.Idx; Path = $rPath; Branch = $rBranch
            LastSHA='<parse-error>'; LastSubject=$json.Substring(0, [Math]::Min(200, $json.Length))
            LastDate=''; AheadOfMain=''; BehindMain=''
            DirtyLines=''; UntrackedCount=''
            Remote=''; MergeRef=''; UnpushedVsOrigin=''
            SubjectsUnique=''
        }
        continue
    }
    $final += [pscustomobject]@{
        Idx = $r.Idx
        Path = $obj.Path
        Branch = $obj.Branch
        LastSHA = $obj.LastSHA
        LastSubject = $obj.LastSubject
        LastDate = $obj.LastDate
        AheadOfMain = $obj.AheadOfMain
        BehindMain = $obj.BehindMain
        DirtyLines = $obj.DirtyLines
        UntrackedCount = $obj.UntrackedCount
        Remote = $obj.Remote
        MergeRef = $obj.MergeRef
        UnpushedVsOrigin = $obj.UnpushedVsOrigin
        SubjectsUnique = $obj.SubjectsUnique
    }
}

$final | Export-Csv -Path $outCsv -NoTypeInformation -Encoding UTF8
Write-Output "Wrote $outCsv with $($final.Count) rows"
$final | Format-Table Idx, Branch, AheadOfMain, BehindMain, DirtyLines, UntrackedCount, LastSubject -AutoSize -Wrap
