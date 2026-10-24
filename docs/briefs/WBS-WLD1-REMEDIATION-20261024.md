# Brief: WBS-WLD1-REMEDIATION-20261024

**Agent**: worker
**Phase**: WBS-WLD1-REMEDIATION-20261024
**Created**: 2026-10-24 07:57:10

---

Own only remediation of the two confirmed WLD1 review findings in this worktree. Preserve all existing implementation changes. Do not edit D:\\Star, dev, W8, or another lane; do not commit, rebase, merge, push, archive, or remove any worktree.

Required work:
1. Fix cleanup-error observability in both the PUBLIC-grant bootstrap and privilege-drift probe in crates/star-pg-adapter/tests/scm_repository_master_postgres.rs. If the probe/task fails and cleanup also fails, report both failures; do not let an unwrap/panic hide cleanup errors. Preserve cleanup attempts for role attributes, grants, schema, and probe rows as applicable.
2. Reconcile the WLD1 lockfile facts. The actual Cargo.lock diff adds only the direct domain-scm edge and its already-present package closure. Update contradictory statements and path counts in the WLD1 brief/report and relevant requirements/basic/detailed/traceability docs. Keep the permitted manifest exact and explicitly exclude unrelated lockfile changes. Do not assert a Cargo lock verification passed.
3. Refresh Cypher header stamp for edited Rust source, run git diff --check and any available targeted static checks. Cargo/Rust 1.99 is currently not found through RTK; do not install it or claim compile/test/PostgreSQL results. No DB test.

Ownership scope: primarily the existing registered WLD1 paths: Cargo.lock, the WLD1 test, WLD1 brief/report, and design/requirements files already in the WLD1 manifest. If another path is required, report it before editing. Do not change implementation behavior outside the two findings.

Deliver exact changed paths, evidence, commands and outcomes, remaining gates, and self-review findings.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
