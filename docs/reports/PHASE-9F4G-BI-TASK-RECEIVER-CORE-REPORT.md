# PHASE-9F4G-BI-TASK-RECEIVER-CORE-REPORT

## §0 目的

为 BI Task event 消费链路增加 provider-neutral、单条有界接收核心，并明确来源认证、owner durability、ACK、隔离和取消之间的顺序。此报告仅覆盖 9F4G 的 receiver core 源码切片，不表示 9F4G 或完整 BI/Agent 产品闭环已完成。

## §1 改动矩阵

| 范围 | 改动 | 结果 |
|---|---|---|
| `bi-owner-runtime` | 增加 `TaskEventReceiver`、transport/authorization/quarantine/Inbox ports 和 `TaskEventQuarantineStore`；固定 route/origin 输入；单并发 semaphore；deadline/cancellation；commit/reverify/ACK；exact replay 与 poison handling | receiver/store 源码已写入；Rust 编译与测试未执行 |
| BI Inbox | 为现有 `TaskEventInboxStore::commit_exact_wire` 实现 receiver commit port；复用 `validated_event_id` 与 16 KiB bound | 调用边界已接入，不代表生产 auth/DB composition |
| BI quarantine | 新增 metadata-only append-only Transaction 表和固定 `SECURITY DEFINER` writer；完整 route/source cursor idempotency；message ID/digest/length/fault replay check；本表无 runtime 直接权限，本迁移授权调用固定 writer | isolated PostgreSQL 18 migration/RLS/grants/WORM/exactness/确定性唯一键锁等待 gates pass；生产身份和 target DB 未验证 |
| Requirements / basic / detailed design | 同步需求 AC-23、基本设计 §6.4/§16.73、详细设计 §14.13.6、Data Design §4.13.29、W/T/M 表与实施计划 §6.153 | 文档状态明确保留 trusted adapter、生产与 Rust 门禁 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `git diff --check` | PASS |
| Cypher Guardian `--changed --stamp` | PASS（4 个受检查源码文件） |
| Rust 1.99.0 fmt/check/test/Clippy | 未运行；本机 `cargo`、`rustc`、`rustfmt` 不可用，不能宣称通过 |
| Isolated PostgreSQL 18 | PASS：新增 quarantine migration 双次应用；missing/mismatched tenant GUC、oversize 输入、role/function privileges、NOINHERIT/owner SET ROLE denial、FORCE RLS/WORM、exact replay、changed digest/message ID/length/fault、route separation，以及两独立会话确定性唯一键锁等待竞争 |
| CodeRabbit | 未完成：本机 `coderabbit.cmd` 调用 `/root/.local/bin/coderabbit` 被 permission denied（exit 1）；不视为独立 review 通过 |
| Target DB / production RLS | 未部署/验证；隔离数据库证据不证明生产 grants、身份、备份/WAL 或 crash/PITR |
| Runtime/transport smoke | 未运行；没有 production transport、trusted service identity、authoritative route-pin registry 或 receiver composition |

## §3 已知缺口

1. Durable quarantine adapter/schema/function are source-level and passed disposable PostgreSQL 18 gates; they are not deployed to the target database.
2. No production transport adapter, trusted service identity, authoritative route-pin registry, receiver composition or endpoint is wired. The SQL function validates route metadata but trusts it from the Rust adapter; `app.tenant_id` is a call-scope guard and does not protect against a compromised runtime that can set the GUC.
3. Target DB roles/grants/RLS, crash/restart, PITR/backup/WAL and production retention/privacy policy remain unverified.
4. No BI query API/UI, Benchmark projection, dispatcher accepted settlement, fan-out or per-consumer replay/gap recovery is implemented.
5. Same-cursor conflicting retries return no receipt and do not append a separate incident row; durable incident visibility and operator alerting remain open.
6. The append-only quarantine table has no quota, partitioning or production retention/restore policy; capacity and privacy gates remain open.
7. No RSS, latency, fairness or multi-consumer load evidence; Rust 1.99.0 gates remain unrun.
8. BI ingress, delivery acceptance and Task/Run execution remain fail closed.

## §4 子代理失败接手清单

本次收尾复用两项独立只读审查：数据库/Rust adapter 审查和需求/设计文档一致性审查均完成；权限描述范围与角色/并发验证缺口已按审查收窄并补测。没有失败的子代理交接；所有本地改动仍在 `codex/phase9f4g-bi-query` 隔离 worktree。

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
| 18 | 数据库只校验 route 元数据格式和内部一致性；不得声称该函数独立认证 transport 或从 route registry 验证 pin。 |
| 19 | `app.tenant_id` 是 tenant call-scope/RLS 上下文；不得把可由 runtime 设置的 GUC 描述为受攻击者约束的认证屏障。 |

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
| v0.2 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 加入固定 BI owner SQLx quarantine store、append-only T schema、完整 route/cursor exactness；记录隔离 PostgreSQL 18 double-apply、RLS/grants/WORM、输入和两种并发 race；同步 SRS/BD/DD/basic/requirements/Data Design；明确 trusted route/GUC 边界及 production/Rust gates | durable metadata-only quarantine persistence slice 与并发 PostgreSQL verification |
| v0.1 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9F4G BI Task event receiver core 的实现范围、文档同步、未运行 Rust gates 与生产阻断项 | 增加 provider-neutral receiver core source slice |
