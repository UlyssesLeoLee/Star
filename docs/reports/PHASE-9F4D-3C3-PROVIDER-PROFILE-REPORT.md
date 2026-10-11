# PHASE-9F4D-3C3-PROVIDER-PROFILE-REPORT

> 日期：2026-10-07
> 状态：🟡 Provider Profile Master 与 owner admission 派生请求切片已验证；生产 resolver/provider/execution 仍 fail-closed
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

把 Run binding 固定的 Infrastructure Provider Profile revision 持久化为 tenant-scoped Master，由 Infrastructure owner 在自己的 admission transaction 中读取并校验 profile digest，再从 profile 资源向量推导 Host 与精确 K3s cluster capacity request。调用方不再提供可任意放大或缩小的 capacity request。本阶段不实现 Profile 编辑器/current resolver、trusted provider observation、跨租户 allocator、生产 consumer 或 Schedule executor。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

| 域 | 改动与证据 | 状态/边界 |
|---|---|---|
| Rust domain contract | 新增 `InfrastructureProviderProfileRevisionV1`、canonical SHA-256 digest、profile-to-binding identity/placement validation 与 `capacity_request_for_binding`；不匹配的 scope/revision/digest/resource fail closed | 🟢 29/29 `domain-infrastructure` tests |
| Infrastructure owner admission | `admit_inbox` 移除外部 request 参数；在 owner/tenant transaction 中读取 binding pin 的 exact profile revision，校验存储 digest 后生成唯一 request | 🟢 `cargo check --all-targets` 与 Infrastructure lib tests 通过；真实 SQLx adapter 由下方 disposable PG runner 验证 |
| Profile persistence | 增加 SCD2 `provider_profile_revision` Master 与 append-only `provider_profile_revision_event` Transaction；tenant FORCE RLS、受限一次性 current-row closure、owner 仅 SELECT | 🟢 累计 8/8 Infrastructure tables FORCE RLS；目标 DB provisioning/grants 尚未验收 |
| Digest mismatch behavior | disposable PostgreSQL SQLx test 篡改持久化 digest；结果为 terminal rejection、0 reservation、1 receipt、1 owner Outbox | 🟢 5/5 ignored SQLx cases 实际运行 |
| W/T/M classification | 累计 8 表分类为 3 Work、4 Transaction、1 Master；3A 六表原分类继续保留，新增 3C3 两表分别归 M/T | 🟢 更新 `docs/data-design/phase9f4d-infrastructure-owner-wtm.md` v0.4 |
| 文档同步 | 同步 requirements v5.93、basic design v5.90、Schedule SRS/BD/DD v0.23/v0.22/v0.23、Infrastructure DD v0.20、automation design v2.19、registry 与实施计划 v6.22 | 🟢 本阶段增量指针已对齐 |
| VM/执行边界 | 未启动 Multipass VM；测试阶段总 inventory 上限仍为 2 个已登记实例，Stopped/Deleted 计入，满额/无效时拒绝创建且不自动 purge | 🔒 仅资源边界记录；未验证 VM/K3s/provider |

## §2 验证摘要（实际执行）

- PostgreSQL/Rust 自动化 runner：`python scripts/automation/phase9f4d_infrastructure_owner_schema.py --pg-bin D:\PostgreSQL\18\bin --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin --target-dir E:\t\phase9f4d3c3-profile`。17/17 runner steps 通过；三份迁移双次 apply；owner RLS、租户隔离、权限、SCD2/WORM、retention checks 通过；8/8 FORCE RLS、5/5 SQLx cases 通过；临时 PostgreSQL 正常停止并清理其独立目录。
- Rust compile：`cargo check --locked -p infrastructure --all-targets -j 4` 通过。
- Domain tests：`cargo test --locked -p domain-infrastructure -j 4`，29/29 通过。
- Infrastructure unit tests：`cargo test --locked -p infrastructure --lib -j 1`，47/47 通过。
- Rust format：本次四个修改的 Rust 文件定向 `rustfmt --check` 通过。`cargo fmt --all -- --check` 未通过，原因是仓库内大量未改文件已有格式差异；没有对全仓运行 formatter 以免引入无关改动。
- CypherGraph Guardian：`python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` 通过，5 个变更源文件已检查并刷新 freshness stamp；未声称 workspace-wide gate 通过。
- CodeRabbit CLI 已安装，但 `coderabbit review --agent -t uncommitted` 无法启动，执行环境返回 `/root/.local/bin/coderabbit: Permission denied`；因此没有外发 diff，CodeRabbit 不构成通过证据。继续完成了本地代码、迁移、文档和 diff 自审。
- 本阶段未创建、删除或修改 Multipass instance；用户确认测试期间磁盘受限，inventory 硬上限为 2 条登记记录。

## §3 已知缺口

1. Profile Master schema 已落地，但可视化 Profile editor、版本写入 API、current resolver 与授权审计入口仍未实现。
2. Binding 的 trusted Project/Run authorization、当前 binding/placement resolver 与 provider observation producer 尚未接线；本 adapter 单独不能启动任务。
3. 目前容量账本按 tenant owner transaction 隔离；同一 Host/cluster 跨 tenant 竞争必须等全局 allocator/ledger 后再启用。
4. 目标数据库 migration provisioning、生产 login/group membership、`SET LOCAL ROLE` pool reset 与部署未验收。
5. Infrastructure Inbox consumer、owner Outbox delivery、retry/reaper/reconciliation、Rust Host Infrastructure Manager、受限 Multipass provider、真实 K3s/CRI readiness、sandbox 与 Worktree bridge 未完成。
6. BI/Benchmark linkage、Schedule worker/executor、取消/恢复、native Hook gate、独立验证与任务卡 CLI 闭环均未完成，Schedule/CLI 保持 fail-closed。
7. 未运行 VM/K3s、多 agent 并发/RSS 压测、workspace-wide build/test 或商业分发验收；本切片不构成这些能力的通过证据。

## §4 子代理失败接手清单

本阶段没有派发子代理；没有子代理产出、失败 lane 或接手任务。

## §5 守门规则（16 项）

1. Run binding 固定 tenant、provider profile identity、revision、digest 与 placement policy。
2. Profile revision 是不可变 Master；只允许当前 revision 被 tenant-scoped actor 一次性关闭以供 successor 接替。
3. Profile revision event 是 append-only Transaction；禁止 UPDATE、DELETE 与 TRUNCATE。
4. Profile digest 必须覆盖全部影响 provider placement 与 Host/cluster 容量请求的字段。
5. Admission 只读取 tenant scope 与 binding pin 完全匹配的 exact revision；不能 fallback 到其他 tenant、profile 或 revision。
6. 持久化 digest 与重算 digest 不同必须终态拒绝，且不得产生 reservation。
7. Capacity request 只能从已校验的 Profile revision 推导；不得接受 caller-supplied request。
8. Profile identity、provider 或 placement 不匹配时 fail closed。
9. Profile 无效、未知 schema 或资源值不可安全持久化时 fail closed。
10. Admission 仍只在 Infrastructure owner transaction 写自身 reservation、receipt、event 与 Outbox；不跨 owner 写表。
11. Owner runtime role 对两个 Profile 表均为 SELECT-only；Profile mutation 不属于 admission owner。
12. 八张 Infrastructure owner/profile 表必须全部 tenant FORCE RLS；生产 role 仍需非 superuser 且 NOBYPASSRLS。
13. Profile schema migration 可重复执行且不得删除历史 revision/event。
14. Schedule 每次 trigger/retry/resume 仍须重新解析并 pin 当前授权、binding、profile revision 和预算。
15. 多租户共享物理 Host/cluster 前必须实现跨租户 allocator；tenant ledger 不可冒充物理全局账本。
16. 测试阶段最多保留 2 个 Multipass inventory 记录，Stopped/Deleted 也计数；达到上限或 inventory 不可解析时拒绝创建，不自动 purge；本阶段不创建 VM。

## §6 签字栏（Mavis 接手审核）

| 角色 | 审核 |
|---|---|
| 架构 | Mavis 接手审核：capacity request 来源收敛到 binding pin 的持久化 Profile revision；跨 owner 和执行边界明确 |
| SRE Lead | Mavis 接手审核：只记录已执行 Cargo/PG/RLS 证据；未声称 workspace、VM 或生产验收 |
| 平台 | Mavis 接手审核：tenant FORCE RLS、profile digest 和 owner read-only grants 验证通过 |
| 评审主持 | Mavis 接手审核：digest mismatch 的终态拒绝和零 reservation 回归用例通过 |
| PM | Mavis 接手审核：文档版本同步；两实例磁盘上限保留；商业执行闭环缺口未隐藏 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 9F4D-3C3 Provider Profile Master、profile-derived owner admission、隔离 PostgreSQL/Rust 实证、未完成生产门与 2 条 Multipass inventory cap | Profile revision 存储与 admission 派生实现完成 |
| v0.2 | 2026-10-07 | Mavis review | Record the cumulative eight-table W/T/M scope, clarify 3A/3B1 historical six-table gates versus current 3C3 profile read-only grants, synchronize active document versions, and include the final CypherGraph Guardian and self-review results. | Final evidence and cross-layer consistency review. |
