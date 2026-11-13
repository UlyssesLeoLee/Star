# Independent Review: 9F4J.3 Runtime Grants Self-Role Guard

> **Conclusion:** PASS_WITH_LIMITATION
>
> **Review status:** Static source/diff review complete; PostgreSQL acceptance NOT_RUN.
>
> **Reviewed lane:** `codex/9f4j3-runtime-grants-self-role-20261108`
>
> **Lane HEAD:** `befe7a89b9d61c6c99bcf549d8e8bc0ce4472242`
>
> **Comparison base:** `dev@6a5c3a46e7fec7e42caad31793b34e67fda8f19f`
>
> **Review date:** 2026-11-08

## §0 Purpose

Review whether the added predicate in `unexpected_star_memberships` prevents a principal from being reported as a member of its own role, while retaining the expected BI runtime membership and the denial checks for other `star_*` roles and platform transport functions.

## §1 Scope and Evidence

- Read the current SQL in `crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs`: principal and membership query at lines 609-650, transport EXECUTE query at lines 652-664, BI membership assertion at lines 676-682, and the exact seven signatures at lines 620-630.
- Compared the registered lane's source change against `dev@6a5c3a46e7fec7e42caad31793b34e67fda8f19f`.
- The source change adds only `AND roles.rolname <> principals.principal_name` to the role join, plus the required source-hash update in the Cypher header. No production source, migration, grant, manifest, or database behavior changed.
- CodeGraph was not used as review evidence because the prior lookup returned unrelated files. This conclusion is based on direct reading of the target SQL and diff.

## §2 Findings

| Severity | Finding | Evidence | Disposition |
|---|---|---|---|
| None | No defect found in the reviewed predicate or its surrounding checks. | The new equality exclusion is scoped to the principal's own `pg_roles` row. | No source change requested. |

## §3 SQL Review

The `principals` CTE still checks both `session_user` and `current_user`. At line 642, the added `roles.rolname <> principals.principal_name` condition removes only the self-pair from the membership candidate set; it does not remove that principal from the other `star_*` role candidates.

The existing `roles.rolname <> 'star_bi_runtime'` exclusion remains. The separate `all_principals_are_bi_runtime_members` assertion still requires both principals to belong to `star_bi_runtime`, so the expected BI group membership remains checked independently.

Both existing denial predicates remain intact for every other matching `star_*` role:

- `pg_has_role(principal_name, role_name, 'MEMBER')`
- `pg_has_role(principal_name, role_name, 'SET')`

The `unexpected_transport_execute` CTE and its seven exact platform transport signatures are unchanged. The self-role filter does not participate in function EXECUTE checks.

## §4 Permission Boundary and Behavior

The change adjusts a test query's classification of role membership. It does not grant or revoke PostgreSQL privileges and does not modify production code or migrations. Its effect is limited to avoiding a self-role row being reported as an unexpected cross-domain membership; checks against other `star_*` roles remain active.

## §5 Verification and Limitations

- Static diff review: PASS.
- Database URL presence: `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` is unset.
- PostgreSQL acceptance: **NOT_RUN**. The test is `#[ignore]`; it was not run, and no database or VM was started.
- No claim is made that the live role grants or catalog behavior have passed acceptance.

## §6 Conclusion and Integration Recommendation

**PASS_WITH_LIMITATION.** The added predicate is a narrow and appropriate self-role exclusion for the membership diagnostic. The expected BI runtime membership assertion, non-allowlisted `star_*` MEMBER/SET checks, and seven exact transport EXECUTE checks remain present. Recommend accepting this test-harness-only change for local `dev` integration while keeping PostgreSQL acceptance explicitly open as `NOT_RUN` until the dedicated disposable database is available.

## §7 Review Record

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v1.0 | 2026-11-08 | Ulysses (12-role company per DEC-008) — Mavis review | Initial independent static review of the self-role predicate; recorded PASS_WITH_LIMITATION and the PostgreSQL NOT_RUN boundary. |
