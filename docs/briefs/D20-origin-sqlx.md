# Brief: D20-origin-sqlx

**Agent**: worker
**Phase**: PHASE-9F4C-D20-SQLX
**Created**: 2026-10-14 02:14:19

---

Own the D20 SQLx integration-test lane only.

Baseline is dev commit 2114c4d267f7e7426c027661f3afb0dc3bfd5183. Work only on a new or narrowly extended integration test under crates/platform-messaging-runtime/tests (prefer native_origin_verification_pg.rs); do not change database migrations, grants, Python runners, product docs, or other crates. Confirm the exact current migration/API contract before writing tests.

Add real PostgreSQL + SQLx coverage for the D20 origin-verification transaction boundary: exact raw-wire digest and delivery identity, valid versus revoked/unknown/mismatched certificate origin, stale or mismatched generation/sequence/message/lease scope, acknowledgement re-verification, exact replay, and rollback/retry when verification or receipt persistence fails. Use a real disposable DB fixture and fail clearly when prerequisites are missing; no mock may be reported as DB proof. Keep native execution/default-off and production composition fail-closed.

Use the repository's CypherGraph Guardian/Cypher header requirements for new code. Run focused Rust formatting/check/tests that this lane can run and report exact commands/results. Do not modify shared dev. Commit the completed lane on codex/d20-origin-sqlx with a subject that includes docs/briefs/D20-origin-sqlx.md. Return changed paths, commit SHA, validation output, blockers, and any gap between tests and intended contract. Keep the phase marked partial.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
