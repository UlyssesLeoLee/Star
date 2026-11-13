# Brief: WLD0-BINDING-HTTP-POSITIVE-20261023

**Agent**: worker
**Phase**: WLD0_BINDING_HTTP_POSITIVE_20261023
**Created**: 2026-10-22 17:51:50

---

Task: Close the next WLD-0 gate by implementing and proving positive authenticated Run-to-Worktree HTTP mutation acceptance against an isolated disposable PostgreSQL fixture.

Owner: worker. Branch: codex/wld0-binding-http-positive-20261023. Worktree: C:\Users\leo19\.codex\worktrees\wld0-binding-http-positive-20261023\Star. Base branch/SHA: dev / e3a59c60b727649500fb4a2d42ac6aa3d16516ea. Current HEAD is that exact base.

Bounded context and evidence:
- Current handler is crates/star-api-rest/src/group_api/run_worktree_bindings.rs::mutate_binding (around lines 278-324). CodeGraph showed it validates authenticated actor and both project:write/worktree:manage scopes, begins an owner transaction, locks the idempotency key and Worktree binding, authorizes Run management, and maintains binding history, audit and Outbox. The graph reported no covering test for mutate_binding.
- Current route tests in crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs only prove unauthenticated requests and missing-scope requests are rejected before DB access; they use connect_lazy with a deliberately unavailable database. The current PHASE-WLD0-BINDING-HTTP-ACCEPTANCE-20261022 report confirms no positive authenticated writes, replay, idempotency conflict, rollback, CAS race, audit/Outbox atomicity, or runtime-role RLS through HTTP. Do not overstate those boundary tests as write acceptance.
- CodeGraph was queried on the current dev index. Its focused route-handler result is source-backed; the full positive path is not indexed/covered. We directly inspected the existing route test and report. No CodeGraph coverage/generation endpoint was exposed.
- Current dev HEAD is e3a59c60b727649500fb4a2d42ac6aa3d16516ea and was clean at dispatch; do not change the integration checkout. The main worktree has user-owned untracked AR-ATLAS files: do not touch them.

Work:
1. Inspect current binding migrations, owner grants/RLS, and disposable DB harness before implementation. Use only an isolated throwaway DB in an already permitted environment. Do not touch the target/system PostgreSQL or port 5432. Follow the Multipass inventory cap of two records, reuse an existing authorized guest if available, never create/purge a VM for this test. Do not require Docker Desktop.
2. Extend the actual Axum HTTP route acceptance with a real database-backed authenticated success path using the application/runtime role (not an in-memory or Noop store). Prove positive bind and detach behavior plus durable owner facts. Add exact replay returning the same receipt without duplicate state/outbox events; reused key with changed request conflicts; stale expected version is rejected; two concurrent CAS writers yield a single winner; failure/rollback leaves no partial binding/audit/outbox facts; cross-tenant/unauthorized visibility and RLS remain denied. If all cases cannot be supported safely, implement the strongest evidenced subset and report each missing case; do not claim closure.
3. Make production-source changes only for a demonstrated defect, within the registered run_worktree_bindings.rs scope. Prefer a new test fixture/runner over altering owner semantics. Any changes must preserve current authorization, SCD2, idempotency, audit and same-transaction Outbox contracts.
4. Add a reproducible bounded runner and static tests only if existing migration/test infrastructure cannot perform this safely. Keep DB/HTTP logs redacted of tokens and secrets. Use random loopback ports, unique temp data dirs, deterministic IDs, cleanup only the runner's own disposable DB after recording evidence.
5. Produce a PHASE report satisfying AGENTS.md's seven-section report structure, append-only dispatcher/acceptance JSONL and exact run log. Keep this lane's report/brief separate; do not edit the shared WBS or Schedule files.
6. Run exact Rust 1.99.0 route tests, disposable PostgreSQL positive/negative/replay/concurrency/RLS acceptance, Python syntax/static tests for the runner, Cypher Guardian on every changed source file, and git diff --check. Record commands and exit codes; distinguish source/unit/fixture evidence from target DB or production readiness. Commit only manifest files. Do not merge, push, archive, delete or clean the worktree.

Required deliverable is an end-to-end authenticated HTTP mutation backed by actual disposable PostgreSQL owner state, or a precise evidence-backed report of the strongest safe partial result. WLD-0 stays open until target database grants/RLS, migration ledger, rollback/replay/races and history reconciliation are verified.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
