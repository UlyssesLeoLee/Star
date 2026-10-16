# PHASE-9F4G-BI-TASK-RECEIVER-CORE-REPORT

## §0 目的

为 BI Task event 消费链路增加 provider-neutral、单条有界接收核心，并明确来源认证、owner durability、ACK、隔离和取消之间的顺序。此报告仅覆盖 9F4G 的 receiver core 源码切片，不表示 9F4G 或完整 BI/Agent 产品闭环已完成。

## §1 改动矩阵

| 范围 | 改动 | 结果 |
|---|---|---|
| `bi-owner-runtime` | 增加 `TaskEventReceiver`、transport/authorization/quarantine/Inbox ports；固定 route/origin 输入；单并发 semaphore；deadline/cancellation；commit/reverify/ACK；exact replay 与 poison handling | 源码已写入，Rust 编译与测试未执行 |
| BI Inbox | 为现有 `TaskEventInboxStore::commit_exact_wire` 实现 receiver commit port；复用 `validated_event_id` 与 16 KiB bound | 调用边界已接入，不代表生产 auth/DB composition |
| Dependencies | 新增 Tokio 与 `tokio-util` CancellationToken 依赖 | Cargo 工具不可用，依赖解析未验证 |
| Requirements / basic / detailed design | 同步需求 AC-23、基本设计 §6.4/§16.73、详细设计 §14.13.6、主要求与实施计划 §6.153 | 文档状态明确保留所有生产门禁 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `git diff --check` | PASS |
| Cypher Guardian `--changed --stamp` | PASS（3 个 Rust source files） |
| Rust 1.99.0 fmt/check/test/Clippy | 未运行；本机 `cargo`、`rustc`、`rustfmt` 不可用，不能宣称通过 |
| PostgreSQL / target DB / RLS | 本切片未改 migration、未运行数据库；目标环境仍未验收 |
| Runtime/transport smoke | 未运行；没有 production transport/auth/quarantine adapter |

## §3 已知缺口

1. 没有可部署的 transport adapter、trusted service identity 或生产组合入口。
2. 没有 durable metadata-only quarantine database adapter；幂等键与 digest/fault conflict 当前只有 port contract。
3. 未验证 target DB、应用角色 grants、FORCE RLS、WORM、并发写、crash/restart、PITR/backup/WAL。
4. 未实现 BI query API/UI、Benchmark projection、dispatcher accepted settlement、fan-out 或逐 consumer replay/gap recovery。
5. 没有 RSS、latency、公平调度或多消费者负载证据；Rust 1.99.0 gates 未运行。
6. BI ingress、delivery acceptance 与 Task/Run execution 继续 fail closed。

## §4 子代理失败接手清单

本切片没有派发子代理，因此没有子代理失败接手项。所有本地变更与审查均在 `codex/phase9f4g-bi-query` 隔离 worktree 内完成。

## §5 守门规则

| # | 守门 |
|---|---|
| 1 | Project → Engineering Run → Worktree → Branch 和 hostless Run owner 不因本阶段改变。 |
| 2 | 只消费固定 route pin，不让 Worktree focus 或远端 Branch 改变 Task/BI owner。 |
| 3 | 传输身份与 route UUID/label 不能单独冒充认证事实。 |
| 4 | transport 必须在 payload 分配前限制 16 KiB wire。 |
| 5 | 不允许无界预取、无界日志或隐藏的内存队列。 |
| 6 | 每个 receiver 一次只处理一条，单实例至多一个 in-flight 调用。 |
| 7 | CancellationToken 与绝对 deadline 覆盖 pull 至 disposition 的整个调用。 |
| 8 | cancel/timeout 可以产生未知 commit/ACK 结果，不得据此虚构失败或 accepted。 |
| 9 | BI owner transaction commit 成功前不得 ACK。 |
| 10 | 每次 ACK 前再次核验 source origin、当前授权和 pinned route。 |
| 11 | Exact replay 返回原 receipt；重复 ACK/disposition 必须安全。 |
| 12 | Quarantine 仅存 route/origin/cursor、digest、长度和受限 fault，不存 wire/payload。 |
| 13 | Quarantine 按来源游标幂等；同键 digest/fault 变化 fail closed。 |
| 14 | Quarantine durable receipt 缺失、无效或写失败时不得 terminal disposition。 |
| 15 | 无 durable terminal support、授权失效或身份不明时保持 unsettled。 |
| 16 | 本阶段不启用生产 ingest、fan-out、accepted settlement 或 execution。 |
| 17 | 文档必须区分已写 source、已运行验证和未实现 production capability。 |

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9F4G BI Task event receiver core 的实现范围、文档同步、未运行 Rust gates 与生产阻断项 | 增加 provider-neutral receiver core source slice |
