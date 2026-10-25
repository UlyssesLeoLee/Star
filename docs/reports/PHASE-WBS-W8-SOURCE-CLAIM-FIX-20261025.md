# PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025：跨终态 source position 唯一认领

> 状态：实现候选，等待独立审查与 PostgreSQL 集成验证；W8 保持 OPEN。
> 工作分支：`UlyssesLeoLee/w8-source-position-claim-fix-20261025`
> 基线：`0ae310440e446f530a210ec413efb526cb038d8f`
> 范围依据：`docs/briefs/WBS-W8-SOURCE-CLAIM-FIX-20261025.md`（提交 `342edf0`）。

## §0 目的

修复 Inbox 与 quarantine 分别认领 provider-neutral source position 时缺少跨终态唯一约束的问题。一个 tenant、consumer、provider、log、generation 下的 sequence，以及非空 opaque cursor，必须由同一 owner-local append-only claim 表唯一认领。Inbox receipt/source-origin 与 quarantine receipt 在各自原有事务中插入时触发共享认领；冲突使该事务回滚，接收器因此不能授权第二种终态 ACK。

本阶段只处理 W8 的 source-position arbitration P1。此报告不代表 W8 整体完成或可发布。

## §1 改动矩阵

| 登记路径 | 改动 | 状态 |
|---|---|---|
| `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql` | 增加共享 claim 表、迁移期跨表 sequence/cursor 冲突检测、既有数据回填、tenant FORCE RLS、owner-only grants、两个终态写入触发器、append-only 保护；触发器在 Inbox/quarantine 原 owner 事务内执行。claim 只保存身份、终态与 wire digest，不保存 wire 内容。 | 已实现，未在 PostgreSQL 执行 |
| `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs` | 增加真实 PostgreSQL receiver 回归：Inbox→quarantine、quarantine→Inbox、两类 exact replay、内容冲突、并发竞争、失败重试无第二回执/ACK；使用 runtime 与 owner audit 两个受限 loopback 连接。 | 已实现并通过 Rust 编译；数据库运行未执行 |
| `docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md` | 记录实现、验证、已知缺口、守门规则与阶段签批。 | 本报告 |

数据库分类：新增 claim ledger 属于 Transaction（T），因为它是 owner 维护的持久化终态认领与审计事实，必须 append-only、保留 tenant RLS。此迁移没有新增 Work（W）临时表或 Master（M）配置表；Inbox source-origin 与 quarantine 仍是已有 T 类事实。

## §2 验证摘要

| 验证 | 实际结果 |
|---|---|
| Rust 工具链 | `rustc 1.99.0 (b940084d7 2026-09-28)` |
| `rtk cargo fmt -p bi-owner-runtime` | exit 0；已格式化本阶段测试文件 |
| `rtk cargo fmt -p bi-owner-runtime -- --check` | exit 0 |
| `rtk cargo check -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | exit 0，0 errors / 4 warnings；warning 是既有 public async trait lint |
| `rtk cargo test -p bi-owner-runtime --test source_position_arbitration_pg --no-run -j 4` | 最终重试 exit 0，集成测试目标成功链接。期间一次并行构建出现 LNK1104 找不到目标 exe；结束后同命令重试通过 |
| `rtk cargo test -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | exit 0，`0 passed, 1 ignored`。被 ignore 的测试没有执行 PostgreSQL 回归 |
| PostgreSQL migration / RLS / mTLS / recovery / settlement | 未运行。当前环境没有 `STAR_BI_TASK_EVENT_TEST_DATABASE_URL`、`STAR_BI_TASK_EVENT_TEST_DATABASE_AUDIT_URL`，且 `psql` 不在 PATH。不得将其标记为通过 |
| `rtk git diff --cached --check` | exit 0；暂存差异仅含登记的三条路径 |
| Cypher Graph Guardian | Rust 集成测试 header 检查 PASS；migration Cypher header digest 已按 Guardian canonicalization 规则复核 |

测试运行需要两个连接到同一个专用 `star_bi_task_event_test` 数据库的 loopback、非默认端口 URL：`STAR_BI_TASK_EVENT_TEST_DATABASE_URL` 必须以 `star_bi_runtime` 登录，仅负责真实接收路径；`STAR_BI_TASK_EVENT_TEST_DATABASE_AUDIT_URL` 必须以 `star_bi_owner` 登录，仅用于带 `app.tenant_id` 的受限审计查询。测试核对两条 URL 指向相同数据库端点，且不会对 runtime 角色开放 claim 直接读取权限。

## §3 已知缺口

1. 最终版 Rust 格式检查、`--no-run` 编译和默认测试复验结果需在本报告定稿前补齐。
2. 专用 PostgreSQL 尚不可用：迁移尚未应用，交叉终态回归、迁移时碰撞拒绝、FORCE RLS、owner/runtime 权限隔离与并发唯一约束都尚未取得真实数据库证据。
3. 需要独立审查迁移锁序、既有行回填、两个 SECURITY DEFINER trigger 的租户范围与 ACK 前提交边界。
4. W8 其他既有门禁仍未完成：mTLS、故障恢复、settlement 及 W8 全链路独立验收；本阶段不修改或替代这些工作。
5. 代码通过静态事务路径检查显示无需扩展已登记 SQL adapter 文件；若 PostgreSQL 集成证据揭示 owner 事务边界不满足要求，应暂停本 lane 并申请精确扩展路径，不能通过报告推定正确。

## §4 子代理失败接手清单

本 lane 未派发子代理，没有子代理失败产出需要接手。父任务并行执行了静态审阅；其提出的 runtime 与 audit 连接权限矛盾已通过双连接设计处理。父任务的独立审阅意见不替代后续独立代码审查与 PostgreSQL 验证。

## §5 守门规则（本阶段适用项）

1. 变更范围仅限登记的 migration、PG integration test、phase report 三条路径。
2. sequence 主键与非空 cursor 唯一索引共享同一 claim ledger；两个触发器只在来源事实原 owner transaction 内认领。
3. `ON CONFLICT DO NOTHING` 之后仅允许同 disposition、sequence、cursor、message ID、digest 完全相同的 replay；跨 disposition 或任一字段不同都抛错并回滚。
4. PostgreSQL 迁移期先锁住两个来源表，再检查历史冲突并回填；发现历史重复必须 fail closed，不自行挑选赢家。
5. claim 与两个终态表均保存 metadata/digest，不存 message wire；claim ledger 仅作 append-only。
6. claim 表启用并强制 tenant RLS；`star_bi_runtime` 没有直接表读写或触发函数执行权。
7. 集成测试用 `star_bi_runtime` 驱动真实接收/ACK，用 `star_bi_owner` 做受 tenant scope 限定的数据库核对，且两者必须连同一专用 loopback 数据库。
8. exact replay 必须返回原有终态；两种顺序的冲突重试都必须失败，不能生成第二 receipt 或第二 ACK。
9. 并发 Inbox/quarantine 竞争只能有一个终态成功并确认；失败方重试不得变成 exact replay 或确认。
10. 工具链按仓库约束使用 Rust 1.99.0，并通过 RTK 执行 Cargo/Git 守门命令。
11. PostgreSQL 环境缺失时报告 `blocked/unrun`，不得把 ignore、编译或静态检查描述成数据库通过。
12. W8 保持 OPEN，直到独立审查及 PG/RLS/mTLS/recovery/settlement 门禁全部取得证据。

## §6 签批栏

| 角色 | 签批 |
|---|---|
| 架构 | 架构师（Mavis 接手审核；阶段状态保持 OPEN） |
| SRE Lead | Mavis 接手审核；PostgreSQL 运维验证待完成 |
| 平台 | Mavis 接手审核；RLS 与角色隔离待真实数据库验证 |
| 评审主持 | Mavis 接手审核；独立审查待完成 |
| PM | Mavis 接手审核；不得将本切片宣传为 W8 已完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建阶段报告；明确共享 position claim 设计、双身份 PG 验证门与 W8 未关闭状态。 | WBS-W8 source-claim-fix 登记 brief |
