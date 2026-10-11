# Brief: ERUN-P2-CLI-RUN-CONTINUE

**Agent**: worker
**Phase**: PHASE-ERUN-P2
**Created**: 2026-10-02 08:57:44

---

# Brief: ERUN-P2-CLI-RUN-LINK

**Agent**: worker
**Phase**: ERUN-P2
**Created**: 2026-10-02 07:29:03

---

Implement the Task Card CLI execution identity bridge to the canonical Engineering Run directory.

Base: origin/dev at 3966dab74f6af157e25b12f205f50bbd8966f861. Private independent worktree; rebase onto latest origin/dev before handing off. Read root AGENTS.md and the narrow requirements/design source files. CodeGraph MCP/index tooling and .codegraph are unavailable; use explicit source reads/rg for this bounded area, and do not claim graph coverage.

Ownership: new additive database migration plus the Rust Task CLI admission path, DTO/signature/fence verification only where necessary, and detailed design specific to task execution. You own crates/star-api-rest/src/group_api/cli_sessions.rs, crates/domain-local-runtime/src/task_execution.rs, crates/star-dto/src/task_run.rs and a new migration if needed. Do not modify the Engineering Run directory migration/API, frontend Run tree, Hooks page, AGENTS.md or broad requirements/basic-design. Other work is happening in separate worktrees; do not touch/revert their files.

Goal: every newly admitted CLI TaskExecutionRun must carry a server-derived canonical engineering_run_id and exact Project/Repository/Branch/Run/Worktree/Task tuple. Never trust an engineering_run_id from a browser body as authorization. Resolve the current Worktree-to-Run binding and current Project/Branch/Run grants in the server transaction that admits the task, bind the resulting Run identity into the persistent TaskExecutionRun snapshot and the signed Local Runtime grant/fence, and compare the binding at spawn/consume. A stale, moved, archived or unauthorized checkout must fail closed. Preserve historically unbound TaskExecutionRun records without guessing/backfilling an owner; define a safe legacy read policy and block an unbound new spawn where required. Keep TaskExecutionRun ID distinct from EngineeringRun ID. Preserve idempotency, existing PTY lifecycle, bounded session listing, cancellation/recovery, native Hook/profile fence and no-path API properties. Prefer a narrow API route evolution over accepting an untrusted run identity.

Acceptance: one vertical server-to-runtime contract for new runs; tenant/tuple/composite FK checks prevent cross-Project/Branch/Run association; no weakening or seed fallback; additive and rerunnable DDL; Cypher manifests updated for changed/new code; document detailed-design deltas and known production gaps in the report; commit only owned files, cite this brief, scripts/automation/engineering_run_directory.py and docs/automation-design.md §4.36. Static review plus cargo check compile target(s) only (Cargo -j 4); no test execution unless separately cleared by coordinator. Report validation commands, source commit, limitations and whether legacy rows are read-only. If a safe end-to-end binding cannot fit, stop at a reviewed design/evidence report and don't ship an unsafe partial authorization path.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)


Continuation: the prior agent handle is missing after interruption. Resume the existing uncommitted draft in C:/Users/leo19/.codex/worktrees/erun-p2-cli-run-link/Star; do not restart or discard it. Current remote dev is 1068ea40. Finish the owned implementation, review it, run appropriate targeted verification (existing tests permitted now, no broad unnecessary tests), commit owned files with author Ulysses. Do not push or merge; coordinator owns serial rebase/integration. Current AGENTS.md new section 0 includes Project/Branch/Run tree, Native Hook, Rust resource bounds, actual Task CLI closed-loop and documentation consistency; follow these, report genuine production blockers. Graph MCP/index tools are unavailable in parent and no .codegraph; use exact source and qualify graph evidence as unavailable. RTK and Orca CLI are unavailable in this host. Briefs stay owned by root. You are not alone; preserve others changes. No unsupported historic narratives.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
