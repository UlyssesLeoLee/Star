# Brief: WBS-W8-MULTIPASS-INVENTORY-PARSER-HARDENING-20261107

**Agent**: worker
**Phase**: WBS-W8-MULTIPASS-INVENTORY-PARSER-HARDENING-20261107
**Created**: 2026-11-07 04:20:59

---

Task: Harden Multipass JSON inventory parsing in the registered W8 lane.

Ownership: You are the sole writer in this isolated worktree: D:\orcaWork\Star\wbs-w8-multipass-inventory-parser-20261107, branch codex/wbs-w8-multipass-inventory-parser-20261107, based on dev@996ae3ae. Do not modify any other worktree or branch.

Allowed paths are exactly:
- scripts/automation/multipass_k3s_gitops.ps1
- scripts/automation/test_multipass_k3s_gitops.ps1
- docs/briefs/WBS-W8-MULTIPASS-INVENTORY-PARSER-HARDENING-20261107.md
- docs/reports/PHASE-W8-MULTIPASS-INVENTORY-PARSER-HARDENING-20261107.md
- docs/reviews/WBS-W8-MULTIPASS-INVENTORY-PARSER-HARDENING-20261107.md
- docs/reports/WBS-DISPATCH-W8-MULTIPASS-INVENTORY-PARSER-HARDENING-20261107.jsonl

Problem: ConvertFrom-MultipassInstanceInventory currently treats an empty JSON object {} and empty/null records as a valid zero-instance inventory. Invoke-CreateVm uses this parser to enforce the two-instance cap; unknown inventory must fail closed.

Requirements:
1. Keep the existing maximum-two inventory and cross-process lock behavior unchanged.
2. Define and enforce explicit accepted inventory shapes. An explicit {"list":[]} may represent an empty inventory. Empty root {}, null, missing list payload, list:null, malformed/wrong-type records, blank name/state and duplicate names must fail closed. Preserve any currently supported non-empty map form only with valid non-empty records.
3. Ensure parser failures propagate to Invoke-CreateVm as a refusal before multipass launch.
4. Add focused synthetic parser tests for accepted empty-list and representative valid populated forms, plus invalid/ambiguous forms. Tests must not invoke multipass or alter VM/inventory state.
5. Update CypherGraph relationship comments in affected PowerShell files based on actual code, refresh source hashes, and run the Cypher Graph Guardian for changed files if available. The test script should accurately describe itself and use the repository's supported PowerShell runtime.
6. Write the phase report and review with exact commands/outcomes; label synthetic tests as parser-only and explicitly NOT evidence of live Multipass behavior. Keep W8 OPEN and stable release NO-GO. Record unavailable runtime/tests honestly.
7. Do not edit the shared WBS file, other WBS items, root D:\Star user content, any main/remote refs, Multipass inventory, VM, K3s, Docker artifacts, or other lane's paths. Do not execute multipass commands.
8. Commit only the six registered files after focused validation and diff checks. Do not rebase, merge, push, or clean up anything; integration is coordinator-owned.

Acceptance evidence: strict parser behavior covered by repeatable local tests; malformed inventory cannot become zero count; exactly {"list":[]} remains valid; no actual infrastructure side effects; review and report match actual diff and validation.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
