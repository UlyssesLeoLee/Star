# Brief: WLD7-COLLAB-SPEC-CITATION-LINK-FIX-20261107

**Agent**: worker
**Phase**: WLD7-COLLAB-SPEC-CITATION-LINK-FIX-20261107
**Created**: 2026-11-07 06:19:58

---

## Objective
Based on dev@b15919ce497e2a6da325aa8e13c6b61534654a2c, correct current references and broken relative links in exactly one source document.

## Owned file and required edits
- Edit only docs/specs/domain-collaboration-spec.md among source documents.
- Change status from Draft v1.2 to Draft v1.3.
- In the current header acceptance-plan reference, update test design v1.7 to v2.6. Keep §4.8, §5.2, and §6.2.5; all three anchors exist in current v2.6 test design.
- In §13.6, identify the current test design version as v2.6.
- Repair the three relative links at the cross-domain collaboration paragraph: from docs/specs/domain-collaboration-spec.md, use ../basic-design.md, ../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md, and ../architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md. These targets exist in latest dev.
- Preserve Revision v1.2 and all older revision history unchanged. Append Revision v1.3 describing only the citation/link correction. Do not claim runtime, implementation, or test completion.
- Do not edit the test design, WBS, data design, or other source documents.

## Exact manifest and guardrails
Only four manifest paths may change: this brief, docs/reports/WBS-DISPATCH-WLD7-COLLAB-SPEC-CITATION-LINK-FIX-20261107.jsonl, docs/specs/domain-collaboration-spec.md, and docs/reports/PHASE-WLD7-COLLAB-SPEC-CITATION-LINK-FIX-20261107.md. Do not touch main, remote, candidate branch/worktree, or the dirty main checkout. Do not merge, rebase, release the MAGOS claim, or clean the worktree.

## Validation and report
Run only document checks: verify all three fixed relative paths resolve, git diff --check, and exact changed-path manifest. Do not run application tests, Cargo, database, cloud, VM/K3s, build, or performance gates. The phase report must follow required §0–§7 structure: purpose, change matrix, verification summary, known gaps, subagent failure handoff, guardrails, sign-off table, and revision history. Explicitly state implementation and runtime remain OPEN/NOT_RUN.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
