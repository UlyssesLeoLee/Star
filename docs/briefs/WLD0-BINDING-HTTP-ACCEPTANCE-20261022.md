# Brief: WLD0-BINDING-HTTP-ACCEPTANCE-20261022

**Agent**: worker
**Phase**: WLD0-BINDING-HTTP-ACCEPTANCE-20261022
**Created**: 2026-10-22 15:59:45

---


Continue WBS WLD-0 from the clean `dev@8578d5048e6d6d10095d3f8797168471cfd632b8` base in this isolated lane.

## Evidence already gathered
- CodeGraph project: `C:\Users\leo19\orca\workspaces\Star\dev-converge-20261007`, current on 2026-10-22 (1,905 files; 37,350 nodes; 117,596 edges). Tier 2 task-directed query on `crates/star-api-rest/src/group_api/engineering_runs.rs` found `GET/POST /api/v1/projects/{project_id}/engineering-runs`, `list_project_runs`/`create_project_run`, and `begin_read`/`begin_write` authorization boundaries. The graph's coverage API is not exposed in this session; the exact route test/source was then read directly.
- `crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs` currently contains only `run_worktree_binding_mutations_require_authentication`, exercising unauthenticated PUT/DELETE with a lazy non-existent database pool and expecting 401.
- `crates/star-api-rest/src/group_api/run_worktree_bindings.rs` contains bind/reassign/detach routes, CAS/transition helpers, owner-local outbox writes, request replay helpers, unit tests for transitions and lock keys, and one ignored PostgreSQL advisory-lock test. Do not duplicate these pure unit tests or overstate them as authenticated database acceptance.
- Current WBS says WLD-0 still lacks authenticated HTTP writes, target app-role grants/RLS, rollback/exact replay, concurrent races, duplicate-current-row handling, and history reconciliation.

## Objective
Extend the narrow route acceptance file with the highest-value missing HTTP authorization/behavior tests that can be made deterministic. First inspect JWT extraction, existing test helpers, migration fixtures, and whether a disposable Postgres fixture is already available. If the existing fixture supports it, add an environment-gated test that proves actual authenticated bind/replay/conflict behavior against an explicitly disposable database. Never use a target/system database or port 5432. If no safe fixture is available, add only useful no-database auth/scope boundary tests, and document exactly which database acceptance remains open; do not fabricate success or build a broad new test database framework in this lane.

## Exact ownership
Only edit `crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs`, this brief, `docs/reports/PHASE-WLD0-BINDING-HTTP-ACCEPTANCE-20261022.md`, and `docs/reports/WLD0-BINDING-HTTP-ACCEPTANCE-20261022.log`. Do not change production routes, migrations, central WBS, schemas, dependencies, or other lanes. This lane is not alone in the repository; preserve others' work and do not revert unrelated edits.

## Validation and report
Use Rust 1.99.0 and `-j 4` if available. Run the focused integration target and any auth tests added. Use the repository CypherGraph Guardian before completion, update the test file's Cypher header with evidence-backed test/endpoint edges, and run `rtk proxy git diff --check`. The phase report must use the repository's required seven-section structure and distinguish passing route tests from unrun target-DB/RLS gates. If Cargo or the safe database fixture is unavailable, record exact commands and errors, leave the test appropriately gated, and preserve the lane.

If all available focused checks pass, make one local commit that references this brief path. Do not rebase, merge, push, archive, delete, or clean any worktree. Report exact commit SHA and residual WLD-0 blockers.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
