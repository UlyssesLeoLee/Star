# PHASE-INFRA-2C3-RESOURCE-LIFECYCLE-REPORT

## 0. 目的

本阶段在 INFRA-2C2 的签名资源注册 writer 上增加资源 lifecycle 写入：精确绑定资源 generation，以独立 Ed25519 签名域记录状态转换，并确保 `retired` Transaction event 与对应 Master SCD2 关闭原子提交。本报告只关闭这段 owner persistence 与隔离验证，不表示 Multipass/K3s provider、Run 执行或整体架构阶段完成。

测试阶段 Multipass inventory 上限继续为 2 条记录（包含 Stopped/Deleted），禁止自动 purge。本阶段未运行 Multipass、未创建 VM、未读取或修改 inventory。

## 1. 改动矩阵与验收

| 区域 | 改动 | 结果与边界 |
|---|---|---|
| 签名契约 | 增加 `InfrastructureManagerResourceLifecycleOperationV1` 与独立 `star.infrastructure.resource-catalog.manager-lifecycle.v1` domain separator；签名绑定 Manager/key、operation/receipt/correlation/actor、观测时间、资源 kind/ID/generation 和目标状态。 | 校验 schema、identity 和签名后才打开 DB；registration V1 signing bytes 不变。生产 key provisioning/rotation 未实现。 |
| Lifecycle writer | 扩展 `PostgresInfrastructureResourceCatalogWriter::transition`，先锁 operation id，再锁 exact resource generation；exact replay 在读取当前状态前成功，冲突重用拒绝。 | 接受 `active ↔ draining` 与 `active/draining → retired`；receipt/event 追加写入，退休时只关闭精确 Master generation。 |
| 数据库约束 | 新增 `2026-10-08-infrastructure-resource-catalog-lifecycle.sql` deferred constraint trigger，拒绝 retired event 未关闭对应 Master generation 的提交。 | 不新增表；累计仍 17 表（4 W / 7 T / 6 M），17/17 FORCE RLS。 |
| SQLx 集成 | 新增实际 PostgreSQL 测试，覆盖五种资源 Master、前置验签拒绝、旧 generation、精确重放/冲突、状态机、并发相同目标、terminal retired 和无 Master close 的数据库负例。 | 15 个接受的 receipt/event facts 对应五个关闭的 Master generation；拒绝操作不增加事实。 |
| Runner | 在既有 C2 runner 增加显式 `--phase c3`；使用独立 output/data 路径并复用累计 migration 链。 | C2 默认路径和历史证据不变。仅 runner 自己启动并清理 loopback PostgreSQL；状态不确定时保留数据。 |
| 设计同步 | 更新 requirements、basic design、Infrastructure DD、Group DD、W/T/M、automation design、registry 与实施计划；增加本阶段报告。 | 生产 key/login、target DB grants/deployment、inventory reconciliation、provider/readiness、BI 和 execution 持续标为未完成。 |

## 2. 验证摘要

| 命令 / 门禁 | 结果 |
|---|---|
| Rust toolchain | 已安装/确认 Rust 1.99.0（`rustc 1.99.0 (b940084d7 2026-09-28)`），与用户选择和 workspace pin 一致。 |
| Rust formatting | `cargo fmt --manifest-path Cargo.toml --package infrastructure -- --check` 通过。 |
| Rust all-target compile | `cargo check --manifest-path Cargo.toml -p infrastructure --all-targets -j 4 --offline` 通过；随后 lifecycle SQLx test 目标再次编译并实际执行。 |
| Rust library tests | `cargo test --manifest-path Cargo.toml -p infrastructure --lib -j 4 --offline`：**59/59 通过**。 |
| Lifecycle integration | Rust 1.99.0 SQLx PostgreSQL integration：**1/1 通过**，覆盖状态转换、重放、冲突、并发与 deferred commit rejection。 |
| PostgreSQL runner | `phase_infra2c2_resource_writer.py --phase c3`：**28/28 steps 通过**。累计 7 个 migration 各双次应用；WORM/RLS 负例、事实后 migration replay、17/17 FORCE RLS 均通过；PostgreSQL 已停止并清理 runner 自有数据目录。 |
| C2 default regression | 同一 runner 不传 `--phase`（默认 C2）：**26/26 steps 通过**，C2 Rust SQLx writer 1/1 与 17/17 FORCE RLS 通过；PostgreSQL 正常停止并清理 runner 自有数据。 |
| Strict Clippy | `cargo clippy -p infrastructure --all-targets --no-deps -j 4 --offline -- -D warnings` 通过。包含依赖的 strict Clippy 被未修改 `star-context/src/actor.rs` 两条 lint 阻断：`unnecessary_map_or` 与 `field_reassign_with_default`；本阶段未修改该文件。 |
| CypherGraph Guardian | `cgg_guard.py --changed --stamp`：3 个变更 source files 检查通过；header 按 lifecycle writer、test、runner 的实际调用与数据关系更新。 |
| Diff | `git diff --check` 通过。 |
| CodeRabbit | 当前环境 `coderabbit` CLI 不在 PATH，无法启动独立审查；未上传 diff，按本地 source/diff/测试结果完成自审。 |
| 外部资源 | Multipass CLI 未运行，VM 创建数为 0，测试 inventory 未访问或变更；没有部署目标 DB，也没有 push。 |

首次 SQLx 运行发现负例 fixture 的字节数组字面量被推导为 `i32`，已显式指定 `u8` 并重跑成功。一次 Windows linker `LNK1104` 未能创建测试 exe；单独重试目标链接后通过，随后完整 runner 28/28 通过。上述失败均已解决，不作为通过证据。

## 3. 已知缺口

- 迁移尚未部署到目标 PostgreSQL；生产 schema owner、NOLOGIN role membership、应用 login、pool role reset 和恢复演练未验收。
- Production Manager identity/key provisioning、rotation/revocation、IPC/process、inventory reconciliation 与 signed live provider commands 未实现；测试公钥/私钥仅为 disposable fixture。
- 本阶段不操作 Multipass/K3s；VM lifecycle、目标 cluster binding、两层容量 reservation、K3s API/node/CRI readiness 与 Runtime sandbox readiness 未实现。
- Worktree-to-guest bridge、取消/恢复、独立验证、Schedule worker、Task Card CLI、Run BI/Benchmark consumer、Rust 桌面 RSS/并行压力验收仍未完成。TaskExecutionRun 保持不可执行。
- `star-context` 的两条 Clippy lint 使带依赖 strict Clippy 失败；本 crate `--no-deps` strict Clippy 通过。Workspace-wide release/test/doc/bench gates 未在本阶段执行，不以 focused gates 代替。

## 4. 子代理失败接手清单

- 本阶段未派发子代理；没有子代理分支或产物需要接手。
- Root 自审修复了 SQLx 负例中的字节类型推导，并处理了 transient Windows linker 输出文件失败；最终重新执行了 SQLx integration 和完整 disposable runner。
- `coderabbit` CLI 未安装/不在 PATH；该工具不可用不是 review pass。变更已通过本地 diff、focused compile/test、strict Clippy（`--no-deps`）、PostgreSQL 和 CypherGraph 检查。

## 5. 守门规则

1. Rust workspace baseline 固定为 Rust 1.99.0；不得以旧工具链的结果替代。
2. Registration V1 signing bytes 保持兼容；lifecycle command 使用独立 domain separator。
3. 验签、schema、key ID 和 identity 校验必须在数据库连接/事务前完成。
4. 一个 operation ID 只代表一组不可变签名事实；精确重放可返回原结果，任何事实漂移必须失败。
5. 新 lifecycle 操作按 operation ID 与 exact resource kind/ID/generation 串行化。
6. 只允许 active→draining、draining→active、active/draining→retired；同态写入和 retired 后转换拒绝。
7. retired event、Manager receipt 与精确 Master generation 的 `valid_to` 关闭必须处于同一 owner 事务。
8. deferred DB trigger 必须在 commit 时拒绝没有 Master close 的 retired event。
9. Receipt/event 为 append-only；FORCE RLS 和 owner-only grant 不能被应用或插件绕过。
10. Owner 不得跨 Run/Platform/Infrastructure bounded context 直接写其他 owner 的表或共享事务。
11. Migration 必须在已有事实前后可重复应用，且不得静默采纳无签名历史事实。
12. 测试阶段 Multipass inventory 上限为 2 条（含 Stopped/Deleted）；容量未知或已满时 fail closed，不自动 purge。
13. PostgreSQL runner 只管理其唯一 loopback server 与被确认停止的自有临时数据目录；不操作 Multipass。
14. Rust、PostgreSQL 和 Clippy 结果只描述实际执行的 scope；本 crate 通过不等于 workspace 全部通过。
15. 未完成 target DB 身份、provider、capacity、Run binding、readiness、sandbox、BI/验证等门之前，Schedule/Task execution 必须保持关闭。

## 6. 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis review — registration/lifecycle 签名隔离、owner 边界和精确 generation 已复核。 |
| SRE Lead | Mavis review — 28/28 disposable PostgreSQL runner 与停机清理证据已复核；目标 DB 未验收。 |
| Platform | Mavis review — Receipt/Event/Master 原子性、WORM、FORCE RLS 和并发串行化已复核。 |
| 评审主持 | Mavis review — focused slice 与未完成 provider/Run/BI/execution 阶段已区分。 |
| PM | Mavis review — Rust 1.99.0、两条 inventory 测试上限、零 VM 和 fail-closed execution 状态已记录。 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 INFRA-2C3 typed lifecycle signature、owner writer、deferred retirement close guard、17 表 W/T/M、59/59 Rust tests、SQLx 1/1、28/28 PostgreSQL runner 与 Clippy/CypherGraph 范围；保留 production/provider/Run/BI/execution 缺口和两条 Multipass 测试上限。 | 用户要求将 Multipass 管理多 K3s 的方向和 Rust 1.99.0 基线带入当前架构阶段。 |
