# Brief: phase9f4a-route-validation-review

**Agent**: reviewer
**Phase**: PHASE-9F4A-ROUTE-VALIDATION
**Created**: 2026-10-04 17:21:37

---

## Task
Perform a read-only acceptance and implementation review of the Phase 9F4A Schedule Rule HTTP route integration test slice before implementation.

## Scope
Inspect only the current branch sources relevant to Schedule Rule routing/authentication: `crates/star-api-rest/src/group_api/schedule_rules.rs`, `group_api.rs`, `main.rs`, the OAuth/JWT extractor, existing route tests, and the 9F4A SRS/DD addenda.

## Questions to answer
1. Is the Schedule Rule router mounted by the production REST entrypoint, and what does current code actually prove?
2. Which route-level checks can be tested without PostgreSQL, and which require a disposable database/real DB state?
3. Identify the smallest useful route tests for all four methods/paths, missing or insufficient scope, request body/page bounds, and private no-store error headers. Note any assumptions about auth token fixtures.
4. Flag contradictions between current SRS/DD/implementation-plan statements and source evidence.

## Constraints
- Read-only: do not edit, stage, commit, run migrations, or start a service.
- Do not claim membership/ACL/Run ownership or write transaction behavior is tested unless the observed test actually exercises PostgreSQL.
- Do not treat an API router unit test as production Auth/provider/database deployment verification.
- Return concise findings with exact file/line references and a minimal test recommendation.

## Acceptance
Return a short evidence-backed review that separates no-DB route/auth contract tests from disposable-PostgreSQL authorization/CAS/idempotency tests, plus any docs correction needed.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
