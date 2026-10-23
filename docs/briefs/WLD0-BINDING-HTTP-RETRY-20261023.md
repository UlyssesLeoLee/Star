# Brief: WLD0-BINDING-HTTP-RETRY-20261023

**Agent**: worker
**Phase**: WLD0-BINDING-HTTP-RETRY
**Created**: 2026-10-23 06:32:39

---

Scope and ownership:
- Work only in this WLD0 worktree: codex/wld0-binding-http-positive-20261023.
- Fix only the stale policy-catalog assertion in scripts/automation/engineering_run_directory.py and its focused test if one exists. Catalog evidence shows permission.project_role_binding has a tenant permissive UPDATE policy and the lock-only restrictive policy is qual=true, with_check=false. Keep the exact five lock-only table cases distinct; preserve the existing expectation for all other policies.
- Do not change the migration, schema, RLS policy semantics, seeded rows, or another worktree. Catalog evidence alone is not proof that the UPDATE is denied. Run the real probe and require SQLSTATE 42501 plus verification that the source row is unchanged.
- Run the WLD0 acceptance using a fresh private UUID PostgreSQL data directory and random loopback port; exclude 5432 and protected port 5555. Do not inspect or alter the system PostgreSQL processes, the prior preserved WLD0 datadir, or unrelated services. Use the checked-in runner file-backed process handles; avoid direct pg_ctl calls that inherit the output pipe.
- On any failure, preserve the new private datadir and logs after stopping only the instance started by this attempt. Delete only that attempts datadir after complete success and confirmed stop.
- Run focused static tests, Python compilation, the real PostgreSQL probe, and the existing Rust HTTP acceptance/full WLD0 gates as far as the runner allows. Run CypherGraph Guardian on changed source/tests and git diff --check. Do not commit, rebase, merge, archive, or clean this worktree.
- Report exact files, commands, exit codes and evidence paths. Keep the WLD0 and Schedule execution status fail-closed until the full WBS gate closes.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
