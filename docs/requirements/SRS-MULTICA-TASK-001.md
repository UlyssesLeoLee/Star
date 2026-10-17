# SRS-MULTICA-TASK-001

> **Task Lifecycle 域要件定義書 v0.24** (6 状态 + 独立 Review Gate; Project/Run owner、bounded Task delivery 与 BI owner projection/quarantine/query 增量)
>
> - 状态: 🟡 Draft v0.24
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: `d4edb3945872f3e8819c39c70e93762cf430e834` (Task Run Outbox delivery substrate; route pin/dispatcher/consumer remain gated)
> - 关联基本設計書: [docs/design/BD-MULTICA-TASK-001.md](../design/BD-MULTICA-TASK-001.md) v0.17
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2
> - 关联 inventory: [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.2 (v33 候选)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> - 日期: 2026-10-17 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-TASK-001 |
| 文书名 | Multica Task Lifecycle 域要件定義書 (v33 候选对齐) |
| 版本 | v0.24 |
| 作成日 | 2026-09-11 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 ADR | ADR-0026 v0.2 §1.2 + §2.1 模式 2 |
| 关联 inventory | inventory §2.2 v33 候选 |
| 上位要件 | SRS-MULTICA-RUNTIME-001 v0.1 (Runtime Registry 是 Task Lifecycle 的前置) |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 |
| 子能力 | 5 子能力 (TK-1 ~ TK-5) × 22 项 (per §6) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ADR-0026 v0.2 §1.2 痛点 + §2.1 模式 2, 定义 STAR 平台 **Multica Task Lifecycle 域** 的需求规格说明书。

**核心方向锚点 (per ADR-0026 v0.2 + 2026-09-11 20:10 JST Ulysses 拍板)**: WBS 现有 4 态 → 6 态 (加 `claimed` + `failed`)，review 状态独立于生命周期状态；subagent complete 先进入 review gate，通过后才进 `completed`。**配套** (跟 SRS-MULTICA-POISON-001 强绑定) session poisoning 标记。

### 1.2 背景 (用户痛点)

STAR / Mavis 当前 root session 模型下, 3 类具体痛点 (per ADR-0026 §1.2 + 守门 #9 实证):

1. **Subagent "succeeded" 不可信** — 守门 #9 实证 10/10 `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded
2. **claim 跟 start 状态不显式** — WBS 现在 4 态, 多个 subagent 抢同一任务时竞争不显式
3. **Complete ≠ 入 done** — subagent 报 complete 直接进 done, 无 review gate 拦截质量
4. **Session 不可恢复无标记** — `(agent, issue) session` resume 必然坏的情况, 反复 resume 同一烂 session (per Multica poisoned.go:10-217)

### 1.3 包含范围 (In-Scope)

5 子能力 (per Multica client.go + poisoned.go line refs):

| 子能力 | Multica 源 | 关键 file:line |
|---|---|---|
| TK-1 6 态状态机扩展 | client.go:227-235 (ClaimTask) + task state machine | pending → claimed → in_progress → completed / failed / cancelled |
| TK-2 4 类 404 区分 | client.go:35-91 (isWorkspaceNotFoundError / isTaskNotFoundError / isRuntimeNotFoundError / isUnauthorizedError) | daemon 知道 server 端删除事件 |
| TK-3 Session poison 标记 (跟 SRS-MULTICA-POISON-001 配套) | poisoned.go:10-217 (FailureReason × 5) | 4 类原因 + GetLastTaskSession 过滤 |
| TK-4 Review gate | Multica 默认 subagent complete → 人工 review → done | subagent_review.py |
| TK-5 WBS row schema 升级 | WBS current 4 态 → 6 态 + 字段 | `STAR-P3-WBS-001.md` 状态定义 |

### 1.4 不含范围 (Out-of-Scope, per ADR-0026 v0.2 §2.2)

- ❌ WebSocket claim (Multica real-time push)
- ❌ Per-runtime claim + machine-level batch claim (per Multica 30s/5s timeout)
- ❌ ReclaimStaleDispatchedTasksForRuntimes (per Multica)
- ❌ 5/10 max task concurrency 限制

### 1.5 受众范围 disclaimer

- 本 SRS 跟 WBS 状态定义一对一映射, 跟 `SRS-STAR-AGENT-RUNTIME-001.md` 平行
- WBS row schema 升级由本 SRS 拍板, 实装由 `scripts/automation/wbs_migrate_v33.py` 落地

### 1.6 Star Worktree 集成边界

Task Lifecycle 是 Run 内的任务事实 owner；产品呈现使用中性任务管理能力名称，内部 domain/package 标识不作为 UI 标签。

- 左侧主树固定为 Project → Engineering Run → Worktree → Branch。Branch 是 Worktree 下的本地 checkout 状态叶；远端 SCM 关联可选，不改变主树。
- Engineering Run 是任务、Task Card、Canvas、Inbox、Workflow、BI/Benchmark 与插件 App 的共同授权和数据范围；Task Card 与 Canvas 同级。
- 选中 Run 内 Worktree 才显示所属 Run 的右侧 App tabs，并将其作为 checkout/CLI focus。Run-level Task list/create 不要求已有 Worktree。
- Task Card canonical owner 为 tenant + Project + Engineering Run；Worktree 只是可选执行焦点。存在 focus 时必须复验该 Worktree 当前绑定到同一 Project/Run。
- CLI/Agent Session 仍需 Runtime/AgentPolicy admission、隔离 sandbox、取消/恢复、独立验证和结果回写；这些条件未完成前卡内 CLI 禁用。
- Canvas、任务、聊天和插件通过版本化 RunContext、授权 command 与 typed entity references 协作，不复制任务状态机或放宽权限。
- 底栏 Chat 保留 WORKTREE 与 GLOBAL 两种 scope。插件按 manifest 声明 capability/scope，由宿主授权和隔离运行时约束。
## §2 用语定义

| 用语 | 定义 |
|---|---|
| **Enqueued** | 任务已入 WBS, 等待被 claim (Mavis 自动派) |
| **Claimed** | subagent 已 claim 但未 start (防多 subagent 抢同一任务) |
| **In Progress** (原 started) | subagent 实际开始执行 |
| **Completed** | subagent 报 success + Mavis review 通过 |
| **Failed** | subagent 报 fail (自动) / Mavis 标记 fail (手动) |
| **Cancelled** | 人工取消 (跟 Failed 区分, per ADR-0026 §1.2) |
| **Session poisoned** | `(agent, issue) session` resume 必然坏, 标 `session_poisoned: bool = true` (per poisoned.go 4 类) |
| **Review gate** | subagent 报 complete → Mavis verify (commit hash, output, artifact) → 才进 done |
| **Stale dispatch** | subagent 报 succeeded 但实际 RPC failed (per 守门 #9 实证) — 不进 in_progress 也不进 completed, 标 stale_dispatch = true |

---

## §3 业务背景

### 3.1 Multica 实证 (per ADR-0026 v0.2 §1.3)

| 文件 | 关键 | 引用 |
|---|---|---|
| `client.go:227` | `ClaimTask` (per-runtime 30s timeout) | TK-1 |
| `client.go:294` | `ClaimTasks` (machine-level batch 5s timeout, MUL-4257) | TK-1 (本 SRS 不引入 batch) |
| `client.go:35-91` | 4 类 404 区分 (workspace / task / runtime / unauthorized) | TK-2 |
| `poisoned.go:10-217` | 4 类 session poison 原因 + 4 classify 函数 | TK-3 (跟 SRS-MULTICA-POISON-001 配套) |
| `runtime_sweeper.go:160-170` | 6 阶段 sweep pipeline | 配套 (runtime liveness 跟 task lifecycle 强绑定) |

### 3.2 拍板来源

- 2026-09-11 20:10 JST Ulysses "multica 的核心功能我原则上都要有"
- 20:43 JST ask_user 选项 form_opt2 (分专题) + inventory_opt1 + code_opt1

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发)
- 守门 #5 (env 安全)
- 守门 #6 (PowerShell only)
- 守门 #9 v27 (RPC fallback) — 本 SRS 直接对齐
- 守门 #11 (缺标比错标) — §1.4 + §8 显式列
- 守门 #12 v21 ([P] docs 同步)
- 守门 #14 v4 (Mavis 审核)

---

## §4 功能需求 (FR, 22 项)

### TK-1 6 态状态机扩展 (FR-1 ~ FR-6)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-1 | WBS row `status` 字段加 `claimed` + `failed` 2 态, 总 6 态: pending / claimed / in_progress / completed / failed / cancelled | P0 |
| FR-2 | `claimed` 显式存在, subagent dispatcher 调用后立即标 claimed (claim → start 间隔期间) | P0 |
| FR-3 | `failed` 跟 `cancelled` 区分: failed = subagent 报 fail (自动) / cancelled = 人工取消 (手动) | P0 |
| FR-4 | 状态机迁移脚本 `wbs_migrate_v33.py` 自动改 41 子项 status 字段 (per 守门 #1 v15) | P0 |
| FR-5 | `stale_dispatch: bool` 字段加进 WBS row, 标守门 #9 v27 RPC 失败的 task | P0 |
| FR-6 | 状态机守门: claimed 后 30s 内必须 start, 否则 reaper 自动回 pending (per Multica 30s claim timeout) | P0 |

### TK-2 4 类 404 区分 (FR-7 ~ FR-10)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-7 | WBS row 加 `workspace_not_found_at: timestamp` 字段, server 端 workspace 删时标 | P0 |
| FR-8 | WBS row 加 `task_not_found_at: timestamp` 字段, server 端 task 删时标 | P0 |
| FR-9 | WBS row 加 `runtime_not_found_at: timestamp` 字段, server 端 runtime 删时标 | P0 |
| FR-10 | automation console 显示 4 类删除事件 + 影响 subagent 数 | P0 |

### TK-3 Session poison 标记 (FR-11 ~ FR-15, 跟 SRS-MULTICA-POISON-001 配套)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-11 | WBS row 加 `session_poisoned: bool` 字段, 标 `(agent, issue) session` resume 必然坏 | P0 |
| FR-12 | WBS row 加 `session_poison_reason: enum` 字段, 5 类原因: IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized | P0 |
| FR-13 | 标 session_poisoned 后, 下次 subagent 接到该 task 自动 fresh session 起步 (不 resume) | P0 |
| FR-14 | session_poisoned 标写入审计 log (per 守门 #1 v15) | P0 |
| FR-15 | automation console 显示 🔴 poisoned 标 + 5 类原因过滤 | P0 |

### TK-4 Review gate (FR-16 ~ FR-19)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-16 | subagent 报 complete → 不直接进 done, 设置 `review_state=pending_review` (或 `in_progress` 带 `awaiting_review: bool`) | P0 |
| FR-17 | Mavis review 3 件事: (a) commit hash 存在 / (b) output 落档 / (c) artifact 落档 | P0 |
| FR-18 | Mavis review 通过 → 进 `completed`; review 失败 → 进 `failed` + 标 review_failed_reason | P0 |
| FR-19 | Review gate 可配置 skip (per 守门 #9 v27 fallback) | P0 |

### TK-5 WBS row schema 升级 (FR-20 ~ FR-22)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-20 | WBS row schema 升级, 6 态 lifecycle status + 独立 review_state + session_poisoned + stale_dispatch + 4 类 404 timestamp | P0 |
| FR-21 | 守门 #13 W-T-M 100% 覆盖: 当前态与 claim/session lease 为 Work projection (短 TTL); 状态转换、完成/失败结果、review 决定为 Transaction append-only; task metadata 为 Master SCD Type 2 | P0 |
| FR-22 | WBS row 加 `task_lifecycle_audit: jsonb` 字段, 每次状态变更 +1 审计 entry | P0 |

---

## §5 非功能需求 (NFR, 5 项)

| NFR | 指标 |
|---|---|
| NFR-1 性能 | 状态机迁移 41 子项 < 5s |
| NFR-2 可靠性 | subagent RPC 失败 → stale_dispatch=true, 不阻塞其他 task |
| NFR-3 可观测 | 每次状态变更写 audit log, console 显示状态机 transition 图 |
| NFR-4 易用 | 状态机守门 = 不可绕过, 但提供 force_skip 路径 (per 守门 #9 v27) |
| NFR-5 安全 | session_poisoned 标不删除, 等人工修 (per 守门 #11) |

---

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #9 v27 RPC fallback | stale_dispatch=true 跟 review gate 配套 |
| 守门 #11 缺标比错标 | session_poisoned 不删标 |
| 守门 #13 W-T-M | 6 态 lifecycle status + 独立 review_state 严格分类 |
| 守门 #14 v4 Mavis 审核 | author=Ulysses |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 41 子项 status 字段迁移漏改 | 中 | 中 | `wbs_migrate_v33.py` 自动改 + `registry_check.py` 校验 |
| Review gate 太严阻塞主流程 | 中 | 高 | 提供 force_skip 配置 (per 守门 #9 v27) |
| Session poison 5 类原因分类不够 | 中 | 中 | Multica 当前 5 类, 后续可扩 (per Multica `taskfailure.ReasonAPIInvalidRequest` 集中) |
| Stale dispatch 标了之后没法识别 | 低 | 中 | automation console 红 banner + 周报 |

---

## §7 验收条件 (AC)

| AC | 描述 |
|---|---|
| AC-1 | 41 子项 status 字段全部迁移, 无 pending 漏改 (registry_check 验证) |
| AC-2 | subagent complete → 必进 `review_state=pending_review`, 不直接进 completed |
| AC-3 | session_poisoned 5 类原因枚举全 |
| AC-4 | stale_dispatch=true 时, console 红 banner 显示 |
| AC-5 | 守门 #13 W-T-M 100% 覆盖 (Work / Transaction / Master) |
| AC-6 | automation console 状态机 transition 图可视化 |
| AC-7 | 4 类 404 timestamp 字段全部落档 |
| AC-8 | audit log 每次状态变更 +1 entry |
| AC-9 | 同一 `work_item_id` 从 Multica、Jira 等价视图、Task Card 与 Canvas 进入时保持身份和状态一致, 不生成并行任务状态机 |
| AC-10 | 任务卡 CLI/Agent Session 同时绑定 `worktree_id` 与 `work_item_id`; CLI 不改变 Multica 任务状态定义 |
| AC-11 | Task Card 索引与 Infinite Canvas 在 Worktree 群组中是同级入口; Canvas 写入通过领域命令和权限校验 |
| AC-12 | 插件可声明范围及 capability 并热插拔注册群组入口; 卸载后其持有的任务事实仍由 Multica/WorkItem 域负责 |
| AC-13 | Profile-bound Run 按 root requirements AC-AEC-017 执行：Run Profile/ResourceBudget/Loop/Task/Hook snapshot、Project-wide pending reservation、reservation audit 与同 `event_id` RunEvent 在一个事务提交；Project epoch 写冲突防止 REPEATABLE READ 并发超额；幂等 replay 不重复预留；quota/fence/scope/revision/capacity 任一检查失败则无 Run、无 spawn |
| AC-14 | Run admission 生成有界 typed one-time Runtime fence，绑定 request fingerprint、scope/lifecycle、catalog revisions、HookSet、ResourceBudget 与 Approved Launch Profile/AgentExecutionProfile 两份当前 ID/version/digest；新 Run 保存 Approved Launch Profile identity 和 binding digest，Run Detail 可追溯；Runtime 必须在进程创建前重验授权/版本并原子 consume 一次；错绑、过期、重放或 reservation lifecycle 不可用时 fail closed，production consumer 未装配前不开放新 Run |
| AC-15 | Engineering Run 默认 Task Cards tab 只读取认证的 Run-scoped API；逐页最多 12 项、响应最多 2 MiB、最多浏览 100 页并只驻留当前页；服务端/宿主认证/capability 不可用时 fail closed，禁止 Worktree legacy、seed 或 mock 回退；卡内 CLI 在实际 admission、sandbox、取消/恢复、独立验证与结果回写全部验收前禁用 |
| AC-16 | 移除产品运行时所有已知旧演示 Task Card 与 fallback，包括 Runtime seed/MSW 中 `wi-001..wi-030`、Tauri MockDb 的 `wi-001..wi-004` 和 browser-dev mock 列表；无 canonical Run provider 时返回明确 unavailable/error，不伪装空任务或本地数据。浏览器只迁移删除精确已知旧 ID 及其演示引用，不前缀清理未知本地/服务器行；测试 fixture 使用 `test-*`，不得进入产品投影 |
| AC-17 | 已接入的 Run-local Engineering Loop controller 只接受经验证且 scope 匹配的 Profile 与固定 Run/Task/Worktree binding；每轮复核 Contract/acceptance/HookSet/Validation identity，输出有界摘要与 stop/drain receipt；独立验证通过只进入 AwaitingReview，不能直接完成 Task。持久化、Run/Auth admission、Schedule、CLI process、Outbox/BI 与跨 Run 公平调度仍需单独验收 |
| AC-18 | Schedule rule 以不可变版本保存 recurrence parser/tzdb、timezone/DST、目标 Project/Branch/EngineeringRun/Worktree/Task、Profile/HookSet identity、overlap/misfire/pause/retry/deadline policy；disabled Rule 必须在 materializer 与 adapter 两层拒绝 occurrence 创建。候选槽和 DST 转换探测各有 32,768 次硬上限，超限时调用方需拆分窗口并按游标续页。occurrence 以 tenant + rule ID + rule version + UTC slot 幂等，并保留 local-time/offset 与 target snapshot；dispatch state 使用 terminal_at + retention_period 驱动 TTL，DB trigger 强制单调连续 fencing、禁止盗取未过期 lease，并保留 append-only 事件；lease-expired event 记录被回收的旧 attempt_no 与旧 fencing_generation，所有表按 tenant RLS。Phase 9F3 的 pinned parser/materializer 与 PostgreSQL adapter 已在 disposable PostgreSQL 18.6 通过重复 migration、FORCE RLS catalog 和 4 个真实 DB integration 场景，domain release tests 26/26；这不等于生产 rule write API、持续 worker、Run admission/Auth/Outbox/BI 或目标 DB grants 已启用，缺少这些依赖时必须 fail closed（根要求 AC-LOOP-007） |
| AC-19 | Task Card 的 canonical owner 为 tenant、Project、Engineering Run。Run-scoped GET/POST /api/v1/engineering-runs/{engineering_run_id}/work-items 在无远端宿主 Branch、无 Worktree focus 时仍可列出与创建；POST 中的 Project/Run identity 必须匹配授权 route，服务端在 owner transaction 复验当前 Project/Run grants 和非归档 Run。focus_worktree_id 可选，存在时验证当前 binding 属于同一 Project/Run；repository/remote Branch 仅是可选上下文。owner transaction 写 task metadata、lifecycle、可选 Worktree association、audit、idempotency 和 schema v2 Outbox。客户端 POST 前将精确 body/key 与 scope/authorization snapshot 写入有界 IndexedDB，沿用每 actor/Run 1 条、全局 32 条、29 天重放门、人工核对/清除和不存凭据规则；本地数据未加密且可能被清理，不承诺 exactly-once 或跨设备恢复。旧未归属行不推断回填。目标 migration、app-role grants、RLS、Rust/HTTP/frontend 验证尚未完成；Run-level lifecycle/edit、Schedule 与 CLI execution 仍关闭，缺依赖 fail closed。

---

| AC | 描述 |
|---|---|
| AC-20 | Before the first external send, the Task delivery owner persists an immutable route pin binding sender/receiver service identity, transport provider/version, Profile revision/digest, endpoint and destination generation, schema version, and exact-wire digest. Retries use the same pin and bytes; revoked or drifted routes fail closed without fallback. The delivery Work expiry is fixed at enqueue as `created_at + 7 days` and is not reset at terminalization; terminal settlement appends a permanent receipt before deleting Work, while still-unsettled expiry appends `retention_elapsed` before deletion. Receiver owner Inbox receipt must commit durably before response, and only a trusted Rust dispatcher that authenticates receiver identity and binds the receipt to the source event, route pin, digest, and current lease may settle accepted delivery. Every Run App, subscribed Schedule/Loop, BI, and Benchmark consumer commits its own Inbox receipt with its projection/outcome. Exact replay is idempotent; identity/content conflict is quarantined. Route-pin migrations and the Run-owner Rust source store now exist. The store performs bounded claim, deterministic envelope preparation, independent Transport Profile `event` authorization, authenticated-context-derived sender/receiver, exact-wire pin verification, fenced retry, and proof-gated invalid-envelope dead-letter; source payload size is bounded by the existing 8 KiB database check. It does not send, settle accepted, write a receiver Inbox or consumer projection, or schedule fairly across Runs. A disposable PostgreSQL 18 synthetic-Outbox fixture applied the migration sources twice and passed only its selected schema/role/RLS/WORM/lease/digest assertions; it does not verify the Rust store, target DB, migration rollback, concurrency or production identity. Rust compilation/formatting remains unverified because the toolchain is unavailable; production activation remains disabled until each owner DB/RLS/rollback/replay gate passes. |
| AC-21 | BI is the first Task-event consumer with its own owner-local durable Inbox and immutable event fact; one BI transaction commits both. Exact replay returns the stable receipt; altered scope/route/wire/content appends a payload-free integrity incident and returns no receipt. Accept only strict supported Task event schemas, <=16 KiB wire and <=8 KiB source payload. PostgreSQL 18.6 disposable migration double-apply, tenant FORCE RLS, WORM, effective runtime least privilege, exact replay and conflict cases must pass. BI's owner DB now generates a fixed projection and does not persist raw/free-form Task payload. There is still no trusted transport adapter to create the crate-private authenticated principal, network receiver/dispatcher, accepted settlement or other consumer/fan-out. The Run-scoped read API source is specified by AC-24; production UI remains open. Rust 1.99 build/fmt, target DB identity/grants, backups/WAL retention, concurrency and crash recovery remain activation gates. |
| AC-22 | BI projection v1 MUST be generated at the owner database boundary from the bounded raw input, not trusted from a caller-provided projection. It preserves only tenant/Project/Run/Task/event scope, aggregate/schema version, event type/time, actor/correlation audit identity, owner-change boolean or allowlisted lifecycle/review state transition. Titles, descriptions, labels, branch/worktree/claimant and arbitrary extension fields MUST NOT be retained. Unknown states fail closed. A transactional backfill MUST preserve Inbox receipt/event/route/wire identity and abort without replacing the raw table if any legacy row cannot be projected. Double-apply, legacy raw sentinel removal, v2 write/replay, least privilege, FORCE RLS and WORM are verified by the 9F4F disposable runner; Rust 1.99.0 rustfmt, `bi-owner-runtime` all-target tests (4/4) and strict all-target Clippy `-D warnings` passed. Workspace-wide Rust gates, target DB, production auth/receiver, query Rust gates, UI and retention remain open. |
| AC-23 | BI receiver MUST process at most one route-pinned delivery per call, enforce a 16 KiB wire cap before transport buffering, allow only one in-flight call, and honor caller cancellation/deadline. It MUST authenticate current source/provider and route generation against exact bytes, validate tenant/event identity, commit the BI owner Inbox/fact transaction before ACK, revalidate source authorization before ACK, and return the stable receipt for exact replay. Malformed/tenant/message identity mismatch and Inbox content conflict MUST use metadata-only durable quarantine keyed by complete immutable route identity plus provider/log generation/log/sequence; exact replay compares message ID, digest, wire length and fault, while any change fails closed without a receipt. Permanent disposition is allowed only after a non-nil committed quarantine receipt and another authorization check. Transport pull/ACK/terminal operations must tolerate cancellation and redelivery idempotently. A fixed BI owner PostgreSQL adapter and schema are source-level and passed isolated PostgreSQL 18 double-apply, tenant/RLS/grants/WORM, invalid-input, exact-replay, changed message/digest/length/fault and deterministic unique-key lock-wait exact/conflict gates; the fixture also checks NOINHERIT and denies runtime `SET ROLE` to the owner. Same-cursor conflicting retries do not append a separate incident row, and the append-only table has no quota/partition/production retention policy. The DB trusts route assertions supplied by the trusted Rust adapter; `app.tenant_id` is call scoping, not a defense against a compromised runtime. Trusted production transport/auth/route registry, target DB, receiver composition, production UI, fan-out, accepted settlement, crash/performance and Rust 1.99.0 gates remain open. No production BI ingress or Task/Run execution is enabled. |
| AC-24 | A Run-scoped BI Task-event query MUST require current Project/Run read authorization before calling a separate least-privilege BI query port, and hold the authorization transaction through the page read, bounded by a six-second wall-clock query-provider timeout. If no query provider is installed, return an unavailable response. Use only the fixed BI owner function; constrain by tenant, Project and Run; return only allowlisted event facts without payload, actor, correlation, route or wire data. Default to 50 rows, cap at 100, and page with a stable `(occurred_at DESC, event_id ASC)` keyset cursor; reject partial/invalid cursors and out-of-range page sizes. Completeness and freshness remain `unknown` absent authoritative checkpoints. Isolated PostgreSQL 18 tests cover migration replay, function-only grants, scope, keyset order, cursor rejection and page bounds. This does not enable production UI, aggregates, Benchmark, receiver ingress, fan-out, or execution; Rust and target role deployment remain open. |

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 6 态 state machine 跟现有 WBS row 字段冲突 (e.g. `status` 当前是 enum 4 态) | P0 | 阻塞 AC-1 | `wbs_migrate_v33.py` 兼容迁移 |
| #2 Session poison 5 类原因 跟现有 session 概念区分 (Mavis 当前 session = Mavis root session, 不等于 `(agent, issue) session`) | P0 | 阻塞 AC-3 | 文档加 disclaimer, 实施时统一命名 |
| #3 Review gate 跟守门 #9 v27 fallback 协调 (v27 已是 fallback 路径) | P0 | 阻塞 AC-2 | force_skip 配置 |
| #4 stale_dispatch=true 时, subagent output 怎么保留? (per 守门 #9 v27) | P1 | 不阻塞 | 暂存 `<task_id>.stale.json` 24h |
| #5 4 类 404 timestamp 跟 automation-design §3.4 横向 audit log 范式协调 | P1 | 不阻塞 | 复用现有 audit_log 字段 |
| #6 状态机守门 "claimed 后 30s 必须 start" 跟守门 #9 v27 30s claim timeout 重复 | P1 | 不阻塞 | 复用同一 timeout 常量 |
| #7 Schedule 9F3 parser、bounded materializer、PostgreSQL occurrence/lease adapter 与隔离 DB 验证已完成；parser/tzdb registry、Rule 写 API、持续 clock/worker、Run 同事务 admission、Outbox/BI consumer、Auth 与目标 DB/RLS/grants 未接通 | P0 | 阻塞生产 Schedule Loop | 保持 producer capability 关闭；后续阶段逐项实现并在目标 PostgreSQL/runtime 身份下验证 |
| #8 BI Inbox/fact source 与 DB-side allowlist projection 已有 disposable PostgreSQL 验收；crate-private authenticated principal 尚无 transport factory。Run-scoped query API/source function 有代码与 PG18 fixture，但新 Rust gates、target DB/role membership、production UI、fan-out/recovery 未验收；Task source Outbox 和 database backups/WAL 仍需由各自 owner 明确 retention | P0 | 阻塞生产 BI ingestion | 保持 BI receiver/consumer capability 关闭；完成认证 adapter、生产 read surface、目标部署、备份/WAL retention 与并发/恢复验收 |

Phase 9E-4C3 已交付条件式 Rust/REST writer 与 PostgreSQL migration slice；9E-4C4 已补上双 Profile 一次性 spawn-fence contract、Run identity migration 和 Run Detail 投影。这些不表示生产 Profile-bound Run 已开放。Local Runtime 既有 `domain-local-runtime::task_execution` 只验证签名 grant、scope、Approved Launch Profile ID/字段、canonical checkout、有效期和一次性 nonce；它未绑定 Launch Profile version/digest、AgentExecutionProfile、catalog/HookSet/ResourceBudget 或 C4 fence，也没有生产 provisioner/OS spawn 消费者。Catalog publisher/production source、Approved Launch Profile authority/provider、真实 Auth/Project ACL、目标数据库与 runtime role grants、allocation-epoch/reservation TTL maintenance、C4 Runtime fence consumer 与 reservation activate/release、Outbox/完整 BI 仍未完成；相关 capability 保持 fail closed。导航遵循 ULYS-235：Settings 主导航中的“高级设置”是父入口，Hooks 规范路由 `/settings/advanced/hooks` 是页面内容区与 Skills/MCP/Plugins 并列的 tab，不是 Worktree Group 子级。

Phase 9F2 交付版本化 rule/occurrence/fence contract 与五表 migration；Phase 9F3 实现 cron/tzdb recurrence、bounded DST/misfire materializer、PostgreSQL rule reader 与 occurrence/dispatch lease adapter，并在一次性 PostgreSQL 18.6 loopback 数据库验证重复 migration、五表 FORCE RLS、append-only、tenant/idempotency、并发 claim/fencing、retry/exhaustion、deadline、reclaim 和 Work TTL。候选槽扫描及 DST 转换探测分别有独立硬上限，各为 32,768；超限窗口需拆分续做。disabled Rule 在物化层 fail closed；lease-expired event 记录旧 attempt/generation；domain release tests 26/26。未实现 parser/tzdb registry、Rule 写 API、长期 worker、occurrence-to-Run/reservation/RunEvent/Outbox 原子 admission、生产 Auth/target DB/BI；Schedule execution 仍不可用，不得把本地测试写作 production readiness。

GAP-8: The Run-owner Rust source store now covers bounded claim, authenticated-context route identity, Profile `event` authorization, deterministic wire preparation, immutable route pin, fenced retry and proof-gated invalid-envelope settlement. It has not been compiled/formatted, and no dispatcher send, receiver Inbox, accepted settlement, per-consumer store, fair Run scheduling, target DB grants/RLS or crash/replay acceptance is implemented or verified. Existing source payload DB constraint is 8 KiB, but this does not prove Rust memory/performance budgets. Keep delivery acceptance and downstream projections disabled until the respective owner gates pass.

---

## §9 关联文档

| 类型 | 文档 |
|---|---|
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.2 + §2.1 模式 2 |
| Inventory | [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.2 v33 候选 |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-POISON-001.md`](../requirements/SRS-MULTICA-POISON-001.md) (Session Poison 配套) |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-RUNTIME-001.md`](../requirements/SRS-MULTICA-RUNTIME-001.md) (Runtime Registry 前置) |
| BD | [`docs/design/BD-MULTICA-TASK-001.md`](../design/BD-MULTICA-TASK-001.md) v0.1 |
| WBS | [`STAR-P3-WBS-001.md`](../../STAR-P3-WBS-001.md) 状态定义 |
| 守门 | AGENTS.md §4.1 守门 #1 v15 / #9 v27 / #11 / #13 |

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 20:43 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（22 FR / 5 NFR / 6 已知缺口 + 5 角色签字栏） | 2026-09-11 20:43 JST ask_user 选项 form_opt2 |
| v0.2 | 2026-09-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加渡口 Worktree 群组集成边界与 AC-9..AC-12; 明确 Multica、Jira 等价视图、Task Card、Canvas 共用 WorkItem 身份、卡内 CLI 与插件注册边界；统一 6 态生命周期、独立 review_state 及 W/T/M 当前态投影/审计分类 | 用户要求按 Worktree 为顶层索引推进渡口需求与基本设计 |
| v0.3 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加 AC-13 并引用 root AC-AEC-017；明确 Run/Profile/ResourceBudget/Loop/Task/Hook/BI/reservation 同事务边界、Project 跨 Worktree quota、REPEATABLE READ allocation epoch 防超额与 idempotency replay；列明 Runtime/DB/Auth/BI/C4 未闭合门；Hooks 维持 ULYS-235 Advanced Settings 并列 tab | Phase 9E-4C3 Run 与资源 reservation 代码切片完成 |
| v0.4 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加 AC-14 并引用 root AC-AEC-018；规定双 Profile scope/revision/budget fence、一次消费、Run identity 与 binding digest 投影；记录 Rust/SQL/API type slice 已交付但 Approved Launch Profile authority、production Runtime consumer、reservation lifecycle 与目标环境仍未完成；Hooks 明确为 Advanced Settings 内容区并列 tab | 推进 Phase 9E-4C4 双 Profile fence 与审计投影 |
| v0.5 | 2026-10-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 补充 Local Runtime 基础 grant 校验已存在但不消费 C4 fence 的事实；明确 ULYS-235 的 Settings 主导航父入口、`/settings/advanced/hooks` 并列标签与 Worktree Group 排除规则 | 用户重申 Hooks 属于高级设置选项卡，并要求保留既有导航层级与路径 |
| v0.6 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加 AC-15：Run-scoped Task Cards bounded list、session/capability fail-closed、无 Worktree/mock 回退及 CLI 前置验收门；同步 Task DD v1.19 与 Group DD v4.34 | 将 Task Cards UI/client 实现及生产门控纳入 Multica Task 需求 |
| v0.7 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加 AC-16：退役 Tauri desktop Task mock rows 与 browser-dev fallback；Run provider 未配置时 fail closed；限定 `test-*` fixtures 与精确本地清理范围，不清除未知服务器 owner 行；同步 Task DD v1.20、Group DD v4.35 与总要求 v5.51 | 全仓检查发现独立 Tauri 桌面端仍残留旧演示任务 |
| v0.8 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加 AC-17：Run-local Rust Engineering Loop 绑定 verified Profile/Run/Task/Worktree 和不可变 Contract/Acceptance/Hook/Validation identities；有界摘要、预算 stop、独立验证与 review gate；同步 Task DD v1.21、Basic Design v5.49 与总要求 v5.52，并明确生产 Loop gates 仍开放 | 实现受限 Engineering Loop 核心并对照 Task 生命周期验收 |
| v0.9 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 增加 AC-18：版本化 Schedule rule、UTC-slot occurrence identity、DST/overlap/misfire/pause/retry/deadline policy、canonical Run target/Profile/HookSet snapshot、TTL/fencing/append-only event 与 tenant RLS；记录 domain contract 和 migration substrate 已落地，parser/worker/API/Run admission/Outbox/目标 DB 仍开放；同步 Task BD v0.2、DD v1.22、根要求 v5.53 与根基本设计 v5.50 | Phase 9F2 Schedule/Occurrence 持久化边界与实现对账 |
| v0.10 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 精确 occurrence 幂等身份为 tenant + rule ID + rule version + UTC slot；规定 terminal-based TTL、单调连续 fencing 与活动 lease 不可被抢占，并同步 SRS/BD/DD/Data Design 与 root 基线版本 | Phase 9F2 对账发现幂等身份表述和实现约束需精确化 |
| v0.11 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 对账 Phase 9F3 pinned cron/tzdb materializer、PostgreSQL persistence/lease adapter、deadline/exhaustion finalizer 与隔离 PostgreSQL 18.6 migration/RLS/4 场景实测；保留 production API/worker/Run admission/Auth/Outbox/BI/target DB 阻断 | 9F3 验证结果需精确反映到 Schedule acceptance 与生产边界 |
| v0.12 | 2026-10-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 明确两个 32,768 扫描上限、disabled Rule 在物化与 adapter 两层 fail closed、lease-expired 审计保留旧 attempt/fencing generation；同步最终 domain release 26/26 与当前 9F4 生产 blockers | 最终源码自审修复安全执行条件、转换扫描预算与审计事件身份对应 |
| v0.13 | 2026-10-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将渡口产品层级统一为 Project → Cloud Branch → Engineering Run → Run Worktree；新增 AC-19 约束有写授权、固定 Worktree 重试目标、server-derived Run owner 和事务 Outbox 的条件式 Task Card 创建；保留 capability/DB/RLS/host session 与 CLI 阻断状态 | 用户要求以本对话确立的导航与 Worktree 归属设计筛选当前实现 |
| v0.23 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修订 AC-23：增加固定 BI owner durable metadata-only quarantine adapter/schema，完整 route/cursor key 与 message ID/digest/length/fault exactness；登记隔离 PG18 双次 migration、RLS/grants/WORM/输入和并发竞态验证；说明 DB/tenant GUC 信任边界及生产 transport/identity/target DB/Rust gates 仍开放 | Phase 9F4G quarantine persistence slice 与 DB concurrency gate |
| v0.24 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 AC-24：Run 授权的只读 BI Task-event query、可选双连接 pool、function-only scope、50/100 keyset 分页及 unknown coverage/freshness；记录 PG18 query gates 与 Rust/target/UI 未验收 | Phase 9F4G BI read API 源码和隔离数据库验证 |
| v0.22 | 2026-10-17 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 AC-23：明确 provider-neutral BI receiver 的单条/16 KiB/单并发/cancel-deadline、current route/origin authorization、owner commit-before-ACK、exact replay、durable metadata-only quarantine 幂等与未验收门禁；production ingress 继续关闭 | Phase 9F4G 推进 BI Task event receiver core |
| v0.21 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 AC-22：BI owner DB 生成并持久化固定字段 projection，legacy raw facts 原子转换且 identity/receipt 保持稳定；PG18.6 回填、双次迁移、raw sentinel 丢弃、重放、RLS/WORM/grants 与 Rust 1.99.0 `bi-owner-runtime` rustfmt/all-target tests (4/4)/strict Clippy 通过；workspace-wide checks、认证 receiver、BI read API/UI、target DB、备份/WAL 仍开放 | 9F4E 自审发现永久保留 raw/free-form Task payload，新增最小化 projection 层 |
| v0.20 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 BI owner Inbox + immutable event fact + payload-free conflict incident 的需求与 AC-21；记录 disposable PostgreSQL 18.6 双次 migration/夹具门通过；明确 crate-private principal 尚无认证 transport、BI projection/fan-out/accepted 与 payload retention 均未关闭，Rust/target DB 未验收 | Phase 9F4E 首个 Task event 消费者持久化切片 |
| v0.19 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对账 Run-owner Rust delivery store 的有界 claim、Profile `event` 授权、已认证路由身份、exact-wire pin、fenced retry 与 proof dead-letter；确认现有 8 KiB payload DB check；明确未编译、未发送、无 accepted/receiver/consumer/fair scheduler | ERUN-P3.3 Rust delivery substrate 实现与需求对账 |
| v0.18 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对账 Task route-pin table/fixed-function migration source；记录 disposable PostgreSQL 18 fixture 双次应用与选定 FORCE RLS、least-privilege、WORM、lease/schema/idempotency/transaction rollback assertions 通过；保留 target DB、migration rollback/downgrade、concurrency 与 receiver/consumer/dispatcher/accepted 关闭 | ERUN-P3.3 增加首发 route pin DDL，但无数据库或 Rust dispatch 验收 |
| v0.17 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修订 AC-20：delivery Work expiry 固定为 enqueue `created_at + 7 days`，不得写作 terminal 后开始 TTL；提前终态与过期均先记录永久 receipt 再删除 Work | 对照 Task delivery DDL 的 `expires_at = created_at + 7 days` 修正文档语义 |
| v0.16 | 2026-10-16 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 AC-20：不可变首发 route pin、exact-wire retry、receiver owner durable Inbox receipt、可信 dispatcher settlement 与 per-consumer Inbox/projection commit；明确 route pin/dispatcher/receiver/consumer stores 和目标 DB gates 未实现，accepted delivery 保持关闭 | Task Run Outbox delivery migration review 发现缺失 route pin 且 ingress receipt 无法证明逐 consumer commit |
| v0.15 | 2026-10-15 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 Project → Run → Worktree → Branch 与中性 UI 文案；将 Task owner 定为 Project/Run 并新增无宿主、无 focus 的 list/create AC；同步 target migration/RLS/validation blockers 与 lifecycle/Schedule/CLI closed status | ERUN-P3.2 host-independent Task Card owner 源码切片 |
| v0.14 | 2026-10-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 扩展 AC-19：在 POST 前写入按 tenant/actor/Project/Run 隔离的有界 IndexedDB pending command，固定当前授权/binding 快照，限制 32 条与 29 天重试窗口；明确本地存储失效、记录缺失、手动清除和非 exactly-once 边界；同步 Task BD/DD 与根要求/基本设计/Group DD/实施计划 | 补齐 Task Card create 在 remount/reload 后的本地恢复切片 |
