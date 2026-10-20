# WLD-6 BI / Benchmark closure audit

**Status:** bounded, read-only source audit; implementation and production acceptance remain open
**Branch / baseline:** `codex/wld6-bi-benchmark-audit-20261020` / `253f52d8c389e3ee19d37833ed502287032b32e9`
**Reviewer:** Mavis 接手审核
**Scope:** Task/Run/Worktree/Hook/Schedule/Loop event delivery into BI and Benchmark

## 0. 目的

核对跨 App 事件从业务 owner、transport 到 BI Inbox、事实投影、查询/API/UI 的闭环状态；区分源码契约、隔离测试资产和生产验收。此报告只基于本 lane 的有界只读检查，不修改产品代码、migration 或数据库。

## 1. Owner 流程与实现矩阵

| Owner / 面 | 现有实现与证据 | 状态与边界 |
|---|---|---|
| Task source owner | `db/migrations/2026-10-16-task-lifecycle-outbox.sql:5-49,50-143` 把 metadata 与 lifecycle 分成独立版本流；lifecycle audit insert 在同一 owner transaction 写入 `task_run_outbox`，要求当前 Project + Run-owned Task。支持事件仅 `task.created/updated/owner_changed/task.lifecycle.changed`。payload 上限由 `2026-10-02-task-run-owner.sql:60` 的数据库约束及 `task_outbox_delivery.rs:97,719` 的 Rust 校验限制为 8 KiB。 | 有 owner-local source Outbox 与事件契约源码；目标 DB、实际 producer 流量和持久化仍未在本审计验证。该契约不覆盖一般 Run、Worktree、Hook、Schedule 或 Loop 事实。 |
| Task delivery owner | `2026-10-16-task-run-outbox-delivery.sql:37-88,104-131,228-305` 定义 Run-scoped 有界 delivery work、lease、最大 12 次尝试、7 天 work retention 及只读的 append-only terminal receipt；claim 限制 batch 1–8、lease 30–300 秒。`task_outbox_delivery.rs:400-450` 执行 bounded claim，`:453-525` 固定 route/profile 和 exact-wire digest，`:527-565` 只暴露 retry/dead。 | Store 只提供底层操作，不是运行中的 dispatcher。SQL `:307-328` 明确拒绝 `accepted` settlement；没有经过认证且绑定到本事件的 BI Inbox receipt 时，不能把 transport append 当作业务接收成功。 |
| Route / transport | `2026-10-16-task-run-outbox-route-pin.sql:6-8,11-50` 将不可变 sender/receiver、provider/profile revision+digest、endpoint/destination generation 和 wire hash pin 下来，明确不保存 payload、wire bytes、URI 或凭证。Native transport 有 exact-append client/receipt；平台 log migration `2026-10-12-platform-native-transport-log.sql:95-164` 保存受限 wire、lease、delivery receipt 与 receipt Outbox。 | Transport primitives 和持久化 schema 分别存在。对 Task 的 sender 到该 transport、以及从 transport 到 BI receiver 的生产绑定未确认；平台 `message.accepted/acked` 只是 transport receipt 事件，不等于 BI Inbox commit。 |
| BI receiving owner | `task_event_receiver.rs:389-445,447-505` 是显式调用、单 in-flight 的 receiver；先验证来源，再 commit BI Inbox，commit 后重验来源，再 ACK。quarantine 路径须先持久化 metadata receipt，再 terminal disposition（`:508-535`）。`task_event_inbox.rs:331-420` 在 BI owner transaction 中调用 `bi.commit_task_event_v2` 并于事务 commit 后返回 receipt。 | 有安全顺序与静态 port 契约，但 receiver 构造本身不启动 worker。对 `TaskEventReceiveTransport` 的全仓定向搜索仅发现同文件测试 fixture 实现；未找到生产 transport adapter。认证 resolver / endpoint / worker 启动和生命周期没有生产证明。 |
| BI Inbox / facts / quarantine | `2026-10-16-bi-task-event-inbox.sql:52-94,96-118` 定义幂等 receipt 与事件事实；`2026-10-16-bi-task-event-projection.sql:25-140,146-206,210-289` 通过 DB 函数验证并投影 allowlist，替换并删除 raw fact 表；fact 具有 tenant FORCE RLS 和 WORM。quarantine schema `2026-10-17-bi-task-event-quarantine.sql:18-66` 仅存 route/cursor、digest、wire length 和固定 fault code，不存 wire/payload。 | 原子 Inbox/fact 与 metadata-only quarantine schema 已实现为源码。`docs/data-design/phase9f4g-bi-task-quarantine-wtm.md:22-35` 明确指出 BI retention/privacy policy、目标角色/grants、production identity、备份/WAL 与运行时门仍开放；没有 target DB 验收。 |
| BI query / API / UI | `task_event_query.rs:170-199,241-312` 提供按 tenant/Project/Run 的有界 keyset page，coverage completeness/freshness 只有 `Unknown`。REST `bi_analytics.rs:112-152` 先授权 actor 与 Run-owned scope，再查可选 BI port；缺配置或超时返回 feature-unavailable。`main.rs:59-87` 仅在 `BI_DATABASE_URL` 存在时安装 port。`RunBiPanel.tsx:48-75` 明示 coverage 未知、空页不代表事件已全部送达、当前没有完整分析或 Benchmark 结果；每页最多 50 条并限制游标历史。 | 有受控的 Task 事件观察面，不是完整 BI/Benchmark 产品。它只显示已观察事实；无 authoritative checkpoint，无法说明完整性、新鲜度、lag 或缺失区间。 |
| Hook / Schedule / Worktree / Loop 事实 | 当前 WBS `WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md:31-32,43-47` 将 WLD-6 定义为跨 owner 事实闭环，并把 WLD-2/3/5 与基础设施事实列为输入依赖。现有实现计划记录独立 Hook ledger/summary、Schedule owner Outbox 与 Run/Worktree binding facts（例如 `WORKTREE-GROUP-IMPL-PLAN-001.md:739-748,766-788,1105,1463`）。 | Task BI Inbox 的 schema/projection 只接受上述 Task metadata/lifecycle 事件；本次未找到这些独立 owner 事实进入该 BI consumer 的完整链路。Loop 到 BI 的版本化事实契约也未在此有界检查中确认。不得把独立 ledger/API 当作已汇总的 Run BI。 |
| Benchmark | 当前 Run 面板将“完整分析或 Benchmark 结果”标为尚未形成（`RunBiPanel.tsx:48-50`）；当前 query item 是 Task 状态事件投影。 | 未发现有证据支持的 Benchmark read model、固定 cohort/formula version、完整数据窗口或结果验证；目前不能标为已实现。 |

## 2. 验证摘要

- 使用 `rtk proxy rg` 对指定 owner、transport、migration、测试与 UI 文件做字面搜索，并以带行号的源码片段复核关键路径；未更改应用代码或数据库。
- 找到 receiver 单元测试：`task_event_receiver.rs:822-932` 覆盖 commit/来源复验先于 ACK、撤权不 ACK、quarantine failure 不 terminally ACK、取消无 transport 副作用。找到 PostgreSQL integration test `task_event_inbox_postgres_tests.rs:87-177` 与 SQL contracts `db/tests/bi-task-event-{inbox,projection,quarantine,query}.sql`。这些是测试资产；本审计未运行它们。
- `scripts/automation/registry.md:56` 记录过隔离 PostgreSQL 18.6 gate 的既有结果，并同时保留目标 DB、生产 identity、backup/WAL 与 Rust 版本门禁。该历史记录不是本轮或目标环境验证。
- 本轮未连接或写入任何数据库；未执行 migration、cargo test、SQL runner、端到端传输或 UI 运行。
- CodeGraph 对当前 lane 未提供匹配且覆盖完整的项目索引；父代理确认当前工具集合没有 `check_index_coverage`。因此采用有界源文件 fallback；“未发现”结论只限于所述定向搜索，不作为全仓不存在证明。

## 3. 已知缺口与下一步工作

按依赖顺序建议：

1. **WLD-6A — 冻结事件目录与身份契约。** 按 owner 列出 Task、Run/Worktree binding、Hook、Schedule、Loop 事实的 event ID、schema/profile version、tenant/Project/Run scope、source cursor、可投影字段、retention 与 replay 语义；每种 owner 保留各自 Outbox/事务。当前 BI consumer 只允许 Task 事件，不能先把多 owner payload 塞入同一未分型结构。
2. **WLD-6B — 完成 Task 端到端 dispatcher 与真实 transport adapter。** 将 delivery claim → immutable route pin → exact append/pull → authenticated BI Inbox receipt → attempt-fenced accepted settlement 串起来。首先补上被 SQL 明确禁用的 accepted receipt 校验；覆盖 worker 生命周期、有界公平调度、取消/drain、租约过期、重试、source revocation 和 endpoint generation drift。只接受 BI owner 已提交的 Inbox receipt，transport ACK 不可代替。
3. **WLD-6C — 接入其它 owner 的独立 Inbox / allowlist projection。** 按 WLD-6A 的冻结 contract 分批接入 Run/Worktree、Hook、Schedule、Loop；接收方本地事务内写 Inbox 与事实，再由本 owner 的投影接口读取。不得跨 owner 共写事务。为每个 consumer 保留 exact replay 稳定 receipt 和冲突隔离。
4. **WLD-6D — 建立 gap/checkpoint 与可恢复读模型。** 以 source cursor + destination durable receipt 形成 per-consumer authoritative checkpoint；实现 bounded replay/rebuild、死信/隔离事件的有审计重放、lag 与缺口展示。Coverage 在 checkpoint 未覆盖窗口、来源不可用或存在 quarantined range 时继续显示 unknown/partial。
5. **WLD-6E — retention、隐私与运行门。** 明确 source Outbox、delivery Work、Inbox/facts、quarantine、native log 的保留期和清理责任；目标 DB owner/runtime grants、FORCE RLS、backup/WAL/PITR、授权身份和故障恢复逐 owner 验证。payload 继续受限并默认不进入 quarantine。
6. **WLD-6F — Benchmark read model。** 只有在 6A–6E 的事件目录和 checkpoint 可证明输入范围后，再版本化 cohort、公式、维度和指标；验证可重复结果、跨 Run 隔离、部分窗口标记和历史重建。不得从分页 UI 或 observed events 推导全量 KPI。
7. **WLD-6G — 性能与发布验收。** 在 Rust 1.99.0 和最多 2 个受测 K3s 实例约束下测量 bounded queue/bytes、per-Run fairness、peak RSS、p95/p99 lag、取消恢复、重放成本与 UI 分页内存；将目标环境结果回填 WBS/设计文档后才关闭 WLD-6。

主要发布阻断：Task accepted settlement 未实现、BI receiver 未接生产 transport/worker、其它 owner 事实未进入 consumer、coverage 恒为 unknown、无 Benchmark 结果模型，以及 retention/production identity/DB 运维验收未完成。WLD-6 未关闭前应保持 BI ingestion 的执行门关闭；查询 UI 可以展示受限 observed facts 并明确其证据范围。

## 4. 子代理失败接管清单

本 lane 为单人只读审计，未派发子代理；无子代理失败或未接管产物。审计边界外的 WLD-2/3/5 与基础设施输入由 WBS 依赖关系管理，不在此报告中代替验收。

## 5. 守门规则

1. 源 owner 仅在自己的业务事实事务中追加 Outbox。
2. 不允许 Run、Transport、BI 或其它 owner 跨数据库共享写事务。
3. 事件必须有版本化 schema、稳定 event ID 与显式 tenant/Project/Run scope。
4. Task Outbox 仅可标记 Task 事实；独立 owner 事件不得伪装为 Task lifecycle。
5. 首次发送固定 sender、receiver、provider/profile digest、endpoint/destination generation 和 exact-wire digest。
6. Delivery claim、lease、attempt、payload bytes、并发数和 deadline 必须有界并支持背压。
7. 只有受信任 BI owner 的已提交 Inbox receipt 可作为 source accepted settlement 证据。
8. Native transport append receipt 或 ACK 不得等同于 BI Inbox transaction commit。
9. 接收先验证来源和 scope，在 BI owner transaction 写 Inbox+projection，commit 后复验并 ACK。
10. exact replay 必须返回原 receipt；身份相同但 wire 内容变化时 fail closed 并隔离。
11. quarantine 只保存必要 route/cursor/digest/fault metadata；必须先 commit quarantine receipt 再 terminal disposition。
12. BI query 必须 tenant/Project/Run scoped、allowlisted、keyset paginated、no-store 且带稳定 cursor。
13. 没有 authoritative checkpoint 时 completeness 与 freshness 必须为 unknown；发现缺口时最多报告 partial。
14. Benchmark 结果必须绑定版本化 cohort/formula 与已验证的输入窗口，不可从 observed facts 推断全量。
15. Source, delivery, Inbox/facts, quarantine 和 transport log 各自的 retention/privacy/backup/WAL policy 必须明确。
16. Target DB owner/runtime role、grants、FORCE RLS、生产身份、恢复与跨 owner crash/replay 必须实测；隔离 fixture 不代替目标环境。
17. 不以源码、测试 fixture、seed、REST 契约或 UI 截图声称生产闭环；未达到完整验收前保持 ingestion/执行门关闭并同步 WBS。

## 6. 签字栏

| 角色 | 审阅记录 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）— 已审阅本次有界审计；生产闭环未批准 |
| SRE Lead | 架构师（Mavis 接手 agent per DEC-008）— 已审阅目标 DB / 恢复缺口；未做环境验收 |
| 平台 | 架构师（Mavis 接手 agent per DEC-008）— 已审阅 owner / transport 边界；未确认生产接线 |
| 评审主持 | 架构师（Mavis 接手 agent per DEC-008）— 已审阅证据范围与限制 |
| PM | 架构师（Mavis 接手 agent per DEC-008）— 已审阅阶段范围；WLD-6 / WLD-7 保持开放 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 WLD-6 owner-by-owner 事件/BI/Benchmark 闭环审计；列出源码、测试资产、生产证明边界及下一步依赖顺序 | WBS WLD-6 bounded read-only audit brief |
