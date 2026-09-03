# 02. STAR Agent Runtime - 基本設計書 (Basic Design)

> **状態**：🟡 Draft v0.1
> **日期**：2026-09-03
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权）
> **依赖**：[`01-SRS-STAR-AGENT-RUNTIME-001.md`](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md)（要件定義書 / SRS）· [ADR-0044 STAR Agent Runtime SRS Baseline](../2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md) · [`AGENTS.md` §4 守门](../../AGENTS.md) · [`docs/automation-design.md`](../../automation-design.md) · [`scripts/automation/registry.md`](../../../scripts/automation/registry.md)
> **关联文档**：[`01-SRS-STAR-AGENT-RUNTIME-001.md`](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) · [`03-detailed-design.md`](03-detailed-design.md)（詳細設計書）
> **平行 view**：[`2026-09-03-langgraph/01-requirements.md`](../2026-09-03-langgraph/01-requirements.md) · [`02-basic-design.md`](../2026-09-03-langgraph/02-basic-design.md) · [`03-detailed-design.md`](../2026-09-03-langgraph/03-detailed-design.md)

> **本 view 范围 (per [SRS-001 §0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md))**: 本基本設計書涵盖的是 **STAR Agent Runtime** 系统 (Rust Hybrid ECS + Lightweight + Event Driven + Shared Runtime + HOT/WARM/COLD) 的 **基本設計**。**不涵盖** LangGraph 任务卡子代理 (per [LangGraph 9/3 02 §0](../2026-09-03-langgraph/02-basic-design.md)) — 那是 LangGraph view, 跟本 view 平行, **不重写** 9 SA Type (SA-01..SA-09) 详细设计, 引用 [LangGraph 9/3 02 §3-§4](../2026-09-03-langgraph/02-basic-design.md) 即可。
>
> **dual-use 提醒 (per AGENTS.md §5 仓库拓扑)**: 本 view 不引用 RGS 仓 + 不建立业务子域↔DDD bounded context 映射. 22 domain-* crate 是 STAR 仓内部 DDD 划分, 跟 5 域 (player/economy/match/social/admin) 非同一分类.

---

## 0. 目的 (Purpose)

本文档基于 [SRS-001 §1-§113](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) 的要件, 定义 **STAR Agent Runtime** 的基本設計：

- 系统架构 (3 层: L0 派发 + L1 ECS + L2 业务)
- Runtime 双模式 (Lightweight < 10 / ECS ≥ 12 + 迟滞区)
- 组件一览 (跟 22 domain-* crate 映射 + 9 SA Type 引用 LangGraph)
- 数据模型 (ECS Component / Event / 状态机)
- 接口设计 (Runtime API + Management API + Agent API)
- 安全/性能/可用性 NFR
- 守门 24 项 + 累积规 v1-v24 統合
- 子代理失败接手 + 已知缺口

> **重要区别 (per [SRS-001 §1.1](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md))**: 本 view 设计的 "Agent Runtime" = 大规模 AI Agent 并发场景的 Rust Runtime (L0 派发 + L1 ECS + 共享池). **不**等于 Mavis worker subagent (worker/explorer/verifier, subprocess + brief 派发) — Mavis worker 是 dispatcher 层, 在 L0 派发层之上. 两套系统并存, L0 派发层可调 worker subagent 走 subprocess 路径 (per 守门 #24 v24).

## 1. 适用范围 (Scope)

### 1.1 包含

- L0 派发层 (Tokio + SQLite + 进程池 + 速率控制 + Backpressure)
- L1 ECS Runtime (bevy_ecs / flecs 选型 + 9 Archetype SA-01..SA-09 引用 LangGraph 9/3 §6.1)
- L2 业务共享池 (LLM Pool / HTTP Pool / MCP Pool / Tool Registry / Retriever Pool)
- Runtime 双模式 (Lightweight < 10 / ECS ≥ 12 + 迟滞区 10-11)
- HOT/WARM/COLD 生命周期 + 转换
- Event Driven + Mailbox + PayloadRef
- Context Store / Memory Store / Event Store (External State)
- 调度器 (Priority + Fairness + Token Budget + Tenant Quota)
- 多租户隔离 + 权限 + Secret
- Crash Recovery + Checkpoint
- Observability + Trace + Benchmark

### 1.2 不包含

- 物理引擎 (Physis 独立产品线, per 2026-09-03 18:14 JST 用户反馈)
- 3D 渲染 / HUD
- 跨机分布式 (per 守门 #3 5 域单仓, 跨机待 P3-F 评估, 本 view ❌ N/A)
- 业务 Agent 具体业务逻辑 (通过 Plugin / System / Tool 扩展, per SRS §83)
- LangGraph 任务卡子代理详细 (引用 [LangGraph 9/3 02](../2026-09-03-langgraph/02-basic-design.md) §3-§4, 不重写)

### 1.3 跟 LangGraph view 区别

| 维度 | LangGraph View (9/3 批) | Agent Runtime View (本 view) |
|---|---|---|
| **关注点** | UI 驱动的 2-level hierarchical Agent (L0 全体代理 + L1 任务卡子代理) | 大规模 AI Agent 并发的 Runtime 基础设施 (派发 + ECS + 共享池) |
| **9 SA Type** | LangGraph subgraph (in-process, LangGraph 协议) | ECS 9 Archetype (in-process, ECS 协议) |
| **Checkpoint** | LangGraph 3-tier (RAM/Redis/DB) | Runtime 3-tier (HOT/WARM/COLD) |
| **通信** | LangGraph 状态 + 边 + reducer | Event Bus + Mailbox + Channel |
| **路径** | `docs/architecture/2026-09-03-langgraph/` | `docs/architecture/2026-09-03-agent-runtime/` |
| **依赖** | LangGraph Python | bevy_ecs / flecs Rust + Tokio |

**关系**: LangGraph view 跟 Agent Runtime view **平行**, 9 SA Type 是**接口**而不是实现 — LangGraph subgraph 实现 SA-XX 业务逻辑, Agent Runtime ECS 提供底层 Runtime. 两者通过 Adapter 模式连接 (per §3.3 组件一览).

## 2. システムアーキテクチャ (System Architecture)

### 2.1 全体構成図 (Overall Architecture)

```
                        ┌────────────────────────────────────────────────┐
                        │  Application Layer (业务 Agent)                  │
                        │  Plugin A / Plugin B / Plugin C                   │
                        │  (per SRS §83-§84 扩展架构)                      │
                        └────────────────────┬───────────────────────────┘
                                             │ Plugin API (Component / System / Tool)
                                             ▼
┌────────────────────────────────────────────────────────────────────────────────────┐
│                            Agent Runtime Core                                       │
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                   Runtime Mode Manager                                      │    │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐         │    │
│  │  │ Lightweight Mode │  │  Hysteresis Zone │  │    ECS Mode      │         │    │
│  │  │   < 10 Agents    │←→│     10-11        │←→│    ≥ 12 Agents   │         │    │
│  │  │  (per SRS §6.1)  │  │ (per SRS §7.2)   │  │  (per SRS §6.2)  │         │    │
│  │  └──────────────────┘  └──────────────────┘  └──────────────────┘         │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                     │
│  ┌──────────────────────────────────┐  ┌─────────────────────────────────────────┐  │
│  │  L0 派发层 (Tokio + SQLite)     │  │  L1 ECS Runtime (bevy_ecs / flecs)      │  │
│  │  (per SRS §29-§34 调度)          │  │  (per LangGraph 9/3 §6.1 9 SA Type)    │  │
│  │  ┌──────────────────────────┐   │  │  ┌─────────────────────────────────┐  │  │
│  │  │ SQLite 任务队列 WAL      │   │  │  │ ECS World                        │  │  │
│  │  │ + 速率控制 + Backpressure │   │  │  │ ┌───────┐ ┌───────┐ ┌───────┐  │  │  │
│  │  │ + 失败重试 + 死信队列     │   │  │  │ │SA-01  │ │SA-02  │ │SA-03  │  │  │  │
│  │  └──────────────────────────┘   │  │  │ │Agent  │ │Agent  │ │Agent  │  │  │  │
│  │  ┌──────────────────────────┐   │  │  │ └───────┘ └───────┘ └───────┘  │  │  │
│  │  │ Tokio Async Dispatcher   │   │  │  │ 9 Archetype (SA-01..SA-09)       │  │  │  │
│  │  │ (单进程 ~30MB)           │   │  │  │ ≤ 50 并发 in-process            │  │  │  │
│  │  └──────────────────────────┘   │  │  └─────────────────────────────────┘  │  │  │
│  │  ┌──────────────────────────┐   │  │  ┌─────────────────────────────────┐  │  │  │
│  │  │ Process Pool (8-16)       │   │  │  │ Systems (12 类)                 │  │  │  │
│  │  │ 常驻 worker 预热         │   │  │  │ Scheduler / Lifecycle / Event   │  │  │  │
│  │  │ (per 守门 #24)            │   │  │  │ Planner / Llm / Tool / Mcp      │  │  │  │
│  │  └──────────────────────────┘   │  │  │ Retrieval / Context / Memory    │  │  │  │
│  │                                  │  │  │ Permission / Persistence / Metrics│  │  │  │
│  │                                  │  │  └─────────────────────────────────┘  │  │  │
│  └──────────────────────────────────┘  └─────────────────────────────────────────┘  │
│                                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                          Shared Runtime                                     │    │
│  │  (per SRS §16-§27 共享池, 引用守门 #24 subprocess 池扩展)                    │    │
│  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐         │    │
│  │  │LLM  │ │MCP  │ │HTTP │ │Tool │ │RAG  │ │Token│ │Rate │ │CB   │         │    │
│  │  │Pool │ │Pool │ │Pool │ │ Reg │ │Pool │ │izer │ │Limit│ │     │         │    │
│  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘         │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                     │
└────────────────────────────────────────────────────────────────────────────────────┘
                                             │
                                             ▼
┌────────────────────────────────────────────────────────────────────────────────────┐
│                            External State Layer                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Context      │  │ Memory       │  │ Event        │  │ Payload      │           │
│  │ Store        │  │ Store        │  │ Store        │  │ Store        │           │
│  │ (L1/L2/L3)   │  │ (S/E/U/W/K)  │  │ (WORM)       │  │ (large msg)  │           │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘           │
│                                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                                │
│  │ Durable      │  │ Fast Cache   │  │ Vector       │                                │
│  │ Store        │  │ (Redis)      │  │ Store        │                                │
│  │ (SQLite/PG)  │  │              │  │ (Qdrant)     │                                │
│  └──────────────┘  └──────────────┘  └──────────────┘                                │
└────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Runtime 双模式架构 (per SRS §6 + §7)

| 维度 | Lightweight Mode | ECS Mode |
|---|---|---|
| **触发条件** | Resident Agent < 10 | Resident Agent ≥ 12 持续 30s |
| **降级条件** | Resident Agent ≤ 8 持续 300s | — |
| **迟滞区** | 10-11 Agent, 保持当前模式 | 同左 |
| **Entity 模型** | Agent = struct 实例 (简单) | Agent = ECS Entity (9 Archetype) |
| **调度** | Tokio async task (1 Agent 1 task) | ECS System (批量 over columns) |
| **生命周期** | 手动 (dispatcher 隐式) | Lifecycle Manager (HOT/WARM/COLD 显式) |
| **优势场景** | 少量 Agent, 简单状态 | 大量 Agent, 高频更新 |
| **STAR 现状** | ✅ P3-A 25 子项全 Lightweight | ⏳ P3-B+ 实装 |

**模式切换一致性 (per SRS §83)**: 切换过程不丢 Event / 不重复 Tool / 不丢 Agent State / 不丢 ContextRef / 不重复 LLM 请求. 零停机迁移 (per SRS §84).

### 2.3 三层职责拆解

| 层 | 职责 | 实现 | 跟 22 domain-* crate 映射 |
|---|---|---|---|
| **L0 派发** | 1M 任务怎么派 (队列 + 调度 + 池) | Tokio + SQLite WAL + Pool | `domain-task` (task_id 生成) + `domain-queue` (新) |
| **L1 ECS** | 任务卡状态机怎么转 (9 SA Archetype) | bevy_ecs / flecs | `domain-agent` (新) + 9 SA Type Adapter |
| **L2 业务共享池** | 共享资源怎么复用 (LLM/HTTP/MCP/Tool/RAG) | Arc + Pool + Mutex/RwLock | `domain-llm` / `domain-mcp` / `domain-tool` (新) |

## 3. コンポーネント一覧 (Components)

### 3.1 L0 派发层组件 (per SRS §5 + §29-§34)

| Component | 责任 | 接口 | 跟 22 domain-* 映射 | 守门 |
|---|---|---|---|---|
| **TaskQueue** | SQLite WAL 持久化任务队列 | `enqueue(task)`, `dequeue()`, `mark_done(id)`, `mark_failed(id, retry)` | `domain-task` | #1 |
| **Dispatcher** | Tokio async 任务调度 (速率控制 + 背压) | `dispatch(task)`, `cancel(task_id)` | (新) `domain-dispatcher` | #19 v19 |
| **ProcessPool** | 常驻 Python worker 池 (8-16, 预热) | `submit(brief, agent_type)`, `pool_size()` | (守门 #24) subprocess pool | #24 v24 |
| **RateLimiter** | 速率控制 (per tenant / per agent) | `acquire(tenant)`, `release(tenant)` | (新) `domain-rate-limiter` | — |
| **Backpressure** | 反压 + 队列溢出处理 | `await_slot()`, `overflow_policy()` | (新) `domain-backpressure` | — |
| **RetryPolicy** | 失败重试 + 死信队列 | `retry_count(task)`, `dlq_push(task)` | (新) `domain-retry` | — |
| **Observer** | 可观测性 (per SRS §61-§62) | `metrics()`, `trace_id`, `latency()` | (新) `domain-observability` | — |

### 3.2 L1 ECS 组件 (per SRS §9-§12 + LangGraph 9/3 §6.1)

| Component | 责任 | Rust 类型 (草案) | 9 SA Archetype 引用 |
|---|---|---|---|
| **AgentIdentity** | 唯一 ID + tenant + agent_type | `struct AgentIdentity { agent_id, tenant_id, agent_type }` | 9 SA 共用 |
| **AgentState** | 12 状态 (per SRS REQ-ECS-002) | `enum AgentState { Idle, Ready, Scheduled, ..., Failed, Suspended, Cancelled }` | 9 SA 共用 |
| **LifecycleState** | HOT/WARM/COLD (per SRS REQ-ECS-003) | `enum LifecycleState { Hot, Warm, Cold }` | 9 SA 共用 |
| **ContextRef** | Context 引用 (不存 Full Context) | `struct ContextRef { context_id: ContextId }` | 9 SA 共用 |
| **MemoryRef** | Memory 引用 | `struct MemoryRef { memory_id: MemoryId }` | 9 SA 共用 |
| **ModelRef** | 模型配置 + 共享 LLM Pool 引用 | `struct ModelRef { provider, model, profile }` | SA-04/05/06/07 (LLM 调用类) |
| **ToolPolicyRef** | Tool 权限引用 | `struct ToolPolicyRef { policy_id }` | SA-05/06/08/09 (Tool 调用类) |
| **McpPolicyRef** | MCP 权限引用 | `struct McpPolicyRef { policy_id }` | SA-06/07 (MCP 调用类) |
| **PermissionRef** | ACL 引用 (不复制 ACL) | `struct PermissionRef { acl_id }` | 9 SA 共用 |
| **TokenBudget** | Token 配额 | `struct TokenBudget { max_ctx, max_out, remaining, cost }` | 9 SA 共用 |
| **Priority** | 优先级 | `enum Priority { Critical, High, Normal, Low, Background }` | 9 SA 共用 |
| **MailboxRef** | Mailbox 引用 (不存大消息) | `struct MailboxRef { mailbox_id }` | 9 SA 共用 |
| **WorkflowRef** | Workflow 引用 | `struct WorkflowRef { workflow_id }` | SA-01/02 (规划类) |

**引用**: 9 SA Type (SA-01..SA-09) 详细定义在 [LangGraph 9/3 02 §3-§4](../2026-09-03-langgraph/02-basic-design.md), 本 view **不重写** (per 拍板 A lg-relation 引用不重写).

### 3.3 L2 业务共享池组件 (per SRS §16-§27)

| Pool | 责任 | 实现 | 跟 22 domain-* 映射 | 守门 |
|---|---|---|---|---|
| **LLM Pool** | 多 Provider 多 Model 共享 | Arc<LLMProviderPool> + Channel | (新) `domain-llm` | — |
| **MCP Pool** | MCP Runtime 共享 (Registry + Connection + Capability Cache) | Arc<McpRegistry> + Pool | (新) `domain-mcp` | — |
| **HTTP Pool** | HTTP Client 共享 (Keep Alive / HTTP/2) | reqwest::Client + Pool | (新) `domain-http` | — |
| **Tool Registry** | 工具定义全局共享 | Arc<ToolRegistry> + HashMap | (新) `domain-tool` | #24 |
| **RAG Pool** | Retriever 共享 (Vector DB + Cache) | Arc<RetrieverPool> + Cache | (新) `domain-rag` | — |
| **Tokenizer** | Tokenizer 共享 | Arc<Tokenizer> | `domain-llm` 复用 | — |
| **Prompt Registry** | Prompt 模板共享 | Arc<PromptRegistry> | (新) `domain-prompt` | — |
| **Provider Registry** | Provider 配置共享 | Arc<ProviderRegistry> | (新) `domain-provider` | — |
| **Rate Limiter** | Provider Rate Limit | Token Bucket | (新) `domain-rate-limiter` | — |
| **Circuit Breaker** | 熔断器 | Arc<CircuitBreaker> | (新) `domain-cb` | — |

### 3.4 9 Systems (per SRS §58 REQ-ECS-SYSTEM + §71)

引用 [SRS-001 §71](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) 13 Systems, 本 view 12 + 1 (跟 LangGraph 9/3 共享 Planner):

| System | 责任 | 触发 | 跟 LangGraph view 关系 |
|---|---|---|---|
| **SchedulerSystem** | Agent 调度 (Ready Queue + Priority) | 每 frame | 独立 (L0 L1 L2) |
| **LifecycleSystem** | HOT/WARM/COLD 状态转换 | 每 frame | 独立 (Runtime 概念) |
| **EventSystem** | Event 路由 + Mailbox 投递 | Event-driven | 跟 LangGraph 状态转换并行 |
| **PlannerSystem** | 任务规划 | 调度时 | 引用 LangGraph 9/3 planner node |
| **LlmSystem** | LLM 请求 + Token 计量 | LLM 调用时 | 独立 (L2 LLM Pool) |
| **ToolSystem** | Tool 调用 + 权限检查 | Tool 调用时 | 跟 LangGraph tool node 协作 |
| **McpSystem** | MCP 调用 | MCP 调用时 | 跟 LangGraph MCP node 协作 |
| **RetrievalSystem** | RAG 检索 | RAG 调用时 | 独立 (L2 RAG Pool) |
| **ContextSystem** | Context 装载/卸载 (Lazy Load) | Context 需要时 | 独立 (L2 Context Store) |
| **MemorySystem** | Memory 读写 | Memory 访问时 | 独立 (L2 Memory Store) |
| **PermissionSystem** | 权限检查 | 任何外部调用 | 跨域 (Tenant + ACL) |
| **PersistenceSystem** | Checkpoint + 持久化 | 任务状态变更 | 跨域 (L2 + DB) |
| **MetricsSystem** | 可观测性指标采集 | 每 frame | 独立 (L0 Observer) |

### 3.5 跟 22 domain-* crate 映射总表

| 22 domain-* crate | 映射到本 view | 状态 |
|---|---|---|
| `domain-task` | L0 TaskQueue | 🟡 部分 (守门 #20 brief 落档) |
| `domain-identity` | L1 AgentIdentity + Permission | 🟡 部分 |
| `domain-permission` | PermissionRef + PermissionSystem | 🟡 部分 |
| `domain-work-item` | L0 任务派发 + L1 状态机 | 🟡 部分 |
| `domain-workspace` | Tenant 隔离 | 🟡 部分 |
| `domain-worktree` | per-task worktree (git evidence) | ✅ (守门 #9) |
| `domain-llm` (待建) | L2 LLM Pool | ⏳ P3-C |
| `domain-mcp` (待建) | L2 MCP Pool | ⏳ P3-C |
| `domain-tool` (待建) | L2 Tool Registry | ⏳ P3-C |
| `domain-rag` (待建) | L2 RAG Pool | ⏳ P3-E |
| `domain-context` (待建) | L2 Context Store | ⏳ P3-D |
| `domain-memory` (待建) | L2 Memory Store | ⏳ P3-D |
| `domain-rate-limiter` (待建) | L0 RateLimiter | ⏳ P3-B |
| `domain-observability` (待建) | L0 Observer | ⏳ P3-B |
| 其他 8 domain-* | 引用但不重定义 | ✅ 现状 |

**新增 8 domain-* crate**: `domain-dispatcher` / `domain-llm` / `domain-mcp` / `domain-tool` / `domain-rag` / `domain-context` / `domain-memory` / `domain-rate-limiter` / `domain-observability`. **新增 9 个, 22+9 = 31 domain-* crate 目标**.

## 4. データモデル (Data Model)

### 4.1 ECS Component 详细定义

```rust
// per SRS §9 REQ-ECS-001 ~ §20 REQ-ECS-012
// 引用 SRS-001 §9-§20, 本节只列关键字段, 详细见 SRS

#[derive(Component)]
pub struct AgentIdentity {
    pub agent_id: AgentId,         // UUID
    pub tenant_id: TenantId,
    pub agent_type: AgentType,     // SA-01..SA-09 (引用 LangGraph 9/3 §6.1)
}

#[derive(Component)]
pub struct AgentState {
    pub current: AgentStateEnum,   // Idle / Ready / Scheduled / Planning / WaitingLlm / ...
    pub prev: AgentStateEnum,      // 用于状态机
    pub since: Instant,
}

#[derive(Component)]
pub struct LifecycleState {
    pub current: LifecycleEnum,    // Hot / Warm / Cold
    pub last_active: Instant,
    pub timeout: Duration,
}

#[derive(Component)]
pub struct ContextRef {
    pub context_id: ContextId,
    pub tier: ContextTier,         // L1 Hot / L2 Recent / L3 Full
    pub loaded: bool,
}

#[derive(Component)]
pub struct MemoryRef {
    pub memory_id: MemoryId,
    pub memory_type: MemoryType,   // Semantic / Episodic / User / Workflow / Knowledge
}

#[derive(Component)]
pub struct ModelRef {
    pub provider: String,          // "openai" / "anthropic" / ...
    pub model: String,             // "gpt-4" / "claude-3" / ...
    pub profile: String,           // "default" / "fast" / ...
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Component)]
pub struct ToolPolicyRef {
    pub policy_id: PolicyId,
    pub tool_allowlist: Vec<String>,
}

#[derive(Component)]
pub struct McpPolicyRef {
    pub policy_id: PolicyId,
    pub server_allowlist: Vec<String>,
}

#[derive(Component)]
pub struct PermissionRef {
    pub acl_id: AclId,
    pub tenant_id: TenantId,
}

#[derive(Component)]
pub struct TokenBudget {
    pub max_context_tokens: u32,
    pub max_output_tokens: u32,
    pub remaining_tokens: u32,
    pub cost_budget_cents: u32,
}

#[derive(Component)]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

#[derive(Component)]
pub struct MailboxRef {
    pub mailbox_id: MailboxId,
    pub unread_count: u32,
}

#[derive(Component)]
pub struct WorkflowRef {
    pub workflow_id: WorkflowId,
    pub step: u32,
    pub total_steps: u32,
}
```

### 4.2 Event 格式 (per SRS §27)

```rust
pub struct AgentEvent {
    pub event_id: EventId,                  // UUID
    pub source_agent_id: Option<AgentId>,
    pub target_agent_id: AgentId,
    pub tenant_id: TenantId,
    pub event_type: EventType,              // UserMessage / AgentMessage / ToolResult / LlmResult / McpEvent / Timer / WorkflowEvent / SystemEvent / ExternalEvent
    pub payload_ref: PayloadRef,            // 大消息存 Payload Store
    pub timestamp: SystemTime,
    pub trace_id: TraceId,
    pub correlation_id: Option<CorrelationId>,
}

pub enum EventType {
    UserMessage,
    AgentMessage,
    ToolResult,
    LlmResult,
    McpEvent,
    Timer,
    WorkflowEvent,
    SystemEvent,
    ExternalEvent,
}
```

### 4.3 Mailbox 消息格式 (per SRS §19 REQ-ECS-011)

```rust
pub struct MailboxMessage {
    pub msg_id: MsgId,
    pub event_id: EventId,
    pub payload_ref: PayloadRef,    // 大消息不内联
    pub timestamp: SystemTime,
    pub read: bool,
}
```

### 4.4 Payload 引用 (per SRS §28 REQ-PAYLOAD-001)

```rust
pub struct PayloadRef {
    pub store_id: StoreId,         // Payload Store 引用
    pub key: String,                // 内部 key
    pub size_bytes: u64,            // 大小 (用于 quota)
    pub ttl: Duration,              // 过期时间
}
```

### 4.5 Context / Memory / Token 配额格式

```rust
// Context Store (per SRS §36 REQ-CTX-001)
pub struct ContextMetadata {
    pub context_id: ContextId,
    pub tenant_id: TenantId,
    pub tier: ContextTier,          // L1 / L2 / L3
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub size_bytes: u64,
    pub refs: Vec<ContextRef>,      // 共享 Context + Agent Delta
}

// Memory Store (per SRS §41 REQ-MEM-001)
pub struct MemoryRecord {
    pub memory_id: MemoryId,
    pub memory_type: MemoryType,    // Semantic / Episodic / User / Workflow / Knowledge
    pub tenant_id: TenantId,
    pub agent_id: Option<AgentId>,
    pub content_ref: PayloadRef,    // 实际内容存 Payload Store
    pub embedding: Option<Vec<f32>>, // for semantic
    pub created_at: SystemTime,
}

// Token Budget
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_cents: u32,
    pub timestamp: SystemTime,
}
```

## 5. インターフェース設計 (Interface Design)

### 5.1 Runtime API (per SRS §85 REQ-API-001)

```rust
// Agent 生命周期 API
pub trait RuntimeApi {
    async fn create_agent(&self, identity: AgentIdentity) -> Result<AgentId>;
    async fn delete_agent(&self, agent_id: AgentId) -> Result<()>;
    async fn get_agent(&self, agent_id: AgentId) -> Result<Agent>;
    async fn send_event(&self, event: AgentEvent) -> Result<EventId>;
    async fn suspend_agent(&self, agent_id: AgentId) -> Result<()>;
    async fn resume_agent(&self, agent_id: AgentId) -> Result<()>;
    async fn cancel_agent(&self, agent_id: AgentId) -> Result<()>;
    async fn get_agent_state(&self, agent_id: AgentId) -> Result<AgentStateSnapshot>;
    async fn get_runtime_metrics(&self) -> Result<RuntimeMetrics>;
}
```

### 5.2 Management API (per SRS §86 REQ-API-MGMT)

```rust
pub trait ManagementApi {
    async fn get_runtime_mode(&self) -> Result<RuntimeMode>;        // Lightweight / ECS
    async fn get_threshold(&self, key: &str) -> Result<ConfigValue>;
    async fn get_hot_limit(&self) -> Result<u32>;
    async fn get_memory_limit(&self) -> Result<MemoryLimit>;
    async fn get_queue_limit(&self) -> Result<QueueLimit>;
    async fn get_tenant_quota(&self, tenant_id: TenantId) -> Result<TenantQuota>;
    async fn get_provider_limit(&self, provider: &str) -> Result<ProviderLimit>;
    // 动态修改是否允许: 基本设计阶段确定 (本 view: 仅 GET, 动态修改待 P3-E 决定)
}
```

### 5.3 Event Bus 接口 (per SRS §13-§15)

```rust
pub trait EventBus {
    async fn publish(&self, event: AgentEvent) -> Result<EventId>;
    async fn subscribe(&self, agent_id: AgentId) -> Result<MailboxStream>;
    async fn unsubscribe(&self, agent_id: AgentId, sub_id: SubId) -> Result<()>;
}

pub trait EventRouter {
    async fn route(&self, event: AgentEvent) -> Result<AgentId>;  // 路由到 target agent
}
```

### 5.4 调度器接口 (per SRS §42 REQ-SCH-001)

```rust
pub trait Scheduler {
    async fn schedule(&self, task: Task) -> Result<ScheduleId>;
    async fn cancel(&self, schedule_id: ScheduleId) -> Result<()>;
    async fn get_ready_queue(&self) -> Result<Vec<Task>>;
    async fn get_hot_slot_usage(&self) -> Result<f32>;          // 0.0-1.0
    async fn get_agent_wait_time(&self, agent_id: AgentId) -> Result<Duration>;
}
```

### 5.5 Lifecycle Manager 接口 (per SRS §25 REQ-LC-MGR)

```rust
pub trait LifecycleManager {
    async fn transition(&self, agent_id: AgentId, target: LifecycleState) -> Result<()>;
    async fn persist(&self, agent_id: AgentId) -> Result<()>;
    async fn restore(&self, agent_id: AgentId) -> Result<Agent>;
    async fn timeout(&self, agent_id: AgentId) -> Result<bool>;
    async fn evict(&self, agent_id: AgentId) -> Result<()>;
}
```

## 6. NFR (Non-Functional Requirements)

### 6.1 性能 NFR (per SRS §72-§74)

| NFR | 目标 | 测量 |
|---|---|---|
| **Logical Agent 数** | 1,000,000 | 单元 / Integration / E2E |
| **HOT Agent 数** | 1,000-5,000 | 同上 |
| **WARM Agent 内存** | < 100 KB / Agent, 优化 10-50 KB | `avg_warm_agent_bytes` 指标 |
| **COLD Agent 内存** | ≈ 0 Runtime RAM | 同上 |
| **Total Runtime RAM (1M logical)** | < 16 GB | 16GB 机器 87 小时派发 (per `1m-orchestrator-l0-l1.html` §3) |
| **L0 派发延迟** | < 100ms p95 | schedule_latency 指标 |
| **L1 ECS 状态转移** | < 1ms p95 | system_latency 指标 |
| **L2 Pool 复用率** | > 90% | pool_utilization 指标 |
| **Throughput** | 200 task/s 持续 (16GB 机器) | tasks_per_sec 指标 |

### 6.2 安全 NFR (per SRS §45-§47 REQ-SEC-001)

| NFR | 措施 |
|---|---|
| **Tenant Isolation** | 22 domain-* per-tenant ACL + RLS 13 類 (per 守门 #13 DB W/T/M) |
| **Tool Permission** | ToolPolicyRef + ToolSystem 统一检查 (per SRS §46) |
| **MCP Permission** | McpPolicyRef + McpSystem 统一检查 |
| **Model Permission** | Provider Quota + ModelRef 验证 |
| **Context Permission** | ContextRef + Tenant 隔离 |
| **Memory Permission** | MemoryRef + Tenant 隔离 |
| **Rate Limit** | Token Bucket per tenant / per agent |
| **Resource Quota** | Tenant Quota (per SRS §43) |
| **Secret 安全** | SecretRef 模式, **不**直接保存 API Key (per SRS §47 + 守门 #5 11:06 JST hard ban) |

### 6.3 可用性 NFR (per SRS §40-§42)

| NFR | 措施 |
|---|---|
| **Crash Recovery** | Persistent State + Restore (per SRS §41 REQ-RECOVERY-001) |
| **Checkpoint** | 任务 step-by-step checkpoint (per SRS §42 REQ-CHECKPOINT-001) |
| **Retry** | max_retry + backoff (per SRS §38 REQ-RETRY-001) |
| **Idempotent** | operation_id + task_id + correlation_id (per SRS §39 REQ-IDEMPOTENT-001) |
| **Persist** | AgentIdentity / State / ContextRef / MemoryRef / TokenBudget / Pending Events (per SRS §40 REQ-PERSIST-001) |

### 6.4 可观测性 NFR (per SRS §61-§62)

| NFR | 措施 |
|---|---|
| **Metrics** | logical_agent_count / resident_agent_count / hot/warm/cold_agent_count / runtime_mode / rss_bytes / heap_bytes / shared_runtime_bytes / ecs_world_bytes / context_cache_bytes / ready_queue_depth / hot_slot_usage / agent_wait_time / llm_latency / tool_latency (per SRS §61 REQ-OBS-001) |
| **Trace** | trace_id 贯穿 Event / Agent / Scheduler / LLM / RAG / Tool / MCP / Persistence (per SRS §62 REQ-TRACE-001) |
| **STAR 现状** | 缺 (per §7 v0.8 token 缺数据), P3-B telemetry 落地 (per G-9) |
| **派发速率** | 16 worker = 3.2 task/s, 32 worker = 6.4 task/s, 200 worker = 40 task/s |

## 7. 守門規則統合 (per AGENTS.md §4 + §4.1 累積規 v1-v24)

本 view 落地需满足 24 项守门 + 24 条累积规. 关键约束 (per [SRS-001 §5](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md)):

| 守门 | 关键内容 | 状态 (本 view) |
|---|---|---|
| #1 | cargo check --workspace --all-targets 0 err | ✅ 41/41 crate 100% 覆盖 (per §7 v0.9) |
| #3 | 5 域独立 Lead, 不接受兼任 | ✅ (本 view 不涉及 5 域映射) |
| #5 | 环境变量安全 (11:06 JST hard ban) | ✅ |
| #6 | PowerShell only | ✅ |
| #7 | 0 unsafe (代码守门) | ✅ |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ (P3-A 25 实证) |
| #12 | 缺标比错标安全 (8/26 拍板) | ✅ |
| #19 | agent 交互 Python 化守门 (9/2 拍板) | ✅ (P3-B L0 dispatcher.py 是 Python, P3-C L1 ECS 是 Rust) |
| #21 | [P] 子项 docs 同步必更新 automation-design §4 + registry.md | ✅ (本 view 落地后 §4.14 + §5.2 追加) |
| #24 | 调试控制台走 subprocess 替代 RPC | ✅ |
| #DB-13 | DB 三類横展開 (W/T/M) 強制分類 (9/1 拍板) | ⏳ P3-D |

**完整 24 + 24 见 AGENTS.md §4 + §4.1.**

## 8. 子エージェント失敗接手 (per 7 子代理派生规则)

| # | 子代理 | 失败模式 | 接手方案 |
|---|---|---|---|
| 1 | **worker** (Mavis dispatcher) | RPC 不可靠 (守门 #9 实证 10/10 失败) | subprocess.run 替代 (守门 #24) |
| 2 | **explorer** | 跨文件 mapping 上下文爆 | 拆任务 + 短 brief |
| 3 | **verifier** | 验证标准歧义 | 显式列 AC + 已知缺口 |
| 4 | **mavis** | 大跨度编排上下文爆 | 阶段化 + token 预算 |
| 5 | **brief 落地** | dispatcher.py brief() 异常 | retry 3x + 死信 (L0 RetryPolicy) |
| 6 | **commit 归因** | git -c user.name 失败 | parent 进程代签 (per 19:39 JST) |
| 7 | **守门 check** | 守门 #1-#24 任一违反 | 阻塞 commit + 报告 |

**派生 (per 守门 #9 主体规则)**: 子代理 status="succeeded" ≠ 实际成功, 必须 `git log -p --follow <wt-branch>` 实证.

## 9. 既知缺口 (Known Gaps, per 守門 #12 缺標比錯標)

引 [SRS-001 §3 G-1~G-12](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md), 关键 5 项 + 3 项新加:

| # | 缺口 | 影响 | 阶段 | 验证 |
|---|---|---|---|---|
| G-1 | L0 SQLite 任务队列未落地 | 1M 派发无持久化 | P3-B L0 PoC | DDD Review |
| G-2 | L1 bevy_ecs / flecs 选型未启 | 9 SA ECS 无运行时 | P3-B 启动 | P3-B 拍板 |
| G-3 | EventBus + Mailbox 未实现 | Agent 间通信无协议 | P3-B | P3-B 拍板 |
| G-4 | Shared LLM/HTTP/MCP Pool 未落地 | 守门 #24 subprocess 池 ≠ ECS 池 | P3-C | P3-C 拍板 |
| G-7 | Crash Recovery + Checkpoint | 任务卡恢复无协议 | P3-D | P3-D 拍板 |
| G-9 | Token 计量 telemetry | 真实数据缺, 改 commit 数 | P3-B telemetry | P3-B 拍板 |
| G-10 | 守门 #1 v18 H2 跨 session 续 | H2 5 domain 类型不兼容 | P3-B 跨 session | DDD Review |
| G-11 | 5 域 Lead 真人 | Mavis 临时代签 (per 守门 #3 反转 B 11:35 JST) | DDD Review | DDD Review |
| **G-13 (新)** | **本 view 详细测试 9 SA Type × ECS Archetype 映射** | 9 SA 业务逻辑 跟 ECS Component 类型兼容性 | P3-B 实装 | P3-B DDD Review |
| **G-14 (新)** | **Process Pool 跟 Tokio 协作的 runtime 隔离** | 8-16 worker 跟 L1 ECS 怎么切换, 需轻量级 runtime 设计 | P3-B L0 PoC | P3-B L0 PoC 实证 |
| **G-15 (新)** | **Tenant Quota 跟 Priority 冲突解决** | 9 SA Archetype 共享 quota 还是 per-archetype quota | P3-D | P3-D 拍板 |

## 10. 签字栏 (5 角色, per 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |

**per 2026-09-03 19:00 JST Ulysses 授权** (默认代签规则 per 19:39 JST + 07:16 JST 反转 + 21:59 JST 第三次强化).

## 11. 修订履歴

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 引用 LangGraph 9/3 02 范式, 落档 Agent Runtime 基本设计; 3 层架构 (L0 派发 + L1 ECS + L2 业务) + Runtime 双模式 + 9 SA Type 引用 + 31 domain-* 目标 + 13 Systems + NFR 性能/安全/可用性/可观测性 + 守门 24 + G-1~G-15 已知缺口 | 2026-09-03 18:48 JST 用户发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步" |

---

## 12. 参考 (Reference)

- [SRS-STAR-AGENT-RUNTIME-001.md v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) (53KB / 113 节, commit `5460d33`)
- [ADR-0044 STAR Agent Runtime SRS Baseline](../2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md) (12KB, commit `5460d33`)
- [docs/architecture/2026-09-03-langgraph/01-requirements.md](../2026-09-03-langgraph/01-requirements.md) §6.1 9 SA Type (引用, 不重写)
- [docs/architecture/2026-09-03-langgraph/02-basic-design.md](../2026-09-03-langgraph/02-basic-design.md) §3-§4 (引用, 不重写)
- [docs/architecture/2026-09-03-langgraph/03-detailed-design.md](../2026-09-03-langgraph/03-detailed-design.md) (LangGraph view 詳細設計, 引用)
- [docs/architecture/preview/1m-orchestrator-l0-l1.html](../preview/1m-orchestrator-l0-l1.html) (1M 派发架构图预览, 16GB 内存账)
- [AGENTS.md §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §5 仓库拓扑 + §6 ADR 索引 + §7 待办](../../AGENTS.md)
- [docs/automation-design.md](../../automation-design.md) §1-§4 任务卡 + §4.13 SRS Baseline
- [docs/automation-design.md §4.14 追加 (本 view 落地后)](../../automation-design.md)
- [scripts/automation/registry.md §5.1 SRS Baseline 索引 + §5.2 (本 view 落地后)](../../../scripts/automation/registry.md)
- [STAR-OLU-001.md v0.1](../../STAR-OLU-001.md) (1 SRE·周 = 1.2M tokens 独立基线)
- [STAR-P3-WBS-001.md v0.6 §7 阻塞 7 项](../../docs/STAR-P3-WBS-001.md) (P3-B 启动前置)
- [HANDOFF-ST-001.md v0.4 §5.3 5 Blocker](../../docs/reports/HANDOFF-ST-001.md) (跨 session 续)

---

# === 基本設計書 結束 ===
