# Brief: WLD3-PG-EXECUTION-RETRY-20261023

**Agent**: worker
**Phase**: WLD3_PG_EXECUTION_RETRY_20261023
**Created**: 2026-10-22 17:51:48

---

Task: Verify the WLD-3 Schedule occurrence Branch snapshot and immutable run-as authorization acceptance against the current dev baseline. This is a read/acceptance lane; do not edit source files or the shared WBS.

Owner: worker. Branch: codex/wld3-occurrence-pg-retry-20261023. Worktree: C:\Users\leo19\.codex\worktrees\wld3-occurrence-pg-retry-20261023\Star. Base branch/SHA: dev / e3a59c60b727649500fb4a2d42ac6aa3d16516ea. Current HEAD is that exact base.

Dependencies and existing evidence:
- WBS WLD-3 remains open; Schedule execution stays disabled.
- Existing older unregistered branch codex/wld3-occurrence-pg-20261022 has one unique commit and dirty/untracked evidence. Do not touch, rebase, cherry-pick, release, archive, or clean that branch/worktree. MAGOS rejected a manifest overlapping its committed files. This lane owns only the four new files listed in its registered manifest.
- CodeGraph on the current dev index found crates/star-api-rest/src/group_api/schedule_rules.rs::authorize_schedule_run_as (lines 637-654) delegating to authorize_run_principal; the same transaction locks and rechecks active Run authorization (lines 669-701). It found the ignored integration test schedule_run_as_authorization_rechecks_canonical_directory_grants at lines 1338+, requiring the disposable canonical ACL fixture. Direct rg confirmed scripts/automation/phase9f3_schedule.py has the ACL harness and star-pg-adapter/tests/schedule_postgres.rs is the DB-backed target. CodeGraph freshness/coverage tooling was unavailable; the source paths were checked directly.
- Previous attempt on the older lane reported PostgreSQL 18.6 29/30 with the embedded Rust ACL test failing before its body at E0631 in worktree_lifecycle.rs:815; fresh pinned Rust 1.99.0 build of schedule_postgres --no-run and domain-automation 30/30 passed. Re-derive current results; do not copy those as current evidence.

Work:
1. Inspect current runner CLI/help and previous failure logs without modifying any source or older evidence.
2. On this new lane, run the focused Rust 1.99.0 ACL test through the repository runner and the schedule_postgres test target. If a disposable PostgreSQL fixture is available, execute the migration/API/ACL harness twice using separate temporary data dirs and unique loopback ports; never touch the system/target DB or port 5432. Check Multipass inventory first; reuse an existing permitted guest only, obey the two-record cap, and do not create/purge an instance.
3. Record exact commands, tool versions, exit codes, disposable DB identity, assertion counts, logs and limitations. Preserve all original evidence. If a prerequisite is missing, record it and keep the lane open; do not simulate success with mocks.
4. Write a PHASE report satisfying AGENTS.md's seven-section report structure, plus append-only JSONL and log. Make only these files: this brief, docs/reports/PHASE-WLD3-PG-EXECUTION-RETRY-20261023.md, docs/reports/PHASE9F3-PG-EXECUTION-RETRY-20261023.log, and docs/reports/WLD3-PG-EXECUTION-RETRY-20261023.jsonl.
5. Run focused Rust 1.99.0 targeted checks and git diff --check. Make no source changes; no Cypher Guardian run is necessary unless source changes are attempted, which are outside this manifest. Commit only the four manifest files if the acceptance report is complete. Do not merge, push, archive, delete or clean anything.

Completion requires current evidence, not the older report. Report whether the WLD-3 acceptance subgate passed; WLD-3 overall remains open until remaining saga, target-DB grants/RLS, restart/crash and worker/admission gates pass.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## 7. Acceptance results (2026-10-23 UTC)

- Environment: existing Multipass guest `informed-impala`; Rust/Cargo 1.99.0; PostgreSQL 18.6. No VM was created/stopped/purged and no request was made to port 5432.
- Focused compile: `cargo test --locked -p star-pg-adapter --test schedule_postgres --no-run -j 4` exited 0 with Rust 1.99.0.
- The checked-in disposable runner's unmodified PostgreSQL startup failed after `initdb` passed (dynamic loopback port 38265; `pg_ctl start` exit 1). The private long-path diagnostic adding `-k <data_dir>` also failed because the Unix socket path exceeded PostgreSQL's 107-byte limit. These are harness/environment failures, not acceptance passes.
- A private copy of the current HEAD runner was then extracted under short path `/tmp/w3r1`; an in-memory wrapper added `-k <unique data_dir>` to `pg_ctl start`. Two separate disposable runs used ports 55845 and 41225 and unique UUID data directories. Each runner invocation completed 32 steps: 31 passed, 1 failed. All migrations (including occurrence Branch snapshot first application and repeat), SQL/RLS/ACL assertions, tenant Outbox checks, and the ignored canonical-directory-ACL Rust test passed. Each focused ACL test passed 1/1 (155 filtered).
- In both runs `schedule_postgres` executed 5 tests: 4 passed and `materialization_is_idempotent_and_tenant_scoped` failed with SQLSTATE 23514, constraint `occurrence_target_snapshot_branch_uuid_check`, at `crates/star-pg-adapter/tests/schedule_postgres.rs:327:10`. The run logs show the test fixture's new occurrence row is rejected by the added Branch UUID constraint. This lane does not edit production source or test fixtures, so the mismatch remains for a separately registered code-fix lane.
- Each run's private PostgreSQL data directory was removed by the disposable runner. A post-run guest process check found no matching runner, compiled test, or `/tmp/w3r1` process. No unrelated PostgreSQL service was stopped or modified.
- The diagnostic wrapper is not the checked-in runner acceptance. The WLD-3 subgate remains **failed/open**, and Schedule execution remains disabled. See `docs/reports/PHASE-WLD3-PG-EXECUTION-RETRY-20261023.md` and the append-only log/JSONL evidence.
