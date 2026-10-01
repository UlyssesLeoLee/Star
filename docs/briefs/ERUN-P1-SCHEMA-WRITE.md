# Brief: ERUN-P1-SCHEMA-WRITE

**Agent**: schema-worker
**Phase**: ERUN-P1
**Created**: 2026-10-01 23:49:56

---

# ERUN-P1-SCHEMA-WRITE

Accepted review correction: registration anchors are immutable T facts; revisions/grants/binding are five M/SCD2 tables; directory event is T, giving 3T/5M/0W. This supersedes the initial anchor classification below. RunContext excludes execution workspace and focus_version is separate. Trusted DML requires current Project manager in addition to tenant/actor/correlation; no implicit read grants.

Owner: schema worker only in `C:/Users/leo19/.codex/worktrees/engineering-run-schema/Star`, branch `codex/engineering-run-schema-20261001`. Root owns Rust API/docs/automation in D:/Star. Other agents may work concurrently; do not revert their work.

Read root AGENTS.md. Source fallback Tier 2; graph/index tools unavailable, no .codegraph. Evidence: 2026-09-29-worktree-group-phase-2b.sql current Worktree Project binding; phase2d owner assignment; task_execution_run is a task attempt and is not Engineering Run. No historical narration without Git proof.

Own only new `db/migrations/2026-10-01-engineering-run-directory.sql`. Implement canonical persisted read foundation, no seed/backfill and no Git or remote effects. Add Cypher structural header. Eight tables: M `scm.cloud_branch`, `scm.cloud_branch_revision`, `permission.cloud_branch_role_binding`, `multica.engineering_run`, `multica.engineering_run_revision`, `permission.engineering_run_role_binding`, `multica.engineering_run_worktree_binding`; T `multica.engineering_directory_event`. Identity tables immutable anchor; revision/grants/binding SCD2 with row UUID separate from entity UUID, active uniqueness, no delete/reopen; fixed tuple composite FKs; effective time checks.

Exact root API SQL contract:
- `scm.cloud_branch`: `branch_id, tenant_id, project_id, repository_id, remote_identity, created_at`. remote_identity is opaque provider reference, not URL or credential. unique tenant/branch, tuple tenant/project/repository/branch.
- revision: `revision_id,tenant_id,branch_id,name,full_ref,head_commit_id` (nullable text), `state` active/archived, `valid_from,valid_to,version` int, actor audit metadata as needed.
- `permission.cloud_branch_role_binding`: `binding_id,tenant_id,branch_id,user_id,role,valid_from,valid_to,version` int. roles tenant_admin/project_admin/developer/viewer/agent like existing project roles. No implicit grant from owner.
- `multica.engineering_run`: `engineering_run_id,tenant_id,project_id,repository_id,branch_id,created_at`, composite FK to Branch.
- revision: `revision_id,tenant_id,engineering_run_id,title,state` draft/active/paused/completed/archived, `owner_user_id` nullable, `valid_from,valid_to,version` int.
- `permission.engineering_run_role_binding`: `binding_id,tenant_id,engineering_run_id,user_id,role,valid_from,valid_to,version` int, same roles.
- `multica.engineering_run_worktree_binding`: `binding_id,tenant_id,project_id,repository_id,branch_id,engineering_run_id,worktree_id,project_binding_id,assigned_by,valid_from,valid_to,version` int. FK exact authoritative p.binding_id; guard project/worktree/repo tuple against p and w; one current Run per Worktree. No workspace owner at Run level; focus workspace is from authoritative WT.
- T event: immutable generic Branch/Run entity audit envelope (Branch required, Run nullable), bounded before/after JSON with table/entity/action/actor/correlation and created_at. Audit all 7 M writes incl Branch grants, not only Run. No outbox delivery claim.

Tenant FORCE RLS SELECT/INSERT/UPDATE on M (no delete), SELECT/INSERT on T with immutable trigger. Include comments W/T/M: seven M, one T, zero W. Server-side txn SET LOCAL tenant/actor/correlation required for writes and audits. Avoid grant bypass, future current rows, hidden physical delete or UPDATE metadata. Existing permission table SCD2 p has tenant+binding_id? Add necessary unique constraint safely for composite FK if needed. Reads revalidate p current and projected w.project_id/w.repo_id, reject missing/mismatched relation.

Inspect SQL carefully. Do not run tests/build or write docs/automation/root files. Commit only your migration with authorized Ulysses author and message referencing this brief plus scripts/automation/engineering_run_directory.py, automation-design.md §4.36 (root will add). Return commit SHA and known gaps. Root independently reviews/integrates and verifies sandbox PostgreSQL; no production migration execution.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
