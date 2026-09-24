//! PI-9 Steering / Follow-up / QueueMode (domain-agent) — W1 `Steering` loop wiring.
//!
//! Per SRS-PI-BORROW-001 §1.3 + §4 FR-30 (`crates/pi-agent-core/src/types.ts`
//! lines 47-55, 278-302 翻译).
//!
//! 本文件范围 = **接口骨架 + FIFO/Priority/Interrupt 三态的 in-memory 调度 +
//! `Steering::apply_steering` 接 PI-3 `AgentLoopBoundary::transform_context` 的
//! 默认实装**:
//!
//! 1. 三个 trait `Steering` / `FollowUp` / `AgentQueue`
//! 2. `QueueMode` 枚举（FIFO / Priority / Interrupt）
//! 3. `QueuedTask` POD 持有待派发的提示
//! 4. `InMemoryAgentQueue` 实现 AgentQueue trait，提供最简调度逻辑
//! 5. `LoopBoundarySteeringHook` = Steering trait 默认实装，把 steering hint
//!    注入 `InternalContext.messages`（用预留的 `InternalRole::Steering` 变体）
//!    后调 `AgentLoopBoundary::transform_context`，真正接入 PI-3 (ULYS-204)
//!    ship 的 loop seam。`NoopSteering` 为无 boundary 场景的退化路径（回
//!    passthrough，不变 context），向后兼容。
//!
//! ## 已知缺口 / 待 PI-* 落地后补
//!
//! - **PI-3 ✅ 已 ship (本 branch cherry-pick `c4145379`)**: `AgentLoopBoundary`
//!   trait + `StreamFn` no-throw 已在本 worktree 可用。本文件 `Steering` 默认
//!   实装就是接 `transform_context`。D-Boy 「走 a」选项兑现。
//! - **PI-4 缺口**: `Tool` trait 5 方法未 ship → 本文件不 import
//!   `domain_tool::Tool`，QueuedTask 用 String 工具名占位；待 PI-4 ship 后
//!   把 String → ToolInvocation 改字段。
//! - **PI-6 缺口**: JSON Delta 协议未 ship → 本文件用 serde_json::Value 描述
//!   task，待 PI-6 ship 后切到 `star_dto::JsonDeltaPayload`。
//! - **跨域协作评论**: SRS-MULTICA-COLLABORATION 协作评论机制未 ship，
//!   `SteeringHook::from_collaboration_comment` = 未在本文件实现（等跨域
//!   Lead 对齐再补接口签名）。
//!
//! 完整 3 周 MVP 工时排程见 issue description §4。
//! 启动条件（per §5）：Stage 4 + PI-6 (ULYS-206 redesign) 已 ship + D-Boy 拍板
//! PI-9 启动。本文件 commit = "W1-Steering-loop-wiring gated by PI-4/PI-6/
//! Collab";PI-4/PI-6/Collab 全 ship 后再 sign off "PI-9 done".
//!
//! ## 守门合规
//!
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - `#[non_exhaustive]` on enums that may grow (per SRS-PI-BORROW-001 NFR-7).
//! - All public items documented (`missing_docs = "deny"` workspace lint).
//! - No reverse dependency on `crates/api` (per 守门 #1 v15 分层 + 守门 #13 d).
//! - 守门 #5: cross-tenant steering 在 `LoopBoundarySteeringHook::apply_steering`
//!   第一步校验 actor_tenant == boundary_ctx.tenant_id，不匹配 → 立即回
//!   `CrossTenantSteeringDenied` 错误（不绕开 boundary）。
//! - 守门 #13 d: hint 注入后通过 `boundary_ctx.audit_sink.emit(...)` 触发
//!   audit event（即使 transform_context 失败也记 "steering-attempted"）。

use std::collections::{BinaryHeap, VecDeque};
use std::sync::Mutex;
use std::time::SystemTime;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{AgentId, AgentSessionId, TenantId};
use crate::loop_boundary::{
    AgentLoopBoundary, InternalContext, InternalMessage, InternalRole, LoopBoundaryContext,
    TransformOutcome,
};

// =====================================================================
// 错误
// =====================================================================

/// PI-9 queue 层错误
#[derive(Debug, Error)]
pub enum AgentQueueError {
    #[error("queue is empty (no task available)")]
    /// 取任务时空队列
    Empty,
    #[error("invalid priority: {0} (must be 0..={MAX_PRIORITY})")]
    /// 优先级非法（< 0 或 > MAX_PRIORITY）
    InvalidPriority(i32),
    #[error("cross-tenant steering denied: actor tenant {actor} vs required {required}")]
    /// 跨租户 steering 拒绝
    CrossTenantSteeringDenied {
        /// 调用方租户
        actor: TenantId,
        /// 必须匹配租户
        required: TenantId,
    },
    #[error("steering transform failed: {0}")]
    /// Steering 注入 transform_context 失败
    SteeringTransformFailed(String),
    #[error("queue internal error: {0}")]
    /// 内部错误
    Internal(String),
}

/// AgentQueue Result 别名
pub type AgentQueueResult<T> = Result<T, AgentQueueError>;

/// 最大优先级（含）
pub const MAX_PRIORITY: i32 = 9;
/// 最小优先级（含）
pub const MIN_PRIORITY: i32 = 0;
/// 中等优先级（FIFO 默认）
pub const DEFAULT_PRIORITY: i32 = 5;

// =====================================================================
// QueueMode -- 队列调度模式（FIFO / Priority / Interrupt）
// =====================================================================

/// **QueueMode** -- 队列调度模式 (SRS-PI-BORROW-001 §1.3 + pi-agent-core/src/types.ts:47-55).
///
/// - `Fifo` (default): 严格先进先出，按 `enqueue` 时间顺序派发。
/// - `Priority`: 按优先级降序派发，相同优先级按 FIFO tie-break。
/// - `Interrupt`: 新任务插队到队首，正在执行的当前任务被抢占（抢占语义
///   由 caller/loop driver 处理；queue 只负责把 Interrupt 任务放最前）。
///
/// `#[non_exhaustive]` 允许未来加新模式（如 `StrictlySerial` / `BatchWindow`）
/// 而不破坏 NFR-4 接口稳定承诺。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum QueueMode {
    /// 先进先出（默认）
    Fifo,
    /// 按优先级降序派发
    Priority,
    /// 抢占式（队首插队）
    Interrupt,
}

impl Default for QueueMode {
    fn default() -> Self {
        Self::Fifo
    }
}

// =====================================================================
// QueuedTask -- 队列任务 POD
// =====================================================================

/// **QueuedTask** -- 队列里的单个待派发任务 (per pi-agent-core/src/types.ts:278-302).
///
/// 不耦合 PI-4 `Tool` trait / PI-6 JSON Delta 协议（占位字段）：
/// - `tool_hint`: `String` 占位，待 PI-4 ship 后收紧为 `ToolInvocation`；
/// - `payload`: `serde_json::Value` 占位，待 PI-6 ship 后收紧为
///   `star_dto::JsonDeltaPayload`。
///
/// `#[non_exhaustive]` 允许 PI-4 / PI-6 ship 后加新字段而不破坏 NFR-4。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct QueuedTask {
    /// 任务唯一 ID
    pub task_id: Uuid,
    /// 所属租户
    pub tenant_id: TenantId,
    /// 目标 Agent
    pub agent_id: AgentId,
    /// 目标 Session
    pub session_id: AgentSessionId,
    /// 用户提示原文
    pub prompt: String,
    /// 工具调用 hint（PI-4 ship 后收紧）
    pub tool_hint: String,
    /// 任务载荷（PI-6 ship 后收紧）
    pub payload: serde_json::Value,
    /// 优先级（`MIN_PRIORITY`..=`MAX_PRIORITY`）
    pub priority: i32,
    /// 队首插队标记（`Interrupt` 模式专用）
    pub interrupt: bool,
    /// 入队时间戳（SystemTime for ordering）
    pub created_at: SystemTime,
}

impl QueuedTask {
    /// 创建新任务（不带 interrupt 标记）
    pub fn new(
        tenant_id: TenantId,
        agent_id: AgentId,
        session_id: AgentSessionId,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            task_id: Uuid::new_v4(),
            tenant_id,
            agent_id,
            session_id,
            prompt: prompt.into(),
            tool_hint: String::new(),
            payload: serde_json::Value::Null,
            priority: DEFAULT_PRIORITY,
            interrupt: false,
            created_at: SystemTime::now(),
        }
    }

    /// 构造为 Interrupt（抢占）任务
    pub fn new_interrupt(
        tenant_id: TenantId,
        agent_id: AgentId,
        session_id: AgentSessionId,
        prompt: impl Into<String>,
    ) -> Self {
        let mut t = Self::new(tenant_id, agent_id, session_id, prompt);
        t.interrupt = true;
        t
    }

    /// 校验 priority 合法
    pub fn validate_priority(p: i32) -> AgentQueueResult<()> {
        if !(MIN_PRIORITY..=MAX_PRIORITY).contains(&p) {
            return Err(AgentQueueError::InvalidPriority(p));
        }
        Ok(())
    }
}

// =====================================================================
// Steering trait -- 用户中途插话引导 agent
// =====================================================================

/// **Steering** -- 用户在 agent 当前任务中途插话，引导方向
/// (per SRS-PI-BORROW-001 §1.3 + §4 FR-30 +
/// `pi-agent-core/src/types.ts:47-55, 218-241`).
///
/// 实现需把 `hint` 注入 agent 当前的 `InternalContext`，并返回更新后的
/// context（`TransformOutcome`），供 loop driver 继续推进。
///
/// 默认实装 = [`LoopBoundarySteeringHook`]（接 PI-3 `AgentLoopBoundary`
/// `transform_context`）；`NoopSteering` 退化回 `Ack` 不改 context。
#[async_trait]
pub trait Steering: Send + Sync {
    /// **apply_steering** -- 把 hint 注入 context，返回更新后的 outcome。
    ///
    /// - `actor_tenant`: 调用方租户（用于守门 #5 cross-tenant 拒绝）；
    /// - `boundary_ctx`: PI-3 `LoopBoundaryContext`（actor / agent / session /
    ///   provider / policy_hooks / audit_sink）；
    /// - `base_context`: 当前 InternalContext（可能为空，但通常由 loop driver
    ///   持有）；
    /// - `hint`: steering 文本（用户在协作评论 / UI 输入的指令）。
    ///
    /// 实现必须：
    /// 1. 校验 actor_tenant == boundary_ctx.tenant_id（守门 #5）；
    /// 2. 把 hint 注入 `InternalContext.messages`（用预留的
    ///    `InternalRole::Steering` 变体）；
    /// 3. 调 `boundary.transform_context(...)` 拿到 `TransformOutcome`；
    /// 4. 触发 audit event（守门 #13 d，通过 `boundary_ctx.audit_sink`）。
    async fn apply_steering(
        &self,
        actor_tenant: TenantId,
        boundary_ctx: &LoopBoundaryContext,
        base_context: InternalContext,
        hint: &str,
    ) -> AgentQueueResult<TransformOutcome>;

    /// **last_steering** -- 返回最近一次成功注入的 hint（用于回放 / 调试）。
    ///
    /// 默认返回 `None`；实现可缓存最后一次 hint。
    async fn last_steering(&self) -> Option<String>;
}

// =====================================================================
// LoopBoundarySteeringHook -- 接 PI-3 transform_context 的实装
// =====================================================================

/// **LoopBoundarySteeringHook** -- `Steering` trait 的默认实装，把 hint 注入
/// `InternalContext` 后调 PI-3 `AgentLoopBoundary::transform_context`
/// 拿到更新后的 `TransformOutcome`。
///
/// 持有 `Arc<dyn AgentLoopBoundary>`（cheap clone）+ `Mutex<Option<String>>`
/// 用于 last_steering 缓存（守门 #5 不要求审计外暴露缓存）。
///
/// 设计要点：
/// - **守门 #5 cross-tenant**: `apply_steering` 第一步校验
///   `actor_tenant == boundary_ctx.tenant_id`，不匹配 → 立即回
///   `CrossTenantSteeringDenied` 错误（不绕开 boundary）；
/// - **守门 #13 d audit**: hint 注入后通过 `boundary_ctx.audit_sink.emit(...)`
///   触发 audit event（即使 transform_context 失败也记 "steering-attempted"）；
/// - **PI-3 transform_context**: hint 作为新的
///   `InternalMessage { role: Steering, content: hint }` append 到
///   `base_context.messages`，再调 boundary.transform_context；boundary
///   的 transform_context 实现（如 `DefaultLoopBoundary`）可能 prune /
///   summarize，输出 `TransformOutcome.context` 已处理过；
/// - **PI-3 4 方法接缝**: 选 `transform_context` 是因为 steering 语义 =
///   "修改即将送给 LLM 的 context"。`before_tool_call` / `after_tool_call`
///   是 PI-5 PolicyHook 路径（已 ship），不在 PI-9 scope；
/// - **`InternalRole::Steering` 复用**: PI-3 的 `InternalRole` enum 已预留
///   `Steering` 变体（loop_boundary.rs:174 注释 "PI-9 deferred"），本文件
///   直接使用，避免引入新 role enum 重复定义（守门 #11 缺标比错标）。
pub struct LoopBoundarySteeringHook {
    boundary: std::sync::Arc<dyn AgentLoopBoundary>,
    last_hint: Mutex<Option<String>>,
}

impl LoopBoundarySteeringHook {
    /// 构造一个新 hook，绑定到具体 `AgentLoopBoundary` 实例。
    pub fn new(boundary: std::sync::Arc<dyn AgentLoopBoundary>) -> Self {
        Self {
            boundary,
            last_hint: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Steering for LoopBoundarySteeringHook {
    async fn apply_steering(
        &self,
        actor_tenant: TenantId,
        boundary_ctx: &LoopBoundaryContext,
        base_context: InternalContext,
        hint: &str,
    ) -> AgentQueueResult<TransformOutcome> {
        // 1. 守门 #5: 跨租户拒绝
        if actor_tenant.as_uuid() != boundary_ctx.tenant_id {
            return Err(AgentQueueError::CrossTenantSteeringDenied {
                actor: actor_tenant,
                required: TenantId::from(boundary_ctx.tenant_id),
            });
        }

        // 2. 构造注入消息（PI-3 InternalRole::Steering 预留）
        let hint_msg = InternalMessage {
            id: Uuid::new_v4(),
            role: InternalRole::Steering,
            content: hint.to_string(),
            tool: None,
            seq: base_context.messages.len() as u64,
        };
        let mut extended = base_context;
        extended.messages.push(hint_msg);

        // 3. audit: 即使 transform 失败也要记录尝试（守门 #13 d）
        let _ = boundary_ctx
            .audit_sink
            .write(crate::policy_hooks::AuditEventSpec::new(
                boundary_ctx.agent_id,
                boundary_ctx.actor.user_id,
                format!(
                    "steering.apply tenant={} session={} hint_len={}",
                    boundary_ctx.tenant_id,
                    boundary_ctx.session_id,
                    hint.len()
                ),
            ));

        // 4. 调 PI-3 transform_context（FR-14）：让 boundary 决定是否 prune /
        //    summarize / pass-through
        let outcome = self
            .boundary
            .transform_context(boundary_ctx, extended)
            .await;

        // 5. 缓存 last_hint（即使 transform_context 被 boundary 改动 messages
        //    也算成功）
        if let Ok(mut g) = self.last_hint.lock() {
            *g = Some(hint.to_string());
        }
        // TransformOutcome 是 struct 而非 Result；默认不返回错误（trait
        // default impl 是 infallible），但保留 try_into 路径以便未来
        // BoundaryError 接缝扩展。
        Ok(outcome)
    }

    async fn last_steering(&self) -> Option<String> {
        self.last_hint.lock().ok().and_then(|g| g.clone())
    }
}

// =====================================================================
// NoopSteering -- 无 boundary 场景的退化实装（向后兼容）
// =====================================================================

/// **NoopSteering** -- 不接任何 boundary，只回 `Ack` 不动 context。
///
/// 用于：
/// - 测试场景（不需要 loop 接线）；
/// - bootstrap / 退化路径（boundary 尚未 ship 或暂时不可用）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSteering;

#[async_trait]
impl Steering for NoopSteering {
    async fn apply_steering(
        &self,
        _actor_tenant: TenantId,
        _boundary_ctx: &LoopBoundaryContext,
        base_context: InternalContext,
        hint: &str,
    ) -> AgentQueueResult<TransformOutcome> {
        let _ = hint;
        Ok(TransformOutcome::passthrough(base_context))
    }

    async fn last_steering(&self) -> Option<String> {
        None
    }
}

// =====================================================================
// FollowUp trait -- 当前任务完成后自动排队后续任务
// =====================================================================

/// **FollowUp** -- 当 agent 当前任务完成后，自动排队后续任务
/// (per SRS-PI-BORROW-001 §1.3 + pi-agent-core/src/types.ts:278-302).
///
/// 实现需把 followup task 入 `AgentQueue`，由 loop driver 在当前任务
/// `Done` 事件触发后自动派发下一个 followup。
#[async_trait]
pub trait FollowUp: Send + Sync {
    /// **enqueue_followup** -- 把 followup task 入 queue。
    ///
    /// - `actor_tenant`: 调用方租户；
    /// - `queue`: 目标 queue（实现内部持有）；
    /// - `task`: 待入队的 followup。
    async fn enqueue_followup(
        &self,
        actor_tenant: TenantId,
        queue: &dyn AgentQueue,
        task: QueuedTask,
    ) -> AgentQueueResult<()>;

    /// **cancel_followup** -- 取消 followup task（按 task_id 查 + 删）。
    async fn cancel_followup(
        &self,
        actor_tenant: TenantId,
        queue: &dyn AgentQueue,
        task_id: Uuid,
    ) -> AgentQueueResult<()>;
}

/// **DefaultFollowUp** -- 默认实装：直接转给 `AgentQueue::enqueue` /
/// `AgentQueue::cancel`。跨租户守门由 queue 内部执行。
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultFollowUp;

#[async_trait]
impl FollowUp for DefaultFollowUp {
    async fn enqueue_followup(
        &self,
        actor_tenant: TenantId,
        queue: &dyn AgentQueue,
        task: QueuedTask,
    ) -> AgentQueueResult<()> {
        if actor_tenant != task.tenant_id {
            return Err(AgentQueueError::CrossTenantSteeringDenied {
                actor: actor_tenant,
                required: task.tenant_id,
            });
        }
        queue.enqueue(task, QueueMode::default()).await
    }

    async fn cancel_followup(
        &self,
        actor_tenant: TenantId,
        queue: &dyn AgentQueue,
        task_id: Uuid,
    ) -> AgentQueueResult<()> {
        queue.cancel(actor_tenant, task_id).await
    }
}

// =====================================================================
// AgentQueue trait -- 队列操作
// =====================================================================

/// **AgentQueue** -- 任务队列抽象 (per pi-agent-core/src/types.ts:278-302).
///
/// 4 个方法：`enqueue` / `take_next` / `cancel` / `len`。
#[async_trait]
pub trait AgentQueue: Send + Sync {
    /// 入队（按 queue.mode 决定调度）
    async fn enqueue(&self, task: QueuedTask, mode: QueueMode) -> AgentQueueResult<()>;

    /// 取下一个任务（FIFO/Priority: pop front; Interrupt: pop front 不变语义）
    async fn take_next(&self) -> AgentQueueResult<QueuedTask>;

    /// 按 task_id 取消任务（守门 #5: actor_tenant 必须匹配任务租户）
    async fn cancel(&self, actor_tenant: TenantId, task_id: Uuid) -> AgentQueueResult<()>;

    /// 当前队列长度
    async fn len(&self) -> usize;

    /// 队列是否为空
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

// =====================================================================
// InMemoryAgentQueue -- 内存实现（默认 Fifo / Priority / Interrupt）
// =====================================================================

/// **InMemoryAgentQueue** -- `AgentQueue` 的内存实现。
///
/// - `Fifo` 模式：`VecDeque<QueuedTask>` 严格 FIFO；
/// - `Priority` 模式：`BinaryHeap<(neg_priority, seq, task)>`，优先级降序 +
///   FIFO tie-break（用 enqueue 序号 `seq` 作 break）；
/// - `Interrupt` 模式：直接 push_front（FIFO 队首插队）。
///
/// 跨租户 fan-in：不限制，允许多租户共享同一 queue（每任务带 tenant_id，
/// 调用方按需守门）。
pub struct InMemoryAgentQueue {
    inner: Mutex<QueueInner>,
}

#[derive(Debug)]
struct QueueInner {
    fifo: VecDeque<QueuedTask>,
    priority: BinaryHeap<PriorityEntry>,
    seq: u64,
}

#[derive(Debug)]
struct PriorityEntry {
    /// 优先级（priority 高 → max-heap 顶 → 先 pop）
    priority: i32,
    /// tie-break: enqueue 顺序（FIFO），seq 小的优先 pop（reverse 让"先入先出"）
    seq: u64,
    /// 任务本体
    task: QueuedTask,
}

impl PartialEq for PriorityEntry {
    fn eq(&self, other: &Self) -> bool {
        // 同 task_id 视为同一 entry（用于 Eq 推导 + 后续可能的 contains 检查）
        self.task.task_id == other.task.task_id
    }
}

impl Eq for PriorityEntry {}

impl Ord for PriorityEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // priority 高者排 max-heap 顶；priority 相同时 seq 小者（先入队者）排顶
        self.priority
            .cmp(&other.priority)
            .then(other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for PriorityEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for InMemoryAgentQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryAgentQueue {
    /// 创建空 queue
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(QueueInner {
                fifo: VecDeque::new(),
                priority: BinaryHeap::new(),
                seq: 0,
            }),
        }
    }
}

#[async_trait]
impl AgentQueue for InMemoryAgentQueue {
    async fn enqueue(&self, task: QueuedTask, mode: QueueMode) -> AgentQueueResult<()> {
        QueuedTask::validate_priority(task.priority)?;
        let mut g = self
            .inner
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("queue poisoned: {e}")))?;
        g.seq += 1;
        let seq = g.seq;
        match mode {
            QueueMode::Fifo | QueueMode::Interrupt => {
                if task.interrupt || matches!(mode, QueueMode::Interrupt) {
                    g.fifo.push_front(task);
                } else {
                    g.fifo.push_back(task);
                }
            }
            QueueMode::Priority => {
                g.priority.push(PriorityEntry {
                    priority: task.priority,
                    seq,
                    task,
                });
            }
        }
        Ok(())
    }

    async fn take_next(&self) -> AgentQueueResult<QueuedTask> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("queue poisoned: {e}")))?;
        // Priority 桶先于 FIFO 桶（如果两者都有任务，priority heap 先出）
        if let Some(entry) = g.priority.pop() {
            return Ok(entry.task);
        }
        g.fifo
            .pop_front()
            .ok_or(AgentQueueError::Empty)
    }

    async fn cancel(&self, actor_tenant: TenantId, task_id: Uuid) -> AgentQueueResult<()> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("queue poisoned: {e}")))?;
        // FIFO 桶
        let before = g.fifo.len();
        g.fifo.retain(|t| {
            !(t.task_id == task_id && t.tenant_id == actor_tenant)
        });
        let removed_fifo = before != g.fifo.len();
        // Priority 桶
        let drained: Vec<PriorityEntry> = std::mem::take(&mut g.priority).into_iter().collect();
        let mut rebuilt = BinaryHeap::new();
        let mut removed_pri = false;
        for e in drained {
            if !removed_pri && e.task.task_id == task_id && e.task.tenant_id == actor_tenant {
                removed_pri = true;
                continue;
            }
            rebuilt.push(e);
        }
        g.priority = rebuilt;
        if !removed_fifo && !removed_pri {
            return Err(AgentQueueError::Internal(format!(
                "task_id={} not found for tenant={}",
                task_id, actor_tenant
            )));
        }
        Ok(())
    }

    async fn len(&self) -> usize {
        let g = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return 0,
        };
        g.fifo.len() + g.priority.len()
    }
}

// =====================================================================
// 单元测试
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loop_boundary::{AgentLoopBoundary, DefaultLoopBoundary};
    use crate::policy_hooks::{AuditEventSpec, AuditSink, NoopAuditSink, PolicyHooks};
    use star_context::ActorContext;
    use std::sync::Arc;

    fn make_task(tenant: TenantId, prompt: &str, priority: i32) -> QueuedTask {
        let mut t = QueuedTask::new(
            tenant,
            AgentId::new(),
            AgentSessionId::new(),
            prompt,
        );
        t.priority = priority;
        t
    }

    fn make_interrupt_task(tenant: TenantId, prompt: &str) -> QueuedTask {
        QueuedTask::new_interrupt(tenant, AgentId::new(), AgentSessionId::new(), prompt)
    }

    fn make_boundary_ctx(tenant_uuid: Uuid) -> LoopBoundaryContext {
        let actor = ActorContext::new(Uuid::new_v4(), tenant_uuid).with_role("project_admin");
        LoopBoundaryContext {
            actor,
            tenant_id: tenant_uuid,
            agent_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            provider: Arc::new(domain_llm::MockProvider::default()),
            policy_hooks: PolicyHooks::new(),
            audit_sink: Arc::new(NoopAuditSink),
        }
    }

    /// 同步 block_on：用 `tokio::runtime::Builder` 建一个 current-thread
    /// runtime 跑 future。本 crate 已经在 dev-deps 引入 tokio，0 新增依赖。
    fn block_on<F: std::future::Future>(fut: F) -> F::Output {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build tokio current-thread runtime");
        rt.block_on(fut)
    }

    #[test]
    fn queue_mode_default_is_fifo() {
        assert_eq!(QueueMode::default(), QueueMode::Fifo);
    }

    #[test]
    fn queued_task_priority_validate() {
        assert!(QueuedTask::validate_priority(MIN_PRIORITY).is_ok());
        assert!(QueuedTask::validate_priority(MAX_PRIORITY).is_ok());
        assert!(QueuedTask::validate_priority(DEFAULT_PRIORITY).is_ok());
        assert!(QueuedTask::validate_priority(-1).is_err());
        assert!(QueuedTask::validate_priority(MAX_PRIORITY + 1).is_err());
    }

    #[test]
    fn in_memory_queue_fifo_ordering() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            q.enqueue(make_task(t, "a", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            q.enqueue(make_task(t, "b", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            q.enqueue(make_task(t, "c", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            assert_eq!(q.take_next().await.unwrap().prompt, "a");
            assert_eq!(q.take_next().await.unwrap().prompt, "b");
            assert_eq!(q.take_next().await.unwrap().prompt, "c");
            assert!(q.take_next().await.is_err());
            assert_eq!(q.len().await, 0);
        });
    }

    #[test]
    fn in_memory_queue_priority_desc_with_fifo_tiebreak() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            // 入队顺序: low, high, mid
            q.enqueue(make_task(t, "low", 1), QueueMode::Priority)
                .await
                .unwrap();
            q.enqueue(make_task(t, "high", 9), QueueMode::Priority)
                .await
                .unwrap();
            q.enqueue(make_task(t, "mid", 5), QueueMode::Priority)
                .await
                .unwrap();
            assert_eq!(q.take_next().await.unwrap().prompt, "high");
            assert_eq!(q.take_next().await.unwrap().prompt, "mid");
            assert_eq!(q.take_next().await.unwrap().prompt, "low");
        });
    }

    #[test]
    fn in_memory_queue_priority_same_priority_fifo_tiebreak() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            q.enqueue(make_task(t, "first", 5), QueueMode::Priority)
                .await
                .unwrap();
            q.enqueue(make_task(t, "second", 5), QueueMode::Priority)
                .await
                .unwrap();
            assert_eq!(q.take_next().await.unwrap().prompt, "first");
            assert_eq!(q.take_next().await.unwrap().prompt, "second");
        });
    }

    #[test]
    fn in_memory_queue_interrupt_push_front() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            q.enqueue(make_task(t, "normal-1", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            q.enqueue(make_task(t, "normal-2", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            q.enqueue(
                make_interrupt_task(t, "interrupt-1"),
                QueueMode::Fifo,
            )
            .await
            .unwrap();
            assert_eq!(q.take_next().await.unwrap().prompt, "interrupt-1");
            assert_eq!(q.take_next().await.unwrap().prompt, "normal-1");
            assert_eq!(q.take_next().await.unwrap().prompt, "normal-2");
        });
    }

    #[test]
    fn in_memory_queue_cancel_cross_tenant_denied() {
        let q = InMemoryAgentQueue::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        block_on(async {
            let task = make_task(t1, "x", DEFAULT_PRIORITY);
            let id = task.task_id;
            q.enqueue(task, QueueMode::Fifo).await.unwrap();
            // t2 试图取消 t1 的任务 → 应失败（任务仍在）
            assert!(q.cancel(t2, id).await.is_err());
            assert_eq!(q.len().await, 1);
            // t1 取消 → 成功
            q.cancel(t1, id).await.unwrap();
            assert_eq!(q.len().await, 0);
        });
    }

    #[test]
    fn noop_steering_returns_passthrough_with_hint_unused() {
        let s = NoopSteering;
        let tenant = TenantId::new();
        let boundary_ctx = make_boundary_ctx(tenant.as_uuid());
        let base = InternalContext::new();
        block_on(async {
            let out = s
                .apply_steering(tenant, &boundary_ctx, base, "do X")
                .await
                .unwrap();
            assert_eq!(out.context.messages.len(), 0, "noop 不应注入 hint");
            assert!(s.last_steering().await.is_none());
        });
    }

    #[test]
    fn loop_boundary_steering_hook_injects_hint_via_transform_context() {
        // 用 DefaultLoopBoundary（pass-through）做 harness，验证 hint 被注入 messages
        let boundary: Arc<dyn AgentLoopBoundary> = Arc::new(DefaultLoopBoundary);
        let hook = LoopBoundarySteeringHook::new(boundary);
        let tenant = TenantId::new();
        let boundary_ctx = make_boundary_ctx(tenant.as_uuid());
        let base = InternalContext::new();
        block_on(async {
            let out = hook
                .apply_steering(tenant, &boundary_ctx, base, "switch to Redis")
                .await
                .unwrap();
            // hint 应作为新 InternalMessage 注入 messages
            assert_eq!(out.context.messages.len(), 1);
            assert!(matches!(
                out.context.messages[0].role,
                InternalRole::Steering
            ));
            assert_eq!(out.context.messages[0].content, "switch to Redis");
            assert!(out.context.messages[0].tool.is_none());
            // last_steering 缓存
            let last = hook.last_steering().await;
            assert_eq!(last.as_deref(), Some("switch to Redis"));
        });
    }

    #[test]
    fn loop_boundary_steering_hook_cross_tenant_denied() {
        let boundary: Arc<dyn AgentLoopBoundary> = Arc::new(DefaultLoopBoundary);
        let hook = LoopBoundarySteeringHook::new(boundary);
        let tenant = TenantId::new();
        let other_tenant = TenantId::new();
        let boundary_ctx = make_boundary_ctx(tenant.as_uuid());
        let base = InternalContext::new();
        block_on(async {
            let res = hook
                .apply_steering(other_tenant, &boundary_ctx, base, "hack")
                .await;
            assert!(matches!(
                res,
                Err(AgentQueueError::CrossTenantSteeringDenied { .. })
            ));
            // last_steering 不应被设置
            assert!(hook.last_steering().await.is_none());
        });
    }

    #[test]
    fn loop_boundary_steering_hook_with_custom_pruning_boundary() {
        // 自定义 boundary：transform_context 直接 truncate messages
        struct TruncateBoundary;
        #[async_trait::async_trait]
        impl AgentLoopBoundary for TruncateBoundary {
            async fn transform_context(
                &self,
                _ctx: &LoopBoundaryContext,
                mut context: InternalContext,
            ) -> TransformOutcome {
                context.messages.clear(); // 强制清空（clippy::manual_clear fix）
                TransformOutcome::passthrough(context)
            }
        }
        let boundary: Arc<dyn AgentLoopBoundary> = Arc::new(TruncateBoundary);
        let hook = LoopBoundarySteeringHook::new(boundary);
        let tenant = TenantId::new();
        let boundary_ctx = make_boundary_ctx(tenant.as_uuid());
        let base = InternalContext::new();
        block_on(async {
            let out = hook
                .apply_steering(tenant, &boundary_ctx, base, "use Postgres")
                .await
                .unwrap();
            // TruncateBoundary 把 messages 清空，验证 hook 不"自作主张"
            assert_eq!(out.context.messages.len(), 0);
        });
    }

    #[test]
    fn default_followup_enqueues_and_cancels() {
        let q = InMemoryAgentQueue::new();
        let fu = DefaultFollowUp;
        let tenant = TenantId::new();
        let task = make_task(tenant, "followup", DEFAULT_PRIORITY);
        let id = task.task_id;
        block_on(async {
            fu.enqueue_followup(tenant, &q, task).await.unwrap();
            assert_eq!(q.len().await, 1);
            fu.cancel_followup(tenant, &q, id).await.unwrap();
            assert_eq!(q.len().await, 0);
        });
    }

    #[test]
    fn default_followup_cross_tenant_enq_denied() {
        let q = InMemoryAgentQueue::new();
        let fu = DefaultFollowUp;
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        let task = make_task(t1, "x", DEFAULT_PRIORITY);
        block_on(async {
            // actor t2 想入队 t1 的 task → 拒绝
            assert!(fu.enqueue_followup(t2, &q, task).await.is_err());
            assert_eq!(q.len().await, 0);
        });
    }

    #[test]
    fn in_memory_queue_invalid_priority_rejected() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let mut task = make_task(t, "x", DEFAULT_PRIORITY);
        task.priority = MAX_PRIORITY + 1;
        block_on(async {
            assert!(q.enqueue(task, QueueMode::Priority).await.is_err());
            assert_eq!(q.len().await, 0);
        });
    }

    #[test]
    fn in_memory_queue_mixed_modes_take_priority_first() {
        // take_next 优先级桶先于 FIFO 桶
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            q.enqueue(make_task(t, "fifo-1", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            q.enqueue(make_task(t, "prio-1", 8), QueueMode::Priority)
                .await
                .unwrap();
            assert_eq!(q.take_next().await.unwrap().prompt, "prio-1");
            assert_eq!(q.take_next().await.unwrap().prompt, "fifo-1");
        });
    }

    #[test]
    fn audit_event_emitted_on_steering_apply() {
        // 自实现 AuditSink 捕获 write 调用，验证守门 #13 d 触发 audit
        struct CaptureSink {
            events: Mutex<Vec<String>>,
        }
        impl CaptureSink {
            fn new() -> Self {
                Self {
                    events: Mutex::new(Vec::new()),
                }
            }
        }
        impl AuditSink for CaptureSink {
            fn write(&self, event: AuditEventSpec) {
                self.events.lock().unwrap().push(event.action);
            }
        }
        let boundary: Arc<dyn AgentLoopBoundary> = Arc::new(DefaultLoopBoundary);
        let hook = LoopBoundarySteeringHook::new(boundary);
        let tenant = TenantId::new();
        let sink = Arc::new(CaptureSink::new());
        let mut ctx = make_boundary_ctx(tenant.as_uuid());
        ctx.audit_sink = sink.clone();
        let base = InternalContext::new();
        block_on(async {
            hook.apply_steering(tenant, &ctx, base, "try something")
                .await
                .unwrap();
            let events = sink.events.lock().unwrap();
            assert_eq!(events.len(), 1, "audit 应该被 write 一次");
            assert!(events[0].starts_with("steering.apply"));
        });
    }

    // Touch unused imports if compile needs them in some cfg
    #[allow(dead_code)]
    fn _unused_audit_spec_marker(_s: AuditEventSpec) {}
}
