# PHASE-WLD1-REPOSITORY-MASTER-20261023

> 状态：WLD1 SCM Repository master persistence source、TOCTOU/ACL/cleanup 修复及 Rust 1.99 selected-package gates 已验证；PostgreSQL runtime、full workspace 和集成验收仍未关闭。
> Lane：`codex/wbs-repository-master-20261023`
> 基线 HEAD：`5059508e5dcea98cd5a7a3aceb99e85ada60da10`
> 目标 dev HEAD（派发时）：`44b027ca09204ce1480667946a37f4900da10691`
> 集成：由父任务 review/integrate；本 lane 未 commit、rebase、merge、push 或 fetch。

## §0 目的

为 WTG-026 / AC-WTG-013 增加 SCM owner 持有的 Repository master domain type、PostgreSQL adapter 与 migration source，并修复独立 review 提出的 2 项 P1 与 1 项 P2：current revision 可被单独关闭后提交；owner role 的非 SCM 权限漂移未在授权 DDL 前拒绝；忽略的 PostgreSQL fixture 未核对连接的真实身份和目标。稳定 `RepositoryId` 独立于 Project、Run、Branch 与 checkout。本 slice 不开放 Worktree lifecycle，也不代表完整 WTG-026 验收完成。

## §1 改动矩阵

| 路径 | 改动与状态 |
|---|---|
| `crates/domain-scm/src/lib.rs` | 增加稳定 Repository master revision/metadata 类型、bounded credential-free 校验与领域单测。 |
| `crates/star-pg-adapter/src/repository/mod.rs` | 注册并导出 SCM master repository adapter。 |
| `crates/star-pg-adapter/src/repository/scm_repository_master.rs` | 新增 tenant-scoped owner adapter：读取、创建、expected-version 修订；显式 role/scope、identity advisory lock、SCD2/CAS。已移除无具体 composition caller 的 raw `PgPool` accessor。 |
| `crates/star-pg-adapter/Cargo.toml` | 增加对 `domain-scm` 的直接依赖。 |
| `Cargo.lock` | 仅增加 `star-pg-adapter` 对已存在 `domain-scm` package 的直接依赖边；当前 diff 无新增 package entry 或其他锁文件变化。Cargo lock/resolution gate 未运行。 |
| `crates/star-pg-adapter/tests/scm_repository_master_postgres.rs` | 增加 `#[ignore]` PostgreSQL acceptance fixture：双次 migration、权限边界/RLS、并发 create、跨租户隔离、actor revision/CAS/SCD2/audit；追加 single-close commit rejection、合法 close+successor、非 SCM direct grant rollback、PUBLIC ACL 排除、双重 opt-in 与真实身份/目标 guard 的回归工件。本测试未运行。 |
| `db/migrations/2026-10-23-scm-repository-master.sql` | 新增 M/SCD2 master、T/append-only audit、FORCE RLS 与最小 owner grant；deferred constraint trigger 在提交时保证每个现存 identity 至少一个 current revision，current partial unique 限制至多一个。role/grant DDL 前预检 memberships、对象所有权、全部 database ACL、其他 database 的 `pg_shdepend` ACL dependency 与当前 database 多类 ACL；漂移 fail closed、不自动跨 schema 撤权。 |
| `docs/requirements.md` | 更新至 v6.99；同步 Rust 1.99 focused gates、same-physical-connection 与 PG runtime 未验收状态。 |
| `docs/requirements/SRS-WORKTREE-CANVAS-001.md` | 更新至 v1.10；AC-WTG-013 记录身份负例与目标包验证、数据库运行时未验收。 |
| `docs/basic-design.md` | 更新至 v7.02；记录显式 PoolConnection、Rust focused gates 与完整运行时边界。 |
| `docs/design/BD-WORKTREE-CANVAS-001.md` | 更新至 v1.11；同步目标包验证、同物理连接事务与未验收门。 |
| `docs/design/DD-WORKTREE-GROUP-001.md` | 更新至 v5.24；同步 TOCTOU/cleanup guard 与 Rust/PG gates。 |
| `docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md` | 更新至 v1.9；AC-WTG-013 对齐源、测试编译、locked tests 与 PG runtime 状态。 |
| `docs/briefs/WBS-WLD1-REPOSITORY-MASTER-20261023.md` | 经 `SubagentDispatcher.brief()` 更新，维护 exact registered manifest、发现修复和验证限制。 |
| `docs/reports/WBS-DISPATCH-WLD1-REPOSITORY-MASTER-20261023.jsonl` | 由 dispatcher 追加 brief/audit 事件。 |
| `docs/reports/PHASE-WLD1-REPOSITORY-MASTER-20261023.md` | 本报告，§0–§7。 |

以上 16 paths 是 WLD1 实现 lane 的登记范围。父级复核流程另在该 checkout 生成 5 个支持文件（FOLLOWUP-REVIEW、PARENT-REVIEW、REMEDIATION、VALIDATION brief 与 remediation dispatch JSONL）；这些文件不属于实现提交，将单独保留在 `dev` 的 WBS 审计材料中。先前“没有改清单外文件”的表述不准确，已在本报告更正。Cargo.lock 仅含 adapter 的直接 `domain-scm` 依赖边，不含新增 package entry 或无关 lockfile 改动。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk git diff --check` | WLD1 源码/文档改动在最终复核中执行；最终结果见收尾报告。 |
| Rust 1.99 targeted rustfmt | 仅对四个本轮 Rust 文件以 `rustfmt --edition 2021 --config skip_children=true --check` 检查，exit 0。 |
| `cargo check --all-targets --locked` | `-p domain-scm -p star-pg-adapter -j 4` exit 0；出现 3 个未改 `domain-kms` deprecated `Nonce::from_slice` warnings。 |
| WLD1 selected-package all-target tests | 父任务使用 Rust 1.99.0，在独立 `CARGO_TARGET_DIR=E:\\DevCache\\cargo\\target-wbs-wld1-verify-20261024` 执行 `cargo test -p domain-scm -p star-pg-adapter --all-targets --locked -j 4 --quiet`，exit 0；`domain-scm` 35 passed、`star-pg-adapter` 35 passed。Schedule PG tests 5 ignored；SCM disposable PG test 1 ignored，target identity guard 1 passed。先前共享 target 首次遇 LNK1104，隔离 target 构建和测试均通过；未清理共享缓存。 |
| `cargo fmt --check` | package fmt check exit 1，仅报告 4 个未改文件：`domain-scm/src/multi_provider_identity.rs`、`domain-scm/tests/multi_provider_identity_test.rs`、`star-pg-adapter/src/repository/automation_schedule.rs`、`star-pg-adapter/tests/schedule_postgres.rs`。本轮 4 个 Rust 文件定向 rustfmt 均通过，未改动上述文件。 |
| CypherGraph Guardian | `cgg_guard.py --changed --stamp` exit 0，4 个 Rust source checked。SQL header stamp 独立 canonical SHA-256 expected/recorded `31b00ddfbd24cefc177528d2468c505337f0a002942fc3fc8e2547e9d1120603`。 |
| PostgreSQL acceptance | 未运行；6 个 `STAR_SCM_REPOSITORY_*` disposable fixture 环境变量均未配置。ignored test 编译通过不等于 migration/RLS/grants/并发/rollback 运行验收。 |
| 其他 gates | full workspace check/test、Clippy、独立 Cargo metadata/lock-resolution、目标 DB migration/RLS/grants/concurrency 均未运行。 |
| `Cargo.lock` | diff 仅新增 `star-pg-adapter` → 已存在 `domain-scm` package 的一条 direct dependency edge；无新 package entry、无无关修改。选定 package `--locked` check/all-target tests 成功；未执行独立 workspace lock/resolution gate。 |

## §3 已知缺口

1. 目标包编译、focused tests 与定向格式检查有 Rust 1.99 实证；package fmt check 的 4 个失败路径均未修改。full workspace、Clippy 和独立 lock/resolution gate 仍开放。PostgreSQL 未运行，Rust success 不证明 SQL migration/runtime behavior。
2. PostgreSQL test 仍 `#[ignore]`，必须配置 operator 创建的一次性隔离 cluster/database、双重 opt-in、精确 `star_scm_test_` database、superuser migration login、不同实际 runtime login、loopback same endpoint 和双连接匹配 marker 后再考虑运行。loopback 与 database marker 不能独立证明整台 VM/cluster 可以删除；若操作者无法保证隔离性，不得运行。
3. owner-role preflight 使用 `pg_shdepend` 检出任何其他 database 上的 owner-role object ACL dependency，并一律 fail closed；该跨库策略刻意保守，因为从当前 database 不在其他库按 schema 分类 ACL。当前库的 ACL 则按 `scm` 以外的 namespace/对象类型检查，`PUBLIC` grantee 不等于 owner role。PostgreSQL 未运行，兼容版本、目录可见性及 fixture 负向断言仍待验证。
4. `Cargo.lock` 属于本 manifest，仅允许 adapter 对 `domain-scm` 的 direct dependency edge；当前 diff 无新增 package entry 或无关 lockfile 改动。被选 package 的 `--locked` compile/test 已通过，但没有单独运行 Cargo metadata/完整 lock-resolution gate。
5. Project↔Repository authorization writer、Run binding writer/migration/API、Host durable root registry/verification agent、真实 lifecycle provider、应用 login membership 与目标 DB rollout 未实现/未验收。Repository master 不构成 Project/Run 可见性、授权、provider 连接或 Worktree 能力；没有相关 gates 时继续 fail closed。

## §4 子代理失败接手清单

本 lane 未派生子代理。独立 WLD1 review 确认先前的 2 P1 与 1 P2 在源码层已修复，未发现新的代码/SQL P1/P2；父任务随后独立重跑 Rust 1.99.0 selected-package all-target tests，70 项通过，PG runtime 仍未验证。父级复核流程生成的 5 个支持文件不在实现 manifest 中，实施提交会排除它们并单独保留到 `dev`。独立 workspace lock/resolution 与 integration 仍由父任务处理。不能把 lane 状态或该报告等同于 commit/merge。

## §5 守门规则

1. 仅修改登记的 16-path manifest。
2. `Cargo.lock` 仅纳入 adapter 对既有 `domain-scm` package 的 direct edge；排除一切无关 lockfile 改动。
3. 本 lane 不 commit、rebase、merge、push、fetch，不触碰 dev/main 或其他 worktree。
4. Git、dispatcher、Cargo 通过 RTK；不安装或绕过工具门禁。
5. Rust workspace MSRV 是 1.99.0；无法运行 toolchain 时不宣称 fmt/build/test/all-targets 通过。
6. ignored PostgreSQL test 的存在不等于 DB 行为通过。
7. migration 在角色/权限 DDL 前 fail closed；不得自动清理非 SCM 直接授权。
8. 每个 identity 的 current revision 必须在提交时恰有一条；single close 拒绝，合法 successor 与 close 同事务。
9. current partial unique 控制至多一条；deferred constraint trigger 检查至少一条。
10. 不把 PUBLIC grantee 视为 owner role direct ACL。
11. 其他 database 上 owner-role object ACL 一律拒绝；只在当前 SCM schema 做权限归一化。
12. 两个 table 均 tenant scoped/FORCE RLS；adapter 每个事务显式设置角色与 scope。
13. Master 与 append-only audit 在同事务写入；test 未运行前不声称数据库行为通过。
14. PostgreSQL fixture 要求双重 opt-in、实连身份/target、superuser migration identity 与 operator 提供的一次性隔离 cluster。
15. 每次 migration transaction 在任何 DDL 前通过同一事务连接重新读取实际 admin identity，并与预检的 admin/runtime identity、loopback endpoint、数据库名及 fixture token 比对；只有完整匹配才执行 DDL。pool 的单连接上限仅用于资源控制，不提供物理连接固定保证；数据库名/marker/loopback 仍不构成 VM/cluster disposal 的独立证明。
16. Repository identity 与 Project association、Run binding、Host root、checkout path 的 owner 分离；source slice 不得扩写为完整 WTG-026。
17. 文档须区分实现存在、检查工件存在、静态检查和未运行验收。
18. Rust/SQL Cypher headers 需与实际符号/关系一致；SQL source hash 手工复算不得冒称 Guardian 扫描 SQL。

## §6 签字栏

| 角色 | 审批者 | 备注 |
|---|---|---|
| 架构 | 架构师（Mavis 接手审核） | 仅 SCM Repository master source slice；完整 WTG-026 未验收。 |
| SRE Lead | Mavis 接手审核 | disposable PostgreSQL 与目标 DB gates 开放。 |
| 平台 | Mavis 接手审核 | 目标 package Rust 1.99 locked check/tests 与 selected test target compilation 通过；package fmt 有四个未改文件差异；PG runtime、full workspace、Clippy 与独立 lock/resolution gates 开放。 |
| 评审主持 | Mavis 接手审核 | same-physical-connection transaction 与 cleanup diagnostics 已有 source/test evidence；PostgreSQL acceptance 仍未运行。 |
| PM | Mavis 接手审核 | 由父任务 review 后决定是否集成。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.4 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 显式 acquire PoolConnection 使 identity recheck 与全部 DDL 使用同一物理连接事务；增加 address/port mismatch 负例并保留两个 probe 分支 cleanup errors。记录 Rust 1.99 selected-package check/tests、ignored target compile、targeted rustfmt、package fmt 的未改文件失败项及 exact Cargo.lock edge；PostgreSQL runtime/full workspace/Clippy/独立 lock-resolution 保持未验收 | WLD1 TOCTOU closure continuation
| v0.5 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更正实现 manifest 与 worktree dirty-path 数的表述，区分 16 个实现路径和父级复核流程额外生成的 5 个支持文件；记录独立复核结论，并把实现 owner 的 test 结果与独立复证状态分开 | WLD1 独立复核收尾与集成范围核对 |
| v0.6 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 父任务在独立 Cargo target dir 对两个目标 package 重跑 Rust 1.99.0 locked all-target tests：`domain-scm` 35 项与 `star-pg-adapter` 35 项通过；Schedule PG 5 项与 SCM PG 1 项仍 ignored，身份 guard 1 项通过。首次共享 target 的 LNK1104 未在隔离 target 复现；共享缓存未清理 | 完成 worktree 收敛前的 selected-package 独立复验 |
| v0.3 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 PG probe 与清理双失败时的诊断遮蔽；将 manifest 锁文件事实统一为精确 16 paths、Cargo.lock 仅增加 direct `domain-scm` edge 且无新增 package entry/无关变更；记录 Cargo/Rust/PostgreSQL gates 仍未运行 | WLD1 独立 review remediation |
| v0.2 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复独立 review 发现的 current revision commit invariant、owner ACL drift 和 PG target identity guard；补充 `pg_shdepend` 跨库 ACL fail closed、login alias/superuser/single-connection correction；同步 SRS/BD/DD/traceability，并精确记录 Cargo/PostgreSQL/cluster disposal gates 仍未通过 | WLD1 remediation 自检与第二轮 source review |
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 SCM Repository master source slice、15 path manifest、跨 actor CAS、权限边界与未执行的 Cargo/PostgreSQL 验证 | WBS-WLD1 Repository master persistence slice |
