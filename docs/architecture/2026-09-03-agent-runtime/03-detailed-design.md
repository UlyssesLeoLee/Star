# 03. STAR Agent Runtime - 詳細設計書 (Detailed Design)

> **状態**：🟡 Draft v0.1
> **日期**：2026-09-03
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权）
> **依赖**：[`SRS-001`](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md)（要件定義書）· [`02-basic-design.md`](02-basic-design.md)（基本設計書）· [ADR-0044](../2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md) · [ADR-0045](../2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md)（本 view 决策）· [`AGENTS.md` §4 守门](../../AGENTS.md)
> **关联文档**：[`SRS-001`](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) · [`02-basic-design.md`](02-basic-design.md)

> **本 view 范围 (per [02 §0](02-basic-design.md))**: 本詳細設計書涵盖的是 **STAR Agent Runtime** 系统的 **実装レベル詳細設計** — module 划分 / 类 / 状态机 / 时序图 / 算法 / 持久化 / 测试. **不涵盖** LangGraph 任务卡子代理 (引用 [LangGraph 9/3 03](../2026-09-03-langgraph/03-detailed-design.md), 不重写).

> **dual-use 提醒 (per AGENTS.md §5)**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射.

---

## 0. 目的 (Purpose)

本文档基于 [02-basic-design.md](02-basic-design.md) 的 3 层架构 + Runtime 双模式 + 9 SA Type 引用, 落地为実装レベル詳細設計:

- 模块设计 (M-01..M-15 责任 / 接口 / 依赖)
- 类设计 (key types 完整 Rust 草案)
- 状态机 (HOT/WARM/COLD 状态转换)
- 时序图 (UC-01..UC-10: dispatch / schedule / recover / migrate / persist / restore / cancel)
- 数据结构 (ECS Component / Event / Mailbox 详细)
- 算法 (调度 / 反压 / 速率 / 限流)
- 错误处理 (Retry 策略 + 异常分类)
- 持久化 (SQLite WAL + 表 schema)
- 测试设计 (UT/IT/E2E/PT 跟 SRS §63-§71 对齐)

## 1. モジュール設計 (Module Design)

### 1.1 モジュール構成図 (Module Structure)

```
crates/                                # STAR Rust workspace (per 22 domain-* crate)
├── domain-agent/                      # 🆕 新建 (L1 ECS Agent 核心)
│   ├── src/
│   │   ├── lib.rs                     # 模块入口
│   │   ├── identity.rs                # AgentIdentity
│   │   ├── state.rs                   # AgentState + 状态机
│   │   ├── lifecycle.rs               # LifecycleState (HOT/WARM/COLD)
│   │   ├── components.rs              # ContextRef / MemoryRef / ModelRef / ...
│   │   ├── scheduler.rs               # Scheduler System
│   │   ├── lifecycle_sys.rs           # Lifecycle System
│   │   └── metrics.rs                 # Metrics System
│   └── Cargo.toml
│
├── domain-dispatcher/                 # 🆕 新建 (L0 派发)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── queue.rs                   # SQLite WAL TaskQueue
│   │   ├── dispatcher.rs              # Tokio async dispatcher
│   │   ├── pool.rs                    # ProcessPool (8-16 worker)
│   │   ├── rate_limiter.rs            # RateLimiter (token bucket)
│   │   ├── backpressure.rs            # Backpressure
│   │   ├── retry.rs                   # RetryPolicy
│   │   ├── observer.rs                # Observer (metrics + trace)
│   │   └── runtime_mode.rs            # RuntimeModeManager (Lightweight / ECS 切换)
│   └── Cargo.toml
│
├── domain-llm/                        # 🆕 新建 (L2 LLM Pool, P3-C)
├── domain-mcp/                        # 🆕 新建 (L2 MCP Pool, P3-C)
├── domain-tool/                       # 🆕 新建 (L2 Tool Registry, P3-C)
├── domain-rag/                        # 🆕 新建 (L2 RAG Pool, P3-E)
├── domain-context/                    # 🆕 新建 (L2 Context Store, P3-D)
├── domain-memory/                     # 🆕 新建 (L2 Memory Store, P3-D)
├── domain-rate-limiter/               # 🆕 新建 (P3-B)
├── domain-observability/              # 🆕 新建 (P3-B)
│
├── domain-task/                       # 现有 (L0 任务派发部分)
├── domain-identity/                   # 现有 (L1 AgentIdentity + Permission 部分)
├── domain-permission/                 # 现有 (PermissionRef 部分)
├── domain-work-item/                  # 现有 (L0 任务派发 + L1 状态机部分)
├── domain-workspace/                  # 现有 (Tenant 隔离)
├── domain-worktree/                   # 现有 (per-task worktree, 守门 #9)
│
└── ... (其他现有 22 domain-* crate)
```

### 1.2 モジュール責任 / 依存関係

| Module | 责任 | 依赖 (Cargo.toml) | 守门 |
|---|---|---|---|
| **domain-agent** | L1 ECS Agent 核心 (Components + Systems) | bevy_ecs, tokio, uuid, serde | #7 0 unsafe |
| **domain-dispatcher** | L0 派发 (Queue + Dispatcher + Pool + RateLimiter + Backpressure + Retry) | tokio, rusqlite, serde, tracing | #7 #24 |
| **domain-llm** | L2 LLM Pool (Provider + Model + Tokenizer) | reqwest, serde, anyhow | #5 #7 |
| **domain-mcp** | L2 MCP Pool (Registry + Connection) | tokio, serde, reqwest | #5 #7 |
| **domain-tool** | L2 Tool Registry (Definition + Schema + Executor + Permission) | serde_json, async-trait | #7 |
| **domain-rag** | L2 RAG Pool (Retriever + Cache + Vector) | reqwest, serde | #5 #7 |
| **domain-context** | L2 Context Store (L1/L2/L3 Tier + Lazy Load) | tokio, serde | #5 #7 |
| **domain-memory** | L2 Memory Store (S/E/U/W/K + Embedding) | tokio, serde, qdrant-client | #5 #7 |
| **domain-rate-limiter** | Token Bucket + Tenant Quota | tokio, dashmap | #7 |
| **domain-observability** | Metrics + Trace (Prometheus + OpenTelemetry) | prometheus, opentelemetry, tracing | #7 |

**新建 9 个 domain-* crate, 总计 22 + 9 = 31 domain-* crate 目标** (per 02 §3.5).

### 1.3 ECS 框架選型 (per SRS §91)

候选: **bevy_ecs** (成熟, 文档全) / **flecs** (C/Rust 双绑, 快) / **自研 Minimal ECS** (极简).

| 维度 | bevy_ecs | flecs | 自研 |
|---|---|---|---|
| **Memory Overhead** | 中 (Archetype-based) | 低 (Sparse set) | 极低 |
| **Dynamic Entity Cost** | O(1) spawn | O(1) | O(1) |
| **Query Cost** | 快 (Archetype filter) | 极快 | 取决于实现 |
| **Serialization** | 支持 (Reflect) | 支持 (meta) | 需自实现 |
| **Concurrency** | 良好 (Send + Sync) | 良好 | 取决于实现 |
| **Lifecycle Support** | 需自实现 System | 需自实现 Observer | 需自实现 |
| **STAR 适用** | ✅ 适合 9 Archetype 业务 | ✅ 适合 1M Entity 列存 | ⚠️ 维护成本高 |
| **P3-B 选型建议** | **推荐** (Rust 生态成熟) | 备选 (低开销) | 备选 (极简) |

**P3-B 选型决策**: 拍板后填入 (per G-2 已知缺口).

## 2. クラス設計 (Class Design)

### 2.1 关键类型 (Key Types)

```rust
// domain-agent/src/lib.rs
use bevy_ecs::prelude::*;
use uuid::Uuid;
use std::time::{Duration, Instant, SystemTime};

// ====== Agent Components (per SRS §9-§20) ======

#[derive(Component, Debug, Clone)]
pub struct AgentIdentity {
    pub agent_id: AgentId,
    pub tenant_id: TenantId,
    pub agent_type: AgentType,        // SA-01..SA-09 引用 LangGraph 9/3 §6.1
    pub created_at: SystemTime,
}

#[derive(Component, Debug, Clone)]
pub struct AgentState {
    pub current: AgentStateEnum,
    pub prev: AgentStateEnum,
    pub since: Instant,
    pub retry_count: u32,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum AgentStateEnum {
    Idle,
    Ready,
    Scheduled,
    Planning,
    WaitingLlm,
    WaitingTool,
    WaitingEvent,
    Processing,
    Completed,
    Failed,
    Suspended,
    Cancelled,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum LifecycleStateEnum {
    Hot,
    Warm,
    Cold,
}

#[derive(Component, Debug, Clone)]
pub struct LifecycleState {
    pub current: LifecycleStateEnum,
    pub last_active: Instant,
    pub hot_timeout: Duration,         // HOT → WARM 超时
    pub warm_timeout: Duration,        // WARM → COLD 超时
}

#[derive(Component, Debug, Clone)]
pub struct ContextRef {
    pub context_id: ContextId,
    pub tier: ContextTier,
    pub loaded: bool,
    pub refs: Vec<ContextId>,          // 共享 Context + Agent Delta
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTier {
    L1Hot,                              // RAM
    L2Recent,                           // Redis
    L3Full,                             // DB
}

#[derive(Component, Debug, Clone)]
pub struct MemoryRef {
    pub memory_id: MemoryId,
    pub memory_type: MemoryType,
    pub embedding_ref: Option<PayloadRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Semantic,
    Episodic,
    User,
    Workflow,
    Knowledge,
}

#[derive(Component, Debug, Clone)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
    pub profile: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Component, Debug, Clone)]
pub struct ToolPolicyRef {
    pub policy_id: PolicyId,
    pub tool_allowlist: Vec<String>,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Component, Debug, Clone)]
pub struct McpPolicyRef {
    pub policy_id: PolicyId,
    pub server_allowlist: Vec<String>,
}

#[derive(Component, Debug, Clone)]
pub struct PermissionRef {
    pub acl_id: AclId,
    pub tenant_id: TenantId,
}

#[derive(Component, Debug, Clone)]
pub struct TokenBudget {
    pub max_context_tokens: u32,
    pub max_output_tokens: u32,
    pub remaining_tokens: u32,
    pub cost_budget_cents: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

#[derive(Component, Debug, Clone)]
pub struct MailboxRef {
    pub mailbox_id: MailboxId,
    pub unread_count: u32,
    pub max_size: u32,
}

#[derive(Component, Debug, Clone)]
pub struct WorkflowRef {
    pub workflow_id: WorkflowId,
    pub step: u32,
    pub total_steps: u32,
    pub checkpoint_id: Option<CheckpointId>,
}

// ====== 类型别名 ======
pub type AgentId = Uuid;
pub type TenantId = Uuid;
pub type ContextId = Uuid;
pub type MemoryId = Uuid;
pub type PolicyId = Uuid;
pub type AclId = Uuid;
pub type MailboxId = Uuid;
pub type WorkflowId = Uuid;
pub type CheckpointId = Uuid;
pub type TraceId = Uuid;
pub type CorrelationId = Uuid;
pub type EventId = Uuid;
pub type MsgId = Uuid;
pub type StoreId = String;
pub type AgentType = String;  // "SA-01".."SA-09"
```

### 2.2 关键类 (Critical Classes)

```rust
// domain-dispatcher/src/dispatcher.rs
pub struct Dispatcher {
    queue: Arc<TaskQueue>,
    pool: Arc<ProcessPool>,
    rate_limiter: Arc<RateLimiter>,
    backpressure: Arc<Backpressure>,
    retry: Arc<RetryPolicy>,
    observer: Arc<Observer>,
    runtime_mode: Arc<RuntimeModeManager>,
}

impl Dispatcher {
    pub async fn dispatch(&self, task: Task) -> Result<ScheduleId> {
        // 1. Acquire rate limit token
        self.rate_limiter.acquire(&task.tenant_id).await?;

        // 2. Check backpressure
        self.backpressure.await_slot().await?;

        // 3. Enqueue to SQLite WAL
        let task_id = self.queue.enqueue(task).await?;

        // 4. Submit to ProcessPool
        let handle = self.pool.submit(task_id).await?;

        // 5. Observe
        self.observer.metrics().inc_dispatched();

        Ok(ScheduleId(task_id))
    }
}

// domain-dispatcher/src/queue.rs
pub struct TaskQueue {
    db: Arc<Mutex<Connection>>,           // rusqlite + WAL
}

impl TaskQueue {
    pub async fn enqueue(&self, task: Task) -> Result<TaskId> {
        // INSERT INTO task_queue (task_id, brief, status, tenant_id, priority, retry_count, created_at)
        //   VALUES (?, ?, 'pending', ?, ?, 0, ?)
    }

    pub async fn dequeue(&self, limit: u32) -> Result<Vec<Task>> {
        // SELECT * FROM task_queue
        //   WHERE status = 'pending'
        //   ORDER BY priority ASC, created_at ASC
        //   LIMIT ?
    }

    pub async fn mark_done(&self, task_id: TaskId) -> Result<()> {
        // UPDATE task_queue SET status = 'done', completed_at = ? WHERE task_id = ?
    }

    pub async fn mark_failed(&self, task_id: TaskId, retry: bool) -> Result<()> {
        // UPDATE task_queue SET status = 'failed', retry_count = retry_count + 1
        //   WHERE task_id = ?
    }
}

// domain-dispatcher/src/pool.rs
pub struct ProcessPool {
    workers: Vec<WorkerHandle>,
    semaphore: Arc<Semaphore>,            // 限流
    config: PoolConfig,
}

#[derive(Clone)]
pub struct PoolConfig {
    pub size: usize,                     // 8-16
    pub worker_path: PathBuf,            // Python worker 路径
    pub warm_up: bool,                    // 预热 imports (per 守门 #24 v24)
    pub idle_timeout: Duration,
}

impl ProcessPool {
    pub async fn submit(&self, task_id: TaskId) -> Result<WorkerHandle> {
        let permit = self.semaphore.acquire().await?;
        let worker = self.workers.iter()
            .find(|w| w.is_idle())
            .or_else(|| self.spawn_new_worker())
            .ok_or(Error::NoAvailableWorker)?;
        worker.assign(task_id)?;
        Ok(worker.handle())
    }
}
```

## 3. 状態機設計 (State Machine)

### 3.1 Agent 状态机 (per SRS REQ-ECS-002 + §35)

```
                    ┌──────────┐
                    │   Idle   │◄────────────────┐
                    └────┬─────┘                 │
                         │ schedule              │ restart
                         ▼                       │
                    ┌──────────┐                 │
                    │  Ready   │                 │
                    └────┬─────┘                 │
                         │ dispatch              │
                         ▼                       │
                    ┌──────────┐                 │
                    │Scheduled │                 │
                    └────┬─────┘                 │
                         │ plan                  │
                         ▼                       │
                    ┌──────────┐                 │
                    │ Planning │                 │
                    └────┬─────┘                 │
                         │ execute               │
                         ▼                       │
                ┌────────────────┐               │
                │  WaitingLlm    │               │
                │  WaitingTool   │               │
                │  WaitingEvent  │               │
                └────────┬───────┘               │
                         │ result                │
                         ▼                       │
                    ┌──────────┐                 │
                    │Processing │                 │
                    └────┬─────┘                 │
                         │ complete              │
                         ▼                       │
                    ┌──────────┐                 │
                    │Completed │─────────────────┘
                    └──────────┘

              异常:    Failed / Suspended / Cancelled
              ┌──────────┐  ┌──────────┐  ┌──────────┐
              │  Failed  │  │Suspended │  │Cancelled │
              └────┬─────┘  └────┬─────┘  └──────────┘
                   │ retry        │ resume
                   ▼              ▼
                 (回到 Ready)  (回到 Scheduled)
```

### 3.2 Lifecycle 状态机 (HOT/WARM/COLD, per SRS REQ-ECS-003 + §10-§12)

```
                  ┌────────────────────────────────────────┐
                  │                                        │
                  ▼                                        │
            ┌──────────┐    timeout 60s    ┌──────────┐   │
   Event ──→│   HOT    │ ────────────────→ │   WARM   │   │
            │ (执行中) │ ←──────────────── │ (等待)   │   │
            └──────────┘    Event          └────┬─────┘   │
                                                │         │
                                       idle 600s│         │
                                                ▼         │
                                          ┌──────────┐    │
                                          │   COLD   │────┘
                                          │(持久化) │
                                          └──────────┘
                                                │ Event
                                                ▼
                                            (Restore)
                                                ↓
                                              WARM
                                                ↓
                                            (Schedule)
                                                ↓
                                              HOT
```

**转换规则**:

| From | To | 触发 | 守门 |
|---|---|---|---|
| (新) | HOT | Event 触发, Schedule | — |
| HOT | WARM | timeout 60s 无活动 | per SRS §30 max_hot_agents |
| WARM | HOT | Event 触发, Schedule | — |
| WARM | COLD | idle 600s (10 分钟) | per SRS §23 |
| COLD | WARM | Event 触发, Restore | per SRS §25 |
| HOT/WARM/COLD | Suspended | suspend_agent() | per SRS §50 |

**配置化 (per SRS §7.4)**:
```yaml
runtime:
  lifecycle:
    hot_to_warm_timeout: 60s
    warm_to_cold_idle_timeout: 600s
    restore_timeout: 30s
    eviction_batch_size: 100
```

## 4. 時系列図 (Sequence Diagrams)

### 4.1 UC-01: Task Dispatch 流程

```
User/Caller     L0 Dispatcher     L0 TaskQueue       L0 ProcessPool       L1 ECS World
    │                 │                  │                  │                  │
    │ dispatch(task)  │                  │                  │                  │
    ├────────────────→│                  │                  │                  │
    │                 │ rate_limit.acquire()                │                  │
    │                 │ (token bucket)    │                  │                  │
    │                 │ backpressure.await_slot()           │                  │
    │                 │ enqueue(task)     │                  │                  │
    │                 ├─────────────────→│                  │                  │
    │                 │                  │ INSERT INTO ...  │                  │
    │                 │ submit(task_id)   │                  │                  │
    │                 ├────────────────────────────────────→│                  │
    │                 │                  │                  │ worker.assign()  │
    │                 │                  │                  │ worker.execute() │
    │                 │                  │                  │ (Python)         │
    │                 │                  │                  │ (调 L1 ECS)      │
    │                 │                  │                  ├─────────────────→│
    │                 │                  │                  │                  │ spawn entity
    │                 │                  │                  │                  │ (Archetype SA-XX)
    │                 │                  │                  │                  │ set Components
    │                 │                  │                  │                  │ run Systems
    │                 │                  │                  │ (callback)       │
    │                 │                  │                  │ result           │
    │                 │                  │                  │ mark_done        │
    │                 │                  │                  │                  │
    │                 │ observer.metrics()                  │                  │
    │                 │ ScheduleId        │                  │                  │
    │←────────────────┤                  │                  │                  │
    │                 │                  │                  │                  │
```

### 4.2 UC-02: Runtime Mode Switch (Lightweight → ECS)

```
Operator       RuntimeModeManager    L0 Dispatcher    L1 ECS World
    │                  │                  │                │
    │ get_status()     │                  │                │
    ├─────────────────→│                  │                │
    │ resident_count   │                  │                │
    │                  │ (12 resident)    │                │
    │ mode=Lightweight │                  │                │
    │←─────────────────┤                  │                │
    │                  │                  │                │
    │ (auto 30s stable)│                  │                │
    │                  │                  │                │
    │                  │ switch_to_ecs()  │                │
    │                  │                  │                │
    │                  │ buffer_events()  │                │
    │                  │ (drain active)   │                │
    │                  │                  │                │
    │                  │ migrate_state    │                │
    │                  ├─────────────────→│                │
    │                  │                  │ drain queue    │
    │                  │                  │ wait workers   │
    │                  │                  │                │
    │                  │ init_ecs_world() │                │
    │                  ├──────────────────────────────────→│
    │                  │                  │                │ create World
    │                  │                  │                │ add Archetypes
    │                  │                  │                │ add Systems
    │                  │                  │                │
    │                  │ resume_dispatch()│                │
    │                  │                  │                │
    │                  │ (no event loss)  │                │
    │ mode=ECS         │                  │                │
    │←─────────────────┤                  │                │
    │                  │                  │                │
```

**一致性保证 (per SRS §83)**: 切换过程中不丢 Event / 不重复 Tool / 不丢 Agent State / 不丢 ContextRef / 不重复 LLM 请求. 用 `buffer_events()` + `migrate_state()` + `resume_dispatch()` 三步保证.

### 4.3 UC-03: HOT → COLD → HOT (Lifecycle)

```
L1 ECS World    LifecycleManager    L2 Context Store    L2 Memory Store    L1 ECS World
    │                  │                  │                  │                │
    │ timeout(60s)     │                  │                │                │
    │←─────────────────┤                  │                │                │
    │                  │                  │                │                │
    │ HOT → WARM       │                  │                │                │
    │ (transition)     │                  │                │                │
    │                  │                  │                │                │
    │ idle 600s        │                  │                │                │
    │←─────────────────┤                  │                │                │
    │                  │                  │                │                │
    │ WARM → COLD      │                  │                │                │
    │                  │ persist(entity)  │                │                │
    │                  ├─────────────────→│                │                │
    │                  │ persist(memory)  │                │                │
    │                  ├─────────────────────────────────→│                │
    │                  │ mark Cold        │                │                │
    │                  │ (RAM ≈ 0)       │                │                │
    │                  │                  │                │                │
    │ (COLD)           │                  │                │                │
    │                  │                  │                │                │
    │ ... Event 触发 ...                  │                │                │
    │                  │                  │                │                │
    │                  │ restore(entity)  │                │                │
    │                  ├─────────────────→│                │                │
    │                  │ load_metadata    │                │                │
    │                  │ (lazy load)      │                │                │
    │                  │                  │                │                │
    │                  │ spawn_entity()   │                │                │
    │                  ├────────────────────────────────────────────────────→│
    │                  │                  │                │ create entity   │
    │                  │                  │                │ set Components  │
    │                  │                  │                │ (lazy ContextRef)│
    │                  │                  │                │                │
    │                  │ schedule()       │                │                │
    │                  ├────────────────────────────────────────────────────→│
    │                  │                  │                │ WARM → HOT      │
    │ (HOT again)      │                  │                │                │
```

### 4.4 UC-04: Backpressure Overflow

```
L0 Dispatcher       L0 TaskQueue      L0 RateLimiter    Caller
    │                    │                 │              │
    │ dispatch(task)     │                 │              │
    ├───────────────────→│                 │              │
    │ acquire(tenant)    │                 │              │
    ├────────────────────────────────────→ │              │
    │ (no token)         │                 │              │
    │ backpressure.await_slot()            │              │
    │ (queue full)       │                 │              │
    │                    │                 │              │
    │ overflow_policy()  │                 │              │
    │ (per config)       │                 │              │
    │ ┌────────────────────────────┐       │              │
    │ │ Reject  | Delay | DropLow  │       │              │
    │ │ Persist | Throttle         │       │              │
    │ └────────────────────────────┘       │              │
    │                    │                 │              │
    │ (e.g. Throttle)    │                 │              │
    │ 429 Too Many Req   │                 │              │
    │←────────────────────────────────────────────────────┤
    │                    │                 │              │
    │ retry_after: 60s   │                 │              │
    │ (per SRS §34)      │                 │              │
```

## 5. データ構造 (Data Structures)

### 5.1 Task Queue 表 (SQLite WAL)

```sql
-- per 守门 #13 DB 三類横展開, task_queue 是 W (Work, 短 TTL) 類
-- per SRS §29-§34 L0 派发调度器

CREATE TABLE task_queue (
    task_id          TEXT PRIMARY KEY,           -- UUID
    brief            TEXT NOT NULL,               -- brief 路径 (per 守门 #20)
    agent_type       TEXT NOT NULL,               -- SA-01..SA-09
    tenant_id        TEXT NOT NULL,
    priority         INTEGER NOT NULL,            -- 0=Critical, 4=Background
    status           TEXT NOT NULL,               -- pending / running / done / failed / dead
    retry_count      INTEGER NOT NULL DEFAULT 0,
    max_retry        INTEGER NOT NULL DEFAULT 3,
    payload          TEXT,                        -- JSON (Task 定义)
    created_at       INTEGER NOT NULL,            -- unix epoch
    started_at       INTEGER,
    completed_at     INTEGER,
    trace_id         TEXT,                        -- UUID, 贯穿 Event/Agent/LLM/...
    correlation_id   TEXT,                        -- UUID
    error            TEXT,                        -- JSON (失败原因)
    -- 守门 #13 W 類派生
    retention_until  INTEGER NOT NULL             -- 7 天后清理
) WITHOUT ROWID;

CREATE INDEX idx_task_queue_status_priority ON task_queue (status, priority, created_at);
CREATE INDEX idx_task_queue_tenant ON task_queue (tenant_id, status);
CREATE INDEX idx_task_queue_trace ON task_queue (trace_id);

-- WAL 模式 (per SRS §29 + 守门 #13)
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA wal_autocheckpoint = 1000;
```

### 5.2 Dead Letter Queue 表 (W 類)

```sql
-- per SRS §34 REQ-SCH-005 死信队列
CREATE TABLE dead_letter_queue (
    task_id          TEXT PRIMARY KEY,
    original_task    TEXT NOT NULL,               -- 完整 Task JSON
    failure_reason   TEXT NOT NULL,
    retry_count      INTEGER NOT NULL,
    first_failed_at  INTEGER NOT NULL,
    last_failed_at   INTEGER NOT NULL,
    -- 守门 #13 W 類派生
    retention_until  INTEGER NOT NULL             -- 30 天后清理 (per 守门 #11 缺标比错标)
) WITHOUT ROWID;
```

### 5.3 Event Store 表 (T 類 append-only)

```sql
-- per SRS §13-§15 Event 持久化 (审计 + 回放)
-- 守门 #13 T 類派生: append-only, 物理删除禁止 + 監査必須 + RLS 13 類必携

CREATE TABLE event_log (
    event_id         TEXT PRIMARY KEY,
    source_agent_id  TEXT,
    target_agent_id  TEXT NOT NULL,
    tenant_id        TEXT NOT NULL,
    event_type       TEXT NOT NULL,
    payload_ref      TEXT,                        -- Payload Store 引用
    timestamp        INTEGER NOT NULL,
    trace_id         TEXT NOT NULL,
    correlation_id   TEXT
) WITHOUT ROWID;

-- 守门 #13 T 類: 物理删除禁止 (REVOKE DELETE)
REVOKE DELETE ON event_log FROM PUBLIC;
REVOKE DELETE ON event_log FROM star_app_role;

-- 守门 #13 T 類: RLS 13 類 (per tenant 隔离)
ALTER TABLE event_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_log FORCE ROW LEVEL SECURITY;
CREATE POLICY policy_event_log_tenant ON event_log
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- 守门 #13 T 類: 月次 BRIN partition + 7 年保持
CREATE INDEX idx_event_log_timestamp_brin ON event_log USING BRIN (timestamp);
```

### 5.4 Agent Checkpoint 表 (T 類)

```sql
-- per SRS §42 REQ-CHECKPOINT-001 长任务 checkpoint
CREATE TABLE agent_checkpoint (
    checkpoint_id    TEXT PRIMARY KEY,
    agent_id         TEXT NOT NULL,
    step             INTEGER NOT NULL,
    total_steps      INTEGER NOT NULL,
    state            TEXT NOT NULL,               -- JSON (Agent Components)
    workflow_ref     TEXT,
    created_at       INTEGER NOT NULL
) WITHOUT ROWID;

CREATE INDEX idx_checkpoint_agent_step ON agent_checkpoint (agent_id, step DESC);
```

### 5.5 Tenant Quota 表 (M 類 SCD-2)

```sql
-- per SRS §43 REQ-TENANT-001 多租户隔离
-- 守门 #13 M 類派生: 物理删除禁止 + SCD Type 2 + RLS 13 類必携

CREATE TABLE tenant_quota (
    tenant_id        TEXT NOT NULL,
    effective_from   INTEGER NOT NULL,
    effective_to     INTEGER,                     -- NULL = current
    agent_limit      INTEGER NOT NULL,
    hot_limit        INTEGER NOT NULL,
    llm_quota        INTEGER NOT NULL,            -- per month
    tool_quota       INTEGER NOT NULL,
    mcp_quota        INTEGER NOT NULL,
    token_quota      INTEGER NOT NULL,            -- per month
    memory_quota     INTEGER NOT NULL,            -- bytes
    PRIMARY KEY (tenant_id, effective_from)
) WITHOUT ROWID;

ALTER TABLE tenant_quota ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_quota FORCE ROW LEVEL SECURITY;
CREATE POLICY policy_tenant_quota_tenant ON tenant_quota
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);
```

## 6. アルゴリズム (Algorithms)

### 6.1 L0 调度算法 (Priority + Fair Scheduling, per SRS §44 REQ-SCH-FAIR)

```python
# domain-dispatcher/src/dispatcher.rs (Python 风格伪代码, 实际是 Rust + Tokio)
async def schedule_task(self, task: Task) -> ScheduleId:
    # 1. Acquire rate limit token (per tenant)
    await self.rate_limiter.acquire(task.tenant_id)

    # 2. Check backpressure (queue < max_size)
    if self.queue.size() >= self.config.max_queue_size:
        match self.config.overflow_policy:
            case "Reject":
                raise Error::QueueFull
            case "Delay":
                await asyncio.sleep(self.config.delay_ms)
            case "DropLow":
                self.queue.drop_low_priority()
            case "Persist":
                self.queue.persist_to_dlq()
            case "Throttle":
                return Err(Throttle { retry_after: 60 })

    # 3. Enqueue with priority
    self.queue.enqueue(
        task_id=task.id,
        priority=task.priority,
        fair_key=(task.tenant_id, task.priority, task.created_at)
    )

    # 4. Dequeue and dispatch
    while True:
        task = self.queue.dequeue_one()  # ORDER BY priority ASC, fair_key ASC
        if task is None:
            break

        # 5. Check fair scheduling
        # - Critical/High: immediate dispatch
        # - Normal/Low/Background: round-robin per tenant
        if task.priority <= Priority::High:
            self.pool.submit(task)
        else:
            # Round-robin: 每 tenant 轮流取一个
            tenant_count = self.fair_scheduler.next_tenant_count(task.tenant_id)
            if tenant_count < self.config.fair_share:
                self.pool.submit(task)
            else:
                # Re-enqueue at tail
                self.queue.requeue(task)
                break

    return ScheduleId(task.id)
```

**复杂度**: O(log N) per enqueue/dequeue (SQLite B-tree 索引). 16 worker 并行, 吞吐量 ~3.2 task/s (5s/task). 200 worker 弹性扩 → 40 task/s.

### 6.2 L1 ECS Query 算法 (Archetype + Sparse Set)

```rust
// domain-agent/src/scheduler.rs
pub fn scheduler_system(
    mut commands: Commands,
    query: Query<(Entity, &AgentIdentity, &AgentState, &Priority), With<Ready>>,
) {
    // 1. Collect ready agents
    let mut ready: Vec<(Entity, Priority, Instant)> = query
        .iter()
        .map(|(e, _, _, p)| (e, p, Instant::now()))
        .collect();

    // 2. Sort by priority + created_at
    ready.sort_by_key(|(e, p, _)| (*p as u8, e.id()));

    // 3. Dispatch to HOT (max_hot_agents limit)
    let max_hot = MAX_HOT_AGENTS.load(Ordering::Relaxed);
    for (entity, _, _) in ready.iter().take(max_hot) {
        commands.entity(*entity).insert(LifecycleState {
            current: LifecycleStateEnum::Hot,
            last_active: Instant::now(),
            ..
        });
    }
}
```

**复杂度**: O(N log N) per frame (sort) + O(K) dispatch (K = max_hot). ECS Archetype-based query 是 O(K) 不是 O(N) (因为只遍历 Ready archetype).

### 6.3 Backpressure 算法 (Bounded Queue + Overflow Policy)

```rust
// domain-dispatcher/src/backpressure.rs
pub struct Backpressure {
    queue_size: Arc<AtomicUsize>,
    max_size: usize,
    overflow_policy: OverflowPolicy,
}

impl Backpressure {
    pub async fn await_slot(&self) -> Result<()> {
        if self.queue_size.load(Ordering::Relaxed) < self.max_size {
            return Ok(());
        }

        match self.overflow_policy {
            OverflowPolicy::Reject => Err(Error::QueueFull),
            OverflowPolicy::Delay { ms } => {
                tokio::time::sleep(Duration::from_millis(ms)).await;
                Ok(())
            }
            OverflowPolicy::DropLow => {
                // Drop lowest priority task
                self.queue.drop_low_priority();
                Ok(())
            }
            OverflowPolicy::Persist => {
                // Move to DLQ
                self.queue.persist_to_dlq();
                Ok(())
            }
            OverflowPolicy::Throttle => {
                Err(Throttle { retry_after: 60 })
            }
        }
    }
}
```

### 6.4 HOT Slot 限制算法 (per SRS §30 REQ-SCH-002)

```rust
// domain-dispatcher/src/dispatcher.rs
pub fn check_hot_slot_available(&self) -> bool {
    let hot_count = self.hot_count.load(Ordering::Relaxed);
    hot_count < self.config.max_hot_agents
}

pub fn dispatch_to_hot(&self, entity: Entity) -> Result<()> {
    if !self.check_hot_slot_available() {
        // Wait for HOT slot
        return Err(Error::HotSlotFull);
    }
    self.hot_count.fetch_add(1, Ordering::Relaxed);
    // ... assign to worker
    Ok(())
}
```

## 7. エラー処理 (Error Handling)

### 7.1 Error 分类 (per SRS §38 REQ-RETRY-001)

| Error 类 | 例子 | Retry 策略 |
|---|---|---|
| **Retryable** | Network timeout, LLM rate limit, Tool temporary failure | max_retry=3, backoff=exponential (1s, 2s, 4s) |
| **Non-Retryable** | Invalid input, Permission denied, Auth failed, Schema mismatch | max_retry=0, 直接 Failed |
| **Recoverable** | Crash recovery, Context lost, Checkpoint not found | restore from checkpoint + retry |
| **Fatal** | OOM, Disk full, Critical invariant violation | 立即 Failed + 报警 |

### 7.2 Retry 实现 (Bounded + Idempotent)

```rust
// domain-dispatcher/src/retry.rs
pub struct RetryPolicy {
    max_retry: u32,
    backoff: ExponentialBackoff,
}

impl RetryPolicy {
    pub async fn retry<F, T, E>(&self, task_id: TaskId, op: F) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> BoxFuture<'static, Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        loop {
            match op().await {
                Ok(v) => return Ok(v),
                Err(e) if attempt < self.max_retry && is_retryable(&e) => {
                    attempt += 1;
                    let delay = self.backoff.delay(attempt);
                    tokio::time::sleep(delay).await;
                    tracing::warn!(task_id = %task_id, attempt = attempt, error = ?e, "retry");
                }
                Err(e) => return Err(RetryError::Exhausted { attempts: attempt, source: e }),
            }
        }
    }
}
```

**幂等性 (per SRS §39 REQ-IDEMPOTENT-001)**: Task 包含 `operation_id` + `task_id` + `agent_id` + `correlation_id`. 重试时 LLM/Tool 收到相同 `operation_id` 走 idempotency key 路径.

## 8. 永続化 (Persistence)

### 8.1 持久化时机 (per SRS §40 REQ-PERSIST-001)

| 时机 | 持久化内容 | 存储 |
|---|---|---|
| **Agent 创建** | AgentIdentity + TokenBudget + Priority | SQLite M 類 (SCD-2) |
| **状态变更** | AgentState + LifecycleState | SQLite T 類 append-only (event_log) |
| **WARM → COLD** | 全 Components (ContextRef + MemoryRef + TokenBudget + WorkflowRef) | SQLite M 類 (agent_checkpoint) |
| **COLD → WARM** | Load checkpoint + lazy ContextRef | SQLite M 類 |
| **Pending Event** | Event 落档 (SRS §40) | SQLite T 類 (event_log) |
| **任务完成** | Task + result + cost | SQLite T 類 (event_log + agent_checkpoint) |
| **Crash** | RAM in-flight entity 全部丢失, 从 SQLite 恢复 | — |

### 8.2 Checkpoint 格式 (per SRS §42 REQ-CHECKPOINT-001)

```json
{
  "checkpoint_id": "uuid",
  "agent_id": "uuid",
  "step": 5,
  "total_steps": 10,
  "state": {
    "AgentIdentity": { "agent_id": "...", "tenant_id": "...", "agent_type": "SA-04" },
    "AgentState": { "current": "WaitingLlm", "prev": "Processing", "since": "..." },
    "ContextRef": { "context_id": "...", "tier": "L2Recent", "loaded": true },
    "TokenBudget": { "max_context_tokens": 8000, "remaining_tokens": 6000 }
  },
  "workflow_ref": "uuid",
  "created_at": "2026-09-03T18:00:00Z"
}
```

## 9. テスト設計 (Test Design, per SRS §63-§71)

### 9.1 UT (Unit Test, per SRS §64)

| 测试类 | 数量 | 守门 |
|---|---|---|
| **AgentIdentity / AgentState / LifecycleState enum** | 30+ | #1 |
| **ContextTier / MemoryType / Priority enum** | 20+ | #1 |
| **TaskQueue CRUD (SQLite in-memory)** | 20+ | #1 |
| **ProcessPool (mock worker)** | 10+ | #1 |
| **RateLimiter (token bucket)** | 15+ | #1 |
| **Backpressure (5 overflow policies)** | 5×5 = 25 | #1 |
| **RetryPolicy (3 retryable, 3 non-retryable, 2 fatal)** | 8+ | #1 |
| **ECS Archetype 9 SA (SA-01..SA-09)** | 9×10 = 90 | #1 |
| **State Machine transitions (12 states)** | 12×3 = 36 | #1 |
| **Lifecycle transitions (HOT/WARM/COLD)** | 3×3 = 9 | #1 |
| **总计** | **~250 UT** | **#1 v12 100% pass** |

### 9.2 IT (Integration Test, per SRS §64)

| 测试类 | 数量 | 守门 |
|---|---|---|
| **L0 Dispatcher + TaskQueue + ProcessPool 集成** | 20+ | #1 |
| **L1 ECS + Scheduler System** | 15+ | #1 |
| **RuntimeModeManager (Lightweight ↔ ECS 切换)** | 10+ | #1 |
| **LifecycleManager (HOT/WARM/COLD 转换 + 持久化)** | 12+ | #1 |
| **EventBus + Mailbox 集成** | 10+ | #1 |
| **总计** | **~70 IT** | **#1 v12 100% pass** |

### 9.3 E2E (End-to-End Test, per SRS §64)

| 测试场景 | 数量 | 守门 |
|---|---|---|
| **1 Agent 场景 (Lightweight 路径)** | 1 | #1 #19 |
| **5 Agent 场景** | 1 | #1 #19 |
| **8 Agent 场景 (守门 ECS 阈值)** | 1 | #1 |
| **9 Agent 场景 (必须 Lightweight)** | 1 | #1 |
| **10-11 Agent 场景 (迟滞区, 不切换)** | 2 | #1 |
| **12 Agent 场景 (进入 ECS)** | 1 | #1 |
| **50 Agent 场景 (9 Archetype 全激活)** | 1 | #1 |
| **100 Agent 场景 (稳态 ECS)** | 1 | #1 |
| **1000 Agent 场景 (HOT slot limit 验证)** | 1 | #1 |
| **总计** | **10 E2E** | **#1 v19 telemetry** |

### 9.4 PT (Performance Test, per SRS §64-§71)

| 测试 | 规模 | 目标 | 守门 |
|---|---|---|---|
| **RSS / Heap / CPU** | 1 / 5 / 8 / 9 / 10 / 12 / 16 / 50 / 100 Agent | ECS Break-even Point 找到 | #1 |
| **P50 / P95 / P99 latency** | 同上 | Schedule latency < 100ms p95 | #1 |
| **Throughput** | 16 / 32 / 200 / 500 worker | 1M 派发 87 小时 | #1 |
| **WARM Agent 内存** | 1000 Agent | < 100 KB / Agent, 优化 10-50 KB | #1 |
| **COLD Agent 内存** | 1000 Agent | ≈ 0 Runtime RAM | #1 |
| **HOT Ratio** | 0% / 1% / 5% / 10% HOT | 不同 HOT ratio 下的 throughput | #1 |
| **稳定性** | 24h / 72h / 7 days | 无 Memory Leak / Task Leak / Queue Growth | #1 |
| **Lifecycle** | Create 100K + Delete 100K | 资源真正回收 | #1 |
| **Mode Switch** | Lightweight → ECS / ECS → Lightweight | Latency + Peak RAM + Event 0 loss | #1 |
| **总计** | **9 PT 套** | **per SRS §72-§74 性能目标** | **#1 v19** |

### 9.5 Benchmark Suite (per SRS §76)

| 对照组 | 描述 | 期望 |
|---|---|---|
| **A. Traditional Agent** | Full Agent Object + Context + Client + Tool Runtime (假设基线) | 1M Agent = 8 TB |
| **B. Lightweight Shared Runtime** | Agent State + Tokio + Shared Runtime (守门 #24) | 1M Agent < 8 GB |
| **C. ECS Runtime** | ECS + Shared Runtime (P3-B+) | 1M Agent < 4 GB |
| **D. Full Hybrid Runtime** | Lightweight + ECS + HOT/WARM/COLD + External Context + Shared Pool + Event Driven (目标) | 1M Agent < 16 GB on 32GB 机器 |

## 10. 守門規則統合 (per AGENTS.md §4 + §4.1)

引 [02 §7](02-basic-design.md), 本 view 全部 24 项守门 + 24 条累积规 v1-v24 需满足:

| 守门 | 关键内容 | 状态 (本 view 落地后) |
|---|---|---|
| #1 | cargo check --workspace --all-targets 0 err | ✅ (per 41/41 crate 100% 覆盖) |
| #3 | 5 域独立 Lead, 不接受兼任 | ✅ (本 view 不涉及 5 域映射) |
| #5 | 环境变量安全 | ✅ |
| #6 | PowerShell only | ✅ |
| #7 | 0 unsafe | ✅ |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ |
| #12 | 缺标比错标安全 | ✅ |
| #19 | agent 交互 Python 化 (per 9/2 拍板) | ✅ (L0 dispatcher.py 是 Python, L1 ECS Rust) |
| #21 | [P] docs 同步必更新 automation-design §4 + registry.md | ✅ (§4.14 + §5.2 同步) |
| #24 | subprocess 替代 RPC | ✅ |
| #DB-13 | DB 三類横展開 (W/T/M) 強制分類 (per 9/1 拍板) | ✅ (per §5.1-§5.5 schema 已分类) |

**完整 24 + 24 见 AGENTS.md §4 + §4.1.**

## 11. 子エージェント失敗接手 (per 7 子代理派生规则)

引 [02 §8](02-basic-design.md), 跟 L0 dispatcher / L1 ECS 强相关:

| # | 子代理 | 失败模式 | 接手方案 |
|---|---|---|---|
| 1 | **L0 Dispatcher (Python worker)** | RPC 不可靠 | subprocess.run (守门 #24) |
| 2 | **L1 ECS System** | Archetype mismatch | 拆 entity 到 2 archetype |
| 3 | **L2 Pool 复用** | Pool exhaustion | rate limit + backpressure |
| 4 | **Lifecycle 转换** | HOT 资源耗尽 | 强制 WARM + 队列等待 |
| 5 | **Context lazy load** | L3 加载超时 | 降级到 L2 Recent |
| 6 | **Checkpoint restore** | Checkpoint corrupt | 重新建 entity (失 1 step) |
| 7 | **Mode switch** | 切换中 Event 丢失 | buffer_events + retry |

## 12. 既知缺口 (per 守門 #12 缺標比錯標)

引 [SRS-001 §3 G-1~G-12](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) + [02 §9 G-13~G-15](02-basic-design.md), 关键实施期 5 项 + 2 项本 view 新加:

| # | 缺口 | 影响 | 阶段 | 验证 |
|---|---|---|---|---|
| G-1 | L0 SQLite 任务队列 (per §5.1 schema) | 1M 派发无持久化 | P3-B L0 PoC | DDD Review |
| G-2 | L1 bevy_ecs / flecs 选型 (per §1.3) | 9 SA ECS 无运行时 | P3-B 启动 | P3-B 拍板 |
| G-3 | EventBus + Mailbox (per §4 时序) | Agent 间通信无协议 | P3-B | P3-B 拍板 |
| G-4 | Shared LLM/HTTP/MCP Pool | 守门 #24 subprocess 池 ≠ ECS 池 | P3-C | P3-C 拍板 |
| G-7 | Crash Recovery + Checkpoint (per §8) | 任务卡恢复无协议 | P3-D | P3-D 拍板 |
| **G-16 (新)** | **9 Archetype SA-01..SA-09 跟 L1 ECS Component 字段兼容性** | 9 SA 业务逻辑用不同字段子集, ECS Archetype 共享 Component 需验证 | P3-B 选型 | P3-B DDD Review |
| **G-17 (新)** | **ProcessPool 跟 ECS World 跨 runtime 切换的 race condition** | 8-16 Python worker 跟 Rust ECS 切换时 in-flight task 怎么 hold | P3-B L0 PoC | P3-B L0 PoC 实证 |

## 13. 签字栏 (5 角色, per 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |

**per 2026-09-03 19:00 JST Ulysses 授权**.

## 14. 関連 ADR + 参考

- [SRS-STAR-AGENT-RUNTIME-001.md v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) (commit `5460d33`)
- [02-basic-design.md](02-basic-design.md) (本 view 基本設計, 同期落档)
- [ADR-0044 STAR Agent Runtime SRS Baseline](../2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md) (commit `5460d33`)
- [ADR-0045 STAR Agent Runtime Basic + Detailed Design Baseline](../2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md) (本 view 决策, 同期落档)
- [docs/architecture/2026-09-03-langgraph/01-requirements.md §6.1 9 SA Type](../2026-09-03-langgraph/01-requirements.md) (引用, 不重写)
- [docs/architecture/2026-09-03-langgraph/02-basic-design.md §3-§4](../2026-09-03-langgraph/02-basic-design.md) (引用, 不重写)
- [docs/architecture/2026-09-03-langgraph/03-detailed-design.md](../2026-09-03-langgraph/03-detailed-design.md) (引用, 不重写)
- [AGENTS.md §3 + §4 + §4.1 + §5 + §6 + §7](../../AGENTS.md)
- [docs/automation-design.md §4.14 (本 view 落地后)](../../automation-design.md)
- [scripts/automation/registry.md §5.2 (本 view 落地后)](../../../scripts/automation/registry.md)
- [STAR-OLU-001.md v0.1](../../STAR-OLU-001.md) (1 SRE·周 = 1.2M tokens 独立基线)
- [STAR-P3-WBS-001.md v0.6 §7 阻塞 7 项](../../docs/STAR-P3-WBS-001.md)
- [HANDOFF-ST-001.md v0.4 §5.3 5 Blocker](../../docs/reports/HANDOFF-ST-001.md)

## 15. 修订履歴

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 引用 02-basic-design.md, 落档 Agent Runtime 詳細設計; 9 模块 (M-01..M-15) + 13 类关键类 (Rust 草案) + 2 状态机 (Agent + Lifecycle) + 4 时序图 (UC-01..UC-04) + 5 表 schema (W/T/M 分类) + 4 算法 + 9 类错误处理 + 持久化 7 时机 + 4 类测试 (UT/IT/E2E/PT) + 9 PT benchmark | 2026-09-03 18:48 JST 用户发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步" |

---

# === 詳細設計書 結束 ===
