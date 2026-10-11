# PHASE-ERUN-DIRECTORY-P2-REPORT

> v0.2 · 2026-10-02 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 状态：ERUN-P2 条件式实现、定向验证并集成到最新 dev 基线；生产 CLI 闭环、Run Apps 迁移和全 Phase 完成门仍开放。
> 自动化：`scripts/automation/engineering_run_directory.py`；`automation-design.md §4.37`；实施计划 §6.68。

## §0 目的

把 canonical EngineeringRun identity 接入现有任务卡 CLI admission、Runtime fence 与 attachment 再授权边界；接通 Run shell 到 Advanced Settings → Hooks 的授权范围深链。EngineeringRun 是协作 owner，TaskExecutionRun 是执行尝试，Worktree 是 checkout focus；三者不复用身份。该切片不迁移 Task/Canvas owner，也不启用缺少生产 provider 的执行能力。

## §1 改动矩阵

| 范围 | 实施 | 责任 |
|---|---|---|
| REST CLI | 当前 directory/三层 grants/revisions/checkout branch/Task association 服务端解析，两事务 exact-match，replay 和 attachment 的 stored/current identity 比较 | CLI worker 独立 worktree |
| DTO/Runtime | strict identity DTO、V2 fence digest、fenced grant signature v3、current-binding compare/原子 consume 与 prepared envelope | CLI worker；root/独立 reviewer 定向审查 |
| DB | nullable additive 身份列与 snapshot、精确复合 FK、strict 25-field shape、new CLI INSERT-only NULL gate；保留 historical/non-CLI NULL | CLI worker；root disposable DDL |
| Hooks UI | hint UUID/重复/完整性门、当前 session 目录查找、显式清除、异步 generation 复验、旧事件隔离；每页≤200/≤20页 | Hook worker 独立 worktree |
| 对账 | requirements v5.46、basic v5.43、Group DD v4.30、Task DD v1.18、plan v5.61、方向指引 v1.2、Infrastructure DD v0.2 与 runner/registry | root；保留 dev 新增 Index/ADR/桌面文档 |

原设计改善：Snapshot 从只有 tuple 扩展为完整 grants/revision/binding 证据；signed fence 固定完整 directory identity；attachment 单独复验，cleanup 不依赖 active Run；深链 Ready 绑定当前 authenticated client 和已确认 scope。改进与尚未实现的生产条件在三层设计同步记录。

## §2 验证摘要

| 证据层 | 本阶段结果 | 限制 |
|---|---|---|
| Hooks worker | TypeScript 通过；HookPolicyPage 15/15 单测通过 | 单测有替身，未证明生产 session/真实目录部署 |
| 整合 frontend | 全量 `tsc --noEmit --incremental false --pretty false` 通过；HookPolicyPage 15/15 单测通过 | source 为 rebase 后 Hook；不代替 browser/desktop runtime |
| CLI Rust | integration 最新代码：`cargo check -p star-api-rest --all-targets -j 4` 通过；Runtime fence 5/5、REST CLI 7/7 通过 | 仓库 runner 暂时排除 desktop 成员并重解 workspace lock；runner 退出后 `Cargo.toml`/`Cargo.lock` 与提交基线干净。存在既有 unused-import/dead-code warning；首次在未合入 CLI lane 的旧 integration 源码上失败，随后按正确顺序合入并重跑通过 |
| PostgreSQL 18 | 本次迁移首轮与重复应用均通过；历史 NULL 保留，新 CLI NULL、tuple-only/缺字段/超限 snapshot、缺 canonical directory/FK 共 5 类拒绝场景通过 | 使用 disposable loopback cluster；legacy audit stand-in；未证明目标库部署、真实角色 RLS、positive admission 或并发唯一键竞态行为 |
| source review | CLI 独立最终复核无剩余 actionable finding；strict snapshot、attachment 与唯一键竞态 fallback 均按源码核对 | CodeGraph/index 不可用，使用 bounded source review；不作 exhaustive graph claim。CodeRabbit CLI 在 WSL 的 `/root/.local/bin/coderabbit` 遇 Permission denied，未得到通过结论 |
| CodeRabbit | 无通过结论 | Windows wrapper 进入 WSL `/root/.local/bin/coderabbit` Permission denied，未更改宿主或安装工具 |
| 产品运行/性能/发布 | 未验收 | 无真实 CLI execution、独立 validation/writeback、桌面打包/RSS/并行 p95 证据 |

DDL guard 仅证明 historical unbound row 保留，以及 new CLI NULL、tuple-only/missing-key/invalid-oversized snapshot、缺 canonical directory/FK 等拒绝场景。超限用额外未知字段构造，不能单独证明 size constraint 路径。Strict SQL shape/FK 不授予 current grants；它们由 REST 复验，Runtime/provider 在使用时仍须实时复验。

DDL evidence：integration `.cache/erun-ddl-verify/` 分步日志；migration SHA-256 `0875B89078B6B5CBEDEF697A33726577F2BAD8055A2101489FFCDF71EAFF209C`。Rust evidence：`.cache/erun-cli-verify/`；REST `--all-targets` check 56.05s，Runtime 5 fence tests 36.55s build + 0.03s run，REST 7 CLI tests 2m26s build + 0.00s run；Cargo lock/manifest 在结束后恢复为干净状态。

## §3 已知缺口

1. 宿主真实 session/IdP/同源反代、可信 SCM Branch ingest、initial grants/Run kickoff/binding writer、目标 DB/runtime roles 仍未装配。
2. Task source 仍属旧 Project/Worktree compatibility model；尚无 canonical Run Task owner、Run Task API/独立 App 和历史归属 reconciliation。Canvas/Inbox/Workflow/Plugin 的 Run owner 亦未迁移，shell 能力继续 false。
3. `TaskCliSessionProvisioner` 没有生产实现；current authority/Profile/catalog/Hook providers、reservation lifecycle、OS sandbox/spawn、bounded output/cancel/recovery、独立验证、结果/BI 写回未接。V2 fence helper 和附件授权 contract 不等于实际安全执行闭环。
4. Attachment 在 REST 签发前重验仅缩短 race window；可信 provider 在使用时必须重新验证当前身份、权限/策略版本及预算，不能凭返回快照授权任意 Shell。status/cancel 保留受权的安全清理语义。
5. Snapshot shape 与 composite FK 已有代码门，不证明目标 role 的 RLS/并发/撤权运行行为。历史 NULL 不猜回填；非 CLI producers 仍待迁移，不能称全部尝试已有 EngineeringRun owner。
6. Hooks 页可忽略迟到响应并停止后续分页，尚未将已发 HTTP 接入 AbortSignal/底层 cancel；每页/页数/缓存有界不等于 transport drain 或实测 RSS 达标。
7. Native Hook 全生命周期、Schedule Loop occurrence/fencing/budget/checkpoint、durable Outbox/Inbox/realtime、Plugin isolation/hot revoke 和完整 Run BI/Benchmark 仍各有完成门。
8. 仓库完整 workspace lock 解析有既有桌面依赖冲突；本阶段用受控 backend-only 临时解析验证，不把定向 checks 称为 workspace/release 全通过。
9. CLI 相同幂等键并发时仅捕获精确唯一键与 SQLSTATE 23505；输家回滚后在新快照重验授权并回放，不创建第二个 Run/fence。尚未用 PostgreSQL 双连接实测竞态，因此 production admission 继续关闭。

下一轮依赖顺序：共享 Run authority port → canonical Task owner/SCD2/reconciliation → Run Task API/App → CLI owner/focus 一致门 → Canvas owner/EntityRef/owner commands → durable 跨 App events；同时推进真实 identity/SCM/grants/Runtime 环境。Phase 1 和全目标保持开放。

## §4 子代理失败接手清单

| lane/审阅 | 可核对证据 | 接手与限制 |
|---|---|---|
| Hook | scoped rebase commit `65c0570c`；integration merge `9a250be8` | 两份 HookPolicyPage 文件；worker typecheck 与 15/15 单测通过；deep link 只作授权后 hint |
| CLI | commit `ab037da2`，并入 integration merge `d897d963`；REST check + Runtime 5/5 + REST 7/7 通过 | 原代理中断后重读现存 draft 续做；相同幂等键竞态只基于源码验证，缺 PostgreSQL 双连接实测 |
| CLI reviewer | `docs/briefs/ERUN-P2-CLI-REVIEW.md`；commit 后独立最终源码复核 | 修正 tuple-only snapshot、attachment 当前授权和 RR 唯一键竞态处理；无剩余 actionable finding |
| root integration | 基于最新 `origin/dev` `78adb5d5`；保留 Hook merge、商用许可设计、Task CLI 与 phase report | 不触碰 D:/Star 的既有 next-env 生成改动或其他 dev checkout/index；未强推 |

CLI lane 以 `--onto origin/dev 3966dab7` 只重放 owned commit；integration 以 `--rebase-merges --onto origin/dev c9286a17` 保留 Hook merge；随后串行合并 CLI lane。未覆盖无关 Canvas 修改；最终 push 与 remote pointer 将在本报告提交后单独核对，不 force push。

`SubagentDispatcher.brief` 真实落地；native collaboration 状态与真实 source/commit/log 对照。既有 dispatcher invoke/verify stub 不作成功证据。需要清理时只归档本次已整合的 managed worktree；旧或用户所有 worktree 不在清理范围。

## §5 守门规则

| # | 本阶段规则 |
|---|---|
| 1 | Ulysses author/审核；不编造历史或代报检查 |
| 2 | worker 独立 worktree，root 串行整合 |
| 3 | scoped rebase 只重放 owned commit；非 force push |
| 4 | 保留用户其他 checkout/index/next-env 改动 |
| 5 | 不打印 secrets，不使用公开 key/seed 冒充会话 |
| 6 | Rust -j4、独立 target，compile/test/timeout 分开 |
| 7 | 原 Cargo.lock 精确恢复，不提交无关 churn |
| 8 | Project/Branch/Run current writer grants 与 revision exact-match |
| 9 | EngineeringRun 与 TaskExecutionRun ID 独立，owner/focus 分离 |
| 10 | strict snapshot + composite FK + new CLI NULL gate |
| 11 | 历史/非 CLI NULL 不猜回填或重启 |
| 12 | V2 fence/signature v3，原子 nonce/fence consume，不启用未接 provider |
| 13 | attachment 再授权；status/cancel 保留安全 cleanup |
| 14 | Hooks 固定高级设置 tab，hint 不进写 body |
| 15 | bounded directory paging 与 session/generation 门，底层 abort 缺口明示 |
| 16 | disposable DB 路径核验、仅自有集群、停止成功后清理；无生产 migration |
| 17 | 三层设计/验收同步；未运行项不标完成，目标仍 active |

## §6 签字栏

| 角色 | 审核范围 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）；身份与 owner/focus 边界，生产门开放 |
| SRE Lead | Mavis 接手审核；检查层次与锁/资源限制，不代报部署/性能 |
| 平台 | Mavis 接手审核；缺 provider/sandbox/runtime 时保持禁用 |
| 评审主持 | Mavis 接手审核；独立源码发现与最终定向复核，不称 CodeRabbit 通过 |
| PM | Mavis 接手审核；阶段结果可追踪，下一轮 Task Run owner 与真实闭环 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容/触发 |
|---|---|---|---|
| v0.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | ERUN-P2 并行 lane、身份/附件/深链门、独立发现与精确验证边界；用户授权 rebase 后 merge dev |
| v0.2 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 吸收唯一键竞态复核、重放验证与商业宽松许可约束；更新最新 dev 基线、commit、测试和 DDL 证据；列出未完成生产门 |
