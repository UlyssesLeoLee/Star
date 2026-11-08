# Phase Report: 9F4J.3 Runtime Grants Self-Role Guard

**Phase:** 9F4J3-RUNTIME-GRANTS-SELF-ROLE-20261108
**Base:** `dev@6a5c3a46e7fec7e42caad31793b34e67fda8f19f`
**Status:** Test-only SQL correction and independent review complete. Focused target compilation passes with an isolated Cargo directory. PostgreSQL acceptance remains `NOT_RUN`. Source commit `620fdd18443565d28f3bb5b8a91d508a6ba39052` was fast-forwarded into local dev.

## §0 Purpose

Correct a false-positive in the existing ignored PostgreSQL privilege acceptance query. The role membership scan must not classify a principal as an unexpected member of its own role; it must continue to detect membership or SET capability to every other non-allowlisted Star role.

## §1 Change Matrix

| Registered file | Change |
|---|---|
| `crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs` | Excluded `roles.rolname = principals.principal_name` from the unexpected-role candidate join. Preserved the `star_bi_runtime` exclusion and both MEMBER and SET checks. Updated the Cypher source hash. |
| `docs/briefs/9F4J3-RUNTIME-GRANTS-SELF-ROLE-20261108.md` | Recorded scope, acceptance, and safety boundaries through the project dispatcher. |
| `docs/reports/WBS-DISPATCH-9F4J3-RUNTIME-GRANTS-SELF-ROLE-20261108.jsonl` | Recorded the brief and focused validation evidence. |
| `docs/reviews/9F4J3-RUNTIME-GRANTS-SELF-ROLE-REVIEW-20261108.md` | Independent review concluded PASS_WITH_LIMITATION; no source changes requested. |
| `docs/reports/PHASE-9F4J3-RUNTIME-GRANTS-SELF-ROLE-20261108.md` | Recorded the change, review, verification, and remaining database limitation. |

This is test-harness-only. It changes no production code, grants, migration, manifest, lockfile, or database policy. The exact seven platform transport function signatures and their EXECUTE checks remain unchanged.

## §2 Verification Summary

| Check | Result | Evidence |
|---|---|---|
| Focused Rust 1.99.1 test-target compilation | **PASS** | `rtk cargo +1.99.1 test -p bi-owner-runtime --test task_event_inbox_runtime_grants_pg --no-run --locked -j 4 --target-dir E:\DevCache\cargo\target\star-9f4j3-self-role-20261108`, exit 0; 1m40s. The temporary 678.8 MiB build directory was removed after verification. Four existing `async_fn_in_trait` warnings remain in `task_event_receiver.rs`. |
| Targeted rustfmt | **PASS** | `rtk proxy rustfmt +1.99.1 --edition 2021 --check crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs`, exit 0. |
| Cypher Graph Guardian | **PASS** | `cgg_guard.py --changed --stamp`, one source file checked. |
| `git diff --check` | **PASS** | Exit 0. |
| Independent review | **PASS_WITH_LIMITATION** | Reviewer confirmed the self-role exclusion is narrow; BI membership, other Star role MEMBER/SET checks, and seven exact EXECUTE checks remain. |
| PostgreSQL acceptance | **NOT_RUN** | `STAR_BI_TASK_EVENT_TEST_DATABASE_URL` is unset and `psql` is unavailable. The ignored PostgreSQL test was not executed; no database or VM was started. |
| Local dev integration | **PASS** | Fast-forwarded `codex/9f4j3-runtime-grants-self-role-20261108` commit `620fdd18443565d28f3bb5b8a91d508a6ba39052` from `dev@6a5c3a46e7fec7e42caad31793b34e67fda8f19f` to local `dev`; post-integration status was clean and `git diff --check` passed. No push. |

A preliminary build using the shared Cargo cache failed while compiling `sqlx-macros-core` with a core metadata-stub diagnostic. A clean, task-specific target directory then compiled the same test target successfully. The shared cache was left untouched.

## §3 Known Gaps

- The role catalog query has not been executed against the designated disposable PostgreSQL database; runtime grants and catalog behavior remain unverified.
- This lane does not prove production RLS, grants, migrations, or application behavior.
- The PostgreSQL test remains `#[ignore]`; enabling it requires a separately provisioned and identity-checked test database.
- The shared Cargo cache failure was not repaired or modified; successful focused compilation used an isolated target directory.

## §4 Subagent Failure / Takeover

- The independent review agent completed the sole review assignment and reported PASS_WITH_LIMITATION.
- No worker failure required takeover. The coordinator owns final integration and post-integration cleanup.

## §5 Guardrails

- Changes are limited to the five files registered by this lane.
- The SQL change removes only each principal's self-pair from the unexpected Star-role membership scan.
- The expected BI runtime membership remains separately asserted; MEMBER and SET checks for every other matching Star role remain active.
- No production source, grant, migration, Cargo manifest, lockfile, database, VM, remote ref, main branch, or unrelated worktree was changed.
- The dedicated PostgreSQL acceptance remains open as NOT_RUN; no live database behavior is claimed.
- Integrate locally only after confirming the reviewed source and dev target have not changed; do not push or publish.

## §6 Sign-off

| Role | Status |
|---|---|
| Architecture / scope | PASS: the change stays inside the registered test-only boundary. |
| Platform / Rust validation | PASS_WITH_LIMITATION: focused compile, format, graph guard, and diff checks pass; shared-cache diagnostic remains unresolved. |
| Database / SRE acceptance | NOT_RUN: no designated PostgreSQL URL or client. |
| Independent review | PASS_WITH_LIMITATION: static review found no blocker; PostgreSQL execution remains open. |
| Product / integration owner | PASS: local fast-forward recorded; PostgreSQL acceptance remains NOT_RUN. |

## §7 Revision History

| Version | Revision | Owner |
|---|---|---|
| v1.0 | Recorded the self-role SQL exclusion, isolated-target compile, static checks, independent review, and PostgreSQL NOT_RUN status. | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
| v1.1 | Recorded fast-forward of source commit `620fdd18443565d28f3bb5b8a91d508a6ba39052` into local dev after post-integration cleanliness and diff checks. | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |
