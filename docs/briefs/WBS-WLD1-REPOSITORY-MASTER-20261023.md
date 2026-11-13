# Brief: WBS-WLD1-REPOSITORY-MASTER-20261023

**Agent**: codex-subagent WLD1 remediation continuation
**Phase**: WBS-WLD1-REPOSITORY-MASTER-20261023
**Created**: 2026-10-24 03:18:06

---

# Brief: WBS-WLD1-REPOSITORY-MASTER-20261023

**Agent**: codex-subagent WLD1 remediation continuation
**Phase**: WBS-WLD1-REPOSITORY-MASTER-20261023
**Updated**: 2026-10-24 (Rust 1.99 focused validation and same-connection TOCTOU closure)

---

# 任务：SCM Repository master persistence slice + independent review remediation

## 目标与完成范围
实现 tenant-owned SCM Repository master domain/adapter/migration source，并完成独立 review 列出的 2 个 P1、1 个 P2 source/test 修复：
1. SCD2 current identity 必须在事务提交时恰有一个 current revision；single-close fail、close+successor 同事务可成功。
2. 迁移改 role/grant 前拒绝 owner-role 非 SCM direct grants、向外角色继承与对象所有权，不跨 schema 自动清理权限。
3. ignored PostgreSQL test 在任何 migration 前核验双 opt-in、实际 session/current identity、不同登录角色、loopback 地址/端口、确认 database 与双 session fixture token；migration admin 是 superuser。实现显式 acquire PoolConnection 并在其物理连接上开启事务，复验身份与全部 DDL 使用同一事务；max_connections(1) 仅是资源上限。

源码、测试与文档已更新。Rust 1.99.0 目标包 locked all-targets check、两个选定包的 35+35 tests、ignored PG target 的 --no-run 编译、address/port identity negative test 与四个本轮 Rust 文件的定向 rustfmt --check 均通过。PostgreSQL runtime 仍未执行，不能标为数据库/集成完成；full workspace、Clippy、独立 Cargo lock/resolution gates 未运行。Cargo.lock 仅新增 star-pg-adapter 到既有 domain-scm package 的一条 direct dependency edge，无新增 package entry/无关改动。

## 当前 lane/限制
- Lane: `codex/wbs-repository-master-20261023`
- Worktree: `C:\Users\leo19\.codex\worktrees\wbs-repository-master-20261023\Star`
- Baseline HEAD: `5059508e5dcea98cd5a7a3aceb99e85ada60da10`
- 目标 dev HEAD（派发时）: `44b027ca09204ce1480667946a37f4900da10691`
- 当前仅父任务有权审查/集成；不得 commit/rebase/merge/push/fetch，勿触碰其他 worktree。
- Rust/Cargo 1.99.0 已通过 PATH 前置 `E:\DevCache\cargo\bin` 后由 RTK 可用；本轮不安装工具。PostgreSQL acceptance 所需的 6 个 STAR_SCM_REPOSITORY_* disposable fixture 环境变量均未配置，因此不运行 ignored DB acceptance。
- CodeGraph 在该 lane 无索引；不要重试，使用精确源文件审阅。
- PostgreSQL 官方 `pg_shdepend` 是集群共享依赖表，`deptype='a'` 表示 ACL role 引用；migration 保守拒绝 owner role 在其他数据库任意对象 ACL，当前数据库按 `scm` 外 ACL catalog 分类。
- Test fixture 的 loopback/database marker 不能独立证明整个 VM/cluster 可销毁，必须由操作员提供一次性隔离集群；若无法保证不可触及生产环境，不运行 ignored test。

## 当前 remediation/source 变更
- Migration：DEFERRABLE INITIALLY DEFERRED constraint trigger 保证至少一个 current；partial unique 限制至多一个；migration 历史 preflight 检查已存在 identity；role/grant DDL 前 preflight 检查 memberships、ownership、集群 database ACL、其他 DB `pg_shdepend` ACL dependency、当前 DB 对象 ACL 与可用 parameter ACL；PUBLIC 不作为该 role direct grant。
- PostgreSQL test：single current close 必须在 commit 失败且无残留；adapter revise 覆盖 close + successor 同事务；schema direct grant drift replay 必须回滚且不改变 role ACL/schema state；PUBLIC-only schema grant 是正例；bootstrap/drift 两个 probe 分支均聚合报告 probe/task 与 cleanup 双失败；identity helper 拒绝 endpoint address 与 port mismatch。纯身份 guard 测试通过；数据库 acceptance 仍 ignored 且未运行。查询 alias 为 login_role，admin 要求 superuser；实现显式 acquire PoolConnection 后在同一物理连接事务中完成 identity recheck 和所有 migration DDL，单连接池不再被描述为连接亲和保证。
- adapter：domain-scm direct dependency 已加；无具体 composition caller 的 raw PgPool accessor 已移除。Cargo.lock 当前仅一条 direct domain-scm edge、无新增 package entry 或无关变化；目标包 locked check/tests 成功，不代表独立 workspace-wide lock/resolution gate 已运行。
- SQL Cypher header 已将 preflight nodes 与当前迁移对应；4 个 Rust source 经 Guardian `--changed --stamp` 通过；SQL header source hash 独立复算匹配。
- 六份 requirements/basic/design/traceability 文档已同步当前版本与验收状态：requirements v6.99、SRS v1.10、basic v7.02、BD v1.11、Group DD v5.24、Traceability v1.9；保留较早修订行作为历史事实。

## 验证现状
- `rtk git diff --check`: 本轮文档收尾后重跑，结果见最终报告。
- Rust toolchain: `rtk rustfmt +1.99.0 --edition 2021 --config skip_children=true --check crates/domain-scm/src/lib.rs crates/star-pg-adapter/src/repository/mod.rs crates/star-pg-adapter/src/repository/scm_repository_master.rs crates/star-pg-adapter/tests/scm_repository_master_postgres.rs` exit 0。
- Compile: `rtk cargo +1.99.0 check --all-targets --locked -p domain-scm -p star-pg-adapter -j 4` exit 0；3 个既存 `domain-kms` deprecated Nonce warnings。
- Ignored test compile: 在隔离 `CARGO_TARGET_DIR=E:\DevCache\cargo\target\wld1-toctou-20261024` 下，`rtk cargo +1.99.0 test --no-run --locked -p star-pg-adapter --test scm_repository_master_postgres -j 4` exit 0。此前共享输出目录尝试遇 LNK1104，未删除/改动旧产物，改用隔离输出目录后成功。
- Runtime Rust tests: `rtk cargo +1.99.0 test --locked -p star-pg-adapter --test scm_repository_master_postgres disposable_target_guard_rejects_mismatched_real_identity_facts -j 4` exit 0，1 passed；`rtk cargo +1.99.0 test --locked -p domain-scm -p star-pg-adapter -j 4` exit 0，domain-scm 35 passed、star-pg-adapter 35 passed。WLD1 PG database acceptance 1 ignored，Schedule PG tests 5 ignored，doc tests pass。
- Formatting: 上述 4 个授权修改 Rust 文件 targeted rustfmt check exit 0。`rtk cargo +1.99.0 fmt --check -p domain-scm -p star-pg-adapter` exit 1，差异只在 4 个未改文件：domain-scm/src/multi_provider_identity.rs、domain-scm/tests/multi_provider_identity_test.rs、star-pg-adapter/src/repository/automation_schedule.rs、star-pg-adapter/tests/schedule_postgres.rs；未格式化这些文件。
- CypherGraph Guardian: `cgg_guard.py --changed --stamp` exit 0，4 个 Rust source checked；migration SHA-256 canonicalization expected/recorded match `31b00ddfbd24cefc177528d2468c505337f0a002942fc3fc8e2547e9d1120603`。
- PostgreSQL acceptance: 未运行。6 个 `STAR_SCM_REPOSITORY_*` disposable fixture 环境变量均 absent；ignored DB test 编译通过不代表迁移、RLS、grants、并发或 rollback 行为已验收。
- 未运行：workspace-wide Cargo check/test/clippy、独立 Cargo metadata/lock-resolution、PostgreSQL migration/ignored acceptance、真实并发/RLS/grants。
- `Cargo.lock`: 精确 diff 仅增加 `star-pg-adapter` 到已存在 `domain-scm` package 的一条 direct dependency edge；无 package entry 和无关变动。
## 精确登记的 16-path manifest
1. `crates/domain-scm/src/lib.rs`
2. `crates/star-pg-adapter/src/repository/mod.rs`
3. `crates/star-pg-adapter/src/repository/scm_repository_master.rs`
4. `crates/star-pg-adapter/Cargo.toml`
5. `crates/star-pg-adapter/tests/scm_repository_master_postgres.rs`
6. `db/migrations/2026-10-23-scm-repository-master.sql`
7. `docs/requirements.md`
8. `docs/requirements/SRS-WORKTREE-CANVAS-001.md`
9. `docs/basic-design.md`
10. `docs/design/BD-WORKTREE-CANVAS-001.md`
11. `docs/design/DD-WORKTREE-GROUP-001.md`
12. `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md`
13. `docs/briefs/WBS-WLD1-REPOSITORY-MASTER-20261023.md`
14. `docs/reports/WBS-DISPATCH-WLD1-REPOSITORY-MASTER-20261023.jsonl`
15. `docs/reports/PHASE-WLD1-REPOSITORY-MASTER-20261023.md`
16. `Cargo.lock` (only the new direct `domain-scm` dependency edge; no new package entry or unrelated change)

No other path may be edited. Do not include unrelated lockfile changes.

## 交接
父任务请独立 review complete lane diff，重点核验 SQL `pg_shdepend` behavior、SQLx/Rust compile viability、isolated PG fixture安全前置与 16-path manifest。Cargo.lock 只纳入 adapter 对 `domain-scm` 的 direct edge；当前 diff 不含新 package entry 或无关 lockfile 变更。不得把本 lane 文件状态当作已合并。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
