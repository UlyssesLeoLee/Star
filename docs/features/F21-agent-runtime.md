# F21 Star Agent 运行时

> 状态摘要: 运行时的纯逻辑内核已在 `crates/star-agent-runtime` (Rust, std-only, 进程内) 与 `scripts/automation/agent_runtime/` (Python 标准库) 两处实现并有单元测试通过; L0 派发 (`star-dispatcher` 内存 PoC、`star-taskqueue` SQLite WAL 队列)、配额、记忆、上下文分层、遥测各自存在但彼此未接线, 也未接入 REST 或前端的真实运行时数据。Rust 侧 ECS 为自研 HashMap 占位, bevy_ecs/flecs 未选型; 持久化、崩溃恢复、取消/超时、真实 Shared LLM/HTTP/MCP Pool、1M 规模基准均未实现。
>
> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

为 STAR 的 Agent 提供一套运行时内核, 使逻辑 Agent 数量与实际运行资源解耦: Agent 由 Identity、State、Lifecycle 与一组引用 (Context/Memory/Model/Tool/Permission/Mailbox/Workflow) 组成, 不持有大对象; 运行时在小规模使用 Lightweight 模式、达到阈值后切换 ECS 模式; 派发经过速率限制、反压、优先级与公平调度、HOT 槽位限制, 并通过有界队列、重试与死信保证有界。

设计依据: `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` §1-§8 (Agent 定义与 ECS 组件)、§10-§12 (状态与组件)、§29-§47 (共享池、调度、重试、持久化)。

### 1.2 范围内 / 范围外

范围内:

- Agent 实体模型 (Identity / State / Lifecycle / 引用组件 / TokenBudget / Priority)。
- Runtime 双模式与切换阈值 (Lightweight / ECS / 迟滞区)。
- 生命周期 HOT/WARM/COLD 转换与超时扫描。
- 调度 (ready 队列、HOT 槽位、租户公平份额)、反压、租户速率限制、重试策略。
- 事件总线与 Mailbox 投递语义。
- L0 派发: 内存 PoC 派发器、SQLite WAL 任务队列、子代理池、检查点存储。
- 租户配额、记忆存储、上下文分层、Token 计量 (作为运行时依赖的同层组件)。
- Python 侧 `AgentRuntime` 门面及其子进程执行池。
- 本功能的唯一前端页面 `/agents` 与其所用的只读 REST 端点 `/api/v1/agents`。

范围外 (由其他功能文档或其他视图承担):

- LangGraph 任务卡子代理与 TMO 任务卡操作 (见 §8 来源 `docs/architecture/2026-09-03-langgraph/`)。
- Agent 会话与 Hook 判定、Local Runtime 心跳 (`routes/runtime.rs`, `local-runtime` 页面)。
- Physis 物理引擎与 3D 渲染。
- 跨机分布式调度与 Kubernetes 部署编排。
- Agent 图谱画布 (F08) 与 Agent 关系 (`/agent-relationships`)。

## 2. 用户与场景

| 角色 | 场景 | 与本功能的关系 |
|---|---|---|
| 平台开发者 (Rust/Python) | 在运行时内核上接入新的派发、调度与生命周期策略 | 直接使用 `star-agent-runtime` 的类型与 `AgentRuntime` 门面 |
| 任务派发方 (守门/编排脚本) | 将 `Task` 投递为可调度单元, 观察 HOT/WARM/COLD 与模式切换 | Python `AgentRuntime.dispatch` / `tick` / `get_runtime_metrics` |
| 运营/管理者 | 在前端查看当前 Agent 花名册并手动同步 | `/agents` 页面 (当前数据为服务端样例) |
| 租户 | 受限于各自速率与配额, 不影响其他租户 | 租户维度的速率限制与公平份额 |

主要用户故事:

- US-1: 作为派发方, 我提交一个带租户与优先级的任务, 运行时按优先级与公平份额排入 HOT 槽位, 超出时保留在 ready 队列。
- US-2: 作为派发方, 任务失败后按最大重试次数回到 Pending, 超过后进入 Dead (死信)。
- US-3: 作为运营者, 我在 `/agents` 看到 Agent 列表, 并能手动触发同步与健康检查。
- US-4: 作为平台开发者, Agent 闲置后自动由 HOT 降为 WARM, 再降为 COLD。

## 3. 需求

优先级: P0 = 运行时内核必需; P1 = 生产化必需; P2 = 扩展。状态标记按 §0 所列的五类 (✅ / 🟡 / ⬜ / 🗑 / ❓)。证据均在 637ed19d 下读取。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-21-01 | Agent 由 Identity + State + Lifecycle + 引用组件构成, 组件只存引用, 不存大对象 | P0 | ✅ | `crates/star-agent-runtime/src/identity.rs:16-41`; `components.rs:73-201` (ContextRef/MemoryRef/ModelRef/ToolPolicyRef/PermissionRef/MailboxRef/WorkflowRef 仅为 ID 包装) |
| FR-21-02 | AgentIdentity 含 agent_id、tenant_id、agent_type; agent_type 为 SA-01..SA-09 加 SA-N | P0 | ✅ | `identity.rs:16-41`; `agent_type.rs:15-65` (`SA_TYPES` 9 项, `SA01`..`SA09`, `SA_N`) |
| FR-21-03 | AgentState 12 状态与合法转换表, 非法转换拒绝且不改状态 | P0 | ✅ | `agent_state.rs:19-35` (`AgentStateEnum` 12 变体), `:79-127` (`can_transition`), `:130-139` (`transition` 非法返回 false) |
| FR-21-04 | AgentState 携带 prev、since、retry_count; fail 计数, retry 回 Ready | P0 | ✅ | `agent_state.rs:48-57`, `:156-165` |
| FR-21-05 | LifecycleState HOT/WARM/COLD, 按空闲时长 sweep 降级, restore 自 COLD 回 WARM | P0 | ✅ | `lifecycle.rs:28-37` (枚举), `:91-115` (`sweep`), `:107-115` (`restore`), 默认 HOT 超时 60s / WARM 超时 600s (`:22-24`) |
| FR-21-06 | LifecycleManager 状态转换表、超时判定、驱逐 | P0 | 🟡 | `lifecycle.rs:194-255`: `transition` 合法集合 `:206-213`, `timeout` `:236-247`, `evict` `:249-255`; `persist` 仅校验存在性 `:219-225` (G-7 持久化未实现) |
| FR-21-07 | Runtime 双模式: 0-8 Lightweight, 9 必为 Lightweight, 10-11 迟滞区, ≥12 ECS 候选 | P0 | ✅ | `runtime_mode.rs:19-25`, `:139-182` (`decide`), `:184-189` (`hysteresis_zone`); 测试 `tests/integration.rs:316-365` (`mode_small_scale_stays_lightweight`, `mode_9_forces_lightweight_from_ecs`, `mode_hysteresis_10_11_stays_current`) |
| FR-21-08 | 进入 ECS 需持续 ≥30s, 退出需持续 ≥300s, 阈值可配置且 fail-safe 校验 | P0 | ✅ | 默认值 `runtime_mode.rs:39-50`; 校验 `:77-87`; 测试 `integration.rs:366-418` (`mode_stable_duration_and_reset`, `mode_exit_requires_300s`, `mode_thresholds_configurable`, `mode_config_fail_safe`) |
| FR-21-09 | 小于 10 个 Agent 禁止启用完整 ECS (SRS §3.2 硬约束) | P0 | 🟡 | 默认 `ecs_enable_threshold=12` (`runtime_mode.rs:41`), 但 `validate` 允许 5 (`:77-87` 仅检查 `==0` 与迟滞区非空); 对 `enable=5, disable=1` 的探针返回 `Ok(())`。Python 侧同理 (`scripts/automation/agent_runtime/config.py:51-98` 无下限) |
| FR-21-10 | ECS World: 组件化存储, 按 Agent 查询组件, 原型 (archetype) 标识 | P1 | 🟡 | `ecs.rs:54-69` (World 为每类组件一个 `HashMap<AgentId, _>`), `:31-` (`ArchetypeId` 为位掩码 `BIT_*` `:36-52`); 非真正的 archetype 表与稀疏集 (DD §6.2 未实现); bevy_ecs/flecs 未选型 (`lib.rs:21-22`, G-2) |
| FR-21-11 | ECS System (SchedulerSystem / LifecycleSystem 等) 批量执行 | P1 | ⬜ | 在 `crates/star-agent-runtime/src` 中未找到 System trait 或系统调度入口 (grep `System` 无结果); Lifecycle 仅有 `sweep` 方法 `lifecycle.rs:91` |
| FR-21-12 | Ready 队列按优先级与入队序出队; HOT 槽位上限; 满则保留等待 | P0 | ✅ | `scheduler.rs:78-88` (字段), `:111-124` (`enqueue_ready`), `:181-215` (`schedule`: `hot_slot_available` 检查与 HotSlotFull reason), `:33-46` (`Ord`); 测试 `integration.rs:452-497` (`scheduler_zero_hot_limit_rejected`, `scheduler_priority_order`, `scheduler_hot_slot_limit`) |
| FR-21-13 | 租户公平份额 (Normal/Low/Background 按租户轮转, Critical/High 不受限) | P0 | ✅ | `scheduler.rs:173-179` (`fair_ok`), `:194-211` (份额计数); 测试 `integration.rs:498-545` (`scheduler_fair_share_round_robin`, `scheduler_critical_bypasses_fair_share`) |
| FR-21-14 | 优先级五级 Critical/High/Normal/Low/Background | P0 | ✅ | `components.rs:47-58` |
| FR-21-15 | 反压: 有界队列, 溢出策略 Reject/Delay/DropLow/Persist/Throttle | P0 | 🟡 | `backpressure.rs:16-31` (`OverflowPolicy`), `:93-145` (`await_slot` 仅返回决策); `Persist` 与 `DropLow` 仅返回决策, 未执行持久化或丢弃; 默认策略 Throttle 60s `:35-40`; 测试 `integration.rs:558-617` |
| FR-21-16 | 租户维度令牌桶限流, 按租户隔离 | P0 | ✅ | `rate_limiter.rs:37-83` (`TokenBucket`), `:86-131` (`TenantRateLimiter`, 每租户一桶, 互斥锁); 测试 `integration.rs:617-652` (`tenant_rate_limiter_isolation`) |
| FR-21-17 | 有界重试 (最大次数、指数退避、可重试判定) | P0 | ✅ | `retry.rs:38-102` (`RetryPolicy::run`, `delay`), `:104-116` (`ErrorKind`, `retryable_by_kind`); 测试 `integration.rs:653-725` |
| FR-21-18 | 幂等键、执行超时、取消 (SRS §49-§52) | P1 | ⬜ | Rust crate 中 `RetryPolicy` 无幂等键字段 (`retry.rs:38-42`); 未找到超时或取消执行路径 (`scheduler.rs` 仅 `release_hot`/`drop_low_priority`); 状态机有 Cancelled 终态 `agent_state.rs:146-148`, 但未接执行器 |
| FR-21-19 | 任务状态与死信: Failed 可重试, 超过上限进入 Dead | P0 | ✅ | `task.rs:16-27` (`TaskStatus` 含 Dead), `:86-94` (`mark_failed`); 测试 `integration.rs:726-748` (`task_mark_failed_retry_then_dead`) |
| FR-21-20 | Rust 侧死信队列 (DLQ) 持久化 | P1 | ⬜ | `crates/star-taskqueue/src` 中 grep `dead` / `dlq` 无结果 (Rust 侧仅 TaskState 6 态, `star-taskqueue/src/lib.rs:44-60`) |
| FR-21-21 | 事件总线: 按目标路由, 有界 mailbox, 取消订阅 | P0 | 🟡 | 运行时内 `event.rs:146-152` (`EventBus` trait), `:173-238` (`InMemoryEventBus`, 有界丢弃最旧 `:215-225`, 静默丢弃); 与 `star-eventbus` 不统一 (见 FR-21-22) |
| FR-21-22 | Mailbox 投递语义 at-most-once / at-least-once / exactly-once | P1 | 🟡 | `crates/star-eventbus/src/mailbox.rs:15-25` (`MailboxMode`), `:51-` (`AtMostOnceMailbox`), `:82-` (`AtLeastOnceMailbox`), `:132-` (`ExactlyOnceMailbox`); 该 crate 未被 `star-agent-runtime` 依赖 (grep 无结果) |
| FR-21-23 | 大 Payload 外置引用 (PayloadRef, 不把大消息放入组件) | P1 | 🟡 | `event.rs:44-52` (`PayloadRef` 含 store_id/key/size_bytes/ttl_seconds); 外部存储未实现 |
| FR-21-24 | L0 派发: 内存 Tokio 派发器, 状态 Pending→Dispatched→Running→Completed/Failed/Aborted | P0 | 🟡 | `crates/star-dispatcher/src/lib.rs:44-53` (`TaskState` 6 态), `:199-268` (`Dispatcher::dispatch`); 状态流转存在, 但派发路径不调用配额与超时, 且队列为内存实现 |
| FR-21-25 | L0 派发持久化: SQLite WAL 任务队列, FIFO 派发, 幂等索引 | P0 | 🟡 | `crates/star-taskqueue/src/lib.rs:146-170` (`SqliteTaskQueue::new` 设 `journal_mode=WAL`, `synchronous=NORMAL`), `crates/star-taskqueue/src/sqlite_backend.rs:12-38` (表 `task_queue` 与三个索引), `:66-134` (`dequeue_pending`: SELECT 后 UPDATE, 无显式事务), `:209-245` (`transition_state`); 无上层调用方 (见 §5) |
| FR-21-26 | 取回按状态列出、按 ID 查询、队列长度、清理已完成任务 | P1 | ✅ | `star-taskqueue/src/lib.rs:122-144` (trait `TaskQueue` 各方法), `sqlite_backend.rs:135-210` (`get_task`, `list_by_state`, `count_all`), `:246-257` (`cleanup_completed`) |
| FR-21-27 | 子代理池并发上限与按原型 (archetype) 注册 | P1 | 🟡 | `star-dispatcher/src/lib.rs:1082-1155` (`SubAgentPool::spawn` 先读后写, 非原子; 上限默认 50 `:1099`); `SubAgentArchetype` 9 类 `:287-325` (与 SA-01..SA-09 对应); 不实际执行子代理 |
| FR-21-28 | 检查点保存与按任务查询 | P1 | 🟡 | `star-dispatcher/src/lib.rs:883-951` (`CheckpointStore`, 进程内); `:1186-1205` (`TopAgent::dispatch_with_checkpoint`); 无持久化、无恢复入口 (SRS §53-§55 未实现) |
| FR-21-29 | 租户配额: 并发、排队数、每分钟任务数 | P1 | 🟡 | `star-dispatcher/src/lib.rs:520-610` (`TenantQuotaTracker::check`), `:611-665` (记录与计数); `Dispatcher::dispatch` 不调用 `check` (lib.rs:199-268); 独立 crate `star-quota` 的 `QuotaChecker::check` 有效 (`crates/star-quota/src/lib.rs:202-220`), 但 `reserve` 只做检查而不写入 `used` (`:222-237`), `used` 仅在 `:186` 的更新路径增加 |
| FR-21-30 | 共享资源池 (LLM/HTTP/MCP/RAG) 有上限获取与释放 | P1 | 🟡 | `star-dispatcher/src/lib.rs:667-745` (`SharedPool::acquire`/`release`/`check_available`); 仅计数资源可用性, 无真实连接池; 设计中的 LLM/HTTP/MCP/RAG Pool 独立实现未在 `crates/star-agent-runtime` 与 `star-dispatcher` 中找到 |
| FR-21-31 | Memory Store 短期/长期分层与按记录读写删 | P2 | 🟡 | `crates/star-memory/src/lib.rs:53-60` (`MemoryLayer`), `:73-88` (`Memory` trait), `:91-120` (`MemoryStore::write_smart` 按 weight 阈值分层); 内存态; REST 门面 `crates/star-api-rest/src/routes/memory.rs:90-100` (GET/POST `/`, GET `/agent/{agent_id}`, DELETE `/record/{record_id}`) |
| FR-21-32 | 上下文分层 L0 全局 / L1 子代理 / L2 共享池 | P2 | 🟡 | `crates/star-context-tiering/src/lib.rs:26-35` (`ContextTier`), `:92-` (`ContextTierContainer` trait), `:122-`, `:176-`, `:244-` (L0/L1/L2 结构); 容器为内存态, 无 lazy load 与压缩实现 |
| FR-21-33 | Token 预算与 Token 计量 (TokenBudget、按调用记录) | P2 | 🟡 | `star-agent-runtime/src/components.rs:151-175` (`TokenBudget` 字段与默认值, 未在任何调度路径中扣减); `crates/star-telemetry/src/meter.rs:48-112` (`TokenMeter::record`, `record_count`); `crates/star-telemetry/src/exporter.rs:20` (`PrometheusExporter`) |
| FR-21-34 | 运行时指标: 延迟百分位、token 计数、错误计数与快照 | P1 | 🟡 | `star-agent-runtime/src/metrics.rs:12-24` (`percentile`, `RuntimeMetrics`), `:90-106` (`Observer` trait), `:111-171` (`InMemoryObserver`); Python `scripts/automation/agent_runtime/metrics.py` 对应实现 |
| FR-21-35 | 可通过门面调用的运行时 API: 创建/删除/查询 Agent, 投递事件, 挂起/恢复/取消, 派发, tick, 指标 | P0 | ✅ | Python `scripts/automation/agent_runtime/runtime.py:86-201` (`create_agent` `:86`, `delete_agent` `:108`, `get_agent` `:116`, `get_agent_state` `:120`, `send_event` `:124`, `suspend_agent` `:131`, `resume_agent` `:135`, `cancel_agent` `:139`, `dispatch` `:149`, `tick` `:188`, `get_runtime_metrics` `:201`); 测试 `scripts/automation/agent_runtime/tests/test_runtime.py` (在 637ed19d 源码上运行 136 个用例全部通过) |
| FR-21-36 | Python 派发路径: 限流 → 反压 → 入队 → 出队调度 → 子进程执行 (守门 #24) | P0 | ✅ | `runtime.py:149-186` (dispatch 六步), `scripts/automation/agent_runtime/process_pool.py:52-145` (`ProcessPool`, `submit` `:68`, `dispatch_subagent` `:109`); `runtime.py:19-22` 为设计注释 |
| FR-21-37 | Python 侧死信队列与保留期 | P1 | ✅ | `scripts/automation/agent_runtime/task_queue.py:35-60` (表 `task_queue`), `:61-72` (`dead_letter_queue`), `:270-292` (`_to_dlq`), `:249-263` (`purge_expired`); 测试 `test_task_queue.py` |
| FR-21-38 | Python 侧 ECS 模式判定随 Agent 数量 tick 更新 | P0 | ✅ | `runtime.py:188-199` (`tick` 扫描 lifecycle 后调用 `mode_manager.decide`), `runtime_mode.py:72-101` (`decide`) |
| FR-21-39 | 配置 fail-safe: 非法配置拒绝, 不得无界 | P0 | 🟡 | Rust `config.rs:93-124` (`RuntimeConfig::validate` 对 `>0` 与超时非零做检查, 但 `RuntimeConfig` 仅被 `lib.rs:60` 重导出, 其字段未被任何运行时组件读取); Python `config.py:87-130` 已接入 `AgentRuntime.__init__` (`runtime.py:54-56`) |
| FR-21-40 | 安全: 不使用 unsafe, 不允许未声明依赖 | P0 | ✅ | `crates/star-agent-runtime/src/lib.rs:33` (`#![forbid(unsafe_code)]`), `Cargo.toml` 无第三方依赖 (`[dependencies]` 为空段, 第 18-19 行注释) |
| FR-21-41 | 多租户隔离: 派发、限流、公平份额按租户区分 | P0 | 🟡 | 调度与限流按 `TenantId` 区分 (`scheduler.rs:111-124`, `rate_limiter.rs:106-123`), 但 Rust 运行时不做 RLS/ACL 校验; 权限引用 `PermissionRef` 仅为 ID (`components.rs:140-147`) |
| FR-21-42 | 可观测性端点与前端展示 Agent 运行状态 | P1 | 🟡 | 页面 `frontend/src/app/(app)/agents/page.tsx` 与 REST `crates/star-api-rest/src/routes/agents.rs:83-97` 存在, 但数据为服务端内存样例 (见 §5 与 §4.7) |
| FR-21-43 | 1M 逻辑 Agent 与 100K WARM 目标的压测/基准 | P2 | ⬜ | `crates/star-agent-runtime`、`star-dispatcher`、`star-taskqueue` 的 Cargo.toml 均无 `[[bench]]` 目标; 基准脚本未找到 (grep `bench` 仅命中注释) |
| FR-21-44 | 部署: Worker 池水平扩展与 Shard | P2 | ❓ | 未在本功能 crate 中找到 K8s 清单; 需核对 `deploy/` 下运行时部署资产 (未核对) |

## 4. 设计

### 4.1 概念模型

运行时的核心实体是 Agent, 其定义 (SRS §8) 在代码中落为以下组成:

- `AgentIdentity` (`identity.rs:16`): 唯一标识、租户、agent_type。
- `AgentState` (`agent_state.rs:48`): 当前状态、前一状态、进入时间、重试计数。
- `LifecycleState` (`lifecycle.rs:39`): HOT/WARM/COLD、最近活跃时间、两个超时阈值。
- 引用组件 (`components.rs`): `ContextRef`、`MemoryRef`、`ModelRef`、`ToolPolicyRef`、`McpPolicyRef`、`PermissionRef`、`MailboxRef`、`WorkflowRef`、`TokenBudget`、`Priority`、`RateLimit`。
- `Task` (`task.rs:31`): 派发单元, 含 brief、agent_type、tenant、priority、状态、重试计数与 trace 标识。

与之并列的是运行时的调度器与资源控制器: `Scheduler`、`RuntimeModeManager`、`Backpressure`、`TenantRateLimiter`、`RetryPolicy`、`InMemoryEventBus`、`LifecycleManager`。

L0 派发 (`star-dispatcher`, `star-taskqueue`) 与 Python 门面是两套并存的派发实现, 二者概念对应但未互相引用 (见 §6)。

### 4.2 数据模型

Rust 进程内组件 (`crates/star-agent-runtime`):

| 结构 | 字段 (摘要) | 位置 |
|---|---|---|
| `AgentIdentity` | agent_id, tenant_id, agent_type | `identity.rs:16-25` |
| `AgentState` | current, prev, since, retry_count | `agent_state.rs:48-57` |
| `LifecycleState` | current, last_active, hot_timeout, warm_timeout | `lifecycle.rs:39-47` |
| `Task` | task_id, brief, agent_type, tenant_id, priority, status, retry_count, max_retry, payload, created_at, trace_id, correlation_id, operation_id | `task.rs:31-46` |
| `AgentEvent` | event_id, source/target agent, tenant, event_type, payload_ref, timestamp, trace_id, correlation_id | `event.rs:57-68` |
| `RuntimeConfig` | runtime_mode, max_hot_agents, 各类请求上限, max_queue_size, overflow_policy, 超时, max_retry, mailbox_max_size, warm 内存预算 | `config.rs:35-52` |

SQLite 表 (`star-taskqueue`): `task_queue(task_id PK, tenant_id, kind, payload, idempotency_key, created_at_ms, state, state_history, attempt)`, 索引 `(state, created_at_ms)`、`(tenant_id)`、`(tenant_id, idempotency_key)` (`sqlite_backend.rs:15-35`)。`compaction_entry` 表在 `compaction.rs:712` 另行定义。

Python 表 (`scripts/automation/agent_runtime/task_queue.py`): `task_queue` 含 status 五态 `pending/running/done/failed/dead`、retry_count、max_retry、retention_until (`:35-58`); `dead_letter_queue` 含 original_task、failure_reason、first/last_failed_at、retention_until (`:61-72`)。

内存态 DB 无 RLS 或分区定义; 上述表均未经过 W/T/M 三类分门别类 (守门 #13), 需在后续 DDL 评审中补充。

### 4.3 接口 (routes, APIs, ports)

Rust 公开接口 (`crates/star-agent-runtime/src/lib.rs:36-77` 导出):

- 状态: `AgentState::transition/cancel/suspend/resume/fail/retry` (`agent_state.rs:130-165`)。
- 生命周期: `LifecycleState::activate/touch/sweep/restore` (`lifecycle.rs:80-115`); trait `LifecycleManager` (`:138-153`) 与内存实现 `InMemoryLifecycleManager` (`:157-255`)。
- ECS: `World::spawn_agent/despawn/insert_component/get_*/lifecycle_counts/agent_ids` (`ecs.rs:71-172`)。
- 模式: `RuntimeModeManager::new/decide/get_mode/switch_count/hysteresis_zone` (`runtime_mode.rs:103-189`)。
- 调度: `Scheduler::new/enqueue_ready/schedule/release_hot/drop_low_priority/drain_deferred/...` (`scheduler.rs:90-224`)。
- 反压: `Backpressure::new/with_policy/has_slot/await_slot` (`backpressure.rs:98-145`)。
- 限流: `TenantRateLimiter::new/acquire/release/bucket_count` (`rate_limiter.rs:92-131`)。
- 重试: `RetryPolicy::new/delay/run` (`retry.rs:54-102`)。
- 事件: `EventBus` trait (`event.rs:146-152`), `EventRouter` (`:156-160`), `InMemoryEventBus` (`:173-238`)。

L0 接口:

- `star-dispatcher`: `Dispatcher::submit/dispatch/close` (`lib.rs:199-272`); `AgentTaskExecutor` trait (`:90-94`); `SubAgentPool::register/spawn/complete/active_count` (`:1082-1156`); `TopAgent::dispatch_with_checkpoint` (`:1186-1205`); `TenantQuotaTracker::register/check/record_dispatch/record_complete` (`:520-665`); `SharedPool::register/acquire/release` (`:683-745`); `MemoryStore`、`ContextStore`、`CheckpointStore` (进程内)。
- `star-taskqueue`: trait `TaskQueue` 八个异步方法 (`lib.rs:122-144`); 构造 `SqliteTaskQueue::new(path)` 与 `in_memory()` (`:146-170`)。
- 跨仓 proto: `crates/star-dispatcher/proto/langgraph_cross_repo.proto` 与 `cross_repo.rs:18-212` (`PhysisServer` 与 `CrossRepoClient` 的进程内模拟); 不是运行中的网络服务。

Python 门面 (`scripts/automation/agent_runtime/runtime.py`): `AgentRuntime` 的十余个方法列于 §3 FR-21-35。

HTTP 端点 (`crates/star-api-rest`):

| 方法 | 路径 | 处理器 | 数据来源 | 位置 |
|---|---|---|---|---|
| GET | `/api/v1/agents` | `list_agents` | 进程内三条样例 | `routes/agents.rs:29-82` (`sample_agents`), `:83-92`, 路由 `:95-97` |
| GET/POST | `/api/v1/memory` | memory router | star-memory 内存态 | `routes/memory.rs:90-100` |
| GET | `/api/v1/memory/agent/{agent_id}` | memory router | 同上 | `routes/memory.rs:98` |
| DELETE | `/api/v1/memory/record/{record_id}` | memory router | 同上 | `routes/memory.rs:99` |

路由挂载: `crates/star-api-rest/src/lib.rs:169-170` (`/api/v1/agents`, `/api/v1/runtime`), `:168` (`/api/v1/memory`)。

没有 HTTP 端点用于派发任务、切换模式、查询生命周期或取消执行。

### 4.4 状态机与关键流程

Agent 状态机 (`agent_state.rs:79-127`, 12 状态):

```
Idle -> Ready -> Scheduled -> Planning -> {WaitingLlm | WaitingTool | WaitingEvent} -> Processing -> Completed -> Idle
Failed -> Ready (retry)        Suspended -> Scheduled (resume)
{Idle..Processing} -> Failed | Suspended ;  {Idle..Failed, Suspended} -> Cancelled (terminal)
```

Lifecycle 状态 (`lifecycle.rs:91-115`): `Hot -> Warm` 当空闲超过 hot_timeout; `Warm -> Cold` 当空闲超过 warm_timeout; `Cold -> Warm` 由 restore 触发。`InMemoryLifecycleManager::transition` 的合法集合为 Hot→Warm、Warm→Hot、Warm→Cold、Cold→Warm (`lifecycle.rs:206-213`), 不允许 Hot→Cold 直接降级。

Runtime 模式 (`runtime_mode.rs:139-182`): Lightweight 在 count ≥ 12 时开始计时, 持续 ≥ 30s 切入 Ecs; Ecs 在 count ≤ max(8, 9) 时开始计时, 持续 ≥ 300s 切回 Lightweight; 10-11 保持当前模式。

派发流程 (Python, `runtime.py:149-186`): rate_limiter.acquire → backpressure.await_slot (Delay 则 sleep) → queue.enqueue → queue.dequeue(limit=max_hot_agents) → scheduler.schedule → 未调度者 requeue → (execute=True 时) ProcessPool.submit → observer.inc_dispatched。

派发流程 (Rust `star-dispatcher`, `lib.rs:223-267`): Pending→Dispatched→Running→执行器 `execute` → Completed / Aborted / Failed。执行器无超时包装; `Dispatcher::dispatch` 不调用 `TenantQuotaTracker::check`。

失败与死信 (Rust, `task.rs:86-94`): `mark_failed(retry)` 递增 retry_count, 若 retry 且未达 max_retry 则回到 Pending, 否则进入 Dead。Rust 侧没有把 Dead 任务写入任何死信表。

### 4.5 权限与安全

- 0 unsafe: `lib.rs:33` 的 `#![forbid(unsafe_code)]`。
- 无第三方依赖的 Rust 内核 (`star-agent-runtime/Cargo.toml` 的 `[dependencies]` 为空); `star-taskqueue` 与 `star-dispatcher` 依赖 tokio、rusqlite 等 (见各自 Cargo.toml)。
- 租户隔离只在调度、限流、公平份额的键上体现 (`TenantId`); 未见 RLS 或 ACL 校验。`PermissionRef` 仅为 ID 包装 (`components.rs:140-147`)。
- 派发与执行未做工具或 MCP 权限校验 (`ToolPolicyRef`、`McpPolicyRef` 仅为 ID)。
- Python 子进程执行 (`process_pool.py:68-97`) 直接运行调用方提供的命令列表; 命令白名单、工作目录约束与超时参数的核对仍需在安全评审中完成。
- Secret 与凭证未在运行时中出现 (未找到相关字段)。

### 4.6 依赖关系

- `star-agent-runtime`: 无外部依赖; 仅依赖 `star-agent-runtime` 内部模块。
- `star-dispatcher`: serde、serde_json、uuid、tokio、thiserror、async-trait (`Cargo.toml`)。
- `star-taskqueue`: 同上加 rusqlite (`bundled`)。
- `star-api-rest`: 依赖 `star-memory` (`Cargo.toml:108`) 与 `agent-domain` (`routes/agents.rs:23`); 不依赖 `star-agent-runtime`。
- `star-agent-runtime` 与 `star-dispatcher`/`star-taskqueue` 之间没有 Cargo 依赖关系。
- `star-cli` 的技能注册表中出现字符串 `star-agent-runtime` (`crates/star-cli/src/skill_registry.rs:168,398`), 用作 adapter 标识而非 crate 依赖。
- Python `agent_runtime` 与 `scripts/automation/` 其他脚本的交互通过子进程完成 (守门 #24)。

### 4.7 页面设计

#### 4.7.1 /agents

文件: `frontend/src/app/(app)/agents/page.tsx` (557 行)。

布局: 由 `frontend/src/app/(app)/layout.tsx:20-49` 提供 `AppShell`; `AppShell` 挂载 `AppHeader` (`frontend/src/components/AppShell.tsx:49`)、主区 `<main>` (`:61`) 与右侧聊天栏 `RightChatRail` (`:77`)。本页面不渲染自己的侧栏, 底栏聊天不在本页面范围内。

组件树 (页面内顶层):

- `AgentsPage` (`page.tsx:81`)
  - 状态提示 `StatusNotice` (`page.tsx:268-269`, 来自 `frontend/src/components/status/StatusNotice`)
  - `PageHeader`、`Stat` (`page.tsx:217-236`, 来自 `@/components/PageHeader`)
  - `CelCard3D` 包装的四个统计卡 (`page.tsx:226-236`)
  - 花名册表格 `<table>` (`page.tsx:282-294`)
  - 数据说明 `agents-data-note` (`page.tsx:257-259`)
  - Argo CLI 不可用提示 `agents-argo-unavailable` (`page.tsx:213-214`)
  - Comms 占位区 `live-activity-placeholder` (`page.tsx:356-361`)
  - 动作区 (`page.tsx:517-531`) 以及动作结果 `agents-action-result` (`page.tsx:545`)

状态矩阵:

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 加载 | `phase === "loading"` (`page.tsx:185-186`) | `StatusNotice state="loading"` | ✅ 实现 |
| 正常 | `phase === "ready"` 且 rows 非空 | 统计卡 + 表格 | 🟡 数据为服务端样例 (三条), 见 §5 |
| 空 | `rows.length === 0` 且非错误 (`page.tsx:196-197`) | `StatusNotice state="empty"` | ✅ 实现 |
| 错误 | `phase === "error"` (`page.tsx:187-195`) | 按错误归类为 unauthorized / disconnected / failed; 重试按钮 `agents-retry` (`:270-274`) | ✅ 实现 |
| 未授权 | `AgentsApiError` code 为 unauthorized | `StatusNotice state="unauthorized"` (`page.tsx:190-191`) | ✅ 实现 (依赖后端返回 401/403) |
| 未接入 | Argo CLI 列表请求失败 | `agents-argo-unavailable` 显示 disconnected (`page.tsx:213-214`) | ✅ 实现 |
| 禁用 | `actionsReachable = phase === "ready"` 为假 (`page.tsx:200`) | 动作按钮 `disabled` 与 `agents-actions-disabled-hint` (`page.tsx:521,530,537`) | ✅ 实现 |

交互:

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 重试加载 (`agents-retry`) | 错误态 | 再次调用 `loadAgents` (`page.tsx:274`) | GET `/api/v1/agents` (`agentsApi.ts:144`) | ✅ 实现 |
| SYNC | phase 为 ready 且非忙碌 (`page.tsx:521`) | 重新拉取花名册, 显示 `syncResult` 计数 (`runSync`) | 同上 | ✅ 实现; 数据仍为样例 |
| VERIFY | 同上 (`page.tsx:530`) | ping 健康检查 | GET `/api/v1/health` (`agentsApi.ts:171-178`) | ✅ 实现 |
| 切换 AutoSagaGuard、IdempotentLock | 任意 | 仅改变前端本地状态 (`page.tsx:101-102,328,337`) | 无 | 🟡 未与后端行为关联 |
| 切换 CEL 调色板与色带数 | 任意 | 仅前端本地状态 (`page.tsx:484,503`) | 无 | 🟡 装饰性 |
| 调度模式拨盘 `dispatchMode` | 任意 | 仅前端本地状态 (`page.tsx:103,343-345`) | 无 | 🟡 未调用派发接口 |

数据: 唯一真实数据源为 `listAgents()` (`frontend/src/lib/agents/agentsApi.ts:144`), 请求 `GET /api/v1/agents`, 返回 `{ total, agents[], note }`; 页面使用其中的 agents 与 note (`page.tsx:85-88`)。`note` 在服务端固定为 "内存样例, 宿主运行时接入后替换 (无 DB)" (`routes/agents.rs:90`)。Argo CLI 数据来自 `/api/v1/argo/nodes`。

与文档设计的差距:

- FR-21-42 要求展示真实运行状态; 当前服务端返回三条样例 Agent (`routes/agents.rs:29-82`), 页面 "正常" 状态实际展示的是样例数据。
- FR-21-35 中的派发与生命周期操作没有对应的前端入口或 REST 端点。
- SRS §8 描述的 Agent 属性 (ModelRef、TokenBudget、Priority、LifecycleState) 在页面中未展示。
- 调度模式拨盘、AutoSagaGuard、IdempotentLock 未与后端行为关联 (FR-21-12, FR-21-18)。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| Rust 运行时内核 (Agent 模型、状态机、生命周期、模式、调度、反压、限流、重试、事件、配置、指标) | `crates/star-agent-runtime/src/` | 🟡 | `lib.rs:36-77`; 52 个集成测试在 637ed19d 源码上全部通过 (`tests/integration.rs`) |
| Rust ECS 占位存储 | `crates/star-agent-runtime/src/ecs.rs` | 🟡 | `ecs.rs:54-172` |
| Python 运行时门面 | `scripts/automation/agent_runtime/runtime.py` | ✅ | `runtime.py:51-229`; 136 个单元测试在 637ed19d 源码上全部通过 |
| Python 任务队列与死信 | `scripts/automation/agent_runtime/task_queue.py` | ✅ | `task_queue.py:35-292`, `tests/test_task_queue.py` |
| Python 子进程池 | `scripts/automation/agent_runtime/process_pool.py` | ✅ | `process_pool.py:52-145` |
| Python 模式、调度、反压、限流、重试、事件、指标 | `scripts/automation/agent_runtime/*.py` | ✅ | 文件存在且被 `runtime.py:19-35` 导入; 对应测试文件在 `tests/` |
| L0 内存派发器与子代理池 | `crates/star-dispatcher/src/lib.rs` | 🟡 | `lib.rs:199-272`, `:1082-1206`; 测试 31 个 (`#[test]`/`#[tokio::test]` 计数, 未运行) |
| L0 SAs 实现 | `crates/star-dispatcher/src/sa_real_impls.rs` | 🟡 | `sa_real_impls.rs:20-272` (CodeReview、TestGen、DocSync、Refactor、DbMigration、DomainDev 等); 测试 12 个 (未运行) |
| 跨仓 proto 与进程内模拟 | `crates/star-dispatcher/src/cross_repo.rs`, `proto/langgraph_cross_repo.proto` | 🟡 | `cross_repo.rs:18-248`; 无网络服务 |
| SQLite WAL 任务队列 | `crates/star-taskqueue/src/` | 🟡 | `lib.rs:122-170`, `sqlite_backend.rs:12-257`, `compaction.rs:712`; 测试 2 个集成 + 4 个单元 (未运行); 无上层调用方 |
| 租户配额 (独立 crate) | `crates/star-quota/src/` | 🟡 | `lib.rs:90-237`; `checker.rs` 含 4 个测试 (未运行) |
| 记忆存储 | `crates/star-memory/src/lib.rs` | 🟡 | `lib.rs:53-120`; `routes/memory.rs:90-100` |
| 上下文分层 | `crates/star-context-tiering/src/lib.rs` | 🟡 | `lib.rs:26-260` |
| 事件总线与 Mailbox 三模式 | `crates/star-eventbus/src/` | 🟡 | `mailbox.rs:15-138`; 与 `star-agent-runtime` 无依赖 |
| Token 计量与 Prometheus 导出器 | `crates/star-telemetry/src/` | 🟡 | `meter.rs:48-112`, `exporter.rs:20` |
| Agent 列表 REST 端点 | `crates/star-api-rest/src/routes/agents.rs` | 🟡 | `routes/agents.rs:29-97` (样例数据) |
| Agent 页面 | `frontend/src/app/(app)/agents/page.tsx` | 🟡 | `page.tsx:81-557`; 数据来自样例 |
| 14 状态 Agent 模型 (另一套状态机) | `crates/agent-domain/src/models/agent.rs:156-` | 🟡 | 与 12 状态 `AgentStateEnum` 的状态名不一致 (见 §8 裁决) |
| 运行时部署资产 | 未找到 | ❓ | 未找到 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| Rust 运行时与 Python 运行时是两套并行实现, 无共享契约 | 行为可能分叉, 派发语义与守门要求不一致 | 确定 Rust 为执行内核、Python 为 LangGraph/守门侧的适配层, 并以共享测试向量覆盖状态机与模式切换 |
| `star-taskqueue`、`star-dispatcher`、`star-quota`、`star-memory`、`star-context-tiering`、`star-telemetry` 无上层调用方 (FR-21-24/25/29/31/32/33) | 基础设施存在但不能被运行时或 REST 使用, 不构成可用闭环 | 在 `star-api-rest` 或派发入口接入实际调用, 并补充端到端测试 |
| 小于 10 个 Agent 禁止 ECS 没有代码下限 (FR-21-09) | 配置可把 ECS 模式开启在 5 个 Agent, 违反 SRS §3.2 | 在 `RuntimeModeConfig::validate` 增加 `ecs_enable_threshold >= 10` 检查, 并补充测试 |
| `RuntimeModeConfig::validate` 对 `ecs_enable_threshold` 为 1 或 2 时做 `- 3` 运算会在 debug 下溢出 panic (`runtime_mode.rs:81`; 以 `enable=2` 探针复现 "attempt to subtract with overflow") | 非法配置导致 panic 而非返回错误 | 在做减法前先检查阈值下界, 改为返回 `RuntimeModeError` |
| ECS 无 System 调度与查询 (FR-21-10/11) | 无法批量处理 Agent 状态, ECS 模式下仍走同一套逐个处理路径 | 选型 bevy_ecs 或 flecs (G-2), 或在现有 World 上补 System 调度 |
| Backpressure 的 `Persist` 与 `DropLow` 只返回决策 (FR-21-15) | 反压策略未真正生效 | 将 Persist 接到 `star-taskqueue`, 将 DropLow 接到 `Scheduler::drop_low_priority` |
| 事件总线静默丢弃最旧消息 (FR-21-21) | 丢失事件无感知 | 改用 `star-eventbus` 的 Mailbox 模式并让丢弃计入指标 |
| 派发与执行无超时、无取消 (FR-21-18) | 失控任务占用 HOT 槽位 | 在 `Dispatcher::dispatch` 增加 tokio 超时与取消令牌, 并将 Cancelled 状态接入 |
| 配额未接入派发路径 (FR-21-29) | 租户超额派发不会被拒绝 | 在 `Dispatcher::dispatch` 调用 `TenantQuotaTracker::check`, 并让 `QuotaChecker::reserve` 写入 `used` |
| `SubAgentPool::spawn` 为 check-then-act 非原子 (FR-21-27) | 并发时可能超过 `max_parallel` | 在同一写锁内完成检查与递增 |
| SQLite 派发 SELECT 后 UPDATE 无事务 (FR-21-25) | 多进程消费者可能重复领取同一任务 | 使用 `BEGIN IMMEDIATE` 或单条原子 UPDATE ... RETURNING |
| Rust 侧无 DLQ (FR-21-20) | Dead 任务不可查询与重放 | 为 `star-taskqueue` 增加 dead-letter 表与接口, 对齐 Python 侧 `dead_letter_queue` |
| 持久化与崩溃恢复未实现 (FR-21-06, FR-21-28) | 进程重启后 Agent 与检查点丢失 | 为 LifecycleManager.persist/restore 接入 SQLite 实现, 并补恢复测试 |
| `/api/v1/agents` 返回样例数据 (FR-21-42) | 页面 "正常" 状态展示的是样例 | 接入 agent-domain 或运行时的真实查询, 样例仅留作测试夹具 |
| Agent 状态存在两套定义 (12 状态 vs 14 状态) | 同名概念语义不一致, 跨层映射困难 | 确定以 `AgentStateEnum` 为运行时状态, 为 agent-domain 提供显式映射 |
| 共享 LLM/HTTP/MCP/RAG 池未实现 (FR-21-30) | 无法限制外部资源并发 | 按 SRS §29-§35 建立共享池, 复用 `SharedPool` 的接口形状 |
| 无基准与 1M 规模验证 (FR-21-43) | 性能目标无证据 | 增加 criterion 或同类基准, 覆盖 1/10/100/1K 规模 |
| TokenBudget 未扣减 (FR-21-33) | 预算字段形同虚设 | 在派发或 LLM 调用路径接入 `TokenMeter::record` 与预算扣减 |
| DDL 未做 W/T/M 分类 (守门 #13) | 表的保留期与审计要求未落地 | 按守门 #13 补充分类与 RLS 声明后再生成迁移 |

## 7. 验收与测试

已运行的测试 (源码取自 637ed19d, 在隔离副本中执行):

- `crates/star-agent-runtime/tests/integration.rs`: 52 个 `#[test]`, 全部通过 (`cargo test --offline`, 结果 `52 passed; 0 failed`)。覆盖 ID 解析、AgentType、状态机、生命周期、ECS spawn/despawn、事件总线与路由、模式切换与迟滞、调度与公平份额、反压、令牌桶、租户限流、重试、Task 死信、指标分位数、配置 fail-safe。
- `scripts/automation/agent_runtime/tests/`: 136 个用例, 全部通过 (`python -I -m pytest agent_runtime/tests`, 结果 `136 passed`)。覆盖反压、配置、事件、模型、进程池、限流、重试、运行时门面、模式、调度、状态机、任务队列、任务图。

存在但未执行的测试 (仅统计 `#[test]` 数量):

- `crates/star-dispatcher/src/lib.rs` 内 31 个测试, `sa_real_tests.rs` 12 个, `cross_repo_tests.rs` 4 个。
- `crates/star-taskqueue/src/lib.rs` 4 个, `compaction.rs` 8 个, `tests/sqlite_persistence_test.rs` 2 个。
- `crates/star-quota/src/lib.rs` 4 个, `checker.rs` 4 个, `tests/rls_13_test.rs` 3 个。
- `crates/star-memory` 3 个 + `tests/memory_layering_test.rs` 3 个; `crates/star-context-tiering` 4 个 + `tests/tiering_3layers_test.rs` 4 个; `crates/star-eventbus` 多个; `crates/star-telemetry` 4 个。
- 这些 crate 依赖工作区配置, 未执行。

探针测试 (隔离副本, 未写入仓库): `RuntimeModeConfig` 在 `ecs_enable_threshold=2` 时 `validate` 触发减法下溢 panic (`runtime_mode.rs:81`); `ecs_enable_threshold=5` 时 `validate` 返回 `Ok(())`。

已验证的内容: 状态机合法与非法转换、模式切换阈值与稳定时长、调度优先级与 HOT 限制、公平份额、租户限流隔离、重试次数与退避、任务死信转换、Python 派发流程与子进程池、Python 死信与保留期。

未验证的内容: 任何端到端派发 (REST 到执行); SQLite 队列在多进程下的并发正确性; 持久化与恢复; 基准与 1M 规模; `/agents` 页面的交互测试 (仅有 `frontend/src/app/(app)/agents/__tests__/agents.test.tsx` 存在, 未运行); 与 `/api/v1/agents` 真实数据对接。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` (1443 行, 含前置的 STAR 映射章节 §0-§7 与正式 §1-§110) | REQ-ECS-001..012、REQ-LC、REQ-SCH、REQ-SHARED、REQ-CTX、REQ-MEM、REQ-RECOVERY 等需求条目 | 前置章节 §1 与 §3 中自报的 "已落地/部分落地" 状态与代码不符 (如声称 "12 节已落地"); 其中 "1M 派发 87 小时" 等数字无基准支撑 | 需求条目保留为 FR-21 的来源; 状态一律以代码为准; 自报状态不采用 |
| `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` (652 行) | 三层架构 (L0/L1/L2)、双模式、组件清单、9 Systems、与 22 domain-* 的映射 | 映射表中的 `domain-dispatcher`、`domain-rate-limiter` 等 "新增" crate 名称在代码中不存在; 实际运行时组件位于 `star-dispatcher`、`star-taskqueue`、`star-quota` 等; `domain-llm`、`domain-mcp`、`domain-tool` 目录存在但其与 Shared Pool 的对应关系未核对 | 组件职责采用; crate 名称以代码为准 (§4.6); 未核对的 domain-* 标 ❓ |
| `docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` (1126 行) | 状态机与生命周期图、L0 调度算法、Backpressure 与 HOT 槽位算法、SQLite 表结构 (§5.1-5.5)、测试设计 | 详细设计中的 SQLite 表与代码 `task_queue` 字段不一致 (代码无 `task_kind` 等列); DLQ 表 §5.2 在 Rust 侧不存在 | 状态机与算法采用; 表结构以 `sqlite_backend.rs:15-35` 为准, 差异列入 §6 |
| `docs/architecture/2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md` | ADR 记录 SRS 基线接受 | 文档自称 "Accepted v1.0", 与代码覆盖度不一致 | 作为决策记录引用; 不作为现状证据 |
| `docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` | ADR 记录基本与详细设计基线 | 同上 | 同上 |
| `docs/specs/domain-agent-spec.md` (360 行) | domain-agent 的职责、实体、接口与验收 | 属于 Agent 领域 (`crates/domain-agent`, `crates/agent-domain`), 与运行时内核职责不同; 其 AgentSession 等概念未在 `star-agent-runtime` 中出现 | 仅在 §5 标注 agent-domain 14 状态模型; 其余内容归属其他功能 |
| `crates/star-agent-runtime/` (源码与测试) | 现状证据 | 与设计文档的若干条目不一致 (如 SRS §3.2 下限缺失) | 代码为现状裁决依据 |
| `crates/star-dispatcher/`、`crates/star-taskqueue/`、`crates/star-quota/`、`crates/star-memory/`、`crates/star-context-tiering/`、`crates/star-eventbus/`、`crates/star-telemetry/` | L0/L2 组件现状 | 多数无上层调用方 | 作为 FR-21 的 🟡 证据 |
| `scripts/automation/agent_runtime/` (Python 运行时) | 第二套运行时现状 | Rust crate 注释声明其为 "双模式" 的另一半, 但两者无共享契约 | 作为 FR-21-35..38 的 ✅ 证据; 差异列入 §6 |
| `crates/star-api-rest/src/routes/agents.rs`、`routes/memory.rs`、`routes/mod.rs`、`lib.rs` | REST 现状 | `/api/v1/agents` 为样例数据 | 作为 FR-21-42 与 §4.3 证据 |
| `frontend/src/app/(app)/agents/page.tsx`、`frontend/src/lib/agents/agentsApi.ts`、`frontend/src/components/AppShell.tsx`、`frontend/src/app/(app)/layout.tsx`、`frontend/next.config.js` | 页面与数据路径 | 页面 "正常" 状态展示样例数据; 多个控件为装饰性 | 作为 §4.7 证据 |
| `docs/arch-live/04-agent-runtime.html` | 另一份 "Agent Runtime" 页面 (约束、状态机与失败分类、Hook 判定、策略继承、Local Runtime 心跳) | 其主题是 Agent 会话与 Hook 判定, 与本功能的运行时内核不同 | 作为范围外来源记录, 不纳入 FR-21 |
| `docs/architecture/2026-09-03-langgraph/` (01/02/03) | 9 SA 类型与 L0/L1 设计 | 与本功能为平行视图, 文档声明不重写 SA 细节 | 范围外; 仅引用 SA 类型名 |
| `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` 与 TMO 相关文档 | 任务卡管理操作 | 属于 LangGraph/TMO 功能 | 范围外 |
| `scripts/automation/registry.md` 与 `docs/automation-design.md` | 守门与脚本索引 | 与运行时的守门关联 (守门 #13、#24) | 仅作守门背景引用 |
| 根 `Cargo.toml` 的 workspace 注释 (WBS 6.2 注册 star-agent-runtime, G.1 注册 star-dispatcher 与 star-taskqueue) | 注册时间线 | 注释中的 "hecs 选型" (`Cargo.toml:279-280`) 与 star-agent-runtime 的 "自研 Minimal ECS" 说明 (`lib.rs:21-22`) 不一致 | 以 star-agent-runtime 源码与 lib.rs 注释为准; hecs 在 workspace 中的使用未核对 (❓) |

本功能未作为唯一证据使用的内容: `docs/arch-live/` 其他页面、`docs/WBS.md` 中的任务项与 AGENTS 的历史报告 (仅用于定位, 不作为现状证据)。
