# F03 Worktree 领域模型与状态机

> 状态摘要: `crates/domain-worktree` 已在内存中实现 Worktree 聚合根、17 个状态枚举、严格迁移表、5 个命令端口与 5 个查询端口，并附带上下文编辑与 Fork 模块；REST 与 MCP 只暴露创建和读取两个操作。17 个状态名与设计 §7.1 仅重合 11 项。Completion Gate、Worktree Isolation、观测投影、Reconciliation、领域事件发布与角色权限均未实现。

> 代码基线: HEAD 1643559f (单一基线); 核对方法: 对 `crates/domain-worktree`、`crates/star-api-rest`、`crates/star-mcp`、`crates/worktree-service`、`crates/worktree-canvas`、`frontend/src/types/ids.ts` 做路径与符号 grep，并逐段阅读工作树文件；所引代码与三份种子文档在工作树与 HEAD 1643559f 之间无差异（路径级 git diff 检查）；核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

- 把 Worktree 建模为独立一级聚合根，持有自身的状态、健康、冲突与 Runtime 绑定，不作为 Repository 或 Branch 的附属字段（spec §1；plan-016 目标 1–3）。
- 以 17 状态严格迁移表约束生命周期，Worktree 状态与 WorkItem 状态相互独立（INV-WT-01、INV-WT-02）。
- 所有读写按 tenant 隔离，每个 Worktree 绑定一个 Runtime（INV-WT-03、INV-WT-08）。
- 通过 WorktreeCommandPort 与 WorktreeQueryPort 对外提供操作边界（spec §4）。

### 1.2 范围

| 类别 | 内容 |
|---|---|
| 范围内 | Worktree 聚合、状态枚举与迁移表、命令/查询端口及内存实现、Repository 抽象、HealthState 与 ConflictState 值对象、按 Repository 的状态计数、ContextEdit 与 Fork（`edit` 模块） |
| 范围外 | Repository、Branch、Commit（domain-scm）；ChangeSet 与 DevelopmentExecution（domain-development）；AgentSession（domain-agent）；ADR-0039 的跨域 Saga 8 步与事件矩阵（见 §6 G24）；Worktree Canvas 与 worktree-service 的并存模型（见 §4.1）；前端页面的视觉与交互 |

## 2. 用户与场景

| 角色 | 场景 | 代码依据 |
|---|---|---|
| 开发者（developer 角色） | 为 WorkItem 创建 Worktree 并绑定 Runtime | crates/star-api-rest/src/routes/worktrees.rs:71-113 |
| Local Runtime | 上报 ahead/behind 与当前会话；是唯一可写观测的 actor | crates/domain-worktree/src/lib.rs:797-799；crates/star-context/src/actor.rs:74 |
| 平台 / Agent | 把 READY 状态的 Worktree 分配给 AgentSession | crates/domain-worktree/src/lib.rs:763-790 |
| 查询方 | 按 WorkItem、按 Agent 列出；按 Repository 查看状态分布 | crates/domain-worktree/src/lib.rs:913-963、987-1012 |
| 放弃者 | 放弃非终态 Worktree | crates/domain-worktree/src/lib.rs:865-894 |
| 上下文编辑者 | 对活跃态 Worktree 增量编辑上下文或 Fork | crates/domain-worktree/src/edit.rs:440-466 |
| tenant_admin / project_admin / viewer | 设计定义的角色（spec §7） | 代码未区分，见 FR-03-25 |

主要用户故事：S1 开发者为 WorkItem 创建 Worktree；S2 Local Runtime 上报观测；S3 平台把 READY 状态的 Worktree 分配给 Agent 会话；S4 按 WorkItem 或 Agent 列出 Worktree；S5 按 Repository 查看状态分布；S6 放弃 Worktree；S7 对活跃态 Worktree 做上下文编辑与 Fork。

## 3. 需求

优先级沿用 plan-016 的阶段划分：P0 为 Phase 1（MVP）核心，P1 为 Phase 2（V1），P2 为 Phase 3（V2）。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-03-01 | 聚合携带 tenant_id、work_item_id、project_id、repository_id、branch、base_branch、runtime_id、owner_user_id、status、health、conflict_state、ahead/behind、时间戳与 version | P0 | ✅ | crates/domain-worktree/src/lib.rs:189-232 |
| FR-03-02 | 聚合携带 spec §2 的其余字段：workspace_id、development_execution_id、dirty_state、changed_files、changed_symbols、test_state、build_state、context_state、feedback_state、synchronization_state、Runtime 类型枚举 | P1 | ⬜ | lib.rs:189-232 中均无这些字段 |
| FR-03-03 | 17 状态枚举 | P0 | 🟡 | 枚举 17 项（lib.rs:63-97）；与设计 §7.1 名称重合 11 项，代码独有 INITIALIZING、COMMITTING、COMPLETED、CHANGES_REQUESTED、FIXING、STALE，设计独有 WAITING_FEEDBACK、FEEDBACK_RECEIVED、VALIDATING、READY_FOR_COMMIT、COMMITTED、PR_OPEN（裁决见 §8） |
| FR-03-04 | 迁移只允许迁移表内的边，其余返回 InvalidTransition | P0 | ✅ | lib.rs:137-181（表 145-172；错误 175-178） |
| FR-03-05 | transition_status 要求调用方给出的 from 与当前状态一致 | P0 | ✅ | lib.rs:850-855 |
| FR-03-06 | abandon 只允许从活跃态进入 ABANDONED，并经迁移表校验 | P1 | 🟡 | abandon 只检查 `is_terminal()`（lib.rs:883-888），不调用 check_status_transition，因此 Created、Blocked、Conflicted 等也可放弃；lib.rs:167 注释写"任何活跃态可"，与表中三条来源（Ready、Assigned、AgentRunning）不一致 |
| FR-03-07 | 每个 Worktree 必绑一个 Runtime（INV-WT-03） | P0 | 🟡 | 仅校验 runtime_id 非 nil UUID（lib.rs:732-734）；WT-001 错误码未定义（见 FR-03-31） |
| FR-03-08 | Status Independence（INV-WT-01）：迁移不写 WorkItem 字段 | P0 | ✅ | 迁移只修改 Worktree 自身字段（lib.rs:857-861）；测试 lib.rs:1472 |
| FR-03-09 | tenant 隔离（INV-WT-08）：跨 tenant 的读写与列表被拒绝 | P0 | ✅ | lib.rs:725-730、769-780、801-812、838-849、871-882、905-910、918-923、945-950、970-975、994-1003 |
| FR-03-10 | 观测上报只接受 Local Runtime actor | P0 | ✅ | lib.rs:797-799；测试 lib.rs:1237 |
| FR-03-11 | 1 WorkItem 对应 0..N 个 Worktree；按 WorkItem 列出 | P0 | ✅ | 字段 lib.rs:195；list_by_work_item lib.rs:913-939 与 683-693 |
| FR-03-12 | 1 Worktree 对应 0..N 个 AgentSession | P1 | 🟡 | 只保存单个 `current_agent_session_id`（lib.rs:213），无多会话集合 |
| FR-03-13 | 分配给 Agent，要求 Worktree 处于 READY | P0 | ✅ | assign_to_agent lib.rs:763-790（检查在 782 行）；迁移边 lib.rs:149；测试 lib.rs:1213 |
| FR-03-14 | 观测更新 ahead、behind、health；STALE 收到观测后回到 READY | P0 | ✅ | lib.rs:813-828 |
| FR-03-15 | 健康分级 Current、PossiblyStale、Offline、Unknown（INV-WT-10） | P1 | 🟡 | 枚举存在（lib.rs:254-264）；观测时只写入 Current（lib.rs:820）；PossiblyStale 与 Offline 无构造处；无基于时钟的分级函数 |
| FR-03-16 | 失联超过 300 秒进入 STALE | P1 | ⬜ | 迁移表允许 Ready、AgentRunning 进入 Stale（lib.rs:164-165），但没有自动触发与计时逻辑 |
| FR-03-17 | Completion Gate 七项检查，未通过不得进入 READY_FOR_REVIEW（INV-WT-06） | P0 | ⬜ | CompletionGateFailed 变体存在（lib.rs:352-353），domain 内无构造处；transition_status 不做门禁（lib.rs:832-863） |
| FR-03-18 | Isolation 检查（INV-WT-09，9 项） | P1 | ⬜ | IsolationFailed 变体存在（lib.rs:354-356），无构造处 |
| FR-03-19 | 文件级冲突检测与 WorktreeConflict 实体（§22.4） | P1 | 🟡 | detect_conflicts 返回同 Repository 内其他活跃 Worktree 的 ID（lib.rs:964-986，简化逻辑见 976 行注释）；无 WorktreeConflict 实体；ConflictState 不被回写 |
| FR-03-20 | Worktree 维度热力图（按 Repository 聚合状态计数） | P1 | 🟡 | heatmap 返回 HeatmapData（lib.rs:987-1012、561-572）；无 100 Worktree / 10k 文件的性能约束与投影表 |
| FR-03-21 | 观测高频写入与 WorktreeStatusObserved 投影（30 天热数据） | P1 | ⬜ | domain-worktree 内无投影类型；同名投影在 crates/worktree-service/src/projection.rs:39（见 §4.1） |
| FR-03-22 | Reconciliation：重连后做 Desired 与 Observed 偏差检测，偏差不静默合并 | P1 | ⬜ | 无 WorktreeReconciliationState 类型；只有 Stale→Ready 的直接迁移（lib.rs:822-825） |
| FR-03-23 | 乐观锁 version：写入时校验期望版本 | P1 | 🟡 | 每次写入递增 version（lib.rs:787、827、860、891）；命令不携带期望版本，不做 CAS 比较 |
| FR-03-24 | 领域事件发布（spec §5 的 7 类 subject，经 outbox） | P2 | ⬜ | domain-worktree 的 src 中无事件类型与 outbox 写入（grep outbox、emit、publish、DomainEvent、nats 均无命中） |
| FR-03-25 | 角色与权限字符串（spec §7：worktree:read、create、assign、abandon 等；tenant_admin、project_admin、developer、viewer） | P1 | ⬜ | domain 无角色判断；唯一的权限拒绝是非 Local Runtime 的观测（lib.rs:797-798）。`require_scope("worktree:read")` 出现在 crates/star-api-rest/src/group_api（如 canvas.rs:1762、engineering_runs.rs:739），属于 API 层，不属于本域 |
| FR-03-26 | Repository 抽象与内存实现 | P0 | ✅ | WorktreeRepository lib.rs:574-601；InMemoryWorktreeRepository lib.rs:640-716 |
| FR-03-27 | PostgreSQL 持久化与 RLS（spec §6，worktree schema） | P0 | ⬜ | domain-worktree 内无 SQL 与迁移；仓内未找到 worktree 建表语句（见 §4.2） |
| FR-03-28 | 上下文编辑只允许活跃态（INV-ED-07） | P1 | 🟡 | ensure_editable 允许 11 个状态（crates/domain-worktree/src/edit.rs:446-466），与模块头注释列出的 4 个活跃态（edit.rs:20-21）不一致 |
| FR-03-29 | Fork：子 Worktree 携带 parent_worktree_id 与 parent_at（INV-ED-02、INV-ED-03） | P1 | 🟡 | ForkSpec 校验 parent_at 不晚于当前时间（edit.rs:180-196）；fork 构造子 Worktree（edit.rs:533 起）；但 Worktree 聚合上没有这两个字段（lib.rs:189-232），edit.rs:29-30 注释却称其为聚合的扩展字段 |
| FR-03-30 | REST 与 MCP 暴露 worktree 操作 | P1 | 🟡 | 仅 create 与 get_by_id 有入口（§4.3）；assign、transition、observe、abandon、列表与 heatmap 均无路由 |
| FR-03-31 | 错误码 WT-001 至 WT-009（spec §8） | P1 | ⬜ | 代码中无 WT- 前缀错误码；WorktreeError 使用 Rust 变体名（lib.rs:328-363） |
| FR-03-32 | MERGED 之后归档（Merged→Archived）与 90 天生命周期策略（plan WT-012） | P2 | 🟡 | 迁移边 lib.rs:172 存在；没有 archive 命令（端口只有 5 个方法，lib.rs:485-520）与定时任务 |

## 4. 设计

### 4.1 概念模型

- **Worktree（聚合根）**：归属一个 tenant、一个 Project、一个 Repository 与一个 WorkItem（只作 ID 引用）；必绑一个 Runtime；可分配一个 Agent 与一个当前 AgentSession。
- **值对象**：HealthState（最后观测时间、staleness）、ConflictState（是否冲突、冲突 Worktree ID 列表、最后检测时间）。
- **派生数据**：WorktreeSummary（列表视图）、HeatmapData（按状态计数）。
- **端口与仓储**：WorktreeCommandPort、WorktreeQueryPort、WorktreeRepository。
- **上下文编辑模块**（`edit.rs`）：WorktreeEditPort、WorktreeEditQueryPort、ContextEdit、ContextEditBatch、ForkSpec。
- **并存模型（不属于本域）**：
  - `crates/worktree-service` 有另一套 Worktree 模型，描述为"9-method CRUD + 7-state machine + Health/Risk + 30-day Status Observed projection"（crates/worktree-service/Cargo.toml:13），状态为 HumanState（crates/worktree-service/src/lifecycle.rs:3）；其 Worktree 结构位于 service.rs:79。
  - `crates/worktree-canvas` 依赖 worktree-service（crates/worktree-canvas/Cargo.toml:19）。
  - 前端另有 WorktreeStatus 联合类型（frontend/src/types/ids.ts:452-464）。
  - 三者的状态词表互不相同，见 §4.4 与 §6 G18、G25。

### 4.2 数据模型

| 实体 | 字段（代码） | 状态 | 证据 |
|---|---|---|---|
| Worktree | id、tenant_id、work_item_id、project_id、repository_id、branch、base_branch、runtime_id、local_path_reference、owner_user_id、assigned_agent_id、current_agent_session_id、status、health、conflict_state、ahead、behind、last_activity_at、created_at、updated_at、version | 🟡 | lib.rs:189-232 |
| HealthState | last_observed_at、staleness | ✅ | lib.rs:235-251 |
| Staleness | Unknown、Current、PossiblyStale、Offline | 🟡 | lib.rs:254-264 |
| ConflictState | has_conflicts、conflicting_worktree_ids、last_detected_at | 🟡（域内从不写入） | lib.rs:267-286 |
| WorktreeSummary | id、tenant_id、work_item_id、status、runtime_id、updated_at | ✅ | lib.rs:463-477 |
| HeatmapData | repository_id、total_worktrees、by_status、generated_at | 🟡 | lib.rs:561-572 |
| CreateWorktreeCommand | tenant_id、project_id、work_item_id、repository_id、branch、base_branch、runtime_id、owner_user_id | ✅ | lib.rs:370-388 |
| AssignWorktreeCommand | tenant_id、worktree_id、agent_id、agent_session_id | ✅ | lib.rs:391-401 |
| RecordObservedStateCommand | tenant_id、worktree_id、ahead、behind、current_agent_session_id | ✅ | lib.rs:404-416 |
| TransitionStatusCommand | tenant_id、worktree_id、from、to、reason | ✅ | lib.rs:419-431 |
| AbandonCommand | tenant_id、worktree_id、reason | ✅ | lib.rs:434-442 |
| ListByWorkItemQuery、ListByAgentQuery | tenant_id 加 work_item_id 或 agent_id | ✅ | lib.rs:445-460 |
| ContextEdit、ContextEditBatch、ForkSpec | 上下文编辑与 Fork 的输入结构 | 🟡 | edit.rs:99-205 |
| WorktreeStatusObserved | 设计 §2 定义的投影，域内无对应类型 | ⬜ | spec §2；lib.rs 中无定义 |
| WorktreeConflict | 设计 §2 定义的冲突实体，域内无对应类型 | ⬜ | spec §2 |
| WorktreeReconciliationState | 设计 §2 定义的值对象，域内无对应类型 | ⬜ | spec §2 |
| worktree.worktree、worktree_status_observed、worktree_conflict、worktree_reconciliation_state 表 | spec §6 定义的 schema | ❓ | 仓内未找到 worktree 建表语句；确认方法是检查 infrastructure 层迁移目录 |

### 4.3 接口

**WorktreeCommandPort**（crates/domain-worktree/src/lib.rs:483-520）

| 方法 | 代码签名要点 | 与 spec §4 的差异 | 状态 |
|---|---|---|---|
| create_worktree | (CreateWorktreeCommand, &ActorContext) → Worktree（lib.rs:487-491） | spec 返回 WorktreeId | ✅ |
| assign_to_agent | (AssignWorktreeCommand, &ActorContext) → Worktree（lib.rs:494-498） | spec 返回 () | ✅ |
| record_observed_state | (RecordObservedStateCommand, &ActorContext) → Worktree（lib.rs:501-505） | 一致 | ✅ |
| transition_status | (TransitionStatusCommand, &ActorContext) → Worktree（lib.rs:508-512） | spec 返回 WorktreeStatus | ✅ |
| abandon | (AbandonCommand, &ActorContext) → Worktree（lib.rs:515-519） | spec 返回 () | ✅ |
| archive | plan WT-004 要求的第 6 个命令 | 代码无 | ⬜ |

**WorktreeQueryPort**（lib.rs:524-559）

| 方法 | 代码签名要点 | 状态 |
|---|---|---|
| get_by_id | (WorktreeId, &ActorContext) → Worktree（lib.rs:526-530） | ✅ |
| list_by_work_item | ListByWorkItemQuery → Vec<WorktreeSummary>（lib.rs:533-537） | ✅ |
| list_by_agent | ListByAgentQuery → Vec<WorktreeSummary>（lib.rs:540-544） | ✅ |
| detect_conflicts | WorktreeId → Vec<WorktreeId>（lib.rs:547-551）；spec 要求 Vec<WorktreeConflict> | 🟡 |
| heatmap | RepositoryId → HeatmapData（lib.rs:554-558） | 🟡 |

**编辑端口**（crates/domain-worktree/src/edit.rs:282-320）

- WorktreeEditPort：apply_context_edits（edit.rs:290）、fork（edit.rs:302）。
- WorktreeEditQueryPort：list_context_edits（edit.rs:309）、list_fork_children（edit.rs:316）。

**仓储端口**：WorktreeRepository（lib.rs:576-601），含 insert、get、update 与三种列表。

**外部入口**

| 入口 | 文件 | 状态 |
|---|---|---|
| POST /api/v1/worktrees | crates/star-api-rest/src/routes/worktrees.rs:71-113；注册于 crates/star-api-rest/src/lib.rs:114 与 279 | ✅（内存 service） |
| GET /api/v1/worktrees/{id} | crates/star-api-rest/src/routes/worktrees.rs:29-55；注册于 lib.rs:115 与 280 | ✅（内存 service） |
| 状态迁移、分配、观测、放弃、列表、热力图的 HTTP 入口 | 无路由 | ⬜ |
| MCP 工具 create_worktree | crates/star-mcp/src/tools/create_worktree.rs:54；分发于 crates/star-mcp/src/transport.rs:359 | ✅（内存 service） |
| MCP 工具 get_worktree | crates/star-mcp/src/tools/get_worktree.rs:39；分发于 transport.rs:358 | ✅（内存 service） |
| 前端 MSW mock（GET /api/worktrees、GET /api/worktrees/:id、POST /api/worktrees/:id/transition） | frontend/src/mocks/handlers/worktrees.ts:1-7 | 🟡（仅 mock，路径无 /api/v1 前缀） |

### 4.4 状态机与关键流程

运行时的迁移只经过 `check_status_transition`（lib.rs:137-181）。两处例外：STALE 收到观测后直接回到 READY（lib.rs:822-825，与表中 Stale→Ready 一致）；abandon 只检查终态（lib.rs:883-888）。

后端迁移表（lib.rs:145-172）：

| 来源状态 | 允许的目标状态 |
|---|---|
| Created | Initializing |
| Initializing | Ready、Blocked |
| Ready | Assigned、Conflicted、Stale、Abandoned |
| Assigned | AgentRunning、Abandoned |
| AgentRunning | Committing、Conflicted、Stale、Abandoned |
| Committing | Completed |
| Completed | ReadyForReview |
| ReadyForReview | Reviewing |
| Reviewing | ChangesRequested、Merged |
| ChangesRequested | Fixing |
| Fixing | AgentRunning、Ready |
| Stale | Ready |
| Merged | Archived |
| Blocked、Conflicted、Abandoned、Archived | 无出边（abandon 例外） |

要点：Created 不能直接到 Ready 或 Assigned，assign 要求 Ready，因此必须经过 Initializing（lib.rs:145-150）；Blocked 与 Conflicted 在迁移表中没有出边，异常态无恢复路径。

代码中的主路径（测试 lib.rs:1080）：Created → Initializing → Ready → Assigned → AgentRunning → Committing → Completed → ReadyForReview → Reviewing → Merged → Archived。

前端 WORKTREE_SM（frontend/src/types/ids.ts:1030-1065）与后端迁移表的差异：

- 只在前端存在的边：assigned→blocked、blocked→assigned、assigned→conflicted、conflicted→assigned、stale→assigned、merged→abandoned、fixing→ready_for_review。
- 只在后端存在的边：initializing→blocked、ready→conflicted、agent_running→conflicted、ready→stale、stale→ready、ready→abandoned、assigned→abandoned、agent_running→abandoned、fixing→agent_running、fixing→ready。
- 两者一致的边：created→initializing、initializing→ready、ready→assigned、assigned→agent_running、agent_running→committing、committing→completed、completed→ready_for_review、ready_for_review→reviewing、reviewing→changes_requested、reviewing→merged、changes_requested→fixing、merged→archived、agent_running→stale。

前端 WorktreeStatus 联合类型（ids.ts:452-464）共 17 个值，名称为 initializing、cloning、syncing、active、dirty、behind、diverged、conflict、committing、pushing、ci_running、review_requested、merged、closed、abandoned、archived、reverted，与后端枚举名称只有 initializing、committing、merged、abandoned、archived 重合。

### 4.5 权限与安全

- **tenant 隔离**：所有读写路径比较 actor 的 tenant 与 Worktree 的 tenant，不一致即拒绝（见 FR-03-09）。
- **观测来源**：只有 `is_local_runtime` 为真的 actor 可以写观测（lib.rs:797-799；字段定义 crates/star-context/src/actor.rs:74）。
- **Local Path Opacity（INV-WT-04）**：`local_path_reference` 字段存在（lib.rs:207），创建时恒为 None（lib.rs:745），没有写入路径。REST 响应不返回该字段（routes/worktrees.rs:38-53），但 domain 的 get_by_id 返回整个聚合（lib.rs:899-912）。状态为 🟡。
- **角色检查**：domain 不做角色判断。assign_to_agent、transition_status、abandon 对同 tenant 的任何 actor 开放（lib.rs:763-894）。spec §7 的权限矩阵未实现（FR-03-25）。
- **REST 身份来源**：routes/worktrees.rs:36 与 77 使用 `ActorContext::default().with_role("developer")`，身份不由请求派生。
- **隔离（INV-WT-09）**：Filesystem、Env、Process、Port、Secret 等 9 项隔离检查未实现（FR-03-18）。

### 4.6 依赖关系

- **crate 依赖**：`star-context`（Cargo.toml 的 path 依赖，提供 ActorContext）；其余为 serde、serde_json、async-trait、thiserror、uuid、chrono、tokio。
- **spec 列出的上游 domain**（domain-tenant、domain-workspace、domain-project、domain-work-item、domain-scm、domain-local-runtime）未出现在 Cargo.toml 中；跨域关系只以 ID 引用。
- **调用方**：
  - crates/star-api-rest（routes/worktrees.rs）。
  - crates/star-mcp（tools/create_worktree.rs、tools/get_worktree.rs、handlers/worktree.rs）。
  - crates/star-desktop/src-tauri（ipc_adapter.rs:40 引入 WorktreeStatus；Cargo.toml 含依赖）。
  - crates/api 与 crates/application 的错误映射（crates/api/src/lib.rs:306-322；crates/application/src/lib.rs:436-464）。
- **ADR-0039 的 Saga**：crates/star-saga/src 中无 worktree 相关代码（grep 无命中）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| Worktree 聚合根与字段 | crates/domain-worktree/src/lib.rs | 🟡 | lib.rs:187-232 |
| 17 状态枚举 | crates/domain-worktree/src/lib.rs | 🟡 | lib.rs:61-98 |
| 迁移表与校验函数 | crates/domain-worktree/src/lib.rs | 🟡 | lib.rs:135-181 |
| HealthState 与 Staleness | crates/domain-worktree/src/lib.rs | 🟡 | lib.rs:234-264 |
| ConflictState | crates/domain-worktree/src/lib.rs | 🟡（域内不写入） | lib.rs:266-286 |
| 命令与查询 DTO | crates/domain-worktree/src/lib.rs | ✅ | lib.rs:369-477 |
| WorktreeCommandPort | crates/domain-worktree/src/lib.rs | 🟡（5 个方法，签名与 spec 不同） | lib.rs:483-520 |
| WorktreeQueryPort | crates/domain-worktree/src/lib.rs | 🟡（detect、heatmap 为简化实现） | lib.rs:522-559 |
| WorktreeRepository 与内存实现 | crates/domain-worktree/src/lib.rs | ✅（仅内存） | lib.rs:574-716 |
| InMemoryWorktreeService | crates/domain-worktree/src/lib.rs | ✅（仅内存） | lib.rs:607-1013 |
| 冲突检测 | crates/domain-worktree/src/lib.rs | 🟡（同 Repository 启发式） | lib.rs:964-986 |
| 热力图 | crates/domain-worktree/src/lib.rs | 🟡 | lib.rs:987-1012 |
| Completion Gate | crates/domain-worktree/src/lib.rs | ⬜（仅错误变体） | lib.rs:351-353 |
| Isolation 检查 | crates/domain-worktree/src/lib.rs | ⬜（仅错误变体） | lib.rs:354-356 |
| Reconciliation | crates/domain-worktree | ⬜ | 无对应类型 |
| 观测投影 | crates/domain-worktree | ⬜ | 无对应类型（另见 crates/worktree-service/src/projection.rs，不属于本域） |
| 上下文编辑与 Fork | crates/domain-worktree/src/edit.rs | 🟡 | 端口 edit.rs:282-320；实现 edit.rs:415-620 |
| REST 创建与读取 | crates/star-api-rest/src/routes/worktrees.rs | ✅（内存） | routes/worktrees.rs:29-113；注册 star-api-rest/src/lib.rs:114-115、279-280 |
| MCP 创建与读取 | crates/star-mcp/src/tools/create_worktree.rs、get_worktree.rs | ✅（内存） | create_worktree.rs:54；get_worktree.rs:39；transport.rs:358-359 |
| 错误映射 | crates/api/src/lib.rs、crates/application/src/lib.rs | ✅ | api/src/lib.rs:306-322；application/src/lib.rs:436-464 |
| 并存的 Worktree 模型 | crates/worktree-service/src | 🟡（与本域并存，不作为本域依据） | service.rs:79；service_impl.rs:35；lifecycle.rs:26-56 |
| 前端状态类型与迁移表 | frontend/src/types/ids.ts | 🟡（与后端两套词表不一致） | ids.ts:452-464、467（lane_id）、1030-1065 |
| 已退役的实现 | — | 未发现 | 本域内无已删除或已替换的 worktree 状态实现 |

## 6. 差距与后续

| 编号 | 缺口 | 影响 | 闭合的下一步 |
|---|---|---|---|
| G1 | 17 状态名称与设计 §7.1 不一致（6 项互异） | 同一状态在 API、前端、设计中名称不同，不能互通 | 确定唯一集合；修订枚举或设计附录 B，给出对照表 |
| G2 | abandon 绕过迁移表，Created、Blocked、Conflicted 等可放弃（lib.rs:883-888） | INV-WT-02 未被满足 | abandon 改为经 check_status_transition；迁移表补 Abandoned 的来源集合；补测试 |
| G3 | Blocked 与 Conflicted 没有出边（lib.rs:145-172） | 异常态只能放弃，无法恢复 | 定义恢复迁移（前端表已有 blocked→assigned、conflicted→assigned 的设计） |
| G4 | Completion Gate 未实现（lib.rs:832-863） | READY_FOR_REVIEW 可在无检查时到达 | 注入 CompletionGate 端口，七项检查由 Project Policy 提供 |
| G5 | Isolation 检查未实现（lib.rs:354-356） | 跨 Worktree 上下文泄露风险（RISK-019） | 定义 IsolationCheck 端口，由 Local Runtime 回报结果 |
| G6 | 时间型 Staleness 未计算，PossiblyStale 与 Offline 无构造（lib.rs:254-264） | UI 无法区分可能过期的 Worktree | 注入时钟，实现 classify(now)，读取时派生 |
| G7 | STALE 无自动触发（lib.rs:164-165） | 失联 Worktree 不进入 STALE | 周期任务，或读取时按 HealthState 派生 |
| G8 | 冲突检测把同 Repository 内全部活跃 Worktree 视为冲突（lib.rs:976-985） | 误报，且不区分文件 | 实现文件级检测；写回 ConflictState；定义 WorktreeConflict |
| G9 | 观测写入与主表未分离（lib.rs:813-828） | 高频观测放大主表写入 | 增加投影表与批量接口（plan WT-006） |
| G10 | Reconciliation 缺失 | 重连后的偏差被静默覆盖 | 实现 WorktreeReconciliationState 与偏差列表 |
| G11 | 无角色与权限检查（lib.rs:763-894） | 同租户的任何 actor 可以分配、迁移、放弃 | 增加授权端口，按 spec §7 字符串校验 |
| G12 | 无 WT-xxx 错误码（lib.rs:328-363） | API 错误模型与 spec §8 不符 | 为 WorktreeError 增加稳定错误码映射 |
| G13 | ContextEdit 的活跃态判定与模块头不一致（edit.rs:446-466 与 edit.rs:20-21） | 文档与实现分裂 | 以设计确定 4 个或 11 个状态，统一代码与注释 |
| G14 | Fork 的父关系未在聚合上持久（lib.rs:189-232） | 子 Worktree 的父关系无法查询 | 在 Worktree 上增加 parent_worktree_id、parent_at，或建立独立关系表 |
| G15 | 只保存单个 current_agent_session_id（lib.rs:213） | 不支持多会话（spec J-WT-02 未决） | 决策后改为集合 |
| G16 | 聚合无 lane_id 字段（lib.rs:189-232） | 不满足 AGENTS.md §0 中"Worktree 属于 Lane 管理的本地 checkout"的约束；前端类型已有可选的 lane_id（ids.ts:467） | 增加 lane_id，并与 Lane 的归属规则对齐 |
| G17 | 聚合缺少 spec §2 的大部分内容字段（FR-03-02） | 变更文件、测试与构建状态不可见 | 分批补充 |
| G18 | Worktree 与 worktree-service 两套模型并存（crates/worktree-service/src/service.rs:79；lifecycle.rs:26-56） | 同一概念有两个状态机与两套端口 | 决定主模型；迁移或收敛另一方 |
| G19 | 无 PostgreSQL 实现与 RLS，内存服务是进程级单例（routes/worktrees.rs:23-26） | 重启丢失数据，多实例状态不一致 | 实现 WorktreeRepository 的 PG 适配；建立 worktree schema（spec §6） |
| G20 | REST 使用硬编码 actor，只有 create 与 get（routes/worktrees.rs:36、77） | 身份不来自请求；其余操作没有 HTTP 入口 | 接入认证与 actor 提取；补全路由 |
| G21 | 无 archive 命令与 90 天生命周期（lib.rs:485-520） | MERGED 之后没有归档路径 | 增加 archive 命令与定时归档（plan WT-012） |
| G22 | 领域事件未发布（FR-03-24） | audit、validation、notification 等订阅方没有输入 | 在迁移事务内写 outbox，按 spec §5 的 subject 发布 |
| G23 | 迁移与观测不做 CAS（lib.rs:850、792-830） | 并发写可能互相覆盖 | 命令携带 expected version 并校验 |
| G24 | ADR-0039 §5.5 的实施证据与代码不符 | 跨域编排的实施状态无法核实 | 修正 ADR 中的 commit 引用，或补充 star-saga 实现后再记录 |
| G25 | 前端两套状态词表，且 ids.ts 注释中的行号引用已偏移（实际为 lib.rs:61-98） | UI 与后端状态不能映射 | 由后端枚举生成前端类型 |
| G26 | frontend/src/lib/group/worktreeGroupApi.ts 调用的 `/api/v1/worktrees/{id}/group-context` 与 `/api/v1/projects/{id}/worktrees` 在 star-api-rest 与 crates/api 中无路由 | 前端调用失败 | 实装端点，或前端降级 |

plan-016 验收标准的核对：

| plan 验收项 | 状态 | 证据 |
|---|---|---|
| 1. worktrees 表包含 22 字段与三级 tenant 隔离 | ❓ | 表结构不在本域内（FR-03-27） |
| 2. 17 状态机完整，转换规则 100% 覆盖 | 🟡 | 状态机存在；无全部 (from, to) 对的穷举测试（§7） |
| 3. 1 WorkItem 对 N Worktree，无直接外键 | ✅ | lib.rs:195、913-939 |
| 4. 观测状态走投影表，1 秒批量节流 | ⬜ | G9 |
| 5. Heatmap 100 Worktree、10k 文件低于 500ms | ⬜ | 无性能测试（§7） |
| 6. Completion 七项检查默认全部必须 | ⬜ | G4 |
| 7. Reconciliation 偏差不静默合并 | ⬜ | G10 |
| 8. Code Review 通过（10 项清单） | ❓ | 评审记录不在代码中 |

## 7. 验收与测试

### 7.1 现有测试

| 测试集 | 位置 | 数量（属性计数） | 覆盖内容 |
|---|---|---|---|
| 域单元测试 | crates/domain-worktree/src/lib.rs:1020 起 | 23 | 枚举与终态（lib.rs:1040、1064）；迁移主路径与非法迁移（1080、1101、1108、1114、1123）；create、transition、assign、observe、abandon；跨 tenant；list、detect、heatmap（1437、1455）；状态独立（1472） |
| 编辑模块测试 | crates/domain-worktree/src/edit.rs:645 起 | 25 | 活跃态与终态的上下文编辑（edit.rs:839、872）；跨 tenant（892、1083）；非法路径（914）；历史（934）；Fork 的未来时间与 nil runtime（968–1058） |
| 集成测试 | crates/domain-worktree/tests/integration_test.rs | 11 | V0 CRUD（58、91）；V1 状态机全链（107）、assign（152）、observe（177）、abandon（220）；V2 跨 tenant（261、278、302）；V3 冲突与热力图（331、367） |
| REST 测试 | crates/star-api-rest/src/lib.rs:523 | 1 | POST /api/v1/worktrees 返回真实实体 |
| MCP 测试 | crates/star-mcp/src/tools/create_worktree.rs、get_worktree.rs、handlers/worktree.rs | 9 | 参数校验与内存 service 路径（create_worktree.rs:111、118、139；get_worktree.rs:77、86、93；handlers/worktree.rs:104、111、135） |

### 7.2 已覆盖

- 迁移表的主路径、若干非法迁移与终态拒绝。
- tenant 隔离的读、写、列表与热力图路径。
- Local Runtime 写观测的权限拒绝。
- 上下文编辑在活跃态与终态的行为，以及 Fork 的时间校验。
- REST 与 MCP 的创建与读取走真实内存 service。

### 7.3 未覆盖或未验证

- 17 个状态之间全部 (from, to) 对的穷举测试（未见）。
- Completion Gate、Isolation、Reconciliation、Staleness 的时间分级、事件发布、权限矩阵、PostgreSQL 持久化、热力图性能。
- 本核对未执行 `cargo test`，各测试的当前通过状态未确认。

### 7.4 plan-016 的 MVP 验收

验收项的逐条状态见 §6 末表。

## 8. 来源与裁决

| 源文档或代码 | 吸收的内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| docs/specs/domain-worktree-spec.md（Draft v0.1） | 职责边界、实体字段、INV-WT-01 至 10、端口签名、事件 subject、表与索引、权限字符串、错误码、AC、风险、Open Issues | 17 状态名与代码 6 项不同；端口返回类型不同；事件、权限、错误码与多数不变量未实现 | 不变量与端口语义保留为设计意图；状态名以 lib.rs:63-97 为当前事实；差异进入 §6 |
| docs/plans/plan-016-worktree-first-class-domain-entity.md（v0.1） | Phase 1 至 3 任务 WT-001 至 012；MVP 验收 1 至 8；非目标 | WT-004 要求 6 个命令（含 archive），代码为 5 个；WT-006 的 1 秒批量节流未实现 | 任务清单作为计划保留；状态按代码标注；archive 标为 ⬜ |
| docs/architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md（v0.2） | D26 至 D32：跨域协作范围、Saga 8 步、事件矩阵、Realtime、Context Graph；§5.5 实施证据 | §5.5 把 commit 863b69b 写成"star-saga worktree 集成"，实际该提交为 docs(adr) 0036；crates/star-saga 中无 worktree 代码 | 跨域编排不纳入本域实现；§5.5 证据标 ❓ |
| crates/domain-worktree/src/lib.rs | 聚合、枚举、迁移表、端口、内存实现、测试 | 与 spec 的状态名与签名有差异（见 §4.3、§4.4） | 当前事实的主要来源 |
| crates/domain-worktree/src/edit.rs | 上下文编辑与 Fork 的实现与测试 | 模块头 INV-ED-07 与 ensure_editable 不一致；模块头称聚合有 parent 字段，但聚合上没有 | 以代码为准，差异进入 §6（G13、G14） |
| crates/domain-worktree/tests/integration_test.rs | 集成测试与 W/T/M 分类注释 | 注释列出的 17 状态与 lib.rs 一致 | 采纳 |
| crates/star-api-rest/src/routes/worktrees.rs；crates/star-api-rest/src/lib.rs | REST 创建与读取、路由注册、集成测试 | 硬编码 actor；其余操作无路由 | 以代码为准 |
| crates/star-mcp/src/tools/create_worktree.rs、get_worktree.rs、handlers/worktree.rs；transport.rs | MCP 工具与分发 | handlers/worktree.rs:61 的资源示例 URI（wt-STAR-1024）为示例数据 | 采纳 create 与 get；示例不计入实装 |
| crates/api/src/lib.rs；crates/application/src/lib.rs | WorktreeError 到 API 错误的映射 | 映射提示文案引用"17 状态机"与"completion gate"，对应逻辑未实现 | 采纳映射；逻辑缺口见 §6 |
| crates/star-desktop/src-tauri/src/ipc_adapter.rs；db_adapter.rs | 仅依赖 WorktreeStatus 枚举；DbWorktree 为独立 demo 结构 | db_adapter.rs:70 注释称与 domain 对齐，实为独立结构 | 只计为消费方 |
| crates/worktree-service/src（service.rs、service_impl.rs、lifecycle.rs、projection.rs；Cargo.toml） | 另一套 Worktree 模型：9 方法 CRUD、7 状态 HumanState、30 天投影 | 聚合、状态名、端口与本域完全不同 | 不作为本域裁决依据；并存问题列入 G18 |
| crates/worktree-canvas/Cargo.toml | 依赖 worktree-service | — | 作为依赖关系的证据 |
| crates/api/src/worktree_picker.rs（头部注释） | 注释称不依赖 worktree-service | — | 仅用于确认依赖关系 |
| crates/star-saga/src | 检索 worktree 相关代码 | 无命中 | 与 ADR-0039 §5.5 所引证据不符，见 G24 |
| frontend/src/types/ids.ts（452-467、1030-1065） | 前端 WorktreeStatus 联合类型、lane_id、WORKTREE_SM | 与后端两套词表不一致；注释行号引用偏移 | 前端展示类型，不作为运行时事实（G25） |
| frontend/src/mocks/handlers/worktrees.ts | mock 端点 | 路径为 /api/worktrees，不含 /api/v1 | 仅 mock |
| frontend/src/lib/group/worktreeGroupApi.ts | 调用的 worktree 端点 | group-context 与 projects/{id}/worktrees 无后端路由 | 列入 G26 |
| crates/star-api-rest/src/group_api/hook_policies.rs | worktree 的 hook-policy 路由 | 属于相邻能力 | 不纳入本文档 |
| AGENTS.md §0（产品导航与 Worktree 管理约束） | Worktree 属于 Lane 管理的本地 checkout；Lane 不依赖远端 Branch | 聚合无 lane_id | 作为目标约束，列入 G16 |
| 用户记忆索引中的 star-api-rest backend takeover 条目 | 对 REST 层"全部为 stub"的描述 | 与 routes/worktrees.rs:29-113 的真实内存调用不符 | 以代码为准 |
| docs/features/ 下同路径的既有文件 | — | — | 不作为依据；以代码与三份种子文档为准 |
