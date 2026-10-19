# Brief: WLD0-BINDING-CONTRACT-ALIGNMENT-20261020

**Agent**: codex-worker
**Phase**: WLD0-CONTRACT
**Created**: 2026-10-20 08:00:42

---

## Target
Reconcile the hostless Project -> Engineering Run -> Worktree -> Branch binding vocabulary in the canonical design documents. Edit only:
- docs/basic-design.md
- docs/design/DD-WORKTREE-GROUP-001.md
- docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md
Read docs/reports/WLD0-BINDING-CONTRACT-REVIEW-20261020.md as evidence; do not rewrite its historical findings.

## Change
Align the two design documents on ownership and version vocabulary:
- Engineering Run Directory owns the Project/Run-authorized EngineeringRunWorktreeBinding relation and its Run Outbox facts.
- Worktree Domain owns checkout lifecycle and Project-to-repository checkout ownership.
- Use binding_id + binding_version for the Run/Worktree relation; project_binding_id + project_binding_version for the Project/repository relation; focus_version for RunContext focus revision.
- Keep Worktree generation and K3s cluster generation as distinct resource-incarnation versions; do not use them as aliases for binding_version.
- Separate local_branch_ref plus observation source/time/freshness from optional remote_branch_id. Unknown or stale observations remain unknown.
- Preserve hostless Runs and optional Worktree focus; require authorized owner commands for attach/rebind/detach.
Update WBS to record only vocabulary/document alignment as complete; keep WLD-0 open for target PostgreSQL grants/RLS, authenticated writes, rollback/replay, races, duplicate-current-row handling and history reconciliation.

## Constraints and ownership
This is a documentation-only lane. Do not change Rust, SQL migrations, generated artifacts, or unrelated files. Do not invent historical claims. You are not alone in the repo; other lanes own frontend/auth and may update separate files. Do not revert or overwrite their work. Keep all product copy free of third-party product-comparison names.

## Acceptance
Show the exact terminology in both design files and the WBS, add a revision-history entry dated 2026-10-20, run focused literal searches and git diff --check, then commit with a message that references docs/briefs/WLD0-BINDING-CONTRACT-ALIGNMENT-20261020.md. Report any remaining ambiguity; do not claim production binding acceptance.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
