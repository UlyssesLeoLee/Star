# Brief: 9F4J3-RUNTIME-GRANTS-SELF-ROLE-20261108

**Agent**: coordinator
**Phase**: 9F4J3-RUNTIME-GRANTS-SELF-ROLE-20261108
**Created**: 2026-11-08 06:31:46

---

Objective: correct a false-positive in the existing ignored PostgreSQL acceptance probe for cross-domain Star role grants. The probe currently treats a principal's own role as an unexpected membership when pg_has_role reports self-membership.

Scope: update only the existing SQL assertion in crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs to exclude roles.rolname equal to principals.principal_name. Preserve the star_bi_runtime exclusion and both MEMBER and SET checks for every other Star role. No production code, grants, migrations, manifests, lockfile, or WBS changes.

Validation: targeted Rust 1.99.1 test-target compile with --no-run --locked -j 4; targeted rustfmt check; Cypher Graph Guardian --changed --stamp; git diff --check. Do not execute the ignored PostgreSQL test. The dedicated PostgreSQL URL is absent, so DB acceptance remains NOT_RUN.

Guardrails: modify only the five registered lane paths. Keep D:\Star user changes, main, remote refs, all other worktrees, and the protected W8 and Rust candidate lanes untouched. After independent review and focused checks, integrate only accepted commits into local dev; do not push or publish.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
