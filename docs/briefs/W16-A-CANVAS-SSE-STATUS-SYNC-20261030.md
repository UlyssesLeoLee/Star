# Brief: W16-A-CANVAS-SSE-STATUS-SYNC-20261030

**Agent**: worker
**Phase**: W16-A-canvas-sse-status-sync
**Created**: 2026-10-30 08:47:33

---

## Objective
Synchronize the Canvas SSE security/revocation status in SRS, basic design, and detailed design from evidence present on the exact registered dev base.

## Required work
1. Inspect the named SRS/BD/DD acceptance rows and the corresponding implementation, migration, and PostgreSQL acceptance evidence in this checkout.
2. Update only the narrowly proven local Canvas SSE revocation acceptance. Mark it PASS only when the exact test is present on this base and passes here; cite the test path, command, and observed result. If the test cannot run or its evidence is missing, leave it OPEN and explain why.
3. Keep HTTP device-revoke/audit, Outbox ownership, production RLS/grants, recovery, performance, and deployment gates OPEN unless this checkout contains direct, current evidence for each exact gate; do not broaden this lane to create implementation or infrastructure.
4. Make SRS, BD, and DD agree on status, scope, terminology, and known gaps. Preserve the user-mandated Project -> Run -> Worktree -> Branch architecture, Rust service ownership, Multipass/K3s target model, and no-competitor-brand product copy.

## Boundaries
Owned files only:
- docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md
- docs/design/BD-W16-PAYLOAD-SECURITY-001.md
- docs/design/DD-W16-PAYLOAD-SECURITY-001.md
- docs/briefs/W16-A-CANVAS-SSE-STATUS-SYNC-20261030.md
- docs/reports/PHASE-W16-A-CANVAS-SSE-STATUS-SYNC-20261030.md
- docs/reports/WBS-DISPATCH-W16-A-CANVAS-SSE-STATUS-SYNC-20261030.jsonl

Do not expand the file set, edit code, claim remote/deployment evidence, or infer PASS from a different worktree. If no qualifying local PASS exists, record the audit and leave the acceptance OPEN.

## Verification and handoff
Review the three-document diff against source evidence, run only the exact in-scope focused acceptance needed to substantiate the status, and record command/result. Use rtk git diff --check. Commit only this lane's accepted documentation changes and report the commit plus any remaining OPEN gates.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
