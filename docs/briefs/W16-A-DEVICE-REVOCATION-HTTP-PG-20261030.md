# Brief: W16-A-DEVICE-REVOCATION-HTTP-PG-20261030

**Agent**: worker
**Phase**: W16-A-device-revoke-http-pg
**Created**: 2026-10-31 03:52:40

---

## Objective
Deliver a real, repeatable PostgreSQL acceptance test for the authenticated HTTP device-revocation flow and its transactional security audit, on the exact registered base.

## Required work
1. Inspect the current route, auth/authorization path, persistence transaction, audit model, migrations, and existing test harness. Use the registered lane only.
2. Implement an integration test that exercises the real HTTP route against a runner-owned disposable loopback PostgreSQL instance. Never point at production, Neon, user data, or an unowned database. Do not use mocks, in-memory substitutes, or seed data as production evidence.
3. Establish and assert the intended repeated-revocation semantics from the current product contract before encoding them. Cover unauthorized/unauthenticated rejection; device and refresh-token revocation plus audit atomicity; rollback behavior; relevant concurrency/idempotency behavior; and requests using the constrained application DB role/RLS path.
4. Add a small automation runner and runner tests that clearly fail closed when PostgreSQL or required binaries/configuration are unavailable, and emit machine-readable PASS/FAIL evidence. Do not add dependencies or touch files outside the registered manifest.
5. Synchronize the automation registry/design, test-design, brief, phase report, and dispatch audit. State exactly what ran, against which disposable DB, and the result. Keep unsupported or production-only claims OPEN.

## Boundaries
Owned files only:
- crates/star-api-rest/tests/device_revocation_http_pg.rs
- scripts/automation/w16a_device_revocation_http_pg_acceptance.py
- scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py
- docs/automation-design.md
- scripts/automation/registry.md
- docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md
- docs/briefs/W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md
- docs/reports/PHASE-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md
- docs/reports/WBS-DISPATCH-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.jsonl

Do not expand the file set. If the actual route, migrations, app-role grants, or production code have defects that require unregistered files, stop and report the exact dependency for replanning; do not patch around them or claim acceptance.

## Verification and handoff
Run the focused runner/test and Rust integration acceptance only when disposable PostgreSQL prerequisites are present; record exact commands and outputs/status. Run formatting/checks required for touched Rust/Python/doc files. If infrastructure is unavailable, report BLOCKED with evidence and leave the criterion OPEN. Commit only this lane's accepted changes and report the commit plus all remaining blockers.

## Execution update (2026-10-31, latest)

Run 20261031T034802-cda7dbb9b0 passed all four named authenticated HTTP acceptance cases on runner-owned disposable PostgreSQL 18 loopback with explicitly selected Rust/Cargo 1.99.1. All 10 runner steps succeeded; PostgreSQL stopped cleanly and only this run data directory was removed. Focused runner unit tests passed 15/15, including a subprocess check that writes 2 MiB on each stream while each saved log remains at most 1 MiB. Rust formatting passed; CypherGraph Guardian passed on the three source files. Automation registry check exited 0 with 0 errors and 205 repository-wide warnings. Exact workspace baseline Rust 1.99.0 compatibility remains NOT_RUN. Test-only global audit/SCD2 prerequisites do not prove production grants/RLS, production audit/SCD2, deployment, recovery, full W16-A, or performance.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
