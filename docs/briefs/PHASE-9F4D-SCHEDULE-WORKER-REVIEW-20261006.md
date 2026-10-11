# Brief: PHASE-9F4D-SCHEDULE-WORKER-REVIEW-20261006

**Agent**: reviewer
**Phase**: PHASE-9F4D-SCHEDULE-WORKER
**Created**: 2026-10-06 18:07:08

---

Scope: read-only independent design review of the next Schedule worker / Run admission implementation slice for Star. Do not edit files or run mutating commands. Inspect the actual current source and migration evidence in these bounded areas: crates/domain-automation/src/schedule.rs; crates/star-pg-adapter/src/repository/automation_schedule.rs; crates/star-api-rest/src/group_api/schedule_rules.rs, execution_resources.rs, cli_sessions.rs, execution_catalogs.rs; Schedule and Run admission migrations; docs/requirements.md, docs/basic-design.md, docs/design/DD-WORKTREE-GROUP-001.md, docs/design/DD-LOCAL-INFRASTRUCTURE-001.md and implementation plan sections 6.74-6.84. CodeGraph is unavailable in this checkout: .codegraph is absent and codegraph_explore returned Transport closed; use targeted direct source reads and do not claim graph evidence. Review the approved requirement: Project > Cloud Branch > Engineering Run > Worktree; Schedule and BI/Benchmark are Run scoped; task CLI binds Worktree+Task; immutable creator run-as is reauthorized on every trigger/retry/resume and revoked permission fails closed; Multipass-managed shared Linux VM/K3s is a future infrastructure provider boundary and must not be exposed to Agent/Plugin/CLI; all queues, batches, logs, process I/O, memory, retries and DB waits stay bounded; enforce resource reservation/fairness/backpressure/cancel/drain; TaskExecutionRun, Reservation, RunEvent, occurrence admitted_run_id and Outbox must be atomic/idempotent and fenced; BI/Benchmark consumer must dedupe and preserve provenance. Deliver a concise evidence-based review: (1) exact existing executable paths and what is absent, (2) recommended cohesive next slice and dependency order, (3) required fail-closed checks and SQL/RLS/idempotency/concurrency tests, (4) files likely to change, (5) unresolved identity/target DB/provider blockers. Separate verified source facts from inference. No invented history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
