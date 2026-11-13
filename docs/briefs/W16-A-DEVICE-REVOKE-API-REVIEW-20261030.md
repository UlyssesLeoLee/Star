# Brief: W16-A-DEVICE-REVOKE-API-REVIEW-20261030

**Agent**: reviewer
**Phase**: W16-A device revoke API evidence review
**Created**: 2026-10-30 06:42:20

---

## Task
Independently audit the existing authenticated device-revocation HTTP endpoint and PostgreSQL owner transaction, because the Canvas SSE acceptance test performs direct SQL revocation and does not exercise the endpoint/audit path. This is a bounded source review, not an implementation task.

## Initial source evidence from dev@be3dd7b42881cc0ebd229a622f87bc5f407e9f69
- `crates/star-api-rest/src/auth/oauth/device_enrollment.rs`: route registration at line 87; `revoke_device` at lines 240-257 extracts `AuthenticatedUser`, parses device UUID, calls repository with actor tenant/user, returns invalid grant for missing/already revoked, else 204.
- `crates/star-pg-adapter/src/repository/device_bindings.rs`: repository `revoke` at lines 341-390 starts one transaction, sets OAuth owner role/scope, increments generation and sets status/revoked_at, revokes actor/device refresh family, inserts audit, commits. `set_oauth_owner_scope` at 394-412 and `insert_audit` at 414-436.
- `db/migrations/2026-10-28-w16-device-auth.sql`: owner role/ACL/RLS, append-only audit trigger and policies; source review must check exact details.
- Candidate Canvas PG test commit `5f437c...` uses direct SQL to revoke and therefore does not prove HTTP handler/audit semantics.

## Review questions
Trace router/middleware auth to the endpoint; verify repository caller identity, transaction and CAS/idempotency semantics, SQL scoping, generation and refresh-family revocation, durable audit, errors, and test coverage. Identify concrete defects separately from missing acceptance evidence. Recommend the minimal next W16-A tests (including unauthorized tenant/actor, stale/already revoked, audit row and concurrent SSE response) without changing implementation. Cite exact files/lines and compare requirements/test design.

## Output and constraints
Write only `docs/reviews/W16-A-DEVICE-REVOKE-API-REVIEW-20261030.md` in this worktree using the seven-section report contract. Do not edit source, migrations, test design, WBS, or candidate runner. Do not run Cargo, PostgreSQL, browser tests, or any database. No commit. Use `rtk` commands. This isolated worktree has no `.codegraph`; use the supplied exact-path evidence and bounded search/read fallback. Review only the dev snapshot above; do not read D:\Star user files or other worktrees.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
