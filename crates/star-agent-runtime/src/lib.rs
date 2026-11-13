/*
@cypher schema=1 source_sha256=fd54a4aef2e241ba4cf8c28a35903209dd323a5f43c5fc3345055bfe60973065
MERGE (self:File {path:"crates/star-agent-runtime/src/lib.rs"})
@endcypher
*/
//! star-agent-runtime — STAR Agent Runtime ECS 骨架 (WBS 6.2)
//!
//! 按 [`SRS-STAR-AGENT-RUNTIME-001`] + 02-basic-design / 03-detailed-design 落地
//! Rust 侧骨架 (大世界 / 高性能模式), 与 Python 侧
//! `scripts/automation/agent_runtime/` (LangGraph 任务图模式) 构成双模式 Runtime。
//!
//! 设计约束 (per SRS):
//! - **Agent = Identity + State + References + Policies** (Agent ≠ Process / Thread /
//!   Runtime, per SRS §110)。组件只放引用, 不放大对象 (per SRS §70)。
//! - **Hybrid Runtime**: Lightweight < 10 / ECS ≥ 12 + 迟滞区 10-11 (per SRS §7)。
//! - **ECS ≠ 游戏 ECS**: 围绕 State / Lifecycle / Event / Scheduler / Resource /
//!   Policy / Query 设计 (per SRS §69)。本骨架为「自研 Minimal ECS」
//!   (03-DD §1.3 备选), bevy_ecs / flecs 选型待 P3-B 拍板后替换 `ecs.rs` 存储层。
//! - **Bounded everything**: 队列/并发/HOT 槽位有界, 禁止无界 (per SRS §44/§47)。
//!
//! 已知缺口 (per 守门 #12 缺标比错标, 显式列):
//! - G-2: bevy_ecs / flecs 选型未启 → 本骨架自研 Minimal ECS 占位
//! - G-3: 真实跨进程 EventBus / 持久化 → 本骨架 in-memory
//! - G-4: Shared LLM/HTTP/MCP Pool → P3-C (本骨架仅接口边界)
//! - G-7: Crash Recovery + Checkpoint → P3-D (本骨架仅 CheckpointId 类型)
//! - Tokio 异步执行层: 骨架为同步纯逻辑, async 接线 (per SRS §45) 待 P3-B
//!
//! 守门: `#![forbid(unsafe_code)]` (per 守门 #7 0 unsafe) + `#![deny(missing_docs)]`
//! (对齐根 workspace lints)。
//!
//! [`SRS-STAR-AGENT-RUNTIME-001`]: ../../docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod agent_state;
pub mod agent_type;
pub mod backpressure;
pub mod components;
pub mod config;
pub mod ecs;
pub mod event;
pub mod id;
pub mod identity;
pub mod lifecycle;
pub mod metrics;
pub mod rate_limiter;
pub mod retry;
pub mod runtime_mode;
pub mod scheduler;
pub mod task;

pub use agent_state::{AgentState, AgentStateEnum};
pub use agent_type::{AgentType, SA_TYPES};
pub use backpressure::{Backpressure, BackpressureDecision, BackpressureError, OverflowPolicy};
pub use components::{
    ContextRef, ContextTier, MailboxRef, McpPolicyRef, MemoryRef, MemoryType, ModelRef,
    PermissionRef, Priority, RateLimit, TokenBudget, ToolPolicyRef, WorkflowRef,
};
pub use config::{ConfigError, RuntimeConfig};
pub use ecs::{ArchetypeId, Component, Stored, World};
pub use event::{
    AgentEvent, EventBus, EventBusError, EventRouter, EventType, InMemoryEventBus, MailboxMessage,
    PayloadRef,
};
pub use id::{
    AclId, AgentId, CheckpointId, ContextId, CorrelationId, EventId, MailboxId, MemoryId, MsgId,
    ParseIdError, PolicyId, TaskId, TenantId, TraceId, WorkflowId,
};
pub use identity::AgentIdentity;
pub use lifecycle::{LifecycleError, LifecycleManager, LifecycleState, LifecycleStateEnum};
pub use metrics::{InMemoryObserver, Observer, RuntimeMetrics};
pub use rate_limiter::{RateLimiterError, TenantRateLimiter, TokenBucket};
pub use retry::{RetryError, RetryPolicy};
pub use runtime_mode::{RuntimeMode, RuntimeModeConfig, RuntimeModeError, RuntimeModeManager};
pub use scheduler::{ScheduleOutcome, Scheduler, SchedulerError};
pub use task::{Task, TaskStatus};
