# Brief: WLD7-COLLAB-DATA-CITATION-FIX-20261107

**Agent**: worker
**Phase**: WLD7-COLLAB-DATA-CITATION-FIX-20261107
**Created**: 2026-11-07 06:19:58

---

## Objective
Based on dev@b15919ce497e2a6da325aa8e13c6b61534654a2c, refresh the current test-design citation in the WLD-7 owner-local data design without changing its logical schema.

## Owned file and required edits
- Edit only docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md among source documents.
- Update the current header test-design reference from v1.7 to v2.6.
- Preserve historical revision v0.4 and all prior revision records unchanged. Append revision v0.5 saying only that the current citation is refreshed to v2.6; logical data, W/T/M classification, migrations, runtime, deployment, and acceptance status did not change.
- Preserve the explicit status that the schema is a logical target and has not been migrated, deployed, or verified against a target database.
- Do not edit the collaboration spec, test design, WBS, or other source documents.

## Exact manifest and guardrails
Only four manifest paths may change: this brief, docs/reports/WBS-DISPATCH-WLD7-COLLAB-DATA-CITATION-FIX-20261107.jsonl, docs/data-design/WLD-7-COLLABORATION-DATA-DESIGN.md, and docs/reports/PHASE-WLD7-COLLAB-DATA-CITATION-FIX-20261107.md. Do not touch main, remote, candidate branch/worktree, or the dirty main checkout. Do not merge, rebase, release the MAGOS claim, or clean the worktree.

## Validation and report
Run only git diff --check and exact changed-path manifest verification. Do not run application tests, Cargo, database, cloud, VM/K3s, build, or performance gates. The phase report must follow required §0–§7 structure: purpose, change matrix, verification summary, known gaps, subagent failure handoff, guardrails, sign-off table, and revision history. Explicitly state implementation and target-database validation remain OPEN/NOT_RUN.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
