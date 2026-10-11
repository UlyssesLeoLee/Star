# PHASE-9F2 Schedule / Occurrence 阶段报告

> 版本：v0.1（2026-10-02）
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 状态：🟢 Domain/schema substrate 阶段完成；生产 Schedule capability 仍关闭

## §0 目的

本阶段把 Schedule Loop 纳入既有 `domain-automation` 与 Engineering Run admission 边界，形成版本化 Rule、幂等 Occurrence、租约 fencing 与终态保留的 Rust/SQL 持久化契约。范围只到 domain/schema substrate；不宣称生产调度器、数据库迁移或 Schedule-to-Run 闭环已可用。

## §1 改动矩阵

| 面向 | 交付 | 状态与边界 |
|---|---|---|
| Rust domain contract | 新增 `domain-automation::schedule`，定义不可变 Rule revision、Project/Branch/EngineeringRun/Repository/Worktree/WorkItem/Profile/HookSet target、tenant + rule ID + rule version + UTC slot occurrence key、local label/offset snapshot 与 lease fence DTO | 已交付；cron 表达式和 IANA timezone 只做有界输入形状校验，没有实际 parser/tzdb catalog |
| Durable schema | 新增五表 migration：Rule revision Master/SCD2、Rule Audit Transaction、Occurrence Transaction、Dispatch Work、Occurrence Event Transaction；复合唯一键、scope FK、append-only/close-only guard 与 tenant FORCE RLS | 源码已交付；未应用 PostgreSQL，DDL/RLS/grants 仍未验收 |
| Dispatch invariants | DB trigger 定义连续递增 fencing generation/attempt、过期 lease reclaim、禁止抢占 active lease、heartbeat owner/expiry 单调、终态不可改写和 terminal-based TTL | migration 源码有对应约束；数据库行为未实证 |
| 验证工具 | 新增 `scripts/automation/phase9f2_schedule_occurrence.py`：日志文件化、单步骤 900 秒上限、Rust 测试、Clippy、rustfmt、migration source-text check；显式 online transient 模式会在执行后精确恢复 `Cargo.lock` | 已交付；默认离线受当前 workspace/offline index 的 `objc2` 版本冲突阻塞，显式 `--online` 路径通过本报告 §2 所列检查 |
| 文档同步 | 同步 Task SRS v0.10、Task BD v0.3、Task DD v1.23、Data Design v0.4、根 requirements/basic design 与实施计划 v5.71、automation-design/registry | 已同步；各处均保留数据库和生产能力未完成状态 |
| 旧 mock Task Card 退役 | 用户授权删除旧演示任务后，全仓精确清理已由 `2f7ac2a3` 完成 | 已确认该提交在 `origin/dev` 历史中；本阶段没有扩大删除未知服务器数据 |

## §2 验证摘要

| 检查 | 结果 | 说明 |
|---|---|---|
| `python scripts/automation/phase9f2_schedule_occurrence.py --online ...` | 通过 | migration source-text contract、rustfmt、`domain-automation` library tests 与定向 Clippy 全通过；`Cargo.lock` 精确恢复 |
| Domain unit tests | 20/20 通过 | 只验证 Rust contract 与既有 crate tests |
| Clippy | 通过 | `--no-deps -D warnings`；仅豁免旧代码已有的 `derivable_impls`、`unnecessary_sort_by`、`bool_assert_comparison` 三类 lint，其余 warning 仍失败 |
| rustfmt | 通过 | 对 `domain-automation` 两个相关源文件执行 `--check` |
| Migration source-text contract | 通过 | 确认五表、复合唯一键、dispatch guard、终态 retention 与 FORCE RLS 源码片段存在；不是 SQL parser 或 PostgreSQL 实证 |
| Python syntax / CypherGraph Guardian | 通过 | runner `py_compile` 无错误；Guardian `--changed --stamp` 对 3 个变更源码文件 PASS |
| Git whitespace / lock integrity | 通过 | `git diff --check` 无错误；runner 报告 `cargo_lock_restored=true` |
| CodeRabbit independent review | 未完成 | Windows wrapper 调用 WSL 后报 `/bin/bash: /root/.local/bin/coderabbit: Permission denied`；不视为 review 通过 |
| PostgreSQL DDL/RLS | 未执行 | Docker API 无法连接；未找到 `psql` / `pg_ctl`，无 disposable PostgreSQL 实例 |

## §3 已知缺口

1. 当前离线 Cargo 依赖索引与 workspace lock 的 Tauri/Wry 依赖不一致：`wry v0.55.1` 要求 `objc2 ^0.6.4`，离线解析选择与该约束冲突。显式 online 验证成功，但没有改写提交中的 `Cargo.lock`。
2. 无 PostgreSQL 环境，因此 migration 的 SQL 语法、幂等重放、RLS、trigger、并发 claim/reclaim、TTL 删除均未在数据库验证；目标 runtime role grants 也未部署。
3. Schedule recurrence parser、固定版本 tzdb catalog、Rule API/persistence adapter、due materializer、worker claim/recovery/heartbeat/retry 尚未实现。
4. Schedule occurrence 到 TaskExecutionRun/reservation/RunEvent/outbox 的同事务 admission、实时 Auth/target/Profile/Hook/quota/fence 复核、BI 与生产 executor 尚未闭合；Schedule capability 必须 fail closed。
5. audit brief 的 dispatcher `verify` 返回失败，`collect_output` 只生成空内容 stub。已收到的审计结论由 root 结合源码复核；dispatcher artifact/status 不能作为自动化子任务成功证据。
6. CodeRabbit 因 WSL wrapper 执行权限错误未能审阅；本阶段仅完成 root 的本地源码/文档自审，未取得独立审阅结论。

## §4 子代理失败接手清单

| 子任务 | 失败/限制 | 接手处理 |
|---|---|---|
| Phase 9F2 read-only audit brief | dispatcher `verify` 失败；`collect_output` 没有可验证正文 | 不依赖该 artifact 声称自动验证成功；root 逐项核对 domain、migration 与设计文档，保留 brief 与空 stub 作为限制记录 |
| 子代理实现 | 本阶段没有可合并的子代理实现提交 | 所有阶段实现由 root worktree 汇总；未把 mailbox 返回状态当作 Git/测试证据 |

## §5 守门规则

| # | 守门要求 |
|---|---|
| 1 | Rule revision 为不可变版本；仅允许一次 close-only `valid_to` 更新，不得删除历史版本。 |
| 2 | 每个 Rule 必须固定 Project/Branch/EngineeringRun/Repository/Worktree/WorkItem 与 Profile/HookSet version/digest。 |
| 3 | occurrence 唯一身份固定为 `(tenant_id, rule_id, rule_version, scheduled_for_utc)`。 |
| 4 | UTC slot 是执行身份；local-time label、offset、parser version 和 tzdb version 只用于解释已 materialize 的时刻。 |
| 5 | DST gap/fold、overlap、misfire、pause、retry、deadline policy 必须版本化并有界。 |
| 6 | bounded overlap 最大并行数不超过 64；catch-up occurrence 上限不超过 256。 |
| 7 | retry 尝试总数不超过 25，backoff 不超过 24 小时；单次 deadline 不超过 24 小时。 |
| 8 | Occurrence/target snapshot 与历史事件 append-only；不得通过更新旧 snapshot 修补新 Rule。 |
| 9 | Dispatch fence generation 与 attempt 只能单调连续增长；每次新 lease claim/reclaim 必须一并推进。 |
| 10 | 未过期 lease 不可被其他 worker 抢占；同 generation heartbeat 必须保留 owner 且不得缩短 lease。 |
| 11 | 过期 lease/retry 转换、generation 检查与后续状态写必须由同一数据库事务适配器完成。 |
| 12 | terminal dispatch 不可重写；Work TTL 从 `terminal_at + retention_period` 起算，过期后才可删除。 |
| 13 | 五张表都必须按 tenant 隔离并使用 FORCE RLS；目标数据库和 runtime grants 未验收前不能声称该规则已运行。 |
| 14 | Run admission 必须再次验证 actor、target、Profile、HookSet、quota、当前 fence 与 DB policy；schema/DTO 不替代授权。 |
| 15 | 缺 parser、worker、Auth/provider、目标 DB 或 Runtime consumer 时，Schedule execution capability 必须 fail closed。 |
| 16 | online Cargo resolver 只能在 `Cargo.lock` 与 HEAD 一致时运行，执行结束须精确恢复 lock 原字节并报告实际模式。 |
| 17 | 测试/源码片段、静态检查和 mailbox 输出不能冒充 PostgreSQL 行为、生产 E2E 或自动化子代理验证证据。 |

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | Mavis 接手审核：domain/schema substrate 与生产启用边界清晰；数据库运行证据仍缺 |
| SRE Lead | Mavis 接手审核：目标 PostgreSQL、RLS/grants、migration rollout 与 worker 运维未验收 |
| 平台 | Mavis 接手审核：Rust contract 与 bounded runner 已交付，生产 Runtime/worker 未装配 |
| 评审主持 | Mavis 接手审核：已列明测试证据、dispatch 验证失败与剩余缺口 |
| PM | Mavis 接手审核：Phase 9F2 完成；9F3/9F4 仍是独立后续阶段 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 9F2 domain/schema substrate、online transient Rust 验证、Cargo.lock 恢复、CypherGraph Guardian 结果以及未完成的 PostgreSQL/worker/Run admission 门 | Phase 9F2 实现完成并与需求、基本设计和详细设计对账 |

## §8 后续环境发现更正（2026-10-02）

9F2 执行当时未发现可用的 `psql` / `pg_ctl`，因此没有执行该阶段 migration；上述判断准确描述 9F2 当时的运行环境与证据。9F3 后续在 `D:\PostgreSQL\18\bin` 发现 PostgreSQL 18.6 工具，并用独立 disposable loopback cluster 验证 9F3 schema 与 adapter。该新环境不追溯改变 9F2 的验证范围：9F2 migration 在 9F2 阶段仍未应用或执行；9F3 的四个数据库场景只验证当时迁移文件与 adapter 路径。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 保留 9F2 原始环境证据，并补充后来发现 PostgreSQL 18.6 及 9F3 实测的时间边界；不将 9F3 实测记作 9F2 验收 | Phase 9F3 发现本机 PostgreSQL 工具但不改变 9F2 当时证据 |
