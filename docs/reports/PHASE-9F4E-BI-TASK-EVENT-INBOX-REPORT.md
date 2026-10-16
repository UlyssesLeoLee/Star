# PHASE-9F4E — BI Task-event Inbox 阶段报告

> **状态**：🟡 持久化切片完成；生产消费者未启用，9F4E 的端到端闭环仍开放
> **日期**：2026-10-16
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **审批者**：架构师 (Mavis 接手 agent per DEC-008)
> **范围**：首个 owner-local BI Task-event Inbox、immutable fact 与 payload-free conflict incident

## §0 目的

交付首个消费 Task Outbox 的 BI-owner 持久化边界：接收受约束的 Task event envelope，使用 route pin 防止身份/路径/字节变化，在 BI owner 的单个事务中写 durable Inbox receipt 与 immutable fact，并在事件身份冲突时只追加不含 payload 的 incident。此阶段不等价于生产消息接收、BI 投影或跨 App 分发完成。

## §1 改动矩阵

| 层 | 文件 / 改动 | 状态 |
|---|---|---|
| Workspace | `Cargo.toml` 注册 `crates/bi-owner-runtime` | 已修改，Rust build 未运行 |
| BI owner | `crates/bi-owner-runtime/src/task_event_inbox.rs`：严格 envelope/schema、16 KiB wire 与 8 KiB JSON payload 上限、route pin、exact replay、冲突隔离、固定 function store | 源码切片；没有 trusted transport factory |
| 数据库 | `db/migrations/2026-10-16-bi-task-event-inbox.sql`：Inbox、immutable fact、payload-free incident、RLS/WORM、受限 function grants | PostgreSQL 18.6 disposable runner 双次 apply 与 fixture 通过；target DB 未验 |
| DB fixture | `db/tests/bi-task-event-inbox.sql` | runner 中执行通过 |
| Runner | `scripts/automation/phase9f4e_bi_task_event_inbox.py` | loopback PostgreSQL 18.6 gates 通过 |
| 需求与设计 | requirements v6.80、Task SRS v0.20、basic design v6.79、Task BD v0.13、Task DD v1.34、Group DD v5.07、Data Design v2.27、实施计划 v7.08 | 增补 9F4E 状态、边界、AC、W/T/M 分类与后续门禁 |

BI 的三个新增持久化表均归属 Transaction；使用 tenant `FORCE ROW LEVEL SECURITY`、append-only/WORM guard。`star_bi_runtime` 不持有直接表 DML 或 schema CREATE，仅调用固定函数。Inbox receipt 与 immutable event fact 在同一个 BI-owner transaction 中提交；exact replay 返回相同稳定 receipt；冲突不返回 receipt，incident 不复制 raw payload。

## §2 验证摘要

| 验证 | 实测结果 |
|---|---|
| PostgreSQL migration / fixture | PostgreSQL 18.6 isolated loopback runner exit 0；重复应用 migration，并运行 fixture。 |
| 幂等和冲突 | 首次提交、exact replay、租户内内容冲突、跨租户 event identity 冲突均按预期处理。 |
| 权限与安全边界 | 验证 runtime 无表级 DML / schema CREATE，固定函数调用、tenant FORCE RLS 与 UPDATE/DELETE/TRUNCATE 不可变场景。 |
| Rust formatting/build/tests | 未运行：当前 shell 无法解析 `cargo`、`rustc`、`rustfmt`。不声称通过。 |
| Target DB / production identity | 未运行；目标数据库、生产登录身份与 grants 尚未部署。 |
| Concurrent crash/recovery / network receive | 未运行 / 未实现。 |

PostgreSQL 结果只证明本地 disposable fixture，不证明生产 deployment、生产身份策略、并发行为或网络 receiver。之前 runner 首次启动产生的 ignored 临时目录未被删除；环境拒绝清理该目录，本报告不以其它删除方式绕过该限制。

## §3 已知缺口

1. `AuthenticatedTaskEventPrincipal` 构造器为 crate-private，目前没有可信 transport adapter/factory 能创建它；无受认证 receiver/API/network wiring，因此生产 BI ingestion 必须保持关闭。
2. 没有 BI projection/read model、Benchmark、Run App 或其他 consumer projection，也没有 fan-out 与 accepted settlement。
3. immutable fact 当前永久保存 raw event payload；没有获批的 retention、purge、容量上限或隐私分类策略。这是 production activation blocker。
4. Rust 1.99.0 build/fmt/tests 未验；不能证明类型、格式、编译、内存上限或运行时行为。
5. Target DB 的 schema rollout、owner login/grants/RLS、并发写、崩溃恢复和重放尚未验收。
6. 16/8 KiB 上限约束输入大小，但没有形成 BI projection 的 RSS、批次并行、公平调度、取消/drain 或 benchmark 证据。
7. `TaskEventInboxStore` 只处理 BI owner-local commit；不发送、不验证跨 owner transaction、不改变 Task delivery accepted 状态。
8. 较早 runner 曾留下 ignored 临时 PostgreSQL 目录，删除操作被环境策略拒绝；实例已停止，目录遗留需要后续按受支持的清理机制处理。

以上缺口未关闭前，不得把 BI Inbox 说成 production-ready，也不得据此开放任务执行或 Schedule/Run delivery。

## §4 子代理失败接手清单

本阶段没有向子代理派发代码实现或测试任务，因而没有未落地的 worker commit 或需要接手的写入任务。之前参与的审查仅为只读核对，不作为本报告的实现或验证证据。所有可审查变更均保留在当前候选分支。

## §5 守门规则

| # | 守门项 |
|---:|---|
| 1 | BI owner 只写自身 Inbox/fact/incident；禁止跨 owner 表写与共享写事务。 |
| 2 | BI Inbox receipt 与 fact 必须在一个 owner transaction 中提交。 |
| 3 | exact replay 必须返回稳定相同 receipt，不重复写事实。 |
| 4 | 身份、tenant、Run scope、route、wire digest 或内容冲突必须不返回 receipt。 |
| 5 | 冲突 incident 不得复制或记录 raw payload。 |
| 6 | sender/receiver service identity 必须来自受信认证上下文，不得接受调用者伪造 UUID。 |
| 7 | route pin 必须固定 transport/profile/destination generation/schema/exact wire digest；重试不得静默切换 provider。 |
| 8 | 只接受严格支持的 envelope/schema 版本；未知字段或不兼容版本 fail closed。 |
| 9 | wire 最大 16 KiB、payload 最大 8 KiB；所有反序列化、复制、队列与并发字节预算都需有界。 |
| 10 | 所有 BI 持久化表必须 tenant FORCE RLS；WORM 数据不得 UPDATE/DELETE/TRUNCATE。 |
| 11 | runtime 角色仅获得固定函数权限；禁止直接表 DML 和 schema CREATE。 |
| 12 | Broker/socket ACK 不能替代 BI transaction commit receipt，也不能结算其他消费者。 |
| 13 | raw payload retention/purge/privacy 与存储容量策略获批并落地前，BI production receiver 保持禁用。 |
| 14 | 每种 consumer 有自己的 Inbox、幂等键、version fencing、projection receipt 和恢复语义。 |
| 15 | 并发、崩溃、重放与 target DB identity/RLS 均需真实门禁；disposable fixture 不替代生产验证。 |
| 16 | Rust 1.99.0 build/fmt/tests、取消/drain、公平调度与 RSS 证据补齐前，不得声称 Rust runtime 或并行能力已验收。 |
| 17 | 本阶段不改变 Project → Run → Worktree → Branch 导航；不将 BI schema/consumer 改成 Worktree owner 或新导航层级。 |

## §6 签字栏

| 角色 | 审阅 |
|---|---|
| 架构 | Mavis 接手审核；生产 transport/consumer 架构仍有 §3 所列缺口 |
| SRE Lead | 待目标 DB、并发、恢复与资源预算门禁 |
| 平台 | Mavis 接手审核；只接受本报告列出的 disposable PostgreSQL 18.6 证据 |
| 评审主持 | Mavis 接手审核 |
| PM | Mavis 接手审核；9F4E 端到端未关闭 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 改动 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 BI owner Task-event Inbox/fact/conflict source、隔离 PostgreSQL 门禁、跨文档同步、验证边界和生产阻断项 | Phase 9F4E 持久化切片进入阶段收敛 |
