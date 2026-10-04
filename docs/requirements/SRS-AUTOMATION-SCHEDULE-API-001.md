# SRS-AUTOMATION-SCHEDULE-API-001 — Run-scoped Schedule Rule Management API

> Status: 🟡 Verified implementation slice; phase closure remains open for route and target-environment gates.
> Version: 0.4 | Date: 2026-10-04
> Parent: `docs/requirements.md` §50 and `docs/specs/domain-automation-spec.md` Schedule contract.

## 1. Purpose and boundary

The API lets an authorized Project member manage recurring Automation rules for one Engineering Run. Every rule targets a current Run-owned Worktree and Task Card and pins the current execution Profile and effective HookSet. Rules are versioned Master data. This API does not start a Run or materialize/dispatch occurrences; those capabilities remain fail-closed until later Phase 9F gates connect the worker, resource reservations, final authorization, Run events, and BI.

## 2. Requirements

| ID | Requirement | Acceptance evidence |
|---|---|---|
| SCHED-API-001 | Read and write routes MUST be nested beneath Project and Engineering Run; a Run grant and active Project binding MUST be checked for every request. | Route and authorization integration tests reject a Run from another Project and missing/expired grants. |
| SCHED-API-002 | Writers MUST have `work-item:write` scope and Project/Run role `tenant_admin`, `project_admin`, or `developer`. Readers MUST have `work-item:read` and current Project/Run access. | Auth matrix tests for each scope and role. |
| SCHED-API-003 | Create/revise MUST resolve an active, non-archived Worktree, current Run binding, current Task↔Worktree link, and current Profile/HookSet inside the write transaction. | PostgreSQL route tests cover stale, foreign, archived, and revoked targets. |
| SCHED-API-004 | Each revision MUST pin parser/tzdb versions, recurrence/DST policies, retry/deadline bounds, and immutable target/profile/HookSet digests. Invalid recurrence input MUST fail before commit. | Domain contract tests and persisted-row assertions. |
| SCHED-API-005 | Rule revisions MUST use compare-and-swap `expected_current_version`; edits close the prior revision and insert a successor. Deletes are not exposed; disable is a successor revision. | Concurrent revision test proves one writer wins; history remains immutable. |
| SCHED-API-006 | Create/revise MUST require an `Idempotency-Key`, replay the same response for the same actor/scope/request, and reject key reuse with a different request hash. | PostgreSQL concurrent retry and key-reuse tests. |
| SCHED-API-007 | Rule Master, audit facts, and an Outbox event MUST commit atomically. Audit/Outbox rows are append-only and tenant-isolated by FORCE RLS. | Disposable PostgreSQL repeat-migration, RLS, rollback, and mutation-rejection tests. |
| SCHED-API-008 | API pages and bodies MUST have explicit upper bounds; authenticated responses MUST be `private, no-store` and vary on Authorization. | Production-router requests verify four unauthenticated rejections, four signed-token missing-scope rejections, a valid-write-scope 16 KiB body-cap rejection, and private response headers. Page-boundary tests remain open. |
| SCHED-API-009 | Idempotency replay rows MUST expire after 24 hours; a matching expired key MUST be removed before reuse, and unrelated stale cleanup MUST have a fixed row cap. | SQL schema checks and exact-key reuse plus bounded cleanup test. |
| SCHED-API-010 | A disabled Rule MUST remain persisted but MUST NOT create an occurrence or authorize any worker execution. | Existing domain/adapter fail-closed tests remain mandatory. |

## 3. Verification status

The cached Rust 1.98.1 toolchain completed focused `rustfmt`, `cargo check --offline --locked -p star-api-rest --all-targets -j 4`, and all six `group_api::schedule_rules` module tests. The workspace explicitly enables `jsonwebtoken` RustCrypto so RS256 signing and verification have one deterministic backend; the valid-token route tests exposed and fixed the prior missing-provider configuration. Through production `build_group_router`, all four unauthenticated method/path cases return 401 with `Cache-Control: private, no-store` and `Vary: Authorization`; signed tokens with the wrong read/write scope return 403 before database access; a valid write-scope POST larger than 16 KiB returns 413. A lazy pool confirms these rejection paths do not reach PostgreSQL. A disposable PostgreSQL 18.6 container applied the 9F2 and 9F4A migrations twice and exercised tenant RLS, append-only Outbox, Run-consistency FK, expired-key reuse, and SCD2 boundary. These are implementation-slice checks only. Role authorization, cross-Project Run access, target currentness/revocation, concurrent CAS/replay, page boundaries, target database deployment, and runtime-role grants remain open; this API is not production-ready and Schedule execution remains fail-closed.

## 4. Revision history

| Version | Date | Change |
|---|---|---|
| v0.1 | 2026-10-03 | Define the initial Run-scoped Rule API contract and retain all execution gates. |
| v0.2 | 2026-10-04 | Record review fixes and focused verification while keeping production gates open. |
| v0.3 | 2026-10-04 | Include Rule-revision currentness and record the direct all-targets/test-binary evidence. |
| v0.4 | 2026-10-04 | Record production-router unauthenticated request coverage and clarify that valid-auth, DB-backed ACL, bounds, and target-environment gates remain open. |
| v0.5 | 2026-10-04 | Select the RustCrypto JWT backend and record signed-token missing-scope plus authenticated body-cap route evidence; keep role, DB ACL, page-boundary, and execution gates open. |
