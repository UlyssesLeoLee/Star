# WBS-W8 Multipass Inventory Parser Hardening — Pre-merge Self-review

**Review class**: author self-review; coordinator independent review is required before integration.
**Status**: READY FOR COORDINATOR REVIEW; W8 remains OPEN; stable release remains NO-GO.

## Scope checked

- `scripts/automation/multipass_k3s_gitops.ps1`
- `scripts/automation/test_multipass_k3s_gitops.ps1`
- The lane brief, dispatcher record, and phase report

The parser now rejects `{}`, JSON `null`, list payloads that are not arrays, unknown list-shape fields, malformed records, non-string or blank names/states, duplicate list names, and empty/null map records. It preserves an explicitly empty `{"list":[]}` and the existing non-empty map record form. The existing instance cap and exclusive lock implementation are unchanged.

The regression harness parses the script AST and loads only `ConvertFrom-MultipassInstanceInventory` and `Invoke-CreateVm`; it does not execute the deployment script entry point. It replaces inventory and command boundaries with local functions, supplies `{}` as invalid input, then checks that creation fails and the command stub remains uncalled.

## Verification evidence

- PowerShell 7.6.9 focused run: PASS, 29 assertions.
- CypherGraph Guardian: PASS, both source headers valid and source hashes current.
- Git whitespace check: PASS.
- No actual Multipass command or VM/K3s operation was run.

## Risks and disposition

1. The explicit empty `list` representation and retained map form have not been compared with live output from the user's supported Multipass version. Parser success is not host integration proof.
2. No concurrency, two-instance cap lifecycle, VM provisioning, K3s readiness, upgrade, or restore test ran.
3. The shared WBS is intentionally outside this lane manifest and is not updated here.

No blocking issue was found in the focused parser behavior. Accept this commit only after a coordinator independently reviews the exact diff and source commit, then integrates through the existing dev lane process. Do not use this self-review as release evidence or as grounds to close W8.
