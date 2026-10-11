# PHASE-INFRA-2C2-MANAGER-RESOURCE-WRITER-REPORT

## 0. 目的

在 INFRA-2C1 的 Host / Multipass / K3s 身份目录上增加由受信 Rust Infrastructure Manager 签名的 owner registration writer。本报告关闭 INFRA-2C2 的签名 receipt、事务写入和隔离 PostgreSQL 验证切片；不代表多 K3s provider、Run 执行闭环或 Worktree-first 全部阶段完成。

## 1. 改动矩阵与验收

| 区域 | 改动 | 证据与边界 |
|---|---|---|
| Rust 签名契约 | 新增类型化 V1 registration，覆盖 HostEnvironment、Multipass instance、K3s cluster、cluster membership 和 Kubernetes Node；Ed25519 校验绑定明确 allowlist 中的 `(manager_id, manager_key_id)`，对稳定序列化使用域分离签名。 | 未知 key、签名变更、超界 operation 在任何 PostgreSQL 访问前拒绝；最多加载 64 个可信 key。生产 key provisioning / rotation 尚未实现。 |
| Owner writer | SQLx writer 用 `star_infrastructure_owner` 设置 owner scope，以 operation identity 的事务 advisory lock 串行化同操作；单事务写不可变签名 receipt、一个精确 generation 的 Master row 和对应 registration event。 | 相同 operation ID/facts 精确重放返回已存 receipt；同 ID 不同 facts 失败关闭。数据库事务不跨 owner。 |
| Migration / WORM | 新增 `2026-10-08-infrastructure-resource-manager-receipts.sql`，增加全局 append-only receipt Transaction、receipt/event digest 与资源绑定、deferred orphan 检查、FORCE RLS 和 owner-only grant。 | 拒绝无签名的 legacy facts；支持已有 signed facts 下安全重应用。receipt 与 event 不允许 UPDATE/DELETE/TRUNCATE。 |
| Rust 集成测试 | 新增真实 SQLx + PostgreSQL writer integration test，执行五种 registration、精确重放、idempotency conflict、无效签名前置拒绝和缺失 parent 的事务回滚。 | 固定测试 Ed25519 seed 仅为 disposable test fixture；不可用于生产凭据。 |
| Disposable runner | 新增 PostgreSQL 18.6 loopback runner，重复应用累计迁移并运行上述真实 Rust integration test，再检查负向约束、迁移重应用和全量 RLS。 | Runner 只管理自己唯一的 PostgreSQL 数据目录；确认 server stopped 后才删除自己的临时目录。没有 Multipass 调用、VM 创建或 inventory 访问。 |
| 文档 | 同步 requirements v6.04、basic design v6.01、Infrastructure DD v0.29、Group DD v4.70、W/T/M v0.9、implementation plan v6.37、automation design v2.26 和 registry v0.73。 | 把 17 张表分类为 4 W / 7 T / 6 M，保留 2C1 的 16 表历史快照。各文档保留生产部署、manager key 与后续执行门。 |
| 资源边界 | 测试阶段 Multipass inventory 仍最多 2 条记录，Stopped/Deleted 也计数。 | 本阶段创建 0 个 VM，不读取、写入、删除或 purge 用户 inventory；上限是测试约束，不是产品拓扑上限。 |

## 2. 验证摘要

| 命令 / 门禁 | 结果 |
|---|---|
| CypherGraph Guardian | `cgg_guard.py --changed --stamp`：6 个变更 source files 检查通过；registry 文件的关系块已补齐。 |
| Rust 格式与编译 | `cargo fmt --manifest-path Cargo.toml --package infrastructure -- --check` 通过；`cargo check --manifest-path Cargo.toml -p infrastructure --lib -j 4 --offline` 通过；integration test target compile-only 通过。 |
| Rust 测试 | `cargo test --manifest-path Cargo.toml -p infrastructure --lib -j 4 --offline`：57/57 通过。 |
| Clippy | infrastructure library 与 `infrastructure_resource_catalog_writer_pg` integration test target 的 `cargo clippy ... --offline --no-deps -- -D warnings` 均通过。 |
| PostgreSQL 18.6 disposable runner | **26/26 steps 通过**；累计 migration chain 两次应用，真实 Rust SQLx writer integration 1/1 通过。 |
| PostgreSQL 约束 | 验证五种 resource registration、exact replay、冲突 replay、无效签名拒绝、缺父资源事务回滚、receipt/event digest 绑定、孤儿/WORM/RLS 负例、存在数据后 migration 重应用。 |
| FORCE RLS | **17/17 Infrastructure tables** 启用并强制 RLS；owner-only / unprivileged-read 约束通过。 |
| 辅助检查 | Python runner AST 语法检查、`git diff --check` 通过。Cargo.lock 有 Git 的 LF/CRLF 转换提示，不是 diff 错误。 |
| 资源与清理 | runner 的 PostgreSQL 停止并确认退出，唯一临时数据目录已清理；Multipass CLI 未运行，VM 数为 0，用户 inventory 未触碰。 |
| 独立代码审查 | CodeRabbit wrapper 因 `/root/.local/bin/coderabbit` 权限拒绝而不可用；没有上传代码。由本地 diff / focused gates 继续审阅。 |
| 工具可用性 | `rtk` 当前不在 PATH；受限于本环境，验证命令经 PowerShell 直接调用。Rust / Cargo 版本为 1.99.0。 |

## 3. 已知缺口

- 尚未将 migration 部署到目标 PostgreSQL，也未验收生产 role provisioning、应用连接身份、pool role reset、目标库备份恢复和运行时授权。
- 生产 Manager key 注册、撤销、轮换、密钥硬件保护与多 Manager 安全策略未实现；本阶段只实现显式 allowlist 校验接口。
- 没有 Multipass inventory reconciliation、签名 lifecycle/update/retire writer、manager process/IPC、受限 Multipass adapter 或多 guest / 多 K3s provider。
- 没有 RunInfrastructureBinding resolver、固定 cluster identity/generation 的 admission、Host 与 cluster 双层容量 reservation、K3s API/node/CRI readiness 或 Runtime sandbox readiness。
- Worktree bridge、取消恢复、独立验证、Schedule executor、任务卡 CLI、BI/Benchmark 消费和桌面性能/RSS 验收均未实现；Execution 继续 fail closed。
- 未执行 workspace-wide Cargo gates、目标平台性能测试、并行压力/故障恢复、生产 migration rehearsal 或完整 UI/E2E；本报告只说明 focused infrastructure slice 的实际证据。

## 4. 子代理失败接手清单

- 本阶段没有派发子代理；root agent 在独立 `infra-2c2-resource-writer` worktree 完成，未接收或丢失其他 agent 产物。
- 自查中发现 append-only receipt 上的 `SELECT ... FOR UPDATE` 会要求不应授予的 UPDATE 权限；已移除该锁，使用同事务 advisory lock 作为 operation idempotency fence，并由 PostgreSQL integration test 验证。
- 自查 runner 清理流程后，要求先确认 `pg_ctl status` 为 stopped，再删除唯一命名的 runner data dir；状态未知时保留目录并 fail closed。
- CodeRabbit 无法启动的环境错误已披露；未把它描述为通过，也未尝试改写系统 wrapper。

## 5. 守门规则

1. Rust 1.99.0 是当前工作区工具链基线；测试通过必须有本阶段实际 Cargo 输出，不从旧报告继承。
2. Manager 签名先于数据库访问验证，并同时绑定 manager ID、key ID、operation ID、resource identity、generation 与规范化 facts。
3. 未知 key、无效签名、越界字段、过期/撤销身份或不一致事实必须 fail closed。
4. 生产 private key 不进入数据库、插件、Agent、Task Card CLI、日志或源代码；测试 seed 仅限 disposable test。
5. Infrastructure owner 只在自己的 PostgreSQL 事务内写 owner receipt、Master 和 event；不得跨 owner 直接写表。
6. 同 operation ID / 同 facts 可幂等回放；同 operation ID / 不同 facts 必须冲突失败。
7. receipt 与 registration event 必须在一个事务内提交；deferred guard 拒绝孤儿 receipt 或无 receipt 的新 signed registration。
8. receipt/event 为 append-only；RLS/grants 必须限制在可信 Infrastructure owner。
9. `FOR UPDATE` 不得应用于仅有 SELECT/INSERT 权限的 append-only 关系；同操作并发序列化使用已授权的 advisory-lock 边界。
10. 多 K3s 的 guest、HostEnvironment、cluster、membership、Node 和 Run binding 是不同身份；VM 或 cluster 数量不得硬编码为产品拓扑上限。
11. Multipass 只能由可信 Rust Host Infrastructure Manager 经受限 adapter 操作；不得向 Agent、Plugin 或任务卡 CLI 暴露 daemon/socket、任意宿主 shell 或默认可写挂载。
12. 测试期最多保留 2 条 Multipass inventory record，包括 Stopped/Deleted；计数未知或已满时 fail closed，禁止自动 purge。
13. 数据库测试 runner 只能管理自己的 loopback PostgreSQL；停止状态未知时必须保留临时数据，不得清理用户目录。
14. 本阶段不创建 Multipass VM、不读取或变更用户 inventory；PostgreSQL gate 不得被描述成 provider、readiness、BI、Benchmark 或任务 CLI 已可用。
15. 每轮 implementation 都要将需求、基本设计、详细设计、W/T/M、实施计划和验收报告与代码对照；未实现能力继续标成开放门。

## 6. 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis review — 签名信任边界、精确 resource generation 与 owner transaction 已复核。 |
| SRE Lead | Mavis review — disposable PostgreSQL 生命周期、26/26 runner 和 stopped 后清理证据已复核；目标 DB 仍未验收。 |
| Platform | Mavis review — receipt/event 原子性、WORM、FORCE RLS 与最小 owner grants 已复核。 |
| 评审主持 | Mavis review — 本切片完成范围与 Manager/provider/Run execution 的未完成门已区分。 |
| PM | Mavis review — 零 VM、两条测试 inventory 上限及任务 CLI fail-closed 状态已记录。 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 INFRA-2C2 签名 receipt、Rust SQLx owner writer、17/17 FORCE RLS、26/26 disposable PostgreSQL gate、Rust 1.99.0、零 VM 和生产执行缺口。 | signed Manager resource registration slice 完成 focused verification。 |
