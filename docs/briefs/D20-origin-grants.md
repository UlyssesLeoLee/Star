# Brief: D20-origin-grants

**Agent**: worker
**Phase**: PHASE-9F4C-D20-GRANTS
**Created**: 2026-10-14 02:14:19

---

Own the D20 target-database owner grants/RLS contract lane only.

Baseline is dev commit 2114c4d267f7e7426c027661f3afb0dc3bfd5183. Inspect the D20 origin-verification migration and the preceding native transport auth/log migrations to identify the intended non-login owner/runtime role and exact least-privilege contract. Do not edit Rust SQLx tests or product requirement/design documents.

Add or correct the dedicated migration/grant contract as needed, and add a reproducible disposable-PostgreSQL verification runner in scripts/automation (prefer extending an existing D20 runner if one is present; otherwise add a narrowly named runner). Verify PUBLIC denial, exact intended role permissions, NOBYPASSRLS behavior, tenant scoping/RLS, and unauthorized principal denial for source verification/ACK. Use only an isolated disposable database; do not alter any target/production DB and do not create Multipass instances. The runner must fail closed on unknown role/migration state and clean up only resources it created.

Keep D20, native cutover, and Schedule/Run execution partial/fail-closed; schema/grant fixtures do not prove production provisioning or live TLS. Use repository CypherGraph Guardian for new/modified code headers. Run focused migration/runner checks and report exact commands/results. Do not modify shared dev or SQLx test files. Commit on codex/d20-origin-grants with subject including docs/briefs/D20-origin-grants.md. Return changed paths, commit SHA, validation output, blockers, and any gap between fixture grants and production deployment.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## Verification (2026-10-14)

- Disposable PostgreSQL result: `result=passed`, `mode=schema_only`, PostgreSQL `18.6 (Ubuntu 18.6-1.pgdg24.04+2)`, 89/89 audit steps passed. This includes D20 migration apply/reapply, grant/RLS contract fixture, a `BYPASSRLS` role-guard rejection and restoration, and untrusted-principal denial for source verification and ACK. The disposable server stopped successfully; the runner removed its owned database directory. `multipass_instances_created=0`; `multipass_inventory_modified=false`.
- Runner command (WSL Ubuntu, PostgreSQL 18): copied this runner into `/tmp/star-d20-origin-grants-20261014/scripts/automation/`, symlinked that temporary root's `db/` to this worktree's `db/`, then ran `/usr/bin/python3 /tmp/star-d20-origin-grants-20261014/scripts/automation/phase9f4d_run_owner_consumer.py --pg-bin /usr/lib/postgresql/18/bin --schema-only`.
- WSL ext4 evidence: `target/phase9f4d-run-owner-consumer/wsl-ext4-run-20261014/runner-result.json`, SHA-256 `CBD897B80ABA98A8EAC02422ED2BC613D5E93A56AC12D1195E1AC5E094271085`; audit `target/phase9f4d-run-owner-consumer/wsl-ext4-run-20261014/audit.jsonl`, SHA-256 `6218847B147E2FABB20BB16A987912F6D9A6996B1C4CE458DF54A1658A08BB40`. These generated artifacts are ignored by `**/target/` and are not commit inputs.
- Host limitation: Windows PostgreSQL `initdb.exe`/`psql.exe` were unavailable. A first attempt on DrvFS failed because PostgreSQL rejected its data-directory permission bits; that runner invocation cleaned its owned directory. Passing verification used the WSL ext4 temporary root. Therefore the fixture is verified on disposable PostgreSQL 18 under WSL, not on a native Windows PostgreSQL installation.
- Production boundary: this verifies the migration and fixture grants only. It does not prove target-environment role provisioning, production deployment grants, live TLS identity, or production traffic. D20 and native transport remain partial/fail-closed.
- Retention: `target/` evidence is ignored and will be removed when this worktree is cleaned. Preserve the summary and audit under a tracked documentation path during integration if long-term reproducibility is required.
