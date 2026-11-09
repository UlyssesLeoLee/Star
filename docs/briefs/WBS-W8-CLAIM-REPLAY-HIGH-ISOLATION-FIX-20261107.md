# Brief: WBS-W8-CLAIM-REPLAY-HIGH-ISOLATION-FIX-20261107

**Agent**: rust-database-engineer
**Phase**: WBS-W8-CLAIM-REPLAY-HIGH-ISOLATION-FIX-20261107
**Created**: 2026-11-07 02:25:34

---

## 目标
修复 W8 source-position lane 的 P2：在 PostgreSQL REPEATABLE READ/SERIALIZABLE 下，对事务快照中已经完整持久化的 quarantine exact replay，当前 record_task_event_quarantine_v2 仍先 INSERT，导致 BEFORE INSERT claim trigger 的 READ COMMITTED gate 在 ON CONFLICT/回执比对前拒绝。只能实现窄的、全字段 exact replay 快路；缺行、身份差异、摘要差异、路由差异、来源不确定或多重匹配都必须 fail closed，不 ACK。

## 基线与证据
候选工作树：D:\orcaWork\Star\w8-source-position-claim-fix-20261025，HEAD 5fe3ea9372242d7d72f9dd416f6437475c870904；当前集成基线 dev@996ae3ae7d6e4c9c075715f3799f5186734d32f0。独立续审 commit 39e49940 / 045b5d97 指出 migration `2026-10-25-bi-task-event-shared-position-claim.sql` 的 claim trigger 在 INSERT 前拒绝非 READ COMMITTED，而旧 quarantine function 于 INSERT 后才检索旧 receipt；source-position PG test 目前只验证高隔离的新 claim 被拒绝，尚无高隔离 exact quarantine replay case。候选的 .codegraph 未初始化；这是受限候选 checkout，只读核对具体 SQL/test 路径，不作穷尽性代码图断言。

## 实施范围
只改本 MAGOS manifest 的五个文件：shared-position-claim SQL migration、`source_position_arbitration_pg.rs`、原 W8 phase report、当前 brief、当前 dispatcher log。保持 `crates/bi-owner-runtime/src/task_event_inbox.rs` 不动（该文件有其他 W8 lane/旧候选重叠）。不改 Cargo.toml/Cargo.lock、其他 migration、WBS 或任何 D:\Star 用户文件。

## 安全设计与验收
1. 在 quarantine INSERT 之前按 owner-scoped source identity（tenant/consumer/provider/log/generation + sequence OR non-null opaque cursor）做有界匹配；多于一条视为冲突。
2. 仅当固定事务快照中唯一已存在的 quarantine receipt 与全部输入有关字段完全匹配（tenant/consumer/provider/log/generation、sequence/cursor、message identity、wire digest/length、sender/receiver、transport profile、endpoint、destination、fault），且唯一共享 claim 的 disposition/source identity/message/digest 也完全匹配时允许 exact replay。触发器 `RETURN NULL` 抑制实际 receipt 行与 claim 写入；外层 stored function 使用原 exact comparison 返回原 quarantine ID/timestamp。缺少回执保持原 `0A000` 隔离拒绝，部分缺失、不唯一或字段不同必须 fail closed。任何 READ COMMITTED 以外的新 claim 都不放行。
3. 新增真实 PostgreSQL 回归设计/实现：高隔离且旧完整 quarantine receipt 在事务快照内时 exact replay 返同一 id；高隔离下来源/内容冲突不得返回 exact replay、不新增终态行且无 terminal ACK；READ COMMITTED 的现有 exact replay/新写入路径不回归。若测试只能做数据库过程级断言，报告清楚区分它和 receiver ACK 行为。
4. 修正阶段报告的“代码/编译通过”和“PG未运行”叙述，写清旧缺口、修复行为与实际验证；W8 继续 OPEN，除非相关 PG 端点和验收实际可用并通过。
5. 在 lane 内先做审阅和 `git diff --check`；用 Rust 1.99.1 执行最小格式/编译 gates。只有既有、身份明确的 disposable PG 环境已配置时才运行 ignored PG suite；不得临时建/删不明数据库或实例，也不打印连接串/secret。没有安全 PG endpoint 时记 NOT_RUN，不能宣称 PG 通过。
6. 只提交五个 manifest 文件；不要 rebase、merge、push、清理 worktree/branch。协调者将独立复审后集成。

## 实施与验证记录（2026-11-07）

- 已修改 shared-position claim trigger：非 READ COMMITTED 下，Inbox/new claim 继续在 claim-ledger 访问前拒绝；quarantine 先做固定快照内的 owner-scoped receipt 候选计数，仅当唯一 receipt 的输入字段完全相同，才查验唯一匹配的 quarantine claim 并 `RETURN NULL`。外层 `record_task_event_quarantine_v2` 因未插入行进入原 exact-replay 比较并返回原 ID/timestamp。
- 已在 ignored PostgreSQL 集成测试中增加 REPEATABLE READ 和 SERIALIZABLE receiver 实例：断言 exact replay 返回原 ID、持久 timestamp 和 receipt/claim 计数不变；同位置更改 wire digest/length 时断言 `23505`、无新增终态、没有 Inbox/quarantine ACK。该测试实际驱动 `TaskEventReceiver` 并检查 `PgDelivery` 测试回调；它不是外部 transport/broker durable ACK 的证据，也不会把函数/trigger 级检查当成 ACK 证据。
- 本地仅两个专用 PostgreSQL 连接环境变量的“已配置”布尔检查结果均为 `False`；ignored suite 本轮 `NOT_RUN`。不创建或清理数据库/VM。
- Rust 1.99.1 `cargo fmt -p bi-owner-runtime -- --check` exit 0；`cargo check -p bi-owner-runtime --test source_position_arbitration_pg -j 4` exit 0、0 errors、4 个既有 `async_fn_in_trait` warnings；`cargo test -p bi-owner-runtime --test source_position_arbitration_pg --no-run -j 4` exit 0，测试目标成功链接但未执行。
- Cypher Graph Guardian `--changed --stamp` exit 0，检查并刷新 1 个 Rust source header；SQL 文件不在其 source-extension 清单，故使用同一工具的 `source_digest` helper 手动刷新 migration header，hash match 检查为 true。
- `git diff --check` exit 0；未跟踪 brief/log 额外通过 trailing-whitespace 检查，JSONL 两条记录均可解析。两个专用 PostgreSQL endpoint 配置布尔值均为 false，ignored PG suite `NOT_RUN`。不将 PostgreSQL 行为、migration 应用、RLS 或 receiver ACK 标为已验证。
- W8 保持 `OPEN`；当前工作是有源码和编译证据的候选修复，不是 PostgreSQL 验收完成。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
