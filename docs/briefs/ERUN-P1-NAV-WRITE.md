# Brief: ERUN-P1-NAV-WRITE

**Agent**: nav-worker
**Phase**: ERUN-P1
**Created**: 2026-10-01 23:56:33

---

# ERUN-P1-NAV-WRITE

Frontend owner checkout: `C:/Users/leo19/.codex/worktrees/engineering-run-nav/Star`, branch `codex/engineering-run-nav-20261001`. Read this brief from D:/Star and checkout AGENTS. Root owns Rust/schema/docs/automation. Other workers exist; do not revert others or edit root checkout. Source fallback Tier2: graph APIs/index not available, no .codegraph; earlier readonly audit is evidence.

Own new frontend Run directory client/session/tree/shell and necessary narrow Sidebar/provider imports. No changes to old Group Apps, Canvas, Task CLI, Index business owner or auth backend. New generated files need matching Cypher header. Do not add/run tests or build; root compiles/integrates and requested verification later. Do not fake production login, use nil actor or public/localStorage credential. Current host source absent is explicit disabled mode; injection accepts real token provider + session generation, no manufactured source. Prefer separate RunDirectoryProvider to avoid activating old Worktree Group live adapters. Session must clear client/cache/focus and abort requests on generation/logout/tenant change. New shell loads canonical context before any focus; App capabilities false must display explicit pending migration. Never route into old Group and claim Run ownership.

Exact backend read contracts root is implementing:
1 GET /api/v1/projects (existing): projects [{project_id,role}], limit,next_cursor, UUID cursor default100/max200.
2 GET /api/v1/projects/{project_id}/branches: {project_id, branches:[{branch_id,project_id,repository_id,name,full_ref,head_commit_id:null|string,state,version}],limit,next_cursor}.
3 GET /api/v1/branches/{branch_id}/engineering-runs: {project_id,branch_id,engineering_runs:[{engineering_run_id,project_id,repository_id,branch_id,title,state,owner_user_id:null|string,version}],limit,next_cursor}.
4 GET /api/v1/engineering-runs/{run_id}/worktrees: {project_id,branch_id,engineering_run_id,worktrees:[{worktree_id,engineering_run_id,project_id,repository_id,workspace_id,name,branch,human_state,machine_state,agent_id,agent_session_id,runtime_id,archived,version,binding_id,binding_version,project_binding_id,project_binding_version}],limit,next_cursor}. No path.
5 GET /api/v1/engineering-runs/{run_id}/context?focus_worktree_id={wt}: {run_context:{tenant_id,project_id,repository_id,branch_id,engineering_run_id,workspace_id:null|string,actor_id,actor_kind,authorization:{project:{binding_id,role,version},branch:{binding_id,role,version},engineering_run:{binding_id,role,version}},granted_scopes,context_version,resolved_at,correlation_id},branch:{DTO},engineering_run:{DTO},focus:null|{WT DTO},capabilities:{run_owned_apps_available:false,execution_admission_available:false}}.
6 GET /api/v1/worktrees/{worktree_id}/run-context for compatibility discovery only, same envelope. Do not redirect old page yet.

New cursor opaque kind+parent-bound, 50 default/max100, query limit/cursor only, reject unknown; client validates envelope and each row parent before display. Current branch grant+run grant+project membership required server-side. Requests Bearer fresh token, no-store, credentials omit, AbortSignal, permitted trusted origin only. New canonical UI path `/projects/{project_id}/branches/{branch_id}/runs/{engineering_run_id}/worktrees/{worktree_id}` (Next page). Parent nodes only expand next layer, do not choose first Worktree as focus. Leaf href sets target; shell API validates tuple matches route and returned focus before rendering. Apps/execution remain blocked until migration; never show seed.

Sidebar: Project is true first tree level, then Branch, Engineering Run, Worktree leaves; aggregate ProjectWorktreeIndexLink can stay auxiliary. Settings advanced peers remain separate, Hooks no tree child. New tree uses authenticated Projects API rather than navStore authority. No session → explicit host session missing message; actor auth errors clear old data immediately and no seed fallback. Limit bounded requests (e.g. 2 concurrently), per-parent max4*50 rows, total parent cache cap16, visible DOM bounded via fixed-row virtualization/flattened tree; load-more/refresh controls and clear errors. Collapse aborts relevant pending request. All directory rows eviction clears expansion descendants so late response cannot reinsert expired data. No polling on collapsed tree.

Commit only frontend owned files, Ulysses author. Message references this brief, scripts/automation/engineering_run_directory.py and docs/automation-design.md §4.36 (root adds). Return SHA, actual files and known gaps, no unsupported runtime/production claims. Root integrates and verifies.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
