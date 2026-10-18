# Brief: wbs-wld0-binding-audit-20261018

**Agent**: worker
**Phase**: WBS-WLD0-AUDIT-20261018
**Created**: 2026-10-18 23:48:42

---

Task: Audit the WLD-0 canonical Run/Worktree binding against current dev source and design.
Owner: This worker owns an isolated audit report only.
Branch/worktree: codex/wbs-wld0-binding; base dev 6f8d5b8f2d00b1515928c9a25e0762a45cf30e4e.
Manifest: docs/briefs/wbs-wld0-binding-audit-20261018.md; docs/reports/WLD-0-RUN-WORKTREE-BINDING-AUDIT-20261018.md; docs/reports/WBS-WLD0-AUDIT-20261018.log.
Read current SRS, Basic Design, DD-WORKTREE-GROUP-001, Run owner W/T/M design, migrations, and concrete APIs/services. Determine whether one canonical versioned binding contract exists. Report exact evidence/gaps for Project-owned hostless Run; optional Run-Worktree relation; Worktree-local Branch observation; owner commands; actor/grants; idempotency; stale/cross-scope rejection; detach/rebind/archive/recovery; DB role/RLS; and lifecycle/BI events. Separate documented intent, source, tests, target DB, and runtime evidence. Propose a precise consistent contract only where sources do not settle it; preserve Project->Run->Worktree->Branch. Cite file paths and section names/line numbers.
No code changes. Do not edit canonical SRS/Basic/DD/data-design docs; give a file-by-file update matrix and proposed wording/acceptance changes for coordinator review. No third-party product comparison wording. Do not claim WLD-0 is implemented.
Validation: use RTK. No .codegraph exists; do not index. Run git diff --check; ensure only the three manifest files changed; self-review and make one audit commit. Do not rebase, merge, push, clean, or delete; coordinator owns integration and preserves all other lane content.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## 7. Audit outcome

Status: completed as a source-backed design audit only. The canonical `engineering_run_worktree_binding` relation is present as a versioned SCD2 Master in the DD/schema, and Project-owned hostless Run creation/read source exists. The end-to-end binding contract is incomplete: no Run↔Worktree write route was found in the reviewed router, branch observation has no freshness/provenance in the Run directory projection, and hostless Run task execution still encounters a legacy host-Branch requirement. The audit report contains exact evidence, proposed contract wording/acceptance gates, and a file-by-file update matrix. No production code, canonical design, database, target runtime, merge, rebase, or push was changed or claimed.

Deliverables: `docs/reports/WLD-0-RUN-WORKTREE-BINDING-AUDIT-20261018.md` and `docs/reports/WBS-WLD0-AUDIT-20261018.log`.
