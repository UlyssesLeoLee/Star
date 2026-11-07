# PHASE-WBS-W8-SOURCE-CLAIM-FIX-20261025：跨终态 source position 唯一认领

> 状态：P1 修复候选与高隔离 quarantine replay P2 已补充源码实现及回归；Rust 1.99.1 定向 compile gate 通过，专用 PostgreSQL endpoints 未配置，本轮数据库回归 NOT_RUN，W8 保持 OPEN。
> 工作分支：`UlyssesLeoLee/w8-source-position-claim-fix-20261025`
> 基线：`0ae310440e446f530a210ec413efb526cb038d8f`
> 本轮 P2 候选基线：`5fe3ea9372242d7d72f9dd416f6437475c870904`；协调者目标 dev 基线：`996ae3ae7d6e4c9c075715f3799f5186734d32f0`。
> 范围依据：`docs/briefs/WBS-W8-SOURCE-CLAIM-FIX-20261025.md`（提交 `342edf0`）、`docs/briefs/WBS-W8-CURSORLESS-CLAIM-GUARD-20261025.md`（提交 `9edb0d78`）、`docs/briefs/W8-CURSOR-MODE-BARRIER-REMEDIATION-20261025.md`（提交 `b3f9e2b9`）、`docs/briefs/W8-READ-COMMITTED-CLAIM-GATE-20261025.md`（提交 `c5169588`）；独立审查 addendum `e361235f` 将 known-first→cursorless-later fail-open 定为 P1，等待复审确认最终分级。

## §0 目的

修复 Inbox 与 quarantine 分别认领 provider-neutral source position 时缺少跨终态唯一约束的问题。一个 tenant、consumer、provider、log、generation 下的 sequence，以及非空 opaque cursor，必须由同一 owner-local append-only claim 表唯一认领。Inbox receipt/source-origin 与 quarantine receipt 在各自原有事务中插入时触发共享认领；冲突使该事务回滚，接收器因此不能授权第二种终态 ACK。

新 claim 不得在同一 source generation 中混用 known 与 cursorless 模式：known-first 后新增 cursorless claim 要失败，cursorless-first 后新增 known claim 也要失败。generation 级 shared/exclusive advisory lock 覆盖两种方向的检查，普通 cursor-bearing 写入保持共享并行。完全匹配的既有 claim replay 仍可返回稳定回执。

shared/exclusive lock 对新 claim 的跨事务可见性证明以 PostgreSQL `READ COMMITTED` 为前提。`VOLATILE` 不会刷新 `REPEATABLE READ` 或 `SERIALIZABLE` 的固定事务快照，因此非 `READ COMMITTED` 的新 Inbox/source-origin claim 仍在 claim-ledger 访问前以 SQLSTATE `0A000` fail closed。Quarantine trigger 仅增加窄的快照内 replay：先按 tenant/consumer/provider/log/generation 与 sequence 或非空 cursor 统计同位置回执，只在唯一回执的全部路由/内容/fault 字段完全相同后，才检查唯一且完全匹配的共享 claim；两者都匹配时 `RETURN NULL` 抑制 INSERT 行与 claim 写入，外层 quarantine function 通过原 exact-replay 比较返回原 receipt ID/timestamp。缺少可见回执仍走 `0A000`；回执不唯一、字段不同、claim 缺失/不唯一/不同均以冲突 fail closed。新 unmatched claim、Inbox claim 与无法证明的 replay 不绕过 READ COMMITTED 门禁。当前只完成源码与测试实现；真实 PostgreSQL 行为仍待运行验证。调用者的新增 claim 写事务需使用 `READ COMMITTED`。

迁移回填可能保留已存在的混合历史 generation，但只能 exact replay；任何新 unmatched claim 不论模式都 fail closed。对混合历史需切换到新的 source-log generation；显式历史协调 API/流程不属于本 lane，必须另行设计审计事实且保留原 append-only claim。

本阶段处理 W8 的 source-position arbitration P1、cursorless-history P2；known-first→cursorless-later fail-open 是独立审查 addendum `e361235f` 标记的 P1 finding，本 lane 已加入对称门禁候选实现，等待独立复审确认。此报告不代表 W8 整体完成或可发布。

## §1 改动矩阵

| 登记路径 | 改动 | 状态 |
|---|---|---|
| `db/migrations/2026-10-25-bi-task-event-shared-position-claim.sql` | 增加共享 claim 表、迁移期跨表 sequence/cursor 冲突检测、既有数据回填、tenant FORCE RLS、owner-only grants、两个终态写入触发器、append-only 保护；全租户 precheck/backfill 显式设 `row_security=off`，无 RLS bypass 权限时 fail closed；generation shared/exclusive advisory lock 保护双向模式检查；known/cursorless partial index 查询历史；非 `READ COMMITTED` 的新 claim fail closed；quarantine 高隔离 replay 只在固定 snapshot 中找到唯一且所有 receipt/claim 字段完全匹配的持久事实时，触发器 `RETURN NULL` 抑制 INSERT 与 claim 写入，外层 function 再返回原 receipt ID/timestamp。claim 只保存身份、终态与 wire digest，不保存 wire 内容。 | 历史 migration 链曾应用于一次性 PostgreSQL 18.6 测试库；本轮 trigger 修改未应用或在 PostgreSQL 验证 |
| `crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs` | 在既有 ignored PostgreSQL 集成测试中增加 `REPEATABLE READ` 与 `SERIALIZABLE` receiver case：精确 quarantine replay 返回相同 durable ID、owner audit 确认原 timestamp 与 receipt/claim 计数不变、`TaskEventReceiver` 调用 terminal quarantine ACK 测试回调；同位置变更 wire digest/length 必须报 PostgreSQL `23505`，不增 receipt 且不调用 ACK 回调。测试用 runtime 与 owner audit 两个受限 loopback 连接，并在专用单连接池上设置会话隔离级别。旧有 READ COMMITTED、mode barrier、并发和 new-claim high-isolation 测试继续保留。该回调不是外部 transport/broker durable ACK 的证据。 | Rust 1.99.1 compile gate 通过；端点未配置，ignored PostgreSQL suite 本轮 NOT_RUN |
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
| 一次性 PostgreSQL 18.6 migration 链 | exit 0；依序应用 Inbox、projection、quarantine、source-origin 与 shared-position-claim 五份 W8 依赖 migration；使用独立 loopback 测试库，未触碰其他 PostgreSQL 数据目录 |
| `rtk cargo test -p bi-owner-runtime --test source_position_arbitration_pg -j 4 -- --ignored --nocapture` | **exit 1**；唯一集成测试在首次 Inbox 调用失败：`Err(Inbox(Database(ColumnNotFound("source_sequence"))))`。SQL function 返回该列，但 Rust SELECT 列表漏选；测试的后续 ACK、quarantine、并发与隔离级别断言均未运行 |
| 2026-11-06 Rust 1.99.1 | `rustc 1.99.1 (58cf8f9c3 2026-10-13)`；`rustfmt 1.10.0-stable (58cf8f9c35 2026-10-13)` |
| `rtk cargo +1.99.1 fmt -p bi-owner-runtime -- --check` | exit 0 |
| `rtk cargo +1.99.1 check -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | exit 0，0 errors / 4 existing `async_fn_in_trait` warnings |
| `rtk cargo +1.99.1 test -p bi-owner-runtime --test source_position_arbitration_pg --no-run -j 4` | exit 0；测试目标成功链接，未执行测试 |
| Current PostgreSQL endpoint availability | 两个专用连接环境变量均未设置；本次未重跑 ignored PG suite，也未修改用户数据库 |
| `rtk git diff --check` | 本轮工作树 diff exit 0 |
| W8 high-isolation replay lane / `rtk proxy git diff --check` | 本轮 trigger、receiver-level 回归与本报告/brief/log 变更；最终 diff 检查结果见本修订记录 |
| `rtk proxy rustup run 1.99.1 cargo fmt -p bi-owner-runtime -- --check` | 首次检查因新增回归的格式差异 exit 1；随后 `cargo fmt -p bi-owner-runtime` exit 0，需以最终复跑结果为准 |
| `rtk proxy rustup run 1.99.1 cargo check -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | exit 0；0 errors，4 个既有 `async_fn_in_trait` warnings；本轮高隔离 receiver regression 编译通过 |
| 高隔离 exact replay PostgreSQL receiver regression | 已加入 `REPEATABLE READ`/`SERIALIZABLE`；两个专用 endpoint 环境变量均未设置，ignored suite NOT_RUN；无实际 DB/ACK 通过证据 |
| PostgreSQL RLS/grants、双模式并发、exact replay、ACK、mTLS、recovery、settlement | 未验证；migration 成功应用不等同于这些运行时验收通过 |
| 2026-11-06 Cypher Graph Guardian（历史记录） | 使用 Guardian 脚本检查并刷新 `task_event_inbox.rs` 与当时的 `source_position_arbitration_pg.rs` header，exit 0；这是先前 adapter lane 的记录 |
| CodeRabbit 独立审查 | 未运行：该 CLI 会将 diff 发往外部服务，本任务没有外发授权；本地 CLI 版本/认证检查因 `/root/.local/bin/coderabbit: Permission denied` 未能启动，未上传代码 |
| 2026-11-06 隔离级别回归（历史记录） | 当时新增的 `REPEATABLE READ` 与 `SERIALIZABLE` 新 claim 数据库断言已编译；该次 ignored suite 在首次 Inbox 查询失败，未到达这些断言 |
| 本轮工具链 | `rustup run 1.99.1 rustc --version` → `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| 本轮 `rtk proxy rustup run 1.99.1 cargo fmt -p bi-owner-runtime -- --check` | exit 0 |
| 本轮 `rtk proxy rustup run 1.99.1 cargo check -p bi-owner-runtime --test source_position_arbitration_pg -j 4` | exit 0；0 errors，4 个既有 `async_fn_in_trait` warnings |
| 本轮 `rtk proxy rustup run 1.99.1 cargo test -p bi-owner-runtime --test source_position_arbitration_pg --no-run -j 4` | exit 0；测试可执行文件链接成功；没有运行 ignored 测试 |
| 本轮 Cypher Graph Guardian | `--changed --stamp` exit 0，检查并刷新 1 个 Rust source file（PG 测试 header）；SQL migration 不在 guardian 支持扩展中，因此按其 `source_digest` 规范手工刷新既有 SQL header stamp。候选 `.codegraph` 未初始化，未初始化或作穷尽性图谱断言 |
| 本轮高隔离 PostgreSQL receiver/ACK 回归 | `REPEATABLE READ`/`SERIALIZABLE` exact ID/timestamp 与匹配冲突/no-ACK 场景已加入 ignored 测试并编译；两个专用 endpoint 配置值的布尔检查均为 `False`，因此本轮 `NOT_RUN`。receiver ACK 计数由测试 Delivery 回调观测，不代表真实外部 transport 的 durable ACK 证明 |
| 本轮 `rtk proxy git diff --check` / `git diff --cached --check` | 两者均 exit 0；另对 brief 与 JSONL dispatch log 检查 trailing whitespace 并逐行解析 JSON，均通过；cached path 清单恰为本 lane 的五个 manifest 文件 |

测试运行需要两个连接到同一个专用 `star_bi_task_event_test` 数据库的 loopback、非默认端口 URL：`STAR_BI_TASK_EVENT_TEST_DATABASE_URL` 必须以 `star_bi_runtime` 登录，仅负责真实接收路径；`STAR_BI_TASK_EVENT_TEST_DATABASE_AUDIT_URL` 必须以 `star_bi_owner` 登录，仅用于带 `app.tenant_id` 的受限审计查询。测试核对两条 URL 指向相同数据库端点，且不会对 runtime 角色开放 claim 直接读取权限。

## §3 已知缺口

1. 一次性 PostgreSQL 18.6 测试库已成功应用五份 W8 依赖 migration。之前的真实 ignored 集成测试在首次 Inbox SQLx 行解码处失败：`source_sequence` 未被 adapter 的 SELECT 列表选出；当前 adapter 已选出该列，测试新增 durable receipt sequence 断言，且两者通过 Rust 1.99.1 编译/链接。由于两个专用 PG endpoint 当前均未设置，修正后没有重跑 ignored PG suite；后续双向 mode barrier、隔离级别、并发、exact replay 与 ACK 断言仍无数据库通过证据。
2. 该迁移的历史扫描和回填要求具备 RLS bypass 的 migration role。`SET LOCAL row_security=off` 会使缺少此权限的执行明确失败，避免按当前 tenant 策略静默漏检；生产迁移身份尚未在 PostgreSQL 环境验证。
3. advisory-lock 屏障明确要求可能产生新 claim 的 owner 写事务为 `READ COMMITTED`；`REPEATABLE READ`/`SERIALIZABLE` 下新 claim fail closed。本轮把 quarantine 的高隔离 exact replay 限定为 trigger 内的 pre-insert snapshot lookup：先要求唯一且所有业务输入字段相同的可见 quarantine receipt，再要求唯一且 disposition/sequence/cursor/message ID/digest 相同的 claim；只有两者完全匹配才 `RETURN NULL` 抑制物理插入和 claim 写入，外层 function 原有冲突比较返回原 ID/timestamp。缺回执保留 `0A000` 新 claim 隔离拒绝，部分/不唯一/不同候选以冲突 fail closed；Inbox 与新 claim 没有扩大例外。该行为有源码及 receiver 回归实现，但本轮 PostgreSQL 端点未设置，运行行为与 ACK 仍未验证。`VOLATILE` 不作为跨隔离级别的可见性保证。仍需验证锁序、既有行回填、共享/排他 advisory-lock key、双向部分索引、SECURITY DEFINER 的 tenant RLS 与终态 ACK 边界。
4. 独立审查 addendum `e361235f` 将 known-first→cursorless-later fail-open 记为 P1；复审报告 `99d85cb5` 确认双向新 claim 屏障在源码层面关闭了该路径。迁移会保留历史上已存在的 mixed-mode claim rows；exact replay 只有在 disposition、sequence、cursor、message ID、digest 全部匹配时才可通过。其他任何新 claim（包括 known→cursorless 与 cursorless→known）均失败。恢复新写入需使用新的 source-log generation；显式历史协调 API/流程不属于本 lane。该结论仍待真实 PostgreSQL 运行验证。
5. 对 mixed legacy generation 的 exact replay 及 known-first→cursorless-later direct trigger fixture 都仍缺真实 PostgreSQL 执行证据。当前 ignored PG suite 已增加 `REPEATABLE READ`/`SERIALIZABLE` receiver-level quarantine 测试：exact replay 检查原 ID、timestamp 与单份终态记录，并实际断言仅 exact replay 发 quarantine ACK；同位置 wire digest/length 冲突断言 PostgreSQL `23505`、无终态 ACK、无新记录。两个专用 endpoint 环境变量均未设置，故这些 receiver/ACK 断言为代码实现而非数据库结果；procedure-level/trigger 编译不能替代实际 PostgreSQL 与 ACK 证明。
6. W8 其他既有门禁仍未完成：mTLS、故障恢复、settlement 及 W8 全链路独立验收；本阶段不修改或替代这些工作。
7. 先前 PostgreSQL 实跑揭示 `crates/bi-owner-runtime/src/task_event_inbox.rs` 的 SQLx SELECT 漏选 `source_sequence`；该文件在 2026-11-06 的先前 MAGOS amendment 中加入并修正，本次高隔离 replay lane 不编辑该文件。先前 amendment 与 `codex/wbs-w8-transport-remediation-20261025` 的已提交改动及 `wbs-w8-bi-native-adapter-20261024` 工作树的未提交改动重叠；当前 lane 继续保留这两个来源且只修改当前登记的五个文件。本机当前没有两个专用 PG endpoint，数据库行为仍待受控 PG 验证。

## §4 子代理失败接手清单

本轮 P2 由协调者派发到隔离 worktree 执行；当前 worker 完成 SQL trigger、ignored PostgreSQL receiver 回归、brief 与本报告更新。没有再向下派发子代理，因此无子代理失败产出需要接手。协调者独立代码审查待完成；源码编译成功不替代审查，也不替代 PostgreSQL 实跑、RLS 与外部 transport ACK 验收。

## §5 守门规则（本阶段适用项）

1. 每个子 lane 仅修改当前 MAGOS manifest 登记路径。本次高隔离 replay lane 限于 shared-position-claim migration、PG integration test、该 phase report、当前 brief 与对应 JSONL dispatch log；不编辑 Inbox adapter、Cargo 文件、其它 migration、WBS 或任何 lane 外文件。
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
14. 工具链按当前仓库基线使用 Rust 1.99.1，并通过 RTK 执行 Cargo/Git 守门命令。
15. PostgreSQL 环境缺失时报告 `blocked/unrun`，不得把 ignore、编译或静态检查描述成数据库通过。
16. W8 保持 OPEN，直到独立审查及 PG/RLS/mTLS/recovery/settlement 门禁全部取得证据。
17. 共享/排他 advisory lock 的 mode-history 证明只用于 PostgreSQL `READ COMMITTED` 下的新 claim；非 `READ COMMITTED` 的 Inbox/new claim 必须在读取 claim、取锁或写入 claim 前拒绝。唯一例外是 quarantine trigger 先在固定 snapshot 中找到唯一且全部 receipt 输入字段完全匹配的既有行后，才读取并验证唯一完全匹配的 shared claim；仅此情况允许 `RETURN NULL` 抑制物理 INSERT/claim 并由外层函数返回原 ID/timestamp。receipt 缺失必须保持 `0A000`，receipt/claim 不唯一、部分缺失或任一 identity/route/content/disposition/profile/endpoint/destination/fault 字段不同必须 fail closed，不得 ACK。`VOLATILE` 不得作为 `REPEATABLE READ`/`SERIALIZABLE` 快照刷新的保证；该行为在实际 PG/ACK 测试通过前保持未验收。

## §6 签批栏

| 角色 | 签批 |
|---|---|
| 架构 | 架构师（Mavis 接手审核；阶段状态保持 OPEN） |
| SRE Lead | Mavis 接手审核；PostgreSQL 运维验证待完成 |
| 平台 | Mavis 接手审核；RLS 与角色隔离待真实数据库验证 |
| 评审主持 | Mavis 接手审核；先前 P1 源码复核 `99d85cb5` 已完成，本轮 P2 独立复审与 PostgreSQL 运行复核待完成 |
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
| v0.10 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录一次性 PostgreSQL 18.6 已成功应用 W8 依赖 migration；真实集成测试在首次 Inbox 调用因 SQLx SELECT 漏选 `source_sequence` 失败。记录该 adapter 文件扩展 manifest 时的 MAGOS 冲突及未修改源码事实；后续数据库验收保持未通过。 | Disposable PostgreSQL integration run and MAGOS manifest gate |
| v0.11 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 MAGOS amendment `94d8b953e690` 后 manifest 扩展为四条路径；修正 Inbox SELECT 的 `source_sequence` 投影并增加 durable receipt sequence 回归断言。Rust 1.99.1 fmt/check/no-run 与 Cypher header 检查通过；因专用 PG endpoints 未设置，修正后的数据库行为仍未验证，W8 保持 OPEN。 | W8 decode failure remediation and focused validation |
| v0.12 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 quarantine BEFORE INSERT 固定 snapshot exact-replay precheck：唯一完整匹配 receipt 后再核验唯一匹配 claim，`RETURN NULL` 抑制 receipt/claim 写入并让旧 stored function 返回同一 receipt ID/timestamp；新增 REPEATABLE READ/SERIALIZABLE receiver callback exact ACK 与变更 wire digest/length 无 ACK/no-new-row 回归。Rust 1.99.1 fmt/check/no-run 通过；专用 PostgreSQL endpoints 未配置，数据库及外部 transport durable ACK 均未验证，P2 与 W8 保持 OPEN。 | WBS-W8-CLAIM-REPLAY-HIGH-ISOLATION-FIX-20261107 |
