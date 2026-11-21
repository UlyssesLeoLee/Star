# F03 Worktree 领域模型与状态机

> 状态摘要: 领域聚合 `crates/domain-worktree` 在内存中实现了 17 状态枚举、严格迁移表、5 个命令端口与 5 个查询端口；REST 与 MCP 只暴露创建与读取。线上持久化使用另一套模型：表 `worktree_canvas_worktree` 以 7 值 `human_state` 与 `archived` 标志表示生命周期，经 group API 可写的只有归档/恢复与 owner 分配两类管理计划。17 状态名与设计 §7.1 只重合 11 个；Completion Gate、Isolation、观测分级、Reconciliation、领域事件与 WT 错误码未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

- 把 Worktree 建模为独立一级聚合根，持有自身的 status、health、conflict_state 与 Runtime 绑定，不作为 Repository 或 Branch 的附属字段（domain-worktree-spec §1；plan-016 目标 1、3）。
- 以 17 状态严格迁移表约束生命周期，Worktree 状态与 WorkItem 状态相互独立（INV-WT-01、INV-WT-02；basic-design.md §7.1）。
- 所有读写按 tenant 隔离，每个 Worktree 绑定一个 Runtime（INV-WT-03、INV-WT-08）。
- 通过 WorktreeCommandPort 与 WorktreeQueryPort 提供操作边界（domain-worktree-spec §4）。线上持久化与画布状态投影之间的关系见 §4.1。

### 1.2 范围

| 类别 | 内容 |
|---|---|
| 范围内 | 域聚合与 17 状态枚举、迁移表；命令与查询端口及内存实现；ContextEdit 与 Fork（`edit` 模块）；线上表 `worktree_canvas_worktree` 的生命周期字段；归档与 owner 管理计划；REST 与 MCP 的创建与读取入口 |
| 范围外 | Repository、Branch、Commit（domain-scm）；ChangeSet 与 DevelopmentExecution（domain-development）；AgentSession（domain-agent，本域仅保存 ID）；ADR-0039 的 8 步 Worktree Orchestration Saga 与事件矩阵（跨域，见 §6）；Worktree 画布、关系与风险视图（F07）；Project、Lane、Run 导航与页面（F01）；导入与 Start-from-picker 路由（`worktree_picker`、`worktree_external`、`worktree-shared-dir`） |

## 2. 用户与场景

| 角色 | 场景 | 代码依据 |
|---|---|---|
| 开发者（developer） | 为 WorkItem 创建 Worktree | 域内 `create_worktree`（crates/domain-worktree/src/lib.rs:720-761）；REST `POST /api/v1/worktrees`（crates/star-api-rest/src/routes/worktrees.rs:71-113）；线上 `POST /api/v1/engineering-runs/{engineering_run_id}/worktrees`（crates/star-api-rest/src/group_api/worktree_lifecycle.rs:411-414, 587） |
| Local Runtime | 上报 ahead、behind 与当前会话；唯一可写观测的 actor | crates/domain-worktree/src/lib.rs:797-799；crates/star-context/src/actor.rs:74 |
| 平台或 Agent | 把 READY 状态的 Worktree 分配给 AgentSession | lib.rs:763-790 |
| 查询方 | 按 WorkItem、按 Agent 列出；按 Repository 查看状态分布 | lib.rs:913-963、987-1012（域内，无 REST 入口） |
| 放弃者 | 放弃非终态 Worktree | lib.rs:865-894 |
| 归档或恢复者 | 发起 `set_archived` 管理计划 | crates/star-api-rest/src/group_api/worktrees.rs:222-225、1392-1393 |
| owner 分配者 | 发起 `assign_owner` 管理计划 | group_api/worktrees.rs:1392 |
| 上下文编辑者、Fork 使用者 | 对活跃态 Worktree 增量编辑上下文，或派生子 Worktree | crates/domain-worktree/src/edit.rs:282-320（域内，无入口） |
| tenant_admin、project_admin、viewer | 设计 §7 的角色 | 域内不区分（FR-03-27）；线上以 scope 与 creator role 判断（group_api/worktree_lifecycle.rs:703） |

主要用户故事：S1 开发者为 WorkItem 创建 Worktree；S2 Local Runtime 上报观测；S3 平台把 READY 状态的 Worktree 分配给 Agent 会话；S4 按 WorkItem 或 Agent 列出；S5 按 Repository 查看状态分布；S6 放弃 Worktree；S7 对活跃态 Worktree 做上下文编辑与 Fork；S8 在线上对 Worktree 归档、恢复或变更 owner。

## 3. 需求

优先级沿用 plan-016 的阶段划分：P0 为 Phase 1（MVP）核心，P1 为 Phase 2（V1），P2 为 Phase 3（V2）。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-03-01 | 聚合携带 spec §2 的标识、关联、Runtime、所有者、status、health、conflict_state、ahead/behind、时间戳与 version | P0 | 🟡 部分实现 | lib.rs:189-232 已有上述字段；缺 workspace_id、development_execution_id、dirty_state、changed_files、changed_symbols、test_state、build_state、context_state、feedback_state、synchronization_state |
| FR-03-02 | 17 状态枚举与 basic-design §7.1 的状态名一致 | P0 | 🟡 部分实现 | lib.rs:63-97 定义 17 个变体。§7.1（basic-design.md:2655-2663）同为 17 名，两者重合 11 个（CREATED、READY、ASSIGNED、AGENT_RUNNING、BLOCKED、CONFLICTED、READY_FOR_REVIEW、REVIEWING、MERGED、ABANDONED、ARCHIVED）；代码独有 INITIALIZING、COMMITTING、COMPLETED、CHANGES_REQUESTED、FIXING、STALE；设计独有 WAITING_FEEDBACK、FEEDBACK_RECEIVED、VALIDATING、READY_FOR_COMMIT、COMMITTED、PR_OPEN |
| FR-03-03 | 迁移只允许迁移表内的边，其余返回 InvalidTransition（INV-WT-02） | P0 | ✅ 已实现 | lib.rs:137-181（表 145-172；错误 175-178） |
| FR-03-04 | 迁移表的边与 §7.1 的迁移表一致 | P0 | 🟡 部分实现 | 代码 24 条边（lib.rs:145-172）。§7.1 的 WAITING_FEEDBACK、VALIDATING、READY_FOR_COMMIT、COMMITTED、PR_OPEN 边在代码中不存在；代码有设计未列的边（Initializing→Blocked、Reviewing→Merged、Fixing→Ready 等）；缺 CONFLICTED→ASSIGNED 与 ABANDONED→ARCHIVED |
| FR-03-05 | transition_status 要求 cmd.from 与当前状态一致 | P0 | ✅ 已实现 | lib.rs:850-855 |
| FR-03-06 | abandon 只能从活跃态经迁移表进入 ABANDONED | P1 | 🟡 部分实现 | abandon 只检查 `is_terminal()`（lib.rs:883-888），不调用 `check_status_transition`；迁移表只给 Ready、Assigned、AgentRunning 三条到 Abandoned（lib.rs:168-170），注释却写“任何活跃态可”（lib.rs:167） |
| FR-03-07 | Status Independence（INV-WT-01）：迁移只修改 Worktree 自身字段 | P0 | ✅ 已实现 | lib.rs:857-861；测试 `status_independence_from_work_item`（lib.rs:1472） |
| FR-03-08 | 每个 Worktree 必绑一个 Runtime（INV-WT-03） | P0 | 🟡 部分实现 | 域内 create 与 fork 只拒绝 nil UUID（lib.rs:732-734；edit.rs:553-556）；缺 WT-001 错误码（FR-03-31）；线上表有 `runtime_id` 列，但为可空（2026-09-29-worktree-group-phase-2b.sql:55），数据库不强制非空 |
| FR-03-09 | 平台不读取 local_path_reference（INV-WT-04） | P1 | ⬜ 未实现 | 域内 `local_path_reference: Option<String>`（lib.rs:207）创建时为 None（lib.rs:745）；线上表 `path TEXT NOT NULL`（2026-09-16-worktree-canvas-worktree.sql:22），且 `PgWorktreeService::create` 写入 `/worktrees/{branch}`（crates/worktree-service/src/pg_worktree_service.rs:203），与 INV-WT-04 的“平台不持有本地路径”相反 |
| FR-03-10 | Local Runtime 重连后比对 Desired 与 Observed，偏差不静默合并（INV-WT-05） | P1 | ⬜ 未实现 | 无 WorktreeReconciliationState 类型；观测上报时 Stale 直接改为 Ready（lib.rs:822-825） |
| FR-03-11 | 进入 READY_FOR_REVIEW 前通过 Completion Gate 七项检查（INV-WT-06） | P0 | ⬜ 未实现 | `CompletionGateFailed` 只出现在错误映射中（crates/api/src/lib.rs:321；crates/application/src/lib.rs:459；crates/star-api-rest/src/error.rs:149；crates/star-mcp/src/error.rs:269），无构造点；`transition_status` 不做门禁（lib.rs:832-863） |
| FR-03-12 | Worktree Isolation 九项检查（INV-WT-09） | P1 | ⬜ 未实现 | `IsolationFailed` 无构造点（lib.rs:354-356） |
| FR-03-13 | 文件级冲突检测与 WorktreeConflict 实体（spec §2、§22.4） | P1 | 🟡 部分实现 | `detect_conflicts` 返回同 Repository 内其他活跃 Worktree 的 ID（lib.rs:964-986，976 行注释写明为简化）；无 WorktreeConflict 类型；ConflictState 只在创建时设为 none（lib.rs:279-285、751） |
| FR-03-14 | 1 WorkItem 对应 0..N 个 Worktree，并可按 WorkItem 列出（INV-WT-07） | P0 | ✅ 已实现 | lib.rs:195；`list_by_work_item`（lib.rs:913-939）；测试 `list_by_work_item_filters`（lib.rs:1353） |
| FR-03-15 | 1 Worktree 对应 0..N 个 AgentSession | P1 | 🟡 部分实现 | 只保存单个 `current_agent_session_id`（lib.rs:213），无会话集合 |
| FR-03-16 | 分配 Agent 要求 Worktree 处于 READY | P0 | ✅ 已实现 | `assign_to_agent`（lib.rs:763-790），迁移检查在 lib.rs:782；测试 `assign_to_agent_requires_ready_state`（lib.rs:1213） |
| FR-03-17 | 观测上报只接受 Local Runtime actor | P0 | ✅ 已实现 | lib.rs:797-799；`ActorContext.is_local_runtime`（crates/star-context/src/actor.rs:74）；测试 lib.rs:1237 |
| FR-03-18 | 观测更新 ahead、behind，并刷新最后观测时间 | P0 | ✅ 已实现 | lib.rs:813-820 |
| FR-03-19 | 健康分级 Current、PossiblyStale、Offline（INV-WT-10） | P1 | 🟡 部分实现 | `Staleness` 枚举存在（lib.rs:254-264）；观测时只写 Current（lib.rs:820）；PossiblyStale 与 Offline 无构造点；60s/300s 阈值未在 crates 与前端 Worktree 相关路径中实现 |
| FR-03-20 | 失联超过 300 秒自动进入 STALE | P1 | ⬜ 未实现 | 迁移表允许 Ready、AgentRunning 进入 Stale（lib.rs:164-165），但无生产代码构造 `WorktreeStatus::Stale`（lib.rs:822 与 edit.rs:465 只读取） |
| FR-03-21 | Repository 维度的状态热力图 | P1 | 🟡 部分实现 | `heatmap`（lib.rs:987-1012）与 `HeatmapData`（lib.rs:561-572）；无 REST 或 MCP 入口；无 100 Worktree / 10k 文件的性能约束与投影 |
| FR-03-22 | 高频观测投影 WorktreeStatusObserved（30 天热数据） | P1 | 🟡 部分实现 | 域内 ⬜；worktree-service 中存在 `WorktreeStatusObserved`（crates/worktree-service/src/projection.rs:39）与 `record_observed`（pg_worktree_service.rs:118）；投影的写入表与保留策略 ❓（确认方法：检索 db/migrations 中 `worktree_status_observed` 的 CREATE 语句，并检索其写入路径） |
| FR-03-23 | 乐观锁 version：写入时校验期望版本 | P1 | 🟡 部分实现 | 域内每次写入递增 version（lib.rs:787、827、860、891），但命令不携带期望版本，无 CAS 比较 |
| FR-03-24 | 管理计划（归档、owner）的期望版本校验与幂等 | P1 | ✅ 已实现 | `expected_version` 字段（group_api/worktrees.rs:111）；版本不符拒绝（worktrees.rs:526）；幂等键写入（worktrees.rs:570）；测试 `unchanged_archive_plan_preserves_worktree_version`（worktrees.rs:1866） |
| FR-03-25 | 领域事件（spec §5 的 7 个 NATS subject）经 outbox 发布 | P2 | ⬜ 未实现 | crates 中无 `star.events.worktree` 命名；域内无事件类型 |
| FR-03-26 | 管理计划事件 worktree.archived、worktree.restored 经 outbox 写入 | P2 | 🟡 部分实现 | `worktree_management_outbox_fact`（worktrees.rs:157-189）；outbox 表（db/migrations/2026-10-20-worktree-management-outbox.sql:31）；只覆盖这两个事件（worktrees.rs:167-168） |
| FR-03-27 | 角色与权限（spec §7） | P1 | 🟡 部分实现 | 域内无角色判断，唯一拒绝为非 Local Runtime 观测（lib.rs:797-798）。API 层有 `worktree:read`（group_api.rs:797）、`worktree:create`（worktree_lifecycle.rs:547）与 creator role 检查（worktree_lifecycle.rs:703）；spec 的 assign、abandon、commit、block、unblock、resolve_conflict 字符串在 crates 中无出现 |
| FR-03-28 | Repository 抽象与内存实现 | P0 | ✅ 已实现 | `WorktreeRepository`（lib.rs:574-601）；`InMemoryWorktreeRepository`（lib.rs:640-716） |
| FR-03-29 | PostgreSQL 持久化与 RLS（spec §6 `worktree.*` schema） | P0 | 🟡 部分实现 | 域内无 SQL；线上表 `worktree_canvas_worktree` 启用并强制 RLS（2026-09-16-worktree-canvas-worktree.sql:61-62）；spec 指定的 `worktree` schema 与其四张表在迁移目录中无定义（迁移目录检索 `worktree.` 前缀的 CREATE 无命中） |
| FR-03-30 | 对外暴露 worktree 操作（spec §4 的全部端口方法） | P1 | 🟡 部分实现 | 生产挂载的只有创建与读取（lib.rs:226-227，经 crates/star-api-rest/src/main.rs:164 与 lib.rs:182 挂载）；MCP 只有 create_worktree 与 get_worktree（crates/star-mcp/src/tools）；assign、transition、observe、abandon、列表、heatmap、detect_conflicts 无入口 |
| FR-03-31 | 错误码 WT-001 至 WT-009（spec §8） | P1 | ⬜ 未实现 | crates 中检索 `WT-0` 无命中；`WorktreeError` 使用 Rust 变体名（lib.rs:328-363） |
| FR-03-32 | 归档：Merged→Archived 与归档时点策略 | P2 | 🟡 部分实现 | 迁移边 Merged→Archived（lib.rs:172）；线上归档经管理计划 `set_archived`（worktrees.rs:167、1392-1393）并经 Hook 门禁（worktrees.rs:203-209、1187-1193）；无 90 天或 30 天定时任务 ❓（确认方法：检索 worker 与 cron 注册点中对 `worktree` 的调用） |
| FR-03-33 | 上下文编辑只允许活跃态（INV-ED-07） | P1 | 🟡 部分实现 | `ensure_editable` 允许 11 个状态（edit.rs:449-459），模块头注释只列 Created、Ready、Assigned、AgentRunning（edit.rs:20-21）；全仓无 `apply_context_edits` 的外部调用 |
| FR-03-34 | Fork：子 Worktree 携带 parent_worktree_id 与 parent_at（INV-ED-02、INV-ED-03） | P1 | 🟡 部分实现 | `ForkSpec` 校验 parent_at 不晚于当前时间（edit.rs:189）；fork 构造的子 Worktree 不含这两个字段（edit.rs:559-586），`Worktree` 结构也不含（lib.rs:189-232）；edit.rs:29-30 注释称其为聚合的扩展字段 |
| FR-03-35 | 线上 human_state 与域内 17 状态之间的映射 | P1 | ⬜ 未实现 | 线上 7 值只有单向字符串化（pg_worktree_repository.rs:147-156），无从域状态到投影的映射函数，也无反向映射 |
| FR-03-36 | 前端 Worktree 状态词表与后端一致 | P2 | ⬜ 未实现 | `WorktreeStatus` 17 值（frontend/src/types/ids.ts:452-460）与域内枚举不同；/worktree 索引只接受这 17 值（frontend/src/app/worktree/page.tsx:167-171、266-270），而线上 `human_state` 为 running、waiting、ready、diverged、conflict、merged、stale（pg_worktree_repository.rs:147-156） |
| FR-03-37 | 前端状态迁移按钮通过后端校验后持久化 | P1 | ⬜ 未实现 | `transitionWorktree` 只修改 Zustand 本地状态（frontend/src/lib/store.ts:467-470），不调用后端，也不使用 WORKTREE_SM 校验（ids.ts:1030-1065）；按钮见 frontend/src/app/worktree/page.tsx:910-918 |

## 4. 设计

### 4.1 概念模型

本功能采用两层状态模型。

- **聚合生命周期层**：`domain-worktree` 的 Worktree 聚合根以 basic-design §7.1 的 17 个状态名为设计基准。选择理由：§7.1 是 domain-worktree-spec 附录 B 与 crate 头注释（crates/domain-worktree/src/lib.rs:10）共同引用的状态基准，spec INV-WT-02 也指向同一节。代码当前的 17 个枚举名与之只重合 11 个，差异列在 FR-03-02 与 §6。
- **线上投影层**：`worktree_canvas_worktree.human_state`（7 值）与 `machine_state`（12 值设计，列为自由文本）承载 DD-WORKTREE-CANVAS-001 §23.1 的人类可读状态。选择理由：线上迁移与 worktree-service 已按该投影落库，且该设计在画布边界内定义了投影。DD 当前为 Draft v1.4。
- 两层之间的映射尚未定义（FR-03-35）。映射实现之前，两层的状态值不得互相替代。

概念实体：

- **Worktree（聚合根）**：归属一个 tenant、一个 Project、一个 Repository 与一个 WorkItem（只作 ID 引用）；绑定一个 Runtime；可分配一个 Agent 与当前 AgentSession。
- **值对象**：HealthState（最后观测时间与 Staleness）、ConflictState。
- **派生数据**：WorktreeSummary（列表视图）、HeatmapData（按状态计数）。
- **端口与仓储**：WorktreeCommandPort、WorktreeQueryPort、WorktreeRepository（lib.rs:485-601）。
- **上下文编辑与 Fork**（`edit.rs`）：WorktreeEditPort、WorktreeEditQueryPort、ContextEdit、ContextEditBatch、ForkSpec。
- **线上投影**：WorktreeStatusObserved（crates/worktree-service/src/projection.rs:39）。

Status Independence 是两层共同的约束：Worktree 的状态迁移不写入 WorkItem 字段（INV-WT-01；lib.rs:857-861）。

### 4.2 数据模型

| 实体 | 字段 | 状态 | 证据 |
|---|---|---|---|
| Worktree（域） | id、tenant_id、work_item_id、project_id、repository_id、branch、base_branch、runtime_id、local_path_reference、owner_user_id、assigned_agent_id、current_agent_session_id、status、health、conflict_state、ahead、behind、last_activity_at、created_at、updated_at、version | 🟡 部分实现 | lib.rs:189-232 |
| HealthState、Staleness（域） | last_observed_at；Unknown、Current、PossiblyStale、Offline | 🟡 部分实现 | lib.rs:235-264 |
| ConflictState（域） | has_conflicts、conflicting_worktree_ids、last_detected_at | 🟡 部分实现 | lib.rs:267-286 |
| ForkSpec、ContextEdit（域） | ForkSpec 含 parent_worktree_id、parent_at；ContextEdit 为 Omit、Replace 两种 op | 🟡 部分实现 | edit.rs:99-112、180-205 |
| worktree_canvas_worktree（线上） | id、repo_id、name、branch、path、human_state、machine_state、ahead、behind、dirty、health_score、last_activity、test_state、risk_count、locked、archived、agent_id、task_id、parent_id、merged_at、created_at、updated_at、valid_from、valid_to、pgpool_version、tenant_id、workspace_id | 🟡 部分实现 | 2026-09-16-worktree-canvas-worktree.sql:22-52 |
| worktree_canvas_worktree 增量列（线上） | 2026-09-25 增加 locked_by、import_source；2026-09-29-phase-2b 增加 project_id、owner_user_id、work_item_id、agent_session_id、runtime_id、pull_request_url（均可空） | ✅ 已实现 | 2026-09-25-ulys-195-worktree-pg-backed.sql:36、40；2026-09-29-worktree-group-phase-2b.sql:50-56 |
| multica.worktree_management_plan（线上） | 管理计划表 | ✅ 已实现 | 2026-09-29-worktree-group-phase-2d.sql:29 |
| multica.worktree_management_outbox（线上） | 管理事件 outbox，带只追加守卫 | ✅ 已实现 | 2026-10-20-worktree-management-outbox.sql:31 |
| WorktreeStatusObserved | 高频观测投影（worktree-service） | 🟡 部分实现 | crates/worktree-service/src/projection.rs:39 |
| WorktreeConflict、WorktreeReconciliationState（域） | 设计 §2 的实体与值对象 | ⬜ 未实现 | 域内无类型定义（lib.rs 与 edit.rs 中无对应名称） |
| `worktree` schema 下的 worktree、worktree_status_observed、worktree_conflict、worktree_reconciliation_state | spec §6 的表定义 | ⬜ 未实现 | 迁移目录中无 `worktree.` 前缀的 CREATE 语句；线上以 `worktree_canvas_worktree` 等名称存在 |

线上表的 RLS、唯一约束与索引：

- 启用并强制 RLS（2026-09-16-worktree-canvas-worktree.sql:61-62），策略按 `app.tenant_id` 与 `app.workspace_id` 过滤。
- 唯一约束：`(repo_id, name)` 与 `(repo_id, path)`（:55、:57）；主键 `id`（:53）。
- 索引包括 `(repo_id, human_state)`（:99-103 区间的索引定义）。

### 4.3 接口

#### 4.3.1 域端口（crates/domain-worktree/src/lib.rs）

WorktreeCommandPort（lib.rs:483-520）：

| 方法 | 签名要点 | 与 spec §4 的差异 | 状态 |
|---|---|---|---|
| create_worktree | (CreateWorktreeCommand, &ActorContext) → Worktree（lib.rs:486-491） | spec 返回 WorktreeId | ✅ 已实现 |
| assign_to_agent | (AssignWorktreeCommand, &ActorContext) → Worktree（lib.rs:494-498） | spec 返回 () | ✅ 已实现 |
| record_observed_state | (RecordObservedStateCommand, &ActorContext) → Worktree（lib.rs:501-505） | spec 返回 () | ✅ 已实现 |
| transition_status | (TransitionStatusCommand, &ActorContext) → Worktree（lib.rs:508-512） | spec 返回 WorktreeStatus | ✅ 已实现 |
| abandon | (AbandonCommand, &ActorContext) → Worktree（lib.rs:515-519） | spec 返回 () | ✅ 已实现 |
| archive | plan-016 WT-004 要求的第 6 个方法 | 不存在；归档经迁移边 Merged→Archived 与线上管理计划实现 | ⬜ 未实现 |

WorktreeQueryPort（lib.rs:524-559）：

| 方法 | 状态 | 说明 |
|---|---|---|
| get_by_id（lib.rs:526-530） | ✅ 已实现 | 跨 tenant 拒绝（lib.rs:905-910） |
| list_by_work_item（lib.rs:533-537） | ✅ 已实现 | 返回 WorktreeSummary |
| list_by_agent（lib.rs:540-544） | ✅ 已实现 | 返回 WorktreeSummary |
| detect_conflicts（lib.rs:547-551） | 🟡 部分实现 | 返回 `Vec<WorktreeId>`，spec 要求 `Vec<WorktreeConflict>` |
| heatmap（lib.rs:554-558） | 🟡 部分实现 | 返回 HeatmapData；域内可调用，无入口 |

上下文编辑与 Fork（crates/domain-worktree/src/edit.rs）：

| 端口方法 | 位置 | 状态 |
|---|---|---|
| WorktreeEditPort::apply_context_edits | edit.rs:290 | 🟡 部分实现（域内实现 edit.rs:478-531，无入口） |
| WorktreeEditPort::fork | edit.rs:302 | 🟡 部分实现（域内实现 edit.rs:533-592，无入口） |
| WorktreeEditQueryPort::list_context_edits | edit.rs:309 | 🟡 部分实现 |
| WorktreeEditQueryPort::list_fork_children | edit.rs:316 | 🟡 部分实现 |

WorktreeRepository（lib.rs:574-601）：insert、get、update、list_by_work_item、list_by_agent、list_by_repository，内存实现见 lib.rs:640-716。状态：✅ 已实现。

#### 4.3.2 对外入口（当前）

| 方法与路径 | 处理位置 | 行为 | 状态 |
|---|---|---|---|
| `POST /api/v1/worktrees` | routes/worktrees.rs:71-113；路由 lib.rs:226 | 调用内存 `InMemoryWorktreeService`（routes/worktrees.rs:22-26）；Project、Repository、Runtime 为随机 ID（:79-83）；`agent_session_id` 未参与命令构造 | 🟡 部分实现 |
| `GET /api/v1/worktrees/{id}` | routes/worktrees.rs:28-55；路由 lib.rs:227 | 读取内存实体 | 🟡 部分实现 |
| MCP `create_worktree` | crates/star-mcp/src/tools/create_worktree.rs（invoke 54 起） | 同一内存服务 | 🟡 部分实现 |
| MCP `get_worktree` | crates/star-mcp/src/tools/get_worktree.rs | 同一内存服务 | 🟡 部分实现 |
| `GET /api/v1/projects/{project_id}/worktrees` | group_api/worktrees.rs:213-216；SQL 330-355 | 线上列表，读取 `worktree_canvas_worktree` | ✅ 已实现 |
| `POST /api/v1/worktrees/{worktree_id}/management-plans` | group_api/worktrees.rs:222-225；handler 448 | 生成计划，operation 为 `set_archived` 或 `assign_owner` | ✅ 已实现 |
| `POST /api/v1/worktrees/{worktree_id}/management-plans/{plan_id}/confirm` | group_api/worktrees.rs:226-229；handler 596 | 校验期望版本（526）后更新（843） | ✅ 已实现 |
| `POST /api/v1/engineering-runs/{engineering_run_id}/worktrees` | group_api/worktree_lifecycle.rs:411-414；handler 587 | 需 `worktree:create` scope（547）与 creator role（703）；无 provider 时返回 `worktree_lifecycle_unavailable`（530-534） | 🟡 部分实现 |
| `PUT`、`DELETE /api/v1/projects/{project_id}/engineering-runs/{run_id}/worktrees/{worktree_id}` | group_api/run_worktree_bindings.rs:239-242；handler 247、290 | 绑定或解绑 Run 与 Worktree | ❓ 未核实（确认方法：逐行阅读 `mutate_binding` 的 SQL 与事务边界，run_worktree_bindings.rs:290 起） |
| `GET /api/v1/worktrees/{worktree_id}/group-context` | group_api.rs:809-810；handler 791；SQL 527-537 | 读取 Worktree 上下文投影 | ✅ 已实现 |
| assign、transition、observe、abandon、heatmap、detect_conflicts、上下文编辑、Fork 的入口 | crates 中检索 | 无路由，无 MCP 工具 | ⬜ 未实现 |

#### 4.3.3 生命周期 provider 注入点

`GroupApiState` 提供三个 setter：`with_worktree_git_lock_observer`（group_api.rs:441-446）、`with_worktree_archive_readiness_observer`（:450-456）、`with_run_worktree_lifecycle_provider`（:459-465）。全仓（含 `crates/star-api-rest/src/main.rs` 与 `crates/star-desktop/src-tauri`）无这三个 setter 的调用点，`RunWorktreeLifecycleProvider` 也无实现。状态：⬜ 未实现（注入点存在，实装不存在）。

### 4.4 状态机与关键流程

#### 4.4.1 域内迁移表（代码实际行为）

| 类别 | 边 | 证据 |
|---|---|---|
| 启动 | Created→Initializing；Initializing→Ready；Initializing→Blocked | lib.rs:145-147 |
| Agent 接管 | Ready→Assigned；Assigned→AgentRunning；AgentRunning→Committing；Committing→Completed | lib.rs:149-152 |
| Review | Completed→ReadyForReview；ReadyForReview→Reviewing；Reviewing→ChangesRequested；Reviewing→Merged；ChangesRequested→Fixing；Fixing→AgentRunning；Fixing→Ready | lib.rs:154-160 |
| 异常 | Ready→Conflicted；AgentRunning→Conflicted；Ready→Stale；AgentRunning→Stale；Stale→Ready | lib.rs:162-166 |
| 放弃 | Ready→Abandoned；Assigned→Abandoned；AgentRunning→Abandoned（`abandon` 方法本身不走此表，见 FR-03-06） | lib.rs:168-170 |
| 归档 | Merged→Archived | lib.rs:172 |

无出边的状态：Blocked、Conflicted、Abandoned、Archived。其中 Blocked 与 Conflicted 进入后，域内没有任何迁移可以离开，`transition_status` 与 `assign_to_agent` 都不会接受它们作为起点（lib.rs:856、782）。`is_terminal()` 把 Merged、Archived、Abandoned 判为终态（lib.rs:124-126）。

#### 4.4.2 设计基准（basic-design §7.1）与代码的边对照

| 设计边（basic-design.md:2668-2685） | 代码 | 状态 |
|---|---|---|
| CREATED → READY（Local Runtime） | 代码为 Created→Initializing→Ready（lib.rs:145-146） | 🟡 部分实现 |
| READY → ASSIGNED | lib.rs:149 | ✅ 已实现 |
| ASSIGNED → AGENT_RUNNING | lib.rs:150 | ✅ 已实现 |
| AGENT_RUNNING → WAITING_FEEDBACK | 无 | ⬜ 未实现 |
| WAITING_FEEDBACK → FEEDBACK_RECEIVED | 无 | ⬜ 未实现 |
| AGENT_RUNNING → VALIDATING | 无 | ⬜ 未实现 |
| VALIDATING → READY_FOR_REVIEW | 无；代码由 Completed→ReadyForReview（lib.rs:154）替代 | ⬜ 未实现 |
| VALIDATING → BLOCKED | 无 | ⬜ 未实现 |
| * → CONFLICTED | 仅 Ready、AgentRunning（lib.rs:162-163） | 🟡 部分实现 |
| CONFLICTED → ASSIGNED | 无；Conflicted 无出边 | ⬜ 未实现 |
| * → ABANDONED | 见 FR-03-06 | 🟡 部分实现 |
| ABANDONED → ARCHIVED（90 天） | 无；Abandoned 无出边 | ⬜ 未实现 |
| READY_FOR_COMMIT → COMMITTED | 无 | ⬜ 未实现 |
| COMMITTED → PR_OPEN | 无 | ⬜ 未实现 |
| PR_OPEN → MERGED | 无；代码为 Reviewing→Merged（lib.rs:157） | 🟡 部分实现 |
| MERGED → ARCHIVED（30 天） | Merged→Archived 存在（lib.rs:172），无定时任务 | 🟡 部分实现 |

§7.1 内部两条归档时点规则不一致（ABANDONED 90 天、MERGED 30 天）；crate 注释写“归档（>90d）”（lib.rs:88）。该差异列入 §6。

#### 4.4.3 线上投影的状态与迁移函数

- 取值：`human_state` 为 running、waiting、ready、diverged、conflict、merged、stale（crates/worktree-service/src/pg_worktree_repository.rs:147-156）；`machine_state` 为 VARCHAR(32)，默认 `git_clean`（2026-09-16-worktree-canvas-worktree.sql:26）。
- 迁移函数：`lifecycle::transition` 定义 7 态与 10 个事件（crates/worktree-service/src/lifecycle.rs:23-52、97）。
- 当前调用：`PgWorktreeService::create` 直接写 `HumanState::Running`（pg_worktree_service.rs:204），随后调用迁移函数并丢弃结果（:223-227）。其余写路径是否调用该函数 ❓（确认方法：检索 `lifecycle_transition` 与 `lifecycle::transition` 的全部调用点）。
- 状态：线上迁移强制 ⬜ 未实现。

#### 4.4.4 线上归档与 owner 管理计划

1. 创建计划：`validate_plan` 校验 operation 与字段组合（group_api/worktrees.rs:1390-1393）；请求哈希（:1385）；计划写入 `multica.worktree_management_plan`。
2. 归档门禁：`management_plan_requires_archive_gate`（:203-209）判定是否需要门禁；`prepare_archive_gate_facts`（:1073）与 `evaluate_archive_gate`（:1187-1193）取得 Hook 评估、Git 锁观测与准入 fence；门禁事实须新鲜（:1311、:812）。
3. 确认计划：`confirm_management_plan`（:596）校验 `expected_version`（:526）与幂等键（:570）。
4. 写入：`UPDATE worktree_canvas_worktree`（:843）。
5. 事件：`worktree_management_outbox_fact`（:157-189）只为 `set_archived` 产生 `worktree.archived` 或 `worktree.restored`；owner 分配不产生归档事件（测试 :1845）。

状态：✅ 已实现（范围仅限归档与 owner；其余生命周期操作无线上路径）。

#### 4.4.5 观测与 Stale（域内）

`record_observed_state` 只接受 Local Runtime；更新 ahead、behind、当前会话与 health（Current）；若状态为 Stale，直接置为 Ready（lib.rs:813-825）。该路径不做 Desired 与 Observed 比对（FR-03-10），也不存在自动进入 Stale 的路径（FR-03-20）。

#### 4.4.6 不变量的执行位置

| 不变量 | 执行位置 | 状态 |
|---|---|---|
| INV-WT-01 Status Independence | 迁移只写 Worktree 字段（lib.rs:857-861） | ✅ 已实现 |
| INV-WT-02 严格迁移 | transition_status（lib.rs:856）与 assign_to_agent（lib.rs:782）调用 check_status_transition；abandon 不调用（lib.rs:883-889） | 🟡 部分实现 |
| INV-WT-06 Completion Gate | 无门禁（lib.rs:832-863） | ⬜ 未实现 |
| INV-WT-10 Stale Display | 只写 Current（lib.rs:820） | 🟡 部分实现 |

### 4.5 权限与安全

| 控制 | 位置 | 状态 |
|---|---|---|
| 域内 tenant 隔离：读写比对 actor tenant 与记录 tenant | lib.rs:725-730、769-780、801-812、838-849、871-882、905-910、918-923、945-950、970-975、994-1003 | ✅ 已实现 |
| 线上 RLS：启用并强制，策略按 `app.tenant_id` 与 `app.workspace_id` 过滤 | 2026-09-16-worktree-canvas-worktree.sql:61-62 与其后的策略段 | ✅ 已实现 |
| 线上列 `tenant_id` 默认 nil UUID | 2026-09-16-worktree-canvas-worktree.sql:51 | 🟡 部分实现（写入路径是否总显式提供 tenant_id ❓，确认方法：检索 crates/worktree-service/src/pg_worktree_repository.rs 中 INSERT 的列表，:288、:361） |
| 观测写入仅 Local Runtime | lib.rs:797-799 | ✅ 已实现（域内）；线上无观测入口 |
| 本地路径不对平台开放（INV-WT-04） | 线上表存 `path`（2026-09-16-worktree-canvas-worktree.sql:22） | ⬜ 未实现（与不变量相反） |
| `worktree:read` scope | group_api.rs:797 | ✅ 已实现 |
| `worktree:create` scope 与 project creator role | worktree_lifecycle.rs:547、598、659；`require_project_creator_role` 703 | ✅ 已实现 |
| 管理计划的授权入口 | `authorize_worktree`（group_api/worktrees.rs:1348） | ✅ 已实现 |
| 管理计划的期望版本与幂等 | worktrees.rs:526、570 | ✅ 已实现 |
| 归档 Hook 门禁 | worktrees.rs:203-209、812、1187-1193 | ✅ 已实现 |
| 域内角色判断（spec §7） | 域内无 | ⬜ 未实现 |
| 设计所列 permission 字符串（assign、abandon、commit、block、unblock、resolve_conflict） | crates 中 `worktree:` 前缀只见 read、create、write | ⬜ 未实现 |

### 4.6 依赖关系

- 域依赖：`star-context`（ActorContext，lib.rs:39）、async-trait、chrono、serde、thiserror、uuid（lib.rs:33-41）。
- 域端口的调用方：star-api-rest（routes/worktrees.rs:11-13）、star-mcp（handlers/worktree.rs:13-14；tools/create_worktree.rs:26-29；tools/get_worktree.rs:16）、application（lib.rs:436-464，错误映射）、api（lib.rs:321）、star-desktop（crates/star-desktop/src-tauri/src/ipc_adapter.rs:40、216，只引用 `WorktreeStatus` 类型）。
- 仅注释引用、不计依赖：domain-local-runtime（lib.rs:62）、domain-permission（lib.rs:52）、domain-validation（invariants.rs:17）。
- 线上依赖：worktree-service（PgWorktreeService、PgWorktreeRepository）；worktree-canvas 依赖 worktree-service（crates/worktree-canvas/Cargo.toml:19）；group_api 直接 SQL 读写 `worktree_canvas_worktree`（group_api.rs:527-537；group_api/worktrees.rs:330-355、843）；infrastructure 的 run_task_card_pg.rs 读取同表（:731、806）。
- 跨域：ADR-0039 D27 的 `CreateWorktree`（domain-worktree）与 `LinkPullRequest`（domain-scm）步骤。Saga 在代码中的实现位置 ❓（确认方法：检索 star-saga 与 application 中的 `CreateWorktree` 步骤）。
- 设计依赖：basic-design §7.1、domain-worktree-spec、DD-WORKTREE-CANVAS-001 §23。

### 4.7 页面设计

4.7 页面设计: 本功能无独立页面，入口见 F01 与 F07.

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 域聚合与状态机 | crates/domain-worktree/src/lib.rs | 🟡 部分实现 | 17 枚举与迁移表存在，与 §7.1 不一致（lib.rs:63-172） |
| 上下文编辑与 Fork | crates/domain-worktree/src/edit.rs | 🟡 部分实现 | 内存实现（edit.rs:415-592），无外部入口 |
| 域集成测试 | crates/domain-worktree/tests/integration_test.rs | ✅ 已实现 | 11 个测试（:58-367） |
| REST 创建与读取 | crates/star-api-rest/src/routes/worktrees.rs；lib.rs:226-227；main.rs:164 | 🟡 部分实现 | 内存服务（routes/worktrees.rs:22-26），ID 随机 |
| 线上管理计划（归档、owner） | crates/star-api-rest/src/group_api/worktrees.rs | ✅ 已实现 | 路由 211-230；validate_plan 1390；UPDATE 843 |
| 线上 Worktree 列表与投影 | group_api/worktrees.rs:211-216、330-355；group_api.rs:697-712 | ✅ 已实现 | PostgreSQL 读取 |
| Run 与 Worktree 绑定 | group_api/run_worktree_bindings.rs:237-245 | ❓ 未核实 | 路由存在；写库细节见 §4.3.2 |
| 线上 Run 内创建（provider 缺失） | group_api/worktree_lifecycle.rs:383-413、530-549、587 | 🟡 部分实现 | 路由与权限存在；provider 无实现，返回不可用（:533） |
| 生命周期 provider 与观测、归档门禁注入点 | group_api.rs:441-465 | ⬜ 未实现 | 只有 setter，全仓无调用点与实现 |
| 线上表与迁移 | db/migrations/2026-09-16-worktree-canvas-worktree.sql 等 | ✅ 已实现 | 表、RLS、索引、增量列 |
| 7 态投影与迁移函数 | crates/worktree-service/src/lifecycle.rs；pg_worktree_service.rs:190-241 | 🟡 部分实现 | 迁移函数定义存在；create 直接写 Running 并丢弃结果（:204、:223-227） |
| 投影类型 | crates/worktree-service/src/projection.rs:39 | 🟡 部分实现 | 类型存在；写入路径 ❓ |
| 服务与仓储（worktree-service） | crates/worktree-service/src/service.rs:79、175-223；pg_worktree_repository.rs | 🟡 部分实现 | trait 含 archive、cleanup_batch 等；线上归档经管理计划而非此 trait（❓ 确认方法：检索 `WorktreeService::archive` 的调用点） |
| MCP 工具 | crates/star-mcp/src/tools/create_worktree.rs、get_worktree.rs；handlers/worktree.rs | 🟡 部分实现 | 内存服务；create_worktree.rs:18 注释引用 `WorktreeStatus::Creating`，枚举中不存在 |
| 前端状态词表 | frontend/src/types/ids.ts:452-460、1030-1065 | 🟡 部分实现 | `WORKTREE_SM.states`（ids.ts:1034-1039）与域枚举的 17 名相同；`WorktreeStatus` 联合类型（ids.ts:452-460）是另一套 17 值，与域同名的只有 initializing、committing、merged、abandoned、archived；WORKTREE_SM 的边为推断（ids.ts:1042） |
| 前端索引归一化 | frontend/src/app/worktree/page.tsx:167-171、266-270 | 🟡 部分实现 | 只接受 ids.ts 的 17 个值，其余抛错（FR-03-36） |
| 前端状态迁移按钮 | frontend/src/lib/store.ts:467-470；frontend/src/app/worktree/page.tsx:910-918 | ⬜ 未实现 | 只修改本地状态（FR-03-37） |
| 画布 7 态类型 | frontend/src/components/worktree-canvas/types.ts:17-24；crates/graph-core/src/state.rs:29-44 | ✅ 已实现 | 类型定义存在 |
| 错误映射 | crates/star-api-rest/src/error.rs:149-150；crates/application/src/lib.rs:436-464；crates/star-mcp/src/error.rs:269-270 | ✅ 已实现 | `WorktreeError` 映射；`CompletionGateFailed`、`IsolationFailed` 只有映射无产生者 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 三套状态词表：域 17 名（11 与 §7.1 重合）、线上 7 值、前端 17 值（与域不同名） | 同一 Worktree 在 API、画布、索引中的状态名不一致 | 以 §7.1 确定聚合状态名；写出域状态到 DD §23.1 投影的显式映射表，并在 domain-worktree 与 worktree-service 之间共用一份枚举 |
| 线上 `human_state` 与前端 `/worktree` 索引的值域不兼容（FR-03-36） | 未归档的 running、waiting、ready、stale 行使索引归一化抛错（page.tsx:266-270） | 统一值域（或映射），前后端共用一份枚举，并加契约测试 |
| 迁移表与 §7.1 的边集不一致；Blocked、Conflicted、Abandoned 无出边（FR-03-04） | 冲突与阻塞无法离开；§7.1 的 CONFLICTED→ASSIGNED、ABANDONED→ARCHIVED 无法到达 | 按 §7.1 补边，或先修订 §7.1 再同步两份表；为每条边补单元测试 |
| abandon 绕过迁移表（FR-03-06） | 任意非终态可放弃，但迁移表注释与实现不一致 | 确定允许放弃的来源状态集合，让 abandon 调用 `check_status_transition` |
| Completion Gate 七项与 Isolation 九项无实现（FR-03-11、FR-03-12） | READY_FOR_REVIEW 进入前没有门禁 | 定义检查结果的输入接口（由 Application 提供），在 transition_status 前校验 |
| 观测分级与自动 STALE 缺失（FR-03-19、FR-03-20） | 过期观测无法区分；失联不进入 Stale | 实现 Staleness 计算（阈值由 Project Policy 配置，spec J-WT-04），由定时任务触发 Ready、AgentRunning→Stale |
| Reconciliation 缺失，Stale→Ready 直接写入（FR-03-10） | 重连时偏差被静默覆盖，违反 INV-WT-05 | 实现 Desired 与 Observed 比对与偏差记录，不直接覆盖 |
| 线上表存储本地 path（FR-03-09） | 与 INV-WT-04 相反 | 表改存不透明引用，路径只由 Local Runtime 解释 |
| 线上 `runtime_id` 可空（FR-03-08） | 数据库不保证 INV-WT-03 | 在创建路径强制非空；为既有行给出回填策略 |
| 线上创建不经域聚合，也无 provider 实现（§4.3.2） | 线上 Run 内创建返回不可用；生产 `POST /api/v1/worktrees` 只写内存 | 确定唯一写入路径：由 provider 或 PgWorktreeRepository 调用域校验，并替换内存服务 |
| 生产 `POST /api/v1/worktrees` 使用随机 Project、Repository、Runtime ID（routes/worktrees.rs:79-83） | 返回的实体不对应真实上下文，且不落库 | 要求调用方提供真实上下文，写入持久化层 |
| 前端迁移按钮只改本地状态（FR-03-37） | 用户操作不持久化、不校验 | 接入后端迁移端点（当前无），或在接入前隐藏按钮 |
| 事件（spec §5）与错误码 WT-xxx（spec §8）未实现（FR-03-25、FR-03-31） | 订阅者无法消费状态变化；错误无稳定码 | 定义事件 envelope 与 WT 错误码表，经 outbox 发布（现仅 worktree.archived、worktree.restored） |
| 域内无角色判断（FR-03-27） | 域操作不检查 actor 角色 | 在 Application 或端口前置校验中检查 permission 字符串，与 group API scope 对齐 |
| 聚合字段缺口（FR-03-01） | 画布与 Validation 无法读取 dirty、changed_files、test_state 等 | 按 spec §2 补字段，或由 worktree-service 投影提供（线上表已有 dirty、test_state、health_score 列） |
| 归档时点冲突（§4.4.2） | 归档行为不确定，无定时任务 | 确定唯一规则并实现定时任务（❓ 未见实现） |
| Fork 与上下文编辑的状态集合与字段（FR-03-33、FR-03-34） | Fork 结果无法追溯父版本；注释与 ensure_editable 不一致 | 把 parent_worktree_id、parent_at 放入 lineage 结构；统一 INV-ED-07 状态集合 |
| heatmap 与 detect_conflicts 无入口（FR-03-13、FR-03-21） | F07 风险视图无数据源 | 暴露只读端点，并在 PostgreSQL 上实现按 Repository 的聚合 |
| MCP 注释引用不存在的 `WorktreeStatus::Creating` | 误导实现者 | 改为 `Created` |

## 7. 验收与测试

### 7.1 现有测试

| 位置 | 数量 | 断言要点 |
|---|---|---|
| crates/domain-worktree/src/lib.rs:1019-1541 | 23（同步 8，异步 15） | 17 个状态枚举计数（:1040）；终态（:1064）；合法路径与非法跳过（:1080、:1101）；终态拒绝（:1108）；ChangesRequested 循环（:1114）；Stale→Ready（:1123）；创建唯一 ID 与初始状态（:1128）；跨 tenant 创建与读取（:1140、:1337）；Runtime 必带（:1151）；迁移 from 不符（:1188）；assign 需 Ready（:1213）；非 Local Runtime 观测被拒（:1237）；abandon 与终态放弃（:1262、:1285）；列表过滤（:1353、:1378）；detect_conflicts（:1437）；heatmap（:1455）；status independence（:1472） |
| crates/domain-worktree/src/edit.rs（测试模块，652 起） | 25 | ContextEdit 校验（路径非空、以 `/` 开头）；ForkSpec 的 parent_at 校验；ensure_editable 的活跃与终态分类；apply、fork、跨 tenant、serde 往返 |
| crates/domain-worktree/tests/integration_test.rs | 11 | it_v0 至 it_v3：创建、读取、列表、Runtime 必带、完整迁移路径、assign 需 Ready、观测需 Local Runtime、abandon 终态、跨 tenant 读写与列表、detect_conflicts、heatmap |
| crates/star-api-rest/src/lib.rs:509-512 | 1 | `POST /api/v1/worktrees` 返回内存实体的 id 与 branch |
| crates/star-api-rest/src/group_api/worktrees.rs:1517-1866 | 测试模块 | 归档门禁的 fail-closed 与新鲜度；Git 锁观测的 unknown 分支；provider 超时；outbox 事实的版本化与最小化；请求哈希稳定性；owner 分配不产生归档事件（:1845）；未变化的归档计划保持版本（:1866） |
| crates/star-api-rest/src/group_api/worktree_lifecycle.rs:859-1029 | 13 | 分支名与路径注入拒绝；候选 ID 不透明；创建角色为 project member writer（:879）；无 Run 绑定时 fail closed（:938）；provider 缺失返回 service unavailable（:955）；幂等关联 ID；receipt 与 Run、Project、Repository 的一致性 |
| crates/worktree-service/src/lifecycle.rs:197 起；pg_worktree_service.rs:588-643；crates/worktree-service/tests/ | 若干 | 7 态迁移路径（lifecycle.rs:197）；PgWorktreeService 构造、订阅；pg_worktree_service_integration.rs、external_worktree_import.rs、start_from_picker_integration.rs |
| crates/star-mcp/tests/st_five_domain_isolation.rs | 引用 domain_worktree | ❓ 未核实（确认方法：阅读该文件中对 domain_worktree 的断言） |

### 7.2 验收场景（spec §10）

| 场景 | 状态 | 证据或缺口 |
|---|---|---|
| 创建 Worktree 必带 Runtime：`runtime_id: null` 返回 422 WT-001 | ⬜ 未实现 | 无 WT-001；nil 返回 `RuntimeRequired`（lib.rs:733）且无 HTTP 映射到 422 WT-001 |
| 17 状态严格迁移：CREATED→READY 成功，CREATED→MERGED 返回 409 WT-002 | 🟡 部分实现 | 域内拒绝非法迁移（lib.rs:175-178、测试 :1101）；CREATED→READY 在代码中不是合法边（代码为 Created→Initializing） |
| Completion Gate 七项：未通过返回 409 WT-003 | ⬜ 未实现 | FR-03-11 |
| Worktree Isolation：跨 Worktree 读取 local_path_reference 返回 403 SEC-006 | ⬜ 未实现 | FR-03-09、FR-03-12 |
| Stale Display：5 分钟前的观测标记 Possibly Stale | ⬜ 未实现 | FR-03-19 |
| Worktree 状态独立于 WorkItem：任意组合同时存在 | ✅ 已实现 | FR-03-07（测试 lib.rs:1472） |

### 7.3 未验证与缺失

- 本文档基于静态阅读与检索，未运行上述测试；测试是否通过、集成测试是否在 PostgreSQL 上执行，均未核实（确认方法：执行 `cargo test -p domain-worktree`、`cargo test -p worktree-service` 与 star-api-rest 的相应测试，并检查 CI 配置）。
- 缺失的测试：17 状态的合法与非法迁移全覆盖（依赖 §7.1 与代码的边集先对齐）；Completion Gate；Isolation；Staleness 阈值；Reconciliation；领域事件；WT 错误码；线上 `human_state` 与前端索引的契约测试；线上 Run 内创建在有 provider 时的端到端测试。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/specs/domain-worktree-spec.md（Draft v0.1） | §1 职责与边界；§2 实体字段；INV-WT-01 至 INV-WT-10；§4 端口签名；§5 事件；§7 权限；§8 错误码；§10 验收场景；附录 B 的 17 状态名 | 端口返回类型与代码不同（create 返回 WorktreeId，代码返回 Worktree）；schema 名 `worktree.*`、NATS subject、WT 错误码在代码中不存在；附录 B 的 17 名与 §7.1 一致但与代码不同 | 不变量与端口职责采纳为需求；返回类型以代码为准并在 §4.3 标注差异；schema、事件、错误码记为 ⬜，不作为当前事实 |
| docs/plans/plan-016-worktree-first-class-domain-entity.md（Planned） | Phase 划分与优先级（P0、P1、P2）；MVP 验收 8 条；Token 估算 | 计划的 Command Port 含 archive（6 方法），代码为 5 方法；计划字段表与代码字段不同；计划把隔离（WT-008）归 domain-local-runtime | 采纳 Phase 划分作为优先级；Command Port 以代码的 5 方法为准，归档经迁移边与线上管理计划实现；隔离归属 local-runtime，不计入本域完成度 |
| docs/architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md（Accepted v0.2） | D27 的 8 步 Saga；D28 事件矩阵；D29 三通道；D30 节点归属；§4 缺口 | 代码中未核实到对应的 Saga 与事件订阅实现（❓，见 §4.6） | 作为跨域上下文，不计入本功能完成度；只把 `CreateWorktree` 步骤作为对本域的依赖 |
| docs/basic-design.md §4.1.2–4.1.3（约 704-735）与 §7.1（2653-2685） | 实体语义；状态机的 17 名与迁移表；归档时点 | §7.1 与代码 17 名的重合为 11；§7.1 两条归档规则（90 天、30 天）不一致；crate 注释为“>90d”（lib.rs:88） | 裁决：17 名与 §7.1 迁移表作为聚合生命周期的设计基准，理由是 spec 附录 B、spec INV-WT-02 与 crate 头注释共同引用该节；代码差异记入 FR-03-02、FR-03-04；归档时点记为待定（§6） |
| docs/design/DD-WORKTREE-CANVAS-001.md（Draft v1.4，§2.1 行 330 起、§23 行 2301-2360） | 7 个 HumanState、12 个 MachineState、状态图与迁移 | 与 §7.1 的 17 名不同；文中未见与 basic-design §7.1 的映射声明（❓ 确认方法：检索该文件对 basic-design §7.1 的引用） | 裁决：采纳为线上与画布的人类可读投影（7 值）；不作为聚合生命周期。理由：线上迁移与 worktree-service 已按该投影落库，且其定义位于画布边界内；Draft 状态意味着仍可能变化，§6 跟踪 |
| docs/features/F01-nav-project-lane.md（路由表 106-109、182-183） | `/worktree`、`/worktree/[id]/group` 与 run 内 Worktree 页的归属 | 无 | 采纳为 §4.7 的入口归属 |
| docs/features/F07-worktree-canvas-relations.md（3、155-156） | `/canvas/worktree-graph` 的归属与画布数据通道状态 | 无 | 采纳为 §4.7 的入口归属 |
| crates/domain-worktree（src/lib.rs、src/edit.rs、tests/integration_test.rs） | 域聚合、端口、迁移表、编辑与 Fork、测试 | 与 spec、§7.1 不一致（见 FR-03-02、FR-03-04、FR-03-34） | 代码为现状的依据；设计冲突时以代码定现状，以 §7.1 定目标 |
| crates/worktree-service（lib.rs:21-60、lifecycle.rs、service.rs、pg_worktree_service.rs、pg_worktree_repository.rs、projection.rs）与 crates/worktree-canvas/Cargo.toml | 线上 7 值模型、PG 写入、投影类型、依赖关系 | create 直接写 Running 并丢弃迁移结果（pg_worktree_service.rs:204、:223-227） | 作为线上现状；7 值模型保留为投影（见 §4.1） |
| crates/graph-core/src/state.rs（HumanState 29-44） | 7 值枚举的权威定义 | 无 | 采纳为 7 值枚举定义 |
| crates/star-api-rest（lib.rs 115-145、165-227；main.rs 155-166；routes/worktrees.rs；group_api.rs 441-465、516-575、697-712、791-822；group_api/worktrees.rs 106-230、330-360、448、526、570、596、843、1390-1393；group_api/worktree_lifecycle.rs 383-413、530-549、587、703；group_api/run_worktree_bindings.rs 237-290） | 生产挂载、线上管理路径、provider 注入点 | 无 | 采纳为线上现状 |
| db/migrations/2026-09-16-worktree-canvas-worktree.sql、2026-09-25-ulys-195-worktree-pg-backed.sql、2026-09-29-worktree-group-phase-2b.sql（:50-56）、2026-09-29-worktree-group-phase-2d.sql（:29）、2026-10-20-worktree-management-outbox.sql（:31） | 线上 schema、RLS、索引、管理计划与 outbox | 表名与 spec §6 的 `worktree.*` 不同 | 采纳为线上 schema；spec §6 记为 ⬜ |
| frontend/src/types/ids.ts（452-460、1030-1065）；frontend/src/app/worktree/page.tsx（167-171、266-270、382、568、910-918）；frontend/src/lib/store.ts（467-470）；frontend/src/components/worktree-canvas/types.ts（17-24） | 前端状态词表、索引归一化、迁移按钮 | 与后端值域不一致；交互仅本地；ids.ts:1031 的行号指向旧位置 | 前端词表不作为权威；差异列入 §6 |
| crates/star-mcp（tools/create_worktree.rs、tools/get_worktree.rs、handlers/worktree.rs）；crates/star-context/src/actor.rs:74 | MCP 入口；Local Runtime 标志 | create_worktree.rs:18 引用不存在的 `Creating` 变体 | 注释视为过期，以枚举为准 |
| crates/domain-local-runtime（lib.rs:62）、crates/domain-permission（lib.rs:52）、crates/domain-validation（invariants.rs:17） | 仅注释提到 domain-worktree | 无依赖关系 | 不作为来源，不计依赖 |
| crates/worktree-shared-dir、group_api 的 worktree_picker 与 worktree_external 路由 | 导入与 Start-from-picker 功能 | 不属于本功能 | 不作为来源；范围外（§1.2） |
