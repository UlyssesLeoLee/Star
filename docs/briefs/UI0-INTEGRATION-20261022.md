# Brief: UI0-INTEGRATION-20261022

**Agent**: worker
**Phase**: UI0_INTEGRATION_20261022
**Created**: 2026-10-22 15:23:51

---

## Goal
Integrate the audited UI-0 shared-Web evidence into the current dev-based candidate and correct documentation discrepancies. This is one WBS evidence slice; UI-0 overall remains OPEN.

## Base and ownership
- Worktree: C:\Users\leo19\.codex\worktrees\ui0-evidence-integration\Star
- Branch: codex/ui0-evidence-integration
- Base: dev at df1c22f66d8d0c3b48baac7e348494ba3466569b
- Source commit: f286b6ac04dddee67ac75ffada4cc73afba4c9e8
- You own only this candidate worktree. Do not edit D:\Star, the WLD-3 worktree, or shared dev.

## Required work
1. Review the exact source commit and its 15-file manifest; cherry-pick f286b6ac only if not already present.
2. Fix the report statement that says the lane did not modify scripts/ui0_bench/playwright_driver.mjs. The commit adds that file. State accurately that the lane includes the bounded driver, while the three audited samples were existing records and were not re-captured for this audit; invocation and actual base URL provenance remain UNKNOWN.
3. Inspect the current UI WBS and implementation plan. Update only the evidence/status fields needed to record the three audited shared-Web runs and the newly available report. Keep UI-0, Tauri/Rust comparisons, renderer choice, thresholds, native RSS/CPU, queue/cache, terminal/canvas, and lifecycle acceptance OPEN.
4. Check terminology against the product naming rules: no competitor-product names in product UI/help/marketing. External tool names in technical evidence are allowed when factual.
5. Run the focused existing UI0 standard-library tests, Node syntax check, and diff/whitespace checks. Do not add or alter benchmark samples, do not run/start/stop local services, and do not claim Rust/Tauri performance evidence.
6. Self-review all changed paths, record exact validation and limitations, and make one coherent local commit on this branch. Do not rebase, merge, push, delete branches, or remove worktrees; coordinator owns integration and cleanup.

## Acceptance
The report and WBS agree with the exact committed evidence; source records remain byte-identical; every open UI0 gate stays open; commit and validation results are reported precisely.

## Evidence limitations
The repository has a CodeGraph index, but this session exposes no index_repository or check_index_coverage API. Do not initialize or rebuild it. Use the available codegraph query for source relationships only if needed, and inspect exact changed files.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
