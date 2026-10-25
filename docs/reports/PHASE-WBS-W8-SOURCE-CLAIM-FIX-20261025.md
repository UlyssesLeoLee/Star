# PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025：跨终态 source position 唯一认领

> 状态：P1 修复候选，等待独立复审与 PostgreSQL 集成验证；W8 保持 OPEN。
> 工作分支：`UlyssesLeoLee/w8-source-position-claim-fix-20261025`
> 基线：`0ae310440e446f530a210ec413efb526cb038d8f`
> 范围依据：`docs/briefs/WBS-W8-SOURCE-CLAIM-FIX-20261025.md`（提交 `342edf0`）、`docs/briefs/WBS-W8-CURSORLESS-CLAIM-GUARD-20261025.md`（提交 `9edb0d78`）、`docs/briefs/W8-CURSOR-MODE-BARRIER-REMEDIATION-20261025.md`（提交 `b3f9e2b9`）、`docs/briefs/W8-READ-COMMITTED-CLAIM-GATE-20261025.md`（提交 `c5169588`）；独立审查 addendum `e361235f` 将 known-first→cursorless-later fail-open 定为 P1，等待复审确认最终分级。

## §0 目的

修复 Inbox 与 quarantine 分别认领 provider-neutral source position 时缺少跨终态唯一约束的问题。一个 tenant、consumer、provider、log、generation 下的 sequence，以及非空 opaque cursor，必须由同一 owner-local append-only claim 表唯一认领。Inbox receipt/source-origin 与 quarantine receipt 在各自原有事务中插入时触发共享认领；冲突使该事务回滚，接收器因此不能授权第二种终态 ACK。

新 claim 不得在同一 source generation 中混用 known 与 cursorless 模式：known-first 后新增 cursorless claim 要失败，cursorless-first 后新增 known claim 也要失败。generation 级 shared/exclusive advisory lock 覆盖两种方向的检查，普通 cursor-bearing 写入保持共享并行。完全匹配的既有 claim replay 仍可返回稳定回执。

shared/exclusive lock 对新 claim 的跨事务可见性证明以 PostgreSQL `READ COMMITTED` 为前提。`VOLATILE` 不会刷新 `REPEATABLE READ` 或 `SERIALIZABLE` 的固定事务快照，因此 claim trigger 在读取 claim、取 claim-generation 锁或写 claim 前检查 `transaction_isolation`；非 `READ COMMITTED` 的新 claim 以 SQLSTATE `0A000` fail closed。完全匹配且已在事务快照中持久化的 Inbox/quarantine exact replay 不创建新终态行或 claim，可以返回原回执并完成对应终态处理；若 snapshot 中缺少回执/来源或任一身份、路由、内容字段不匹配，则不得 ACK/终止为 exact replay。该边界来自源代码与独立复审，真实 PostgreSQL 行为仍待运行验证。调用者的新增 claim 写事务需使用 `READ COMMITTED`。

迁移回填可能保留已存在的混合历史 generation，但只能 exact replay；任何新 unmatched claim 不论模式都 fail closed。对混合历史需切换到新的 source-log generation；显式历史协调 API/流程不属于本 lane，必须另行设计审计事实且保留原 append-only claim。

本阶段处理 W8 的 source-position arbitration P1、cursorless-history P2；known-first→cursorless-later fail-open 是独立审查 addendum `e361235f` 标记的 P1 finding，本 lane 已加入对称门禁候选实现，等待独立复审确认。此报告不代表 W8 整体完成或可发布。

## §1 改动矩阵

| 登记路径 | 改动 | 状态 |
|---|---|---|
| `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql` | 增加共享 claim 表、迁移期跨表 sequence/cursor 冲突检测、既有数据回填、tenant FORCE RLS、owner-only grants、两个终态写入触发器、append-only 保护；全租户 precheck/backfill 显式设 `row_security=off`，无 RLS bypass 权限时 fail closed；generation shared/exclusive advisory lock 保护双向模式检查；known/cursorless partial index 查询历史；新 claim trigger 在任何 claim-ledger 读/写/取锁前仅允许 `READ COMMITTED`，避免将 `VOLATILE` 误作快照刷新保证。非该隔离级别的新 claim 以 `0A000` fail closed；完全匹配的既有终态 replay 不产生新 receipt/claim，按既有 origin 与 Inbox/fact 或 quarantine 字段核验后返回原回执。claim 只保存身份、终态与 wire digest，不保存 wire 内容。 | 已实现，未在 PostgreSQL 执行 |
| `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs` | 增加真实 PostgreSQL receiver/trigger 回归：cursorless-first 后 known receiver 被拒绝且无 ACK；known-first 后不同 sequence 的 cursorless trigger 写入被拒绝；混合历史 backfill exact receipt replay 及两种新 claim 拒绝；同 cursor 不同 sequence；并发双模式竞争仅一方提交；`REPEATABLE READ` 与 `SERIALIZABLE` 的 cursorless 新 claim 均应被隔离门禁拒绝且无 receipt。测试用 runtime 与 owner audit 两个受限 loopback 连接；真实 PG 尚未运行。 | 已实现，Rust 验证待本次复验；数据库运行未执行 |
| `docs/reports/PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025.md` | 记录实现、验证、已知缺口、守门规则与阶段签批。 | 本报告 |

数据库分类：新增 claim ledger 属于 Transaction（T），因为它是 owner 维护的持久化终态认领与审计事实，必须 append-only、保留 tenant RLS。此迁移没有新增 Work（W）临时表或 Master（M）配置表；Inbox source-origin 与 quarantine 仍是已有 T 类事实。

## §2 验证摘要

| 验证 | 实际结果 |
|---|---|
| Rust 工具链 | `rustc 1.99.0 (b940084d7 2026-09-28)` |
| `rtk cargo fmt -p bi-owner-runtime` | exit 0；已格式化本阶段测试文件 |
| `rtk cargo fmt -p bi-owner-runtime` | exit 0 |
| `rtk cargo fmt -p bi-owner-runtime -- --check` | READ COMMITTED gate 与隔离回归后的运行 exit 0 |
| `rtk cargo check -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | READ COMMITTED gate 与隔离回归后的运行 exit 0，0 errors / 4 warnings；warning 是既有 public async trait lint |
| `rtk cargo test -p bi-owner-runtime --test source_position_arbitration_pg --no-run -j 4` | READ COMMITTED gate 与隔离回归后的运行 exit 0，集成测试目标成功链接 |
| `rtk cargo test -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | exit 0，`0 passed, 1 ignored`。新增的 REPEATABLE READ/SERIALIZABLE 回归只编译，因该集成测试被 ignore 而未执行 |
| PostgreSQL migration / RLS / mTLS / recovery / settlement | 未运行。PG integration test 默认标记 ignored；本轮无数据库执行结果，不得将其标记为通过 |
| `rtk git diff --check` / 暂存范围 | `rtk git diff --check` exit 0；暂存后仍需核对只含登记的三条路径 |
| Cypher Graph Guardian | `cgg_guard.py --root <lane> --stamp crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs` exit 0，Rust test header stamp 已刷新；SQL header digest `d2ee89e43fabbc107a5336bdd791e67349b50c879c07b948eee4f762450465e8` 与 canonicalization 结果匹配 |
| CodeRabbit 独立审查 | 未运行：该 CLI 会将 diff 发往外部服务，本任务没有外发授权；本地 CLI 版本/认证检查因 `/root/.local/bin/coderabbit: Permission denied` 未能启动，未上传代码 |
| 隔离级别回归 | 新增 `REPEATABLE READ` 与 `SERIALIZABLE` 数据库行为断言；随 ignored PG integration suite 编译，但本机无 PostgreSQL，未实际运行 |

测试运行需要两个连接到同一个专用 `star_bi_task_event_test` 数据库的 loopback、非默认端口 URL：`STAR_BI_TASK_EVENT_TEST_DATABASE_URL` 必须以 `star_bi_runtime` 登录，仅负责真实接收路径；`STAR_BI_TASK_EVENT_TEST_DATABASE_AUDIT_URL` 必须以 `star_bi_owner` 登录，仅用于带 `app.tenant_id` 的受限审计查询。测试核对两条 URL 指向相同数据库端点，且不会对 runtime 角色开放 claim 直接读取权限。

## §3 已知缺口

1. 专用 PostgreSQL 尚不可用：迁移尚未应用，交叉终态、双向 mode barrier、隔离级别 fail-closed、sequence/cursor 唯一约束、迁移时碰撞拒绝、FORCE RLS、owner/runtime 权限隔离与 advisory-lock 并发行为均未取得真实数据库证据。当前 Rust 命令仅做检查/链接；集成测试被 ignore，不能证明 concurrency、隔离门禁或 ACK 行为。
2. 该迁移的历史扫描和回填要求具备 RLS bypass 的 migration role。`SET LOCAL row_security=off` 会使缺少此权限的执行明确失败，避免按当前 tenant 策略静默漏检；生产迁移身份尚未在 PostgreSQL 环境验证。
3. advisory-lock 屏障明确要求可能产生新 claim 的 owner 写事务为 `READ COMMITTED`；`REPEATABLE READ`/`SERIALIZABLE` 下命中新 claim trigger 会 fail closed。Inbox/quarantine exact replay 若从事务 snapshot 读到已提交且全字段匹配的终态记录，会返回旧回执/执行原终态处理，不创建新 receipt/claim；该路径不触发 claim isolation trigger。独立复审确认这不是重复写缺陷，但报告原“含 exact replay 一律拒绝”的承诺与实现不符，故按新 claim 边界修正。`VOLATILE` 只影响函数快照行为，不作为跨隔离级别的可见性保证。真实 PostgreSQL 尚未验证 exact replay 的高隔离行为；仍需复核锁序、既有行回填、共享/排他 advisory-lock key、一致的隔离设置、双向部分索引查询、两个 SECURITY DEFINER trigger 的租户范围与 ACK/终态处理前提交边界。
4. 独立审查 addendum `e361235f` 将 known-first→cursorless-later fail-open 记为 P1；复审报告 `99d85cb5` 确认双向新 claim 屏障在源码层面关闭了该路径。迁移会保留历史上已存在的 mixed-mode claim rows；exact replay 只有在 disposition、sequence、cursor、message ID、digest 全部匹配时才可通过。其他任何新 claim（包括 known→cursorless 与 cursorless→known）均失败。恢复新写入需使用新的 source-log generation；显式历史协调 API/流程不属于本 lane。该结论仍待真实 PostgreSQL 运行验证。
5. 对 mixed legacy generation 的 Inbox exact replay 豁免尚未在真实 PostgreSQL 上执行验证；ignored PG suite 构造 owner-scoped 历史数据并走 receiver 回放路径，但本机未运行该数据库测试。known-first→cursorless-later 用 owner-scoped direct trigger fixture 验证 claim/receipt 回滚；该 fixture 不调用 transport ACK，不能作为实际 ACK 行为的数据库证据。高隔离级别的完整 Inbox/quarantine exact replay 也没有独立 PG 回归，目前仅有源代码路径审查。
6. W8 其他既有门禁仍未完成：mTLS、故障恢复、settlement 及 W8 全链路独立验收；本阶段不修改或替代这些工作。
7. 代码通过静态事务路径检查显示无需扩展已登记 SQL adapter 文件；若 PostgreSQL 集成证据揭示 owner 事务边界不满足要求，应暂停本 lane 并申请精确扩展路径，不能通过报告推定正确。

## §4 子代理失败接手清单

本 lane 未派发子代理，没有子代理失败产出需要接手。父任务并行执行了静态审阅；其提出的 runtime 与 audit 连接权限矛盾已通过双连接设计处理。父任务的独立审阅意见不替代后续独立代码审查与 PostgreSQL 验证。

## §5 守门规则（本阶段适用项）

1. 变更范围仅限登记的 migration、PG integration test、phase report 三条路径。
2. sequence 主键与非空 cursor 唯一索引共享同一 claim ledger；两个触发器只在来源事实原 owner transaction 内认领。
3. `ON CONFLICT DO NOTHING` 之后仅允许同 disposition、sequence、cursor、message ID、digest 完全相同的 replay；跨 disposition 或任一字段不同都抛错并回滚。
4. PostgreSQL 迁移期必须由具备全租户 RLS bypass 的迁移身份执行；先锁住两个来源表，再检查历史冲突并回填；发现历史重复或权限不足必须 fail closed，不自行挑选赢家或静默漏行。
5. claim 与两个终态表均保存 metadata/digest，不存 message wire；claim ledger 仅作 append-only。
6. claim 表启用并强制 tenant RLS；`star_bi_runtime` 没有直接表读写或触发函数执行权。
7. 集成测试用 `star_bi_runtime` 驱动真实接收/ACK，用 `star_bi_owner` 做受 tenant scope 限定的数据库核对，且两者必须连同一专用 loopback 数据库。
8. exact replay 必须返回原有终态；两种顺序的冲突重试都必须失败，不能生成第二 receipt 或第二 ACK。
9. 并发 Inbox/quarantine 竞争只能有一个终态成功并确认；失败方重试不得变成 exact replay 或确认。
10. 同 generation 的 cursor-bearing claim 取得 shared advisory lock；cursorless claim 取得对应 exclusive lock。共享 scope 字符串必须一致，且不得引入普通 cursor-bearing 写入之间的全局串行化。
11. 对每个新 claim 在对应 advisory lock 下双向查找 opposite cursor mode：cursorless partial index 支撑 NULL history；cursor unique partial index 支撑已知 cursor history。同 generation 若已混合，只允许 disposition、sequence、cursor、message ID、digest 完全一致的既有 replay。
12. 迁移回填保留历史 mixed-mode rows；任何新的 unmatched claim 两种模式都失败。正常恢复使用 fresh source-log generation；不得删除或覆盖 append-only claim 来解除门禁。
13. 测试需证明 known-first→cursorless-later 与 cursorless-first→known-later 均失败；混合历史 exact replay 稳定、两种新 claim 均被拒绝；并发 known/null contender 最多一方提交；不同 sequence 复用相同 opaque cursor 仍失败。
14. 工具链按仓库约束使用 Rust 1.99.0，并通过 RTK 执行 Cargo/Git 守门命令。
15. PostgreSQL 环境缺失时报告 `blocked/unrun`，不得把 ignore、编译或静态检查描述成数据库通过。
16. W8 保持 OPEN，直到独立审查及 PG/RLS/mTLS/recovery/settlement 门禁全部取得证据。
17. 共享/排他 advisory lock 的 mode-history 证明只用于 PostgreSQL `READ COMMITTED` 下的新 claim；trigger 必须在任何 claim-ledger 读取、锁或写入前拒绝其它事务隔离级别。不得把 `VOLATILE` 当作 `REPEATABLE READ`/`SERIALIZABLE` 快照刷新的保证。已存在且完整匹配的 Inbox/quarantine exact replay 可复用旧终态回执而不创建 claim；不能将此例外用于新 claim、来源缺失或字段冲突。

## §6 签批栏

| 角色 | 签批 |
|---|---|
| 架构 | 架构师（Mavis 接手审核；阶段状态保持 OPEN） |
| SRE Lead | Mavis 接手审核；PostgreSQL 运维验证待完成 |
| 平台 | Mavis 接手审核；RLS 与角色隔离待真实数据库验证 |
| 评审主持 | Mavis 接手审核；独立源代码审查完成（`99d85cb5`），PostgreSQL 运行复核待完成 |
| PM | Mavis 接手审核；不得将本切片宣传为 W8 已完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建阶段报告；明确共享 position claim 设计、双身份 PG 验证门与 W8 未关闭状态。 | WBS-W8 source-claim-fix 登记 brief |
| v0.2 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 cursorless-history fail-closed barrier、shared/exclusive advisory-lock、partial index、不同 sequence 相同 cursor 与历史 null-cursor quarantine 回归、migration RLS-bypass fail-closed 要求及 generation rollover/reconciliation 说明；PG 结果仍为 blocked/unrun。 | Independent review addendum `e098fe4061e23045049aa08c0c2bef75cec6561a` 与 cursorless guard brief `9edb0d78` |
| v0.3 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 为 known-first→cursorless-later fail-open 加入双向 cursor-mode 门禁与 exact replay 豁免；增加顺序、mixed-history 与双模式并发 PostgreSQL 回归。数据库/RLS/concurrency 实跑仍未执行。 | W8 cursor-mode barrier remediation brief `b3f9e2b9` |
| v0.4 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确记录独立 addendum `e361235f` 将该 fail-open 定为 P1、修复候选等待复审；披露 known-first cursorless fixture 不运行 transport ACK；将过期验证条目改为本轮实际执行状态。 | Independent review addendum `e361235f` |
| v0.5 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 CodeRabbit 未运行：外发 diff 未授权；本地 CLI 状态检查因权限错误未启动。 | 提交前自审 |
| v0.6 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按固定事务快照边界补 READ COMMITTED 前置条件；trigger 对非 READ COMMITTED（含 exact replay）fail closed，并增加 REPEATABLE READ/SERIALIZABLE PostgreSQL 回归断言。 | W8 read-committed claim gate brief `c5169588` |
| v0.7 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 复跑 Rust 1.99 format/check/no-run/default-test gates；编译通过，PG integration 仍为 1 ignored，未取得隔离/并发/RLS 实际证据。 | READ COMMITTED gate verification |
| v0.8 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 独立复审 `99d85cb5` 将“非 READ COMMITTED 下 exact replay 也由 claim trigger 拒绝”的 P2 定性为门禁范围表述不匹配，而非重复写缺陷；修正文档为新 claim 必须 READ COMMITTED，完全匹配且已持久化的 Inbox/quarantine replay 可复用旧回执且不新增 claim。高隔离 exact replay、RLS、并发和 ACK/终态实跑仍未执行，W8 保持 OPEN。 | W8 symmetric claim barrier independent review |
| v0.9 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录复审 `99d85cb5` 对双向新 claim 屏障的源码级通过；将历史 P1 从“等待复审”改为“源码已确认、PG 待验证”，并补充 mixed-history/high-isolation exact replay 与终态 ACK 的数据库验证缺口。 | W8 remediation review completion |
