# PHASE-9F3 Schedule Recurrence / PostgreSQL Adapter 阶段报告

> 版本：v0.1.5（2026-10-03）
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 状态：🟢 pinned recurrence 与 disposable PostgreSQL adapter substrate 完成；生产 Schedule capability 仍 fail closed

## §0 目的

在 9F2 版本化 Rule/Occurrence schema 契约上补齐受限 cron/tzdb materializer、租约与持久化适配器，并用隔离 PostgreSQL 验证幂等、tenant RLS、并发 claim/fencing、retry、deadline、exhaustion 和 TTL 语义。该阶段不提供生产 Rule API、常驻 worker 或 Schedule-to-Run admission，不得据此开启生产执行能力。

## §1 改动矩阵

| 面向 | 交付 | 状态与边界 |
|---|---|---|
| Pinned recurrence | 固定 `cron 0.17.0` / `chrono-tz 0.10.4` build identity；支持 5/6-field cron 输入、IANA zone、DST gap/fold、Skip/CoalesceLatest/CatchUp；候选槽扫描和小时级 DST transition 探测各有 32,768 步硬上限，页大小 1–256；禁用 Rule fail closed | 最新 domain debug/release tests 均 26/26；规则版本不支持或 Rule 已禁用时拒绝 materialize，长窗口转换探测超限时要求拆分续页 |
| PostgreSQL repository | 新增 Rule 快照读取、幂等 occurrence/dispatch/event 原子写、租约 claim/reclaim、单调 generation、heartbeat fence、retry、finish 及终态 Work TTL 清理 | 使用 tenant-local RLS scope；当前仅 adapter substrate，无生产 worker/API/grants |
| Deadline/exhaustion 与审计 | claim transaction 内有界 sweep 到期 occurrence 与耗尽最终 lease，将其终态化并 append `deadline_expired` / `dispatch_failed` 事件；未 claim 的 generation 以 NULL 写入审计事件；lease reclaim 事件记录被回收的旧 attempt/generation | PostgreSQL integration 覆盖 deadline、最大尝试耗尽、过期 lease reclaim 与旧 attempt/fence 对应；生产调度与容量运行尚未验收 |
| Migration | 增加 terminal event 类型和 pending/retry_wait → failed 合法转换 | PostgreSQL 18.6 disposable cluster 上首次及重复 apply 均成功；不代表目标环境 rollout 已验证 |
| Desktop/workspace scaffold | 将 Tauri manifest 放回 `src-tauri/` package 根，明确 lib/bin/build script wiring，修复 desktop icon 资源与 CI cache 路径；Windows agent shell adapter 按平台使用 `cmd.exe /C`；更新 Tauri README，区分 fail-closed Task provider 与非权威 Worktree/Canvas demo fixtures | 清除 workspace all-targets gate 的既存入口/manifest 阻塞；`star-desktop` 与 `agent-bridge` 对应测试通过；README 不再把旧 Task mock 写成可用 IPC |
| 验证 runner | 新增 `scripts/automation/phase9f3_schedule.py`，文件化日志、每步骤 900 秒上限、随机 loopback cluster、双次 migration、5 表 FORCE RLS catalog check、非 superuser runtime role、测试后停服确认和仅自有目录 cleanup | 本地 bounded runner 通过；PostgreSQL 目标配置、部署权限及服务运行手册仍开放 |
| 文档 | 同步 requirements v5.56、basic design v5.53、Task SRS v0.12、BD v0.5、DD v1.25、Data Design v0.6、Implementation Plan v5.77、automation-design v1.3、registry v0.43 | 设计声明与已测范围分开；disabled Rule 双层 fail closed、lease-expired audit 保留旧 attempt/fencing generation；9F4 生产 API/worker/admission 门仍开放 |
| 旧 mock Task Card 退役 | `2f7ac2a3` 已合入 `origin/dev` | 作为既有交付核验；本阶段不删除未识别服务端数据或保留的 `test-*` fixtures |

## §2 验证摘要

| 检查 | 结果 | 证据与范围 |
|---|---|---|
| 9F3 bounded runner | 通过，exit 0（最后一次运行早于 DST 探测独立上限修正） | `scripts/automation/phase9f3_schedule.py`；步骤上限 900 秒，结果日志在 ignored `.cache/phase9f3-schedule/`；该后续修正只改 domain materializer，PostgreSQL adapter/migration 源未变化 |
| Domain tests | 26/26 通过 | 最新 checkout 的 `cargo test --locked -p domain-automation --lib -j 4`；包含 disabled Rule 与独立 DST transition scan 上限回归断言 |
| PostgreSQL adapter all-targets compile | 通过 | `.cache/phase9f3-schedule/schedule-adapter-check.stdout.log` |
| Integration harness compile | 通过 | runner 编译目标测试后由 direct test harness 在 PG 生命周期内运行 |
| Clippy | 通过，`-D warnings` | 定向 `domain-automation` / `star-pg-adapter` all-targets；精确豁免既有 lint：`derivable_impls`、`unnecessary_sort_by`、`bool_assert_comparison`、`empty_line_after_outer_attr`、`empty_line_after_doc_comments`、`needless_borrows_for_generic_args`、`let_unit_value` |
| PostgreSQL 18.6 migration | 首次与重复应用均通过 | 独立 loopback-only disposable cluster；不连接目标数据库 |
| FORCE RLS / runtime role | 通过 | 五张表的 FORCE RLS catalog assertion；适配器测试使用非 superuser tenant runtime role |
| PostgreSQL adapter scenarios | 4/4 通过 | 幂等 + tenant RLS + append-only；并发 claim + fencing + heartbeat；retry + terminal TTL + expired lease reclaim（验证事件对应旧 attempt/fence）；deadline + attempt exhaustion |
| PostgreSQL cleanup | 通过 | 停服命令确认 `server stopped` 后清理随机自建 cluster；测试未写入项目数据库 |
| `cargo check --workspace --all-targets` | 通过，exit 0 | 原先 `-j 4` 检查通过；最终代码再以 `-j 2` 重试也通过，存在仓库既有 warnings |
| Desktop unit tests | 28/28 通过 | `cargo test --locked -p star-desktop --lib -j 4` |
| Agent bridge release tests | 30/30 通过 | `cargo test --locked -p agent-bridge --lib --release -j 4`；覆盖 Windows `cmd.exe /C` 路径 |
| Affected domain release tests | 26/26 通过 | 最新 checkout 的 `cargo test --locked -p domain-automation --lib --release -j 1` |
| Workspace release library tests | 最终复跑未完成，exit -1 | 之前完整 workspace suite 曾通过；后续 `-j 2` 与 `-j 1` 复跑均在编译 `star-api-rest` / `application` targets 时异常结束，无 Rust 编译或测试错误诊断；本次新增的 bounded transition probe 单独通过 domain debug/release tests，但未再触发完整 workspace suite；不计作最终全 workspace pass |
| Workspace Clippy | 通过，exit 0 | `cargo clippy --workspace --all-targets -j 4 --locked`；默认 warning 模式，含既有 warnings，不等同 `-D warnings` |
| Workspace release all-targets build | 通过，exit 0 | `cargo build --workspace --release --all-targets -j 4 --locked` |
| Workspace rustdoc | 通过，exit 0 | `cargo doc --workspace --no-deps --release -j 4 --locked`；存在既有 broken intra-doc links、Cypher comment 与 HTML tag warnings |
| Workspace benchmark compile | 通过，exit 0 | `cargo bench --workspace --no-run -j 4 --locked`；只编译 benchmark executables，不运行基准或声称性能测量 |
| Post-review focused rustdoc | 通过，exit 0 | `cargo doc --locked -p domain-automation -p star-pg-adapter --no-deps --release -j 4`；仅有既有 rustdoc warnings |
| CypherGraph Guardian | 通过 | `--changed --stamp` 检查最新改动的 14 个源码文件 |
| Automation registry check | 0 errors / 191 warnings | 220 个既有脚本中有 191 个历史未登记警告；本次 `phase9f3_schedule.py` 已登记 |
| `git diff --check` | 通过 | 无 whitespace errors |
| `cargo fmt --all -- --check` | 失败 | 报告大量未改 workspace 文件的既存 formatting diffs；本次实际改动的 5 个 Rust 文件已单独以 rustfmt `--check --edition 2024` 通过，未重排无关文件 |
| CodeRabbit | 未完成 | CLI 因 WSL `/root/.local/bin/coderabbit: Permission denied` 退出 1；不视为审查通过 |

## §3 已知缺口

1. 本地 PostgreSQL 18.6 disposable cluster 的成功结果不证明目标数据库版本、extensions、权限、runtime grants、migration rollout 或备份恢复策略已通过。
2. 无生产 Rule API/授权端点、materializer worker、lease owner supervisor、调度启停与告警；仍不能运行生产 Schedule。
3. occurrence 到 TaskExecutionRun、Run reservation、RunEvent、outbox 的原子 admission 及当前 Auth、Branch/Run/Worktree、Profile/HookSet、quota、资源预算与 policy recheck 尚未连接。
4. Schedule 执行与既有 Agent/BI/Benchmark 数据流、UI、pause/resume/cancel 操作、运维观测和 target DB 尚未闭合。
5. `cargo fmt --all -- --check` 被大量未改 workspace 文件的既有 formatting drift 阻塞；本次修改的 Rust 文件已分别通过 focused rustfmt 检查。未对无关文件做全仓格式重写。
6. workspace all-targets check（最终 `-j 2`）、Clippy、release build、workspace rustdoc、benchmark compile、最新 domain debug/release tests 26/26 与此前 9F3 targeted PostgreSQL runner 均已通过；PostgreSQL runner 在新增独立 DST transition probe bound 前执行，之后 adapter/migration 源未变化。完整 workspace release lib suite 曾通过，之后两次全量复跑均 exit -1 且无 Rust diagnostic；新增的 domain bounded-scan 修正之后也未重试完整 workspace suite，因此不宣称最终树上的全 workspace release test pass。`cargo fmt --all -- --check` 仍因大量未改 workspace 文件的既有 formatting drift 失败；变更 Rust 文件 focused rustfmt 通过。CodeRabbit 因 WSL permission denied 未完成，人工 diff review 不替代该自动审查结果。
7. 9F4 生产 Rule API、常驻 worker、occurrence→Run/reservation/RunEvent/Outbox 原子 admission、Auth/ACL/current fence recheck、BI consumer、目标 DB/runtime grants 与运行观测尚未完成；Schedule producer capability 必须保持 fail closed。
8. 9F2 报告原先记录执行当时未找到 PostgreSQL 工具；§8 已追加后续发现 18.6 工具的时间边界，不追溯改写 9F2 验证范围。

## §4 子代理失败接手清单

| 子任务 | 失败/限制 | 接手处理 |
|---|---|---|
| Phase 9F3 实现与验证 | 本阶段没有派发实现子代理；本地实现和 runner 结果可直接复核 | root 复核差异、runner 日志、数据库角色和进程结束状态；不以不存在的子代理产出作为证据 |
| Phase 9F2 read-only audit brief | 既有 9F2 报告记录 dispatcher `verify` 失败且输出为空 | 保留在 9F2 报告作为历史限制，不影响本阶段直接构建与 PostgreSQL 实测证据 |

## §5 守门规则

| # | 守门要求 |
|---|---|
| 1 | 新建与消费 occurrence 必须固定 parser/tzdb 版本；不支持的身份拒绝执行。 |
| 2 | 每个 Rule revision 固定 tenant、Project/Branch/EngineeringRun/Repository/Worktree/WorkItem 与 Profile/HookSet snapshot。 |
| 3 | occurrence 唯一键为 `(tenant_id, rule_id, rule_version, scheduled_for_utc)`；重复 materialize 只能得到同一持久事实。 |
| 4 | tenant scope 与 FORCE RLS 必须在 adapter transaction 内；schema/DTO 不能替代 API 权限校验。 |
| 5 | claim、过期 lease reclaim、generation fencing、heartbeat 和状态推进由同一 DB adapter transaction 控制；worker 不得绕过当前 fence。 |
| 6 | occurrence deadline 与 attempt exhaustion 必须被有界终态化并发出 append-only 事件；到期事件不能伪造未发生的 lease generation。 |
| 7 | terminal Work retention 从 terminal timestamp 开始；过期后才允许删除 Work，不得删除 Transaction audit/event 事实。 |
| 8 | candidate scan 与小时级 DST transition probe 各 ≤32,768 步、页 ≤256；超限窗口必须分块续页，不允许无界 CPU 扫描或内存驻留。 |
| 9 | 生产 DB migration、runtime role/grants 与目标 PostgreSQL 版本未验证前，生产 Schedule capability 保持关闭。 |
| 10 | Run admission 必须重新验证 actor、target、Profile、HookSet、quota、当前 fence 与 DB policy，并与 Run/Outbox admission 原子提交。 |
| 11 | 阶段报告按实际日志区分 targeted、workspace 与 production E2E 结果，不得互相替代。 |

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | Mavis 接手审核：pinned materializer 与 lease adapter 的生产边界明确；Run admission 仍缺 |
| SRE Lead | Mavis 接手审核：disposable PG 成功；目标数据库 rollout、grants、worker 运维未验收 |
| 平台 | Mavis 接手审核：current domain release 26/26、adapter/isolated PostgreSQL（DST probe cap 前）、focused rustfmt/Clippy、workspace all-targets check（-j 2）、Clippy/release build/rustdoc/bench compile 已通过；全量 workspace release test 最终复跑出现无诊断 exit -1，最新 bounded-scan 修正后未重跑；全仓 fmt drift 与既有 warnings 已披露 |
| 评审主持 | Mavis 接手审核：人工复核 recurrence、tenant RLS/fencing、审计事实、disabled Rule fail-closed、desktop wiring 与文档差异；CodeRabbit 因 WSL permission denied 未完成 |
| PM | Mavis 接手审核：9F3 targeted PostgreSQL 与 workspace check/build/doc/bench gates 完成；最终 bounded-scan 修正后的 domain debug/release tests 26/26、Clippy 与 focused rustfmt 通过；完整 workspace release test 受环境异常中断且未在最新小修后重跑；commit `4de395d4` 已快进集成到本地 `dev`，普通 `git push origin dev` 因 SSH `kex_exchange_identification: Connection closed by remote host` 失败，远端未确认更新；9F4 尚未开始 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 pinned recurrence、PostgreSQL lease adapter、4 个 DB scenario、targeted gates、文档对账与 9F4 生产缺口 | Phase 9F3 实测完成 |
| v0.1.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 workspace all-targets check 被 base 中的 star-desktop truncated main.rs 阻塞、全仓 fmt baseline drift、targeted fmt 仍通过；CodeRabbit WSL permission denied；明确暂停 dev 集成并调整后续 gate | lockfile 完整纳入 workspace package graph 后执行集成前检查 |
| v0.1.2 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 Tauri package/lib/bin/build wiring、icons 与 CI cache 路径；按平台修复 Windows agent shell 调用；记录 desktop/workspace tests、workspace check/Clippy/release tests/release build 通过，撤销 v0.1.1 已被实证推翻的 workspace 阻塞结论；保留全仓 fmt drift、CodeRabbit 不可用及 9F4 生产缺口 | 完成集成前 workspace/release gate 并复核既有 desktop scaffold |
| v0.1.3 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 自审强化 disabled Rule materialization fail closed，并修正 lease-expired BI/audit event 的 attempt 与旧 fencing generation 对齐；domain release 25/25、4/4 isolated-PG 场景、workspace check/doc/benchmark compile 通过；如实记录全 workspace release test 最终复跑 exit -1、全仓 fmt drift 与 9F4 缺口 | 集成前复核发现 disabled Rule 执行风险和审计 attempt/fence 对应问题 |
| v0.1.4 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加独立 DST transition probe 32,768 步上限与超限拆窗语义；最新 domain debug/release tests 26/26、Clippy/rustfmt 通过；透明记录 PostgreSQL 4/4 runner 在该 domain-only 修正前运行、完整 workspace suite 与 CodeRabbit 限制及 9F4 缺口 | 自审发现长窗口稀疏 Schedule 在 ShiftForward 策略下的时区探测 CPU 无独立上限 |
| v0.1.5 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 9F3 commit `4de395d4` 已本地快进集成到 `dev`；push 因远端 SSH 连接关闭失败，明确远端尚未确认更新；9F4 保持未开始 | 完成本地 dev 集成并验证远端写入受网络连接阻塞 |
