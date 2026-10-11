# PHASE-9F4C-D17-TRANSPORT-PROFILE-REPORT

> v0.3 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：Transport Profile Master、revision publisher 与 uncached resolver 首片已实现；disposable PostgreSQL/SQLx gate 通过。D17 整体、native transport、Schedule/Run execution 和 Rust 1.99.0 正式工具链门仍开放。

## §0 目的

建立 Platform Messaging owner 管理的 deployment+tenant scoped Transport Profile revision Master、可信上下文 resolver seam、append-only Audit/Transactional Outbox facts，并把本机 Rust 服务的模块复用原则落实到 D17。此报告只验收已完成的 profile 持久化切片，不表示真实消息服务、BI consumer、Agent executor 或产品 phase 已闭环。

## §1 改动矩阵

| 范围 | 完成内容 |
|---|---|
| Rust profile contract | 新增 `transport_profile.rs`：有界 provider/version/endpoint/route definition、canonical digest、生命周期 revision、opaque secret reference、authenticated-session seam 和 exact route authorization。复审时将 Rust 字段首字符校验对齐 PostgreSQL schema 正则，避免仅在写库时失败。 |
| PostgreSQL owner/resolver | 新增 `transport_profile_pg.rs`：同 owner transaction 内 CAS successor、exact idempotency replay、未知 commit 结果 fail closed；resolver 按 trusted deployment/tenant 查单一 active profile，不缓存。 |
| Master 与 facts | 新增 `2026-10-12-platform-transport-profile-master.sql`：一张 SCD2 Master、一张 append-only Audit、一张 append-only BI/Benchmark Outbox；trigger 在同一事务发出 Audit/Outbox。 |
| 安全与 schema gate | 新增 SQL assertions，检查双 scope FORCE RLS、restricted roles、最小 grants、secret-free Outbox、固定 search_path、非 PUBLIC trigger execute 与 deferred successor。 |
| SQLx 验收 | 新增 `transport_profile_pg.rs` integration target，验证 create/exact replay、CAS 冲突、successor、identity/route resolve、并发单赢家、revoke 与历史 revision replay。 |
| Runner | 扩展 `phase9f4d_run_owner_consumer.py` 的 `--schema-only --transport-profile-tests`；D17 migration 双次 apply，运行 schema assertions 与真实 SQLx test；该选项不启动 NATS 或 Multipass。 |
| 本机 Rust 模块复用 | 只将 AssetsLake version/schema、trace、causation、idempotency 的事件元数据语义适配到 Star Outbox；没有复制其 dry-run relay 或将其视作 durable transport。当前 AssetsLake HEAD 与目标文件 hash 经只读复核，源仓其余 17 个脏路径未触碰。其余本机候选、逐文件 hash、许可和依赖边界详见 `docs/design/DD-RUST-SERVER-REUSE-001.md`。 |
| 设计同步 | 同步 `docs/requirements.md`、Schedule SRS、`docs/basic-design.md`、Schedule BD/DD、`docs/data-design.md`、实施计划、自动化设计与 registry；保留 production identity/provider/BI/性能验收缺口。 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| PostgreSQL disposable runner `--schema-only --transport-profile-tests --ignore-rust-version` | PASS，PostgreSQL 18.6，共 71/71 steps；两次应用 D17 migration、schema/RLS/grant/WORM assertions 及既有 prerequisite chain 均完成。复测 Audit：`target/phase9f4d-run-owner-consumer/c4329797-381b-4cce-befd-76b153d5f01b/audit.jsonl`。临时 PostgreSQL 已停止并清理；未创建 NATS 或 Multipass 实例。 |
| D17 SQLx | PASS，`profile_master_cas_outbox_rls_and_resolver` 1/1（0.13s）。实际运行使用 Rust 1.98.1 diagnostic toolchain 与 `--ignore-rust-version`。 |
| `platform-messaging-runtime` crate unit tests（含 D17） | PASS：default 27/27；`--no-default-features --features durable-quarantine` 21/21。ignored SQLx target 编译成功；无 PostgreSQL runner 时保持 ignored。 |
| Cargo all-target checks | PASS，`platform-messaging-runtime` default 与 `--no-default-features --features durable-quarantine` 两种配置；均用 Rust 1.98.1 `--ignore-rust-version`。default 配置有 3 条既有 `domain-kms` deprecation warnings。 |
| Rust 1.99.0 正式 gate | NOT RUN；当前安装列表无 1.99.0。1.98.1 诊断结果不替代正式门。 |
| 格式 | PASS：本次变更 Rust 文件使用 rustfmt `--check --config skip_children=true`。包级 `cargo fmt --check` 仍被未修改的既有 `admission_delivery.rs` 排版差异阻断；该文件上的意外格式改动已还原。 |
| Python / patch hygiene | PASS：automation runner `py_compile`、`git diff --check`。 |
| Graph headers | PASS：`cgg_guard.py --changed --stamp` 检查 5 个变更源码文件；两个 SQL header 另行验证 block 数量、canonical repository path 和按 guardian canonicalization 计算的 freshness hash。 |
| Automation registry | PASS，registry checker 0 errors；全仓既有 197 warnings，主要是历史脚本未入索引，不把它们写成新 D17 错误。D17 runner 行及 §4.88/v0.95 已更新。 |
| CodeGraph coverage | 工具未提供 `check_index_coverage`；查询当前未返回新 D17 文件，因此不声称图覆盖完整。保留既有 `.codegraph/` 用户数据，没有初始化或重建索引。 |
| CodeRabbit | 未能启动：`coderabbit auth status` 返回 `/bin/bash: /root/.local/bin/coderabbit: Permission denied`，无独立 CodeRabbit review 结果。提交前 secret-pattern scan 检查 18 个变更文件，0 个匹配。 |
| Cargo release/workspace gates | 未运行：workspace all-targets、Clippy、release tests、release build、doc 与 bench `--no-run` 不计为已通过。 |

## §3 已知缺口

1. Rust 1.99.0 尚未安装；本报告所有 Cargo 证据都是 Rust 1.98.1 诊断运行。
2. `AuthenticatedTransportSession` 是信任上下文注入 seam，不构成生产认证；服务身份 provider、证书/credential rotation 与生产 composition 未接入。
3. 当前数据库测试身份不证明目标数据库 runtime grants、生产部署角色、RLS identity mapping 或迁移 rollout。
4. Resolver 返回的是读取时的 immutable snapshot。下游每次 owner mutation/terminal ACK 仍须复验当前 profile、service identity 与授权；实现真实 native provider 前不得把 snapshot 当长期授权缓存。
5. 本阶段没有 native durable append/pull/ack provider、可靠恢复、poison replay body、receiver consumer 或完整应用 NATS dependency removal。
6. Outbox 已在 owner transaction 写入，但没有生产 consumer、BI/Benchmark projection、告警与 retention/archival 运维策略。
7. 没有 RSS、吞吐、跨租户公平调度、并发压力、内存缓存或 Rust 桌面 UI 性能测量。
8. 没有 Multipass/K3s 实例启动；当前测试实例上限仍为 2。VM readiness 也不会替代 Run binding、sandbox、原生 Hook、取消恢复和独立验证。
9. D17 不实现 Schedule worker、Run reservation/RunEvent consumer 或 task-card CLI execution；这些功能必须继续 fail closed。
10. 代码库 `.codegraph/` 是已有未跟踪数据；由于覆盖工具不可用，没有重建或覆盖它。

## §4 子代理失败接手清单

本 D17 切片没有新派发子代理；无失败 RPC、遗失 worker patch 或待接手 branch。实现提交 `7ef61ee1` 在隔离 worktree `codex/phase9f4c-admission-quarantine` 完成审查，并已 fast-forward 集成到本地 `dev`；本报告只记录这笔定向集成，不代表其他本地分支已收敛。本机来源仓 `E:/AssetsLake` 保持只读，D17 仅适配字段语义，没有源代码复制。

## §5 守门规则

1. Profile Master、Audit、Outbox 由 Platform Messaging owner 持有；不得跨 owner 共享写事务。
2. 查询身份来自已认证 deployment/tenant context，不接受消息 body、UUID 参数或 profile digest 作为认证证明。
3. 新 revision 必须按 profile/tenant/deployment 维度 CAS；冲突不得覆盖当前 revision。
4. Exact idempotency replay 只有在 actor、scope、request、correlation、生命周期、definition 和 digest 全部相同时返回原 revision。
5. Master payload 不可变；唯一允许的旧 revision 更新是事务内关闭 `valid_to` 并同时写入精确 successor。
6. Audit/Outbox append-only、无物理删除和 TTL purge；触发器以同一数据库事务写入。
7. 所有三张表强制 deployment+tenant FORCE RLS；runtime roles 使用最小权限，resolver 只有受限读取。
8. Profile/Outbox 不存 credential、private key、token 或 secret-manager reference；Master 只可存有界 opaque secret reference。
9. Profile digest 由 canonical typed 内容确定性计算并在读取时验证；未知 schema/digest 必须拒绝。
10. Provider、endpoint、log generation policy、sender、recipient 和 message kind 均有长度/集合上限；禁止用户控制任意 broker URL。
11. 初次 claim 固定 provider/version/profile digest/endpoint；旧 Work 重试不得静默切换 profile。
12. 任何持久化副作用或 ACK 前复验当前身份、授权、profile generation 与 owner fence；撤权和未知提交结果 fail closed。
13. Cursor 仅在同一 provider/profile/endpoint/log generation 内解释，禁止跨 generation 比较或复用 dedupe 窗口。
14. Outbox 版本化包含 trace/causation/idempotency 身份；BI 消费只能经授权 owner API/projection，不跨 owner 直接改表。
15. AssetsLake 可按模块直接参考/复用；每次实际复制仍固定源 HEAD、目标文件 hash、源工作区、权属、许可证/NOTICE 与依赖闭包。AGPL-3.0 不等于禁止商用，义务须按实际修改/链接/分发/网络提供方式落实。
16. AssetsLake dry-run/no-broker relay 不作为 durable acceptance；source worktree 保持只读；`.codegraph/` 用户数据不得重建、暂存或提交。
17. D17 disposable PostgreSQL 与 Rust 1.98.1 诊断结果不等于正式 Rust 1.99.0 gate、生产身份、native transport、BI 或整个 9F4C phase 完成；测试 Multipass inventory 上限仍为 2。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis：owner 边界、profile pin 与 native transport 替换契约已与代码/DB 事实核对；生产身份和消息路径仍开放。 |
| SRE Lead | Mavis：D17 disposable PostgreSQL 71/71、migration replay 与 SQLx 1/1 通过；生产部署、恢复、容量和性能未验收。 |
| 平台 | Mavis：Rust 类型、SCD2 Master、append-only facts 与版本化事件元数据已实现；Rust 1.99.0 和 target DB grants 未验收。 |
| 评审主持 | Mavis：定向验证与图头检查通过；CodeRabbit 因 Permission denied 未运行成功，不记为 review pass。 |
| PM | Mavis：仅关闭 D17 profile owner/schema/resolver 首片，继续推进 Worktree-first 产品其余 phases；Schedule/Run execution 保持 fail closed。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-12 | Ulysses — Mavis 接手审核 | 记录 D17 Transport Profile Master/resolver、AssetsLake metadata 适配、验证证据和生产闭环缺口。 |
| v0.2 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 自审修正 Rust 与 PostgreSQL 标识字段首字符验证差异；复跑 default 27 项、durable-quarantine 21 项 crate unit tests、两种 all-target 配置及 71 步 disposable PostgreSQL/SQLx 验收，更新审计路径与文件扫描数。 |
| v0.3 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录实现提交 `7ef61ee1` 已定向 fast-forward 到本地 `dev`；保留其他分支与 worktree 状态。 |
