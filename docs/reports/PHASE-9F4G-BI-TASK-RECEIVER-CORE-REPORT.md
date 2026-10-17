# PHASE-9F4G-BI-TASK-RECEIVER-CORE-REPORT

## §0 目的

记录 Phase 9F4G 从 provider-neutral BI Task-event receiver/quarantine source，推进到 Run-scoped、只读、有界 BI Task-event query API 的当前切片。报告不宣称 Phase 9F4G、BI/Benchmark 闭环或 Worktree-first Group Apps 全部完成。

## §1 改动矩阵

| 范围 | 改动 | 结果 |
|---|---|---|
| Run BI API | 新增 `GET /api/v1/engineering-runs/{engineering_run_id}/bi/task-events`；先做 actor/scope 与当前 Project/Run 授权，并在独立 BI 查询期间保持授权事务和 grant locks | 路由已接入 Group API；无远端宿主 Branch 依赖 |
| Query port/store | 新增 `TaskEventQueryPort`、有界 request/cursor/page 类型与 SQLx `PgTaskEventQueryStore` | 固定函数读取、默认 50/最大 100、稳定 `(occurred_at DESC, event_id ASC)` keyset；单次最多取 101 行 |
| BI 数据访问 | 新增 `bi.list_task_event_page_v1`；可选 `BI_DATABASE_URL`，pool 上限 2、acquire timeout 2 秒，空或非法 URL 启动失败，provider 缺失返回 503 | 运行角色仅 EXECUTE，不直接 SELECT facts；未增加表 |
| 自查修复 | 补齐 async-trait 直接依赖、Cargo.lock 与公开 API rustdoc；移除重复 import/unused mut；查询 provider 增加 6 秒总超时；request constructor 拒绝直接构造的 nil cursor | TOML/lock parse、Guardian 与 whitespace checks 通过；Rust compile/tests 未运行 |
| 数据最小化 | response 仅含 allowlisted event/work-item IDs、版本、事件类型/时间、lifecycle/review 转移和 owner-change；coverage/freshness 为 `unknown` | payload、actor、correlation、route、wire 不出现在查询 response；没有 checkpoint 时不声称完整或新鲜 |
| 需求与设计 | 同步总需求 v6.84、基本设计 v6.83、Task SRS v0.24、BD v0.17、DD v1.38、Data Design v2.30、实施计划 v7.12 | 明确 source/fixture 已完成，Rust/target DB/production UI 与其余消费者仍开放 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `phase9f4e_bi_task_event_inbox.py --pg-bin D:\PostgreSQL\18\bin` | PASS：隔离 PostgreSQL 18 验证 BI Inbox/projection/quarantine/query migration；新 query migration 双次 apply、function-only grants、tenant/Project scope、相同时间戳 keyset paging、partial cursor/page bound 拒绝与 payload omission 断言通过 |
| CodeGraph | PASS：候选 worktree 已同步当前源码；定向结构查询与源码核查，不宣称全仓覆盖。 |
| `git diff --check` | PASS |
| CypherGraph Guardian | PASS（变更 Rust 源文件 stamp 结果见本轮验证；如后续再改源文件需重跑） |
| Rust 1.99.0 fmt/build/test/Clippy | 未运行：当前 PATH 无 `cargo`、`rustc`、`rustfmt`；不得宣称 Rust gates 通过 |
| 生产数据库/角色 | 未部署或验证；隔离 fixture 不证明 target DB grants、login membership、备份/WAL 或 PITR |
| UI/runtime | 未验证 BI UI、aggregates、Benchmark projection 或生产 receiver；此 API 为读接口 source slice |
| 独立 CodeRabbit review | CLI prerequisite 失败：coderabbit --version 返回 /root/.local/bin/coderabbit: Permission denied（exit 1）；未执行远端review。三个子代理审查不等同CodeRabbit通过 |

## §3 已知缺口

1. 新 Rust query API/store 尚未由 Rust 1.99.0 格式化、编译或测试；workspace gates 也未运行。
2. `BI_DATABASE_URL` 所用 target login 的 `star_bi_runtime` membership/effective grants、migration rollout 和生产连接尚未验收。
3. API 的授权事务保持打开直到 BI 页读取完成；虽有 DB statement/lock timeout、有界 2-connection BI pool 和 2 秒 acquire timeout，新增 provider 6 秒总超时以释放授权事务；仍需生产负载下测量 auth pool 占用与延迟。
4. 当前 coverage/freshness 固定为 unknown；尚无 authoritative consumer checkpoint、gap reconciliation、BI aggregate 或 Benchmark projection。
5. 可信 source transport/identity、route registry、receiver composition、dispatcher accepted settlement、fan-out、crash/restart recovery、备份/WAL/PITR 和 quarantine retention/privacy policy 仍未完成；ingress 与执行继续 fail closed。
6. 未完成 RSS、延迟、公平性及并发负载测量；没有 BI UI。

## §4 子代理失败接手清单

本轮派发代码、文档和拓扑三个独立审查子代理；代码审查发现依赖、rustdoc 与总超时缺口，子代理已修复，主代理复核了 manifest/lock、6 秒 timeout 和 cursor 构造边界。文档与拓扑审查已返回，无失败交接。全部改动位于候选 worktree；最新 Orca 检查中 dev 为 inactive、无 terminal/PTY，合并前仍须重新核对 HEAD 与 clean state。拓扑审查确认候选基线只领先 dev 两个 BI receiver/quarantine 提交；neutral-product-copy 的有效文案已由 f1d2fa40、6cd18fa5、e17e2a1a 选择性纳入 dev，余下旧架构差异不采纳。

## §5 守门规则

| # | 守门 |
|---|---|
| 1 | 主导航与 owner scope 仍为 Project → Engineering Run → Worktree → Branch；查询不要求远端宿主 Branch。 |
| 2 | 每次 query 必须重新校验 actor 的当前 Project/Run read authorization；授权事务持续到分页读取结束。 |
| 3 | BI 查询与主授权数据库使用独立 pool；最多 2 个连接，不启用无界队列或后台查询 worker。 |
| 4 | 未配置 query provider 返回 unavailable；空/非法 URL 不得悄悄连接默认数据库。 |
| 5 | 数据库只开放固定 owner function，不给 API runtime 直接读取 facts 的表权限。 |
| 6 | tenant GUC 是 scope consistency guard，不是 transport/auth 证据；Project/Run 授权由 API owner transaction 决定。 |
| 7 | 页面默认 50、上限 100；游标字段必须成对并受格式/身份校验。 |
| 8 | keyset 顺序固定 `(occurred_at DESC, event_id ASC)`；不可用 offset 或单独 UUID/time 游标。 |
| 9 | 响应只包含批准的投影；不得暴露 event payload、actor、correlation、route 或 wire。 |
| 10 | 没有 durable checkpoint 时，coverage 与 freshness 保持 unknown；缺事件不得解释为零事件。 |
| 11 | Disposable PostgreSQL 结果不证明目标 DB migration、role membership、生产授权或故障恢复。 |
| 12 | Rust 编译、格式、测试和 Clippy 未运行时，不得把本切片标成 Rust gates passed。 |
| 13 | BI UI、aggregate、Benchmark、receiver、fan-out、recovery 与执行继续分别验收，不能由本 read API 推断完成。 |
| 14 | 查询 provider 必须受总 wall-clock timeout 约束，超时释放授权事务并返回 unavailable。 |
| 15 | 合并由主代理串行完成；目标 checkout 的 owner、HEAD、clean state 与依赖范围必须在写入前复核。 |

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
| v0.3 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 Run-scoped BI Task-event read API、固定 DB function、可选两连接 query pool、授权持锁、50/100 keyset、unknown coverage/freshness；记录隔离 PG18 query fixture 通过及 Rust/target/production blockers | Phase 9F4G BI read surface 源码切片 |
| v0.2 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 加入固定 BI owner SQLx quarantine store、append-only T schema、完整 route/cursor exactness；记录隔离 PostgreSQL 18 double-apply、RLS/grants/WORM、输入和两种并发 race；同步 SRS/BD/DD/basic/requirements/Data Design；明确 trusted route/GUC 边界及 production/Rust gates | durable metadata-only quarantine persistence slice 与并发 PostgreSQL verification |
| v0.1 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Phase 9F4G BI Task event receiver core 的实现范围、文档同步、未运行 Rust gates 与生产阻断项 | 增加 provider-neutral receiver core source slice |
