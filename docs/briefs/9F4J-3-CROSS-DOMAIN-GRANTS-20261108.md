# Brief: 9F4J-3-CROSS-DOMAIN-GRANTS-20261108

**Agent**: worker
**Phase**: WBS-9F4J-3-CROSS-DOMAIN-GRANTS-20261108
**Created**: 2026-11-07 17:18:43

---

Objective: strengthen the existing ignored PostgreSQL grants acceptance test for 9F4J.3's newly reviewed cross-owner privilege boundary. This is a test and evidence lane; it does not change application code or database grants.

File ownership: only
- crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs
- docs/briefs/9F4J-3-CROSS-DOMAIN-GRANTS-20261108.md
- docs/reports/PHASE-9F4J-3-CROSS-DOMAIN-GRANTS-20261108.md
- docs/reports/WBS-DISPATCH-9F4J-3-CROSS-DOMAIN-GRANTS-20261108.jsonl

Required work:
1. Read the current test and the exact existing platform transport migrations that grant EXECUTE to star_platform_transport_runtime. Treat those migrations as read-only evidence; do not edit them.
2. Extend the dedicated opt-in PostgreSQL role/grant acceptance to prove the authenticated BI login/current role cannot inherit or SET ROLE to star_platform_transport_runtime or any other non-allowlisted Star runtime role. Keep star_bi_runtime as the explicit required BI runtime membership where the existing contract expects it; do not rely on suffix-only *_owner checks.
3. Check effective EXECUTE on the exact cross-domain transport functions found in migrations and report offending function identities. Add the smallest meaningful role matrix / SQL assertions to reject a BI writer that also has transport privileges.
4. Preserve the existing safety gates: #[ignore], exact disposable database identity before probes, loopback/non-default port, no target DB creation, and no automatic credential output. Target PostgreSQL acceptance remains NOT_RUN without the designated database.
5. Update the registered phase report and dispatch record with exact compile/format/diff/CGG results, review status, and NOT_RUN DB results. Keep the report seven-section structure.
6. Update the Rust file's Cypher graph header using CypherGraph Guardian. Do not run workspace-wide formatting or modify unrelated files.

Validation: under Rust 1.99.1 run the focused test-target compile with -j 4 (no execution of the ignored PG test), format-check only this Rust file, run cgg_guard.py --changed --stamp, and git diff --check. Report all outputs honestly. Do not update Cargo.lock or Cargo manifests.

Do not modify crates/star-api-rest, D:\\Star user files, Cargo files, migrations, WBS, main, remote, protected W8/Dependabot worktrees/branches, or any other lane. Do not claim production PG grants/RLS acceptance. Commit only the four registered files after self-review; do not merge dev. The coordinator will independently review and integrate accepted commits.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
