#!/usr/bin/env pwsh
#requires -Version 7.0
# @cypher schema=1 source_sha256=d4c0869999e63685eafc80d1bee6d45751277ce27cd6e2532f088b1f72678d85
# MERGE (self:File {path:"scripts/automation/test_multipass_k3s_gitops.ps1"})
# MERGE (suite:Symbol {id:"scripts/automation/test_multipass_k3s_gitops.ps1::Invoke-ParserTests",kind:"function"})
# MERGE (load_function:Symbol {id:"scripts/automation/test_multipass_k3s_gitops.ps1::Get-SourceFunctionText",kind:"function"})
# MERGE (assert_equal:Symbol {id:"scripts/automation/test_multipass_k3s_gitops.ps1::Assert-Equal",kind:"function"})
# MERGE (assert_throws:Symbol {id:"scripts/automation/test_multipass_k3s_gitops.ps1::Assert-Throws",kind:"function"})
# MERGE (pwsh_ast:ExternalService {id:"PowerShell.Language.Parser"})
# MERGE (scriptblock_type:Type {id:"System.Management.Automation.ScriptBlock"})
# MERGE (scriptblock_ast_type:Type {id:"System.Management.Automation.Language.ScriptBlockAst"})
# MERGE (function_ast_type:Type {id:"System.Management.Automation.Language.FunctionDefinitionAst"})
# MERGE (target_parser:Symbol {id:"scripts/automation/multipass_k3s_gitops.ps1::ConvertFrom-MultipassInstanceInventory",kind:"function"})
# MERGE (target_create:Symbol {id:"scripts/automation/multipass_k3s_gitops.ps1::Invoke-CreateVm",kind:"function"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (self)-[:DEFINES]->(load_function)
# MERGE (self)-[:DEFINES]->(assert_equal)
# MERGE (self)-[:DEFINES]->(assert_throws)
# MERGE (suite)-[:CALLS]->(pwsh_ast)
# MERGE (suite)-[:USES_TYPE]->(scriptblock_type)
# MERGE (load_function)-[:USES_TYPE]->(scriptblock_ast_type)
# MERGE (load_function)-[:USES_TYPE]->(function_ast_type)
# MERGE (load_function)-[:CALLS]->(scriptblock_type)
# MERGE (suite)-[:CALLS]->(load_function)
# MERGE (suite)-[:CALLS]->(assert_equal)
# MERGE (suite)-[:CALLS]->(assert_throws)
# MERGE (suite)-[:TESTS]->(target_parser)
# MERGE (suite)-[:TESTS]->(target_create)
# @endcypher

<#
.SYNOPSIS
  Focused parser-only tests for Multipass instance inventory validation.
.DESCRIPTION
  Loads only the parser and VM-create function definitions from the deployment
  script's PowerShell AST. Does not execute the script entry point or call a real
  Multipass executable. A local function stub confirms malformed inventory is
  rejected before any create-path Multipass call.
.EXAMPLE
  pwsh -NoProfile -File scripts/automation/test_multipass_k3s_gitops.ps1
##>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-SourceFunctionText {
  param(
    [Parameter(Mandatory)][System.Management.Automation.Language.ScriptBlockAst]$RootAst,
    [Parameter(Mandatory)][string]$FunctionName
  )

  $functionAst = $RootAst.FindAll({
    param($node)
    $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
      $node.Name -eq $FunctionName
  }, $true) | Select-Object -First 1
  if ($null -eq $functionAst) {
    throw "Function '$FunctionName' was not found in the deployment script AST."
  }
  return $functionAst.Extent.Text
}

function Assert-Equal {
  param(
    [Parameter(Mandatory)]$Expected,
    [Parameter(Mandatory)]$Actual,
    [Parameter(Mandatory)][string]$CaseName
  )
  if ($Expected -cne $Actual) {
    throw "$CaseName`: expected '$Expected', received '$Actual'."
  }
  $script:Passed++
}

function Assert-Throws {
  param(
    [Parameter(Mandatory)][scriptblock]$Action,
    [Parameter(Mandatory)][string]$CaseName
  )
  $threw = $false
  try {
    $null = & $Action
  } catch {
    $threw = $true
  }
  if (-not $threw) {
    throw "$CaseName`: expected the parser to reject the inventory."
  }
  $script:Passed++
}

function Invoke-ParserTests {
  $script:Passed = 0
  $sourcePath = Join-Path $PSScriptRoot 'multipass_k3s_gitops.ps1'
  $tokens = $null
  $parseErrors = $null
  $sourceAst = [System.Management.Automation.Language.Parser]::ParseFile(
    $sourcePath,
    [ref]$tokens,
    [ref]$parseErrors
  )
  if ($parseErrors.Count -gt 0) {
    throw "Deployment script has PowerShell parse errors: $($parseErrors[0].Message)"
  }

  . ([scriptblock]::Create((Get-SourceFunctionText -RootAst $sourceAst -FunctionName 'ConvertFrom-MultipassInstanceInventory')))
  . ([scriptblock]::Create((Get-SourceFunctionText -RootAst $sourceAst -FunctionName 'Invoke-CreateVm')))

  $emptyInventory = @(ConvertFrom-MultipassInstanceInventory -Json '{"list":[]}')
  Assert-Equal -Expected 0 -Actual $emptyInventory.Count -CaseName 'explicit empty list is accepted'

  $listInventory = @(ConvertFrom-MultipassInstanceInventory -Json '{"list":[{"name":"star-a","state":"Running"},{"name":"star-b","state":"Stopped"}]}')
  Assert-Equal -Expected 2 -Actual $listInventory.Count -CaseName 'list inventory count'
  Assert-Equal -Expected 'star-a' -Actual $listInventory[0].Name -CaseName 'list inventory name'
  Assert-Equal -Expected 'Stopped' -Actual $listInventory[1].State -CaseName 'list inventory state'

  $mapInventory = @(ConvertFrom-MultipassInstanceInventory -Json '{"star-map":[{"state":"Stopped"}]}')
  Assert-Equal -Expected 1 -Actual $mapInventory.Count -CaseName 'map inventory count'
  Assert-Equal -Expected 'star-map' -Actual $mapInventory[0].Name -CaseName 'map inventory name'

  $singleMapInventory = @(ConvertFrom-MultipassInstanceInventory -Json '{"star-single":{"state":"Suspended"}}')
  Assert-Equal -Expected 'Suspended' -Actual $singleMapInventory[0].State -CaseName 'single map record state'

  $invalidInventories = @(
    @{ Name = 'empty root object'; Json = '{}' },
    @{ Name = 'JSON null'; Json = 'null' },
    @{ Name = 'root array'; Json = '[]' },
    @{ Name = 'list null'; Json = '{"list":null}' },
    @{ Name = 'list object'; Json = '{"list":{}}' },
    @{ Name = 'list scalar'; Json = '{"list":"star-a"}' },
    @{ Name = 'unknown list property'; Json = '{"list":[],"instances":[]}' },
    @{ Name = 'null list item'; Json = '{"list":[null]}' },
    @{ Name = 'missing list item fields'; Json = '{"list":[{}]}' },
    @{ Name = 'non-string name'; Json = '{"list":[{"name":1,"state":"Running"}]}' },
    @{ Name = 'blank name'; Json = '{"list":[{"name":" ","state":"Running"}]}' },
    @{ Name = 'non-string state'; Json = '{"list":[{"name":"star-a","state":1}]}' },
    @{ Name = 'blank state'; Json = '{"list":[{"name":"star-a","state":""}]}' },
    @{ Name = 'duplicate names'; Json = '{"list":[{"name":"star-a","state":"Running"},{"name":"star-a","state":"Stopped"}]}' },
    @{ Name = 'empty map records'; Json = '{"star-a":[]}' },
    @{ Name = 'null map records'; Json = '{"star-a":null}' },
    @{ Name = 'map record without state'; Json = '{"star-a":[{}]}' },
    @{ Name = 'map record with blank state'; Json = '{"star-a":[{"state":" "}]}' },
    @{ Name = 'malformed JSON'; Json = '{"list":' }
  )
  foreach ($case in $invalidInventories) {
    Assert-Throws -Action { ConvertFrom-MultipassInstanceInventory -Json $case.Json } -CaseName $case.Name
  }

  $script:MultipassInvocationCount = 0
  $script:FailMessages = [System.Collections.Generic.List[string]]::new()
  $script:InstanceName = 'star-parser-test'
  $script:MaxTestPhaseInstances = 2
  $script:Cpus = 1
  $script:Memory = '1G'
  $script:Disk = '4G'
  $script:CloudInit = 'unused-cloud-init'
  function Enter-MultipassInstanceCapLock { return $null }
  function Get-MultipassInstanceInventory {
    ConvertFrom-MultipassInstanceInventory -Json '{}'
  }
  function Write-Stage { param([string]$Msg) }
  function Write-Done { param([string]$Msg) }
  function Write-Info { param([string]$Msg) }
  function Write-Fail { param([string]$Msg) $script:FailMessages.Add($Msg) }
  function multipass { $script:MultipassInvocationCount++; throw 'Unexpected host command invocation.' }

  $createResult = Invoke-CreateVm
  Assert-Equal -Expected $false -Actual $createResult -CaseName 'invalid inventory refuses VM creation'
  Assert-Equal -Expected 0 -Actual $script:MultipassInvocationCount -CaseName 'invalid inventory refuses before multipass launch'
  if ($script:FailMessages.Count -eq 0) {
    throw 'The VM create path did not record its fail-closed inventory refusal.'
  }
  $script:Passed++

  Write-Output "PASS: $script:Passed parser and fail-closed create-path assertions."
}

Invoke-ParserTests
