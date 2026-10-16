# Phase 9F4D — Task Outbox Rust Store Slice Report

> 日期：2026-10-16
> 状态：🟡 source slice complete; compile, database, dispatcher and production gates remain open
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

记录 Task Run Outbox 的 Run-owner Rust persistence/wire-preparation slice，并核对实现与 requirements、basic design、Task SRS/BD/DD、Group DD、Data Design 和 implementation plan 的一致性。本报告只覆盖有界 claim、route/profile authorization、immutable route pin、exact-wire preparation、retry 与受控 dead-letter；不把 source code 视为可用投递闭环。

## §1 改动矩阵

| 文件/范围 | 改动 | 当前结果 |
|---|---|---|
| `crates/run-owner-runtime/src/task_outbox_delivery.rs` | 增加 bounded Task Outbox claim、envelope validation/wire preparation、route-pin readback、lease-fenced retry、invalid-envelope proof dead-letter 与 payload/wire redacted Debug | source slice；无网络发送、accepted settlement、receiver Inbox 或 consumer |
| `crates/platform-messaging-runtime/src/transport_profile.rs` | 增加独立 `event` message kind、profile digest tag 与已认证 context 的 direction/sender/recipient accessors | Rust source；数据库 migration 未运行 |
| `db/migrations/2026-10-16-platform-transport-profile-event-kind.sql` | 扩展 Profile message-kind constraint，允许显式 `event` 授权 | migration source；未在 PostgreSQL 执行 |
| `crates/run-owner-runtime/src/lib.rs` | 导出 Task delivery store/proof 类型并更新 Cypher relationship header | source wiring |
| requirements/basic design/Task SRS/BD/DD/Group DD/Data Design/implementation plan | 同步能力、授权边界、版本、限制和后续阶段 | 文档已对账；运行验收未关闭 |

## §2 验证摘要

- CypherGraph Guardian：`cgg_guard.py --changed --stamp` 通过，检查并盖章 3 个 Rust 源文件。
- `git diff --check` 通过，只有 Rust 文件的 CRLF→LF 行尾提示；Rust 编译、rustfmt、Clippy、单元测试均未运行，当前 PowerShell `Get-Command` 未找到 `cargo`、`rustc`、`rustfmt` 或 `rtk`。
- PostgreSQL migration/SQLx/RLS/grant/并发验证未运行。本地既有 Task delivery/route-pin migration fixture 的历史结果不覆盖本阶段 Rust store 或新增的 Profile `event` migration。
- CodeRabbit CLI 已尝试调用；安装 wrapper 转到 `/root/.local/bin/coderabbit` 后因 `Permission denied` 退出。该失败不是 clean review，剩余变更按本地源码与 diff 手工审查。
- 测试环境未创建 Multipass guest；既有两实例上限不受本 source-only slice 影响。

## §3 已知缺口

1. Rust 工具链不可用，代码尚未编译、格式化或做测试验证。
2. Profile `event` migration 未在隔离 PostgreSQL 执行；生产数据库迁移、应用角色 grants、FORCE RLS 与真实服务身份未验证。
3. 该 store 不发送网络消息，也不在每次发送前重新解析 Profile/endpoint generation。
4. 未实现 receiver owner 的 append-if-absent Inbox、durable receipt 验证和 accepted settlement。
5. 未实现 Run App、Schedule/Loop、Canvas、BI、Benchmark 各自的 Inbox/projection transaction、gap recovery 或 projection parity。
6. tenant claim 的 SQL 顺序不是 Run 间公平调度；无公平轮转、全局 in-flight byte budget、取消/drain worker 或 peak RSS 测量。
7. Route factory 的 destination label/generation 仍须来自 deployment-owned configuration；未来 sender 必须维持此来源约束并在实际发送前复验身份。
8. 历史 route-pin PostgreSQL fixture 不包含新 Rust store、新 Profile migration、生产 runtime grants 或并发测试，不能用作本阶段通过证据。

## §4 子代理失败接手清单

| 审查 | 结果 | 接手处理 |
|---|---|---|
| Task route/profile audit | 识别了 route identity 可由调用方传入和有效事件可被 caller-selected dead-letter 两项真实缺口 | 已改为从 `AuthenticatedTransportContext` 派生 sender/receiver，并要求 `event` Profile 授权；dead-letter 现在要求私有 invalid-envelope proof |
| Payload bound audit | 报告未在查询中发现 payload 上限 | 复核完整迁移链后发现 `2026-10-02-task-run-owner.sql` 已有 `octet_length(payload::TEXT) <= 8192` CHECK，后续 owner/lifecycle migrations 未删除该 CHECK；故否决“缺少 source DB byte bound”结论，不重复添加迁移。此结论来自 DDL 静态检查，目标 DB 当前状态仍未验证 |
| Receiver audit | 确认当前无 Task-specific receiver Inbox writer/accepted receipt path，并指出 quarantine 与恢复仍未实现 | 作为后续 dispatcher/receiver/consumer 阶段门保留；当前不开放 accepted |
| DB harness audit | 现有隔离 runner 不覆盖新 Task/Profile migrations；无法据此证明新 Rust store | 不复用其通过结果；新 migration 与 Rust DB 行为仍列为未验证 |

以上 agent 工作均为只读审查；没有子代理代码产出需要合并或丢弃。

## §5 守门规则

1. 导航与 owner scope 继续遵循 Project → Engineering Run → Worktree → Branch；事件 owner 不由 Worktree focus 或远端 Branch 改变。
2. Task business fact 与 Outbox source 仍须由 Task owner 在同一事务提交。
3. Claim batch 限制为 1–8；source payload 由数据库既有 8 KiB JSONB text constraint 限制。
4. Profile `event` 是独立 message kind，不借用 Schedule `Result` 或其他类别授权。
5. sender/receiver 由已认证 transport context 派生，不接受 Task payload 中的路由身份。
6. Profile 与 endpoint 的 deployment、tenant、identity、revision、digest 和 endpoint label 必须一致。
7. destination label/generation 只可来自可信 deployment configuration，不得来自任务卡、事件正文或用户请求。
8. 首个外发 route pin 必须不可变；retry 不得切换 provider、endpoint 或 recipient。
9. Retry/settlement 必须由数据库当前 worker、attempt 和 lease fence 判定。
10. Wire payload、credential、secret 或 endpoint URI 不进入 route pin，source payload/prepared wire 也不得通过 Debug/log 输出；当前 Debug 实现做 redaction。
11. Dead-letter 仅接受 envelope validation 失败生成的 proof；业务有效事件不能用自由 reason 丢弃。
12. 传输 ACK、socket write 或 broker persistence 均不等于 receiver owner durable Inbox commit。
13. Accepted 只能在可信 receiver 身份和 source/route/digest/lease receipt 绑定验证后启用；当前关闭。
14. 每个 consumer 独立提交 Inbox receipt 与自己的 projection/outcome，不跨 owner 写表或共享事务。
15. 不以 occurred_at 或 UUID 顺序作为 commit order/high-water cursor；replay 必须有界且可对账。
16. 正式 Rust baseline 为 1.99.0；当前缺工具链时不得声称 compile/fmt/test 通过。
17. Multipass 测试 inventory 最多两个实例；本 slice 未启动虚拟机，也不改变该限制。

## §6 签批栏

| 角色 | 审核决定 | 范围/限制 |
|---|---|---|
| 架构师 | 🟢 Mavis 接手终审 | Run-owner 与 transport-profile 授权边界；生产 dispatch 仍关闭 |
| SRE Lead | 🟢 Mavis 接手终审 | 数据库/迁移/恢复验证尚未运行，不代表环境门通过 |
| 平台 Lead | 🟢 Mavis 接手终审 | Rust source contract 静态检查；工具链缺失，未签核编译质量 |
| 评审主持 | 🟢 Mavis 接手终审 | 文档及 Cypher relationship header 对账；CodeRabbit 未完成 |
| PM | 🟢 Mavis 接手终审 | 只交付 source slice，不宣称业务闭环或商业产品可用 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 bounded Task delivery store、独立 Profile `event` authorization、authenticated route identity、exact-wire pin、proof dead-letter 与未关闭的 Rust/DB/dispatcher/receiver/consumer/fairness gates | ERUN-P3.3 Rust delivery source 实现与文档对账 |
