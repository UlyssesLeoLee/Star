# Brief: 9F4J-3-TASK-EVENT-COMPOSITION-20261107

**Agent**: worker
**Phase**: WBS-9F4J-3
**Created**: 2026-11-07 17:42:31

---

## Objective
Complete the 9F4J.3 fixes identified by independent review in the existing registered source lane. Keep native ingestion opt-in and fail-closed.

## Independent review findings to fix
1. P1: BI writer authorization currently rejects `_owner` role memberships but can still inherit another `star_*` runtime such as `star_platform_transport_runtime`. Require the writer's session_user/current_user to belong only to the explicitly allowlisted `star_bi_runtime` among Star roles, and reject effective EXECUTE on the seven exact `platform_transport` function signatures already enumerated in the merged `task_event_inbox_runtime_grants_pg.rs` acceptance test. Keep application/query/writer identity separation. Do not broaden grants.
2. P2: A failed shutdown drain or JoinError currently marks readiness Blocked but may still let main return `Ok(())`. Propagate shutdown failure to the process exit result. Add focused tests for timeout/join failure propagation where the existing seams permit.

## File ownership and integration
Only edit the eight MAGOS-registered paths in this lane. Do not edit BI owner core, transport runtime, migrations, Group WBS, protected W8/Dependabot sources, D:\Star user files, main, or remote. The current dev target advanced from 01563e93 to eccdd6f5 with a separate cross-domain grants test commit; do not rebase or merge during implementation. Coordinator will rebase this private lane onto current dev after commit, rerun affected checks and independent review, then decide integration.

## Required verification
Use Rust 1.99.1, focused `rtk cargo check -p star-api-rest --all-targets -j 4`, focused `rtk cargo test -p star-api-rest --test bi_task_event_ingestion -j 4`, targeted rustfmt, Cypher Graph Guardian for changed Rust files, and `rtk git diff --check`. Report PostgreSQL role/grants/RLS/transport/deployment acceptance as NOT_RUN unless it truly ran against a designated disposable DB. Update the seven-section phase report and dispatch JSONL with the exact review fixes, validation, remaining gates, and no production claim. Commit only registered files; do not merge dev.

## Architecture guardrails
Keep BI owner independent of platform transport; composition belongs in star-api-rest. Preserve pinned provider/profile/version/digest/endpoint, explicit drain/migration on drift, commit-before-ack, bounded in-flight work, cancellation, and fail-closed readiness. The existing cross-domain grants test in current dev is a compile-checked but ignored PostgreSQL acceptance harness; do not claim its SQL executed.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
