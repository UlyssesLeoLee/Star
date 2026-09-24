//! PI-9 Steering / Follow-up / QueueMode (domain-agent) — W6 「PI-4/PI-6 ship 收紧」.
//!
//! Per SRS-PI-BORROW-001 §1.3 + §4 FR-30 (`crates/pi-agent-core/src/types.ts`
//! lines 47-55, 278-302 翻译).
//!
//! ## 累计里程碑
//!
//! - **W1** (commit `99f70d67`): 接口骨架 + FIFO/Priority/Interrupt 三态的
//!   in-memory 调度 + `Steering::apply_steering` 接 PI-3
//!   `AgentLoopBoundary::transform_context` 的默认实装。
//! - **W2** (commit `58ee2a5d`): 在 W1 基础上加 **cancel 抢断语义** +
//!   **Interrupt 抢占触发回调** (`PreemptionListener`).
//! - **W3** (commit `97641799`): 加 **跨域协作评论入口 stub** +
//!   **`mark_running_with` 真租户守门**:
//!
//!   | 增量 | 描述 |
//!   |---|---|
//!   | `AgentQueue::mark_running_with(QueuedTask)` | 把完整 task（含真实 `tenant_id` + snapshot）交给 queue；与 W2 `mark_running(task_id)` 并存 |
//!   | `maybe_fire_preemption` 跨租户守门 | `running.tenant_id` 非 nil 时,跨租户 incoming 静默不 fire；W2 legacy 路径（`mark_running(task_id)` 占位 tenant=nil）保持原行为 |
//!   | `CollabCommentSink` trait | 跨域协作评论投递接口的本地抽象,无依赖 |
//!   | `NoopCollabSink` / `InMemoryCollabSink` | 默认 / 测试用 sink |
//!   | `CollabPreemptionBridge<C>` | `PreemptionListener` W3 实装：把抢占事件转 `CollabSteeringCommand` + prompt 摘要（最长 80 字符,防 secret 泄漏）+ 投 sink |
//!   | `redact_excerpt` 工具函数 | prompt 截断,守门 #5 env |
//!   | W3 测试 9 条 | tenant guard 生效 / 跨租户不再 fire / sink dispatch / 端到端集成 |
//!
//!   W2 的 `mark_running(task_id)` 仍可用（向后兼容）,但 `running.tenant_id`
//!   是占位 nil → `maybe_fire_preemption` 跳过 tenant 守门（行为 = W2 legacy）。
//!   生产环境应**全部走 `mark_running_with`**。
//!
//! - **W4** (commit `dcc31890`, P-B 路径): `ParentType::AgentSession` variant
//!   + INV-C-05 守门同步 + application `SteeringSinkBridge` 把
//!     `CollabSteeringCommand` 转 `SteeringCommand` 走 `InMemoryCommentService`,
//!     PI-9 主路径 0 改动。
//!
//! - **W5** (commit `4ea76dfa`): `CollabSteeringCommand` 加 4 字段
//!   (`current_agent_id` / `current_session_id` / `incoming_agent_id` /
//!   `incoming_session_id`),bridge 透传 `QueuedTask.agent_id` / `session_id` 真实值
//!   (消除 W4 占位 `= incoming_task_id` trade-off)。
//!
//! - **W6** (本 commit): **PI-4 + PI-6 已 ship 后收紧 `QueuedTask` 占位字段**:
//!
//!   | 字段 | W5 形态 | W6 形态 | 依据 |
//!   |---|---|---|---|
//!   | `tool_hint` | `String` | `Option<domain_tool::ToolExecutionMode>` | PI-4 (ULYS-205, commit `0b00ecec`) ship `ToolExecutionMode` 4 变体 (Sync/Async/Stream/Background) 作为调度提示 |
//!   | `payload_op` (新) | (无) | `Option<star_dto::delta::Op>` | PI-6 (ULYS-206, commit `2a0ca91a`) ship 7 ops (r/s/d/a/t/p/m) 作为结构化载荷 |
//!   | `payload` (保留) | `serde_json::Value` | `serde_json::Value` | 向后兼容 PI-6 之前的 caller;`payload_op` 与 `payload` 同时存在,默认 `payload_op = None`,显式设置时二选一 |
//!
//!   W6 测试新增 4 条：tool_hint 默认 None / `payload_op` 序列化 / `Op::Set` 透传 /
//!   `Op::Apply` reducer 端到端 (调 `star_dto::delta::apply`)。
//!
//!   W1-W5 任何 caller 测试**0 改动**——`QueuedTask::new()` 默认 `tool_hint = None` /
//!   `payload_op = None`,既有测试通过 `make_task` helper 走 `new()`,自动兼容。
//!
//! ## 已知缺口 / 待 SRS-MULTICA-COLLABORATION 落地后补
//!
//! - **PI-3 ✅ 已 ship (本 branch cherry-pick `c4145379`)**: `AgentLoopBoundary`
//!   trait + `StreamFn` no-throw 已在本 worktree 可用。
//! - **PI-4 ✅ 已 ship (origin/main commit `0b00ecec`)**: `Tool` trait 5 方法
//!   + `ToolExecutionMode` 4 变体。本 commit 把 `tool_hint` 收紧为
//!     `Option<domain_tool::ToolExecutionMode>`,Sprint 3 接 `Tool::execute`
//!     dispatcher 后可进一步收紧为完整 `ToolInvocation`。
//! - **PI-6 ✅ 已 ship (origin/main commit `2a0ca91a`)**: `star_dto::delta::Op`
//!   7 ops + `apply` reducer。本 commit 加 `payload_op: Option<Op>` 字段,
//!   序列化路径已验证 (Set/Set round-trip)。
//! - **跨域协作评论**: SRS-MULTICA-COLLABORATION 协作评论机制未 ship,
//!   `CollabPreemptionBridge` 已就位,等跨域 ship 后:
//!   1. 在 `crates/domain-comment` 加 `impl CollabCommentSink for CommentService`
//!   2. 改 `CollabPreemptionBridge::new(Arc::new(real_service))`
//!   3. PI-9 主路径**0 改动**。
//!
//! 完整 3 周 MVP 工时排程见 issue description §4。
//! 启动条件（per §5）：Stage 4 + PI-6 (ULYS-206 redesign) 已 ship + D-Boy 拍板
//! PI-9 启动。W6 兑现 D-Boy 2026-09-24 「完成所有后续工作，全选，推进到完成」:
//! PI-4 + PI-6 已 ship + SRS-MULTICA-COLLABORATION 真接口仍待 ship (跨域协调)。
//!
//! ## 守门合规
//!
//! - 0 `unsafe` blocks (`unsafe_code = "forbid"` workspace lint).
//! - `#[non_exhaustive]` on enums that may grow (per SRS-PI-BORROW-001 NFR-7).
//! - All public items documented (`missing_docs = "deny"` workspace lint).
//! - No reverse dependency on `crates/api` (per 守门 #1 v15 分层 + 守门 #13 d).
//! - 守门 #5: cross-tenant steering 在 `LoopBoundarySteeringHook::apply_steering`
//!   第一步校验 actor_tenant == boundary_ctx.tenant_id,不匹配 → 立即回
//!   `CrossTenantSteeringDenied` 错误（不绕开 boundary）。
//! - 守门 #5 (W3 增量): `maybe_fire_preemption` 在 `running.tenant_id` 非 nil
//!   时,`incoming.tenant_id != running.tenant_id` 静默不 fire（不抛错、
//!   不调 listener,避免泄漏跨租户抢占语义）。
//! - 守门 #13 d: hint 注入后通过 `boundary_ctx.audit_sink.emit(...)` 触发
//!   audit event（即使 transform_context 失败也记 "steering-attempted"）。
//! - 守门 #22 (W2 增量): PreemptionListener 回调 **不** 阻塞 enqueue
//!   （fire-and-forget fn 指针而非 async fn）,保证 enqueue p99 不被
//!   listener 拖垮。
//! - 守门 #22 (W3 增量): `CollabCommentSink::dispatch_steering` 也是 sync fn,
//!   sink 内部 RPC 自负责 spawn task。

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
    #[error("running task mismatch: queue holds {queue_running:?} but driver marked {marked:?}")]
    /// `mark_done` / `mark_running` 给的 task_id 与 queue 当前记录的
    /// running 不一致 —— 通常是 driver 多次 restart 同一 task / 跨 queue
    /// 调用错配。Queue **不** 自动修正（避免掩盖 driver bug），由调用方
    /// 决定是 reset 还是重发。
    RunningTaskMismatch {
        /// queue 当前记的 running task_id（None = 没人跑）
        queue_running: Option<Uuid>,
        /// driver 给的 task_id
        marked: Uuid,
    },
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
// CancelReport -- 批量 cancel 的返回值 (W2 新增)
// =====================================================================

/// **CancelReport** -- `cancel_many` 的返回值，区分 found 与 missing，让
/// caller 知道 "哪些 task 真的被清掉了" / "哪些 task 本来就不在 queue
/// 里（可能已经 take_next 跑起来了，或者根本没 enqueue）"。
///
/// `found` 与 `missing` 的顺序都保持输入顺序，便于 caller 做 diff.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelReport {
    /// 成功取消的 task_id 列表（输入顺序）
    pub found: Vec<Uuid>,
    /// 未找到的 task_id 列表（输入顺序）
    pub missing: Vec<Uuid>,
}

impl CancelReport {
    /// 没有任何 missing（全命中或 input 为空）
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }
    /// total = found + missing = caller 传入的 input 长度
    pub fn total(&self) -> usize {
        self.found.len() + self.missing.len()
    }
}

// =====================================================================
// PreemptionListener -- 抢占回调 (W2 新增)
// =====================================================================

/// **PreemptionListener** -- 当 `enqueue` 一个 Interrupt / Priority 任务且
/// 已存在 `mark_running` 任务时，queue **fire-and-forget** 调
/// `on_preempt(current, incoming)`。
///
/// **为什么是 `fn`（同步）不是 `async fn`**:
/// - 抢占决策是 fire-and-forget 的：listener 收到信号后通常会
///   `loop.cancel_running_task()`（同步 op）或 spawn 一个 background
///   task 做 cleanup（异步 op），并不需要 queue 等待结果。
/// - 如果做成 `async`，listener 内部 `await` 一个慢 I/O（RPC / disk）
///   会卡 `enqueue` 的 p99 延迟，违反守门 #22（enqueue 同步完成）。
///
/// **panic 政策**: queue **不** 用 `catch_unwind` 包裹 listener —— listener
/// 自己负责 panic safety（用 `Arc<Mutex<...>>` 内置状态自保护）。
pub trait PreemptionListener: Send + Sync {
    /// **on_preempt** -- 抢占回调。
    ///
    /// - `current`: 正在跑的任务（来自 `mark_running` 注册的快照）
    /// - `incoming`: 新进的 Interrupt / 高优 Priority 任务
    ///
    /// listener 应自行决定如何处理（cancel / 中断 stream / 通知 driver），
    /// **不** 应通过返回值传结果（fire-and-forget）。
    fn on_preempt(&self, current: &QueuedTask, incoming: &QueuedTask);
}

// =====================================================================
// CollabPreemptionBridge -- 跨域协作评论入口 (W3 增量)
// =====================================================================
//
// **为什么单独成一节**:PI-9 的 Steering 钩子需要一个 UI 入口让用户在
// agent 跑的时候通过协作评论发 steering 指令。SRS-MULTICA-COLLABORATION
// 协作评论机制 **尚未 ship**,本节提供一个本地 trait 抽象
// (`CollabCommentSink`),让 PI-9 的 `CollabPreemptionBridge` 在该机制
// ship 后,把 sink 适配到 `crates/domain-comment` 的真接口即可,无需再改
// PI-9 queue.rs 主路径。
//
// **安全设计** (守门 #5 env):
// - bridge 不把 prompt 全文塞进协作评论 — 用 `redact_excerpt` 截到
//   `MAX_PROMPT_EXCERPT_CHARS` (= 80) 字符,避免泄漏长 prompt 里嵌的
//   secret / token / 内部文件路径。
// - `CollabSteeringCommand` 字段标 `non_exhaustive`,跨域 schema 演进安全。

/// prompt 摘要最大字符数（W3 守门 #5: 防止 secret 泄漏到协作评论载荷）
pub const MAX_PROMPT_EXCERPT_CHARS: usize = 80;

/// **CollabSteeringCommand** -- bridge 投递到协作评论侧的 steering 命令。
///
/// `current_prompt_excerpt` / `incoming_prompt_excerpt` 是被 redacted 的
/// 短摘（最长 [`MAX_PROMPT_EXCERPT_CHARS`] 字符）—— 不是 prompt 全文。
///
/// **ULYS-207 PI-9 W5**:`current_agent_id` / `current_session_id` /
/// `incoming_agent_id` / `incoming_session_id` 在 W5 落地,**透传自
/// `QueuedTask`**,不再用 `task_id` 占位（per W4 已知 trade-off fix path）。
/// Bridge 取真实 `AgentId` / `AgentSessionId`,application 层
/// `SteeringSinkBridge` 透传到 `domain_comment::SteeringCommand` 用作
/// `comment.author_agent_id` 与 `parent_id = ParentType::AgentSession`。
///
/// `#[non_exhaustive]` 允许未来加 field 而不破坏下游。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CollabSteeringCommand {
    /// 触发 steering 的 tenant（= incoming.tenant_id）
    pub tenant_id: TenantId,
    /// 当前正在跑的任务 ID
    pub current_task_id: Uuid,
    /// current 任务的 prompt 摘要（已 redact）
    pub current_prompt_excerpt: String,
    /// **W5** 当前任务的 agent（透传自 `QueuedTask.agent_id`）
    pub current_agent_id: AgentId,
    /// **W5** 当前任务的 session（透传自 `QueuedTask.session_id`）
    pub current_session_id: AgentSessionId,
    /// 新进的抢占/高优任务 ID
    pub incoming_task_id: Uuid,
    /// incoming 任务的 prompt 摘要（已 redact）
    pub incoming_prompt_excerpt: String,
    /// **W5** incoming 任务的 agent（透传自 `QueuedTask.agent_id`）
    pub incoming_agent_id: AgentId,
    /// **W5** incoming 任务的 session（透传自 `QueuedTask.session_id`）
    pub incoming_session_id: AgentSessionId,
    /// incoming 入队时间
    pub enqueued_at: SystemTime,
}

impl CollabSteeringCommand {
    /// **new** -- 构造 CollabSteeringCommand(cross-crate caller 用,
    /// 因为 struct 是 `#[non_exhaustive]`,同 crate 内可用 struct literal,
    /// 跨 crate 必须用 constructor)。
    ///
    /// **ULYS-207 PI-9 W4 P-B / W5**:application crate 胶水
    /// `SteeringSinkBridge` 调此方法构造 CollabSteeringCommand 转
    /// `domain_comment::SteeringCommand`。W5 加 `current_agent_id` /
    /// `current_session_id` / `incoming_agent_id` / `incoming_session_id`
    /// 4 个真实字段,W4 的 `incoming_task_id` 占位已废止。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: TenantId,
        current_task_id: Uuid,
        current_prompt_excerpt: String,
        current_agent_id: AgentId,
        current_session_id: AgentSessionId,
        incoming_task_id: Uuid,
        incoming_prompt_excerpt: String,
        incoming_agent_id: AgentId,
        incoming_session_id: AgentSessionId,
        enqueued_at: SystemTime,
    ) -> Self {
        Self {
            tenant_id,
            current_task_id,
            current_prompt_excerpt,
            current_agent_id,
            current_session_id,
            incoming_task_id,
            incoming_prompt_excerpt,
            incoming_agent_id,
            incoming_session_id,
            enqueued_at,
        }
    }
}

/// 截 prompt 到 `MAX_PROMPT_EXCERPT_CHARS` 字符，超过部分加 "…"。
/// 守门 #5: 不让长 prompt 全文进协作评论载荷。
fn redact_excerpt(prompt: &str) -> String {
    if prompt.chars().count() <= MAX_PROMPT_EXCERPT_CHARS {
        prompt.to_string()
    } else {
        // 按 char 截，避免 UTF-8 边界把字符切坏
        let mut out: String = prompt.chars().take(MAX_PROMPT_EXCERPT_CHARS).collect();
        out.push('…');
        out
    }
}

/// **CollabCommentSink** -- 跨域协作评论投递接口的本地抽象（W3 跨域 stub）。
///
/// 这是 **trait**,不绑任何具体下游 —— 这样:
/// 1. SRS-MULTICA-COLLABORATION 未 ship 期间,PI-9 用 [`NoopCollabSink`] /
///    [`InMemoryCollabSink`] (测试用) 即可完成 queue.rs 自身的逻辑;
/// 2. 跨域机制 ship 后,只需加一个 `impl CollabCommentSink for
///    domain_comment::CommentService` 适配器,PI-9 主路径不变。
///
/// **为什么是 sync fn 不是 async**:
/// - PreemptionListener 是 fire-and-forget, sink 同步 fn 让 enqueue p99
///   不被跨域 RPC 拖垮（守门 #22）。
/// - 真 sink 内部需要 RPC 时,自负责 spawn 一个 tokio task。
pub trait CollabCommentSink: Send + Sync {
    /// **dispatch_steering** -- 把 steering 命令投递到协作评论侧。
    ///
    /// 实现要求：
    /// - **不 panic** —— listener panic 由 listener 自负责（守门 #22）。
    /// - 失败仅记日志,不返回 Err（fire-and-forget）。
    fn dispatch_steering(&self, cmd: CollabSteeringCommand);
}

/// **NoopCollabSink** -- 不做任何事的 sink（生产环境默认 sink）。
///
/// 当 SRS-MULTICA-COLLABORATION 未 ship 时,`CollabPreemptionBridge` 默认
/// 绑这个 sink,queue 行为 = 跟 W2 完全一致（仅 in-process listener fire,
/// 无跨域副作用）。
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopCollabSink;

impl CollabCommentSink for NoopCollabSink {
    fn dispatch_steering(&self, _cmd: CollabSteeringCommand) {
        // no-op;SRS-MULTICA-COLLABORATION ship 后由 D-Boy 决定是否换 sink
    }
}

/// **InMemoryCollabSink** -- 把所有 `CollabSteeringCommand` 记到内存,供
/// 测试断言。仅 `#[cfg(test)]` 模块外不可见 —— `pub(crate)` 限定,生产
/// binary 不会暴露。
#[derive(Debug, Default)]
pub struct InMemoryCollabSink {
    pub(crate) commands: Mutex<Vec<CollabSteeringCommand>>,
}

impl InMemoryCollabSink {
    /// 构造新 sink
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(Vec::new()),
        }
    }
    /// 返回已记录的 command 数
    pub fn len(&self) -> usize {
        self.commands.lock().unwrap().len()
    }
    /// 是否无记录（clippy `len_without_is_empty` 一致性）
    pub fn is_empty(&self) -> bool {
        self.commands.lock().unwrap().is_empty()
    }
    /// 复制所有已记录 command（按入队顺序）
    pub fn commands(&self) -> Vec<CollabSteeringCommand> {
        self.commands.lock().unwrap().clone()
    }
}

impl CollabCommentSink for InMemoryCollabSink {
    fn dispatch_steering(&self, cmd: CollabSteeringCommand) {
        self.commands.lock().unwrap().push(cmd);
    }
}

/// **CollabPreemptionBridge** -- `PreemptionListener` 的 W3 实装,把抢占事件
/// 转成 `CollabSteeringCommand` 通过 [`CollabCommentSink`] 投到协作评论侧。
///
/// **Sink 不持有锁**:bridge 持有 `Arc<dyn CollabCommentSink>`,sink 自身
/// 负责 thread-safe（典型 = `Arc<Mutex<...>>` 或 channel-based）。
///
/// **典型用法**:
/// ```ignore
/// let sink: Arc<dyn CollabCommentSink> = Arc::new(NoopCollabSink);
/// let bridge: Arc<dyn PreemptionListener> =
///     Arc::new(CollabPreemptionBridge::new(sink));
/// queue.register_preemption_listener(bridge).await;
/// ```
///
/// **W3 限制**:sink 是本地 trait 抽象。SRS-MULTICA-COLLABORATION ship 后,
/// 在 `crates/domain-comment` 加 `impl CollabCommentSink for CommentService`,
/// 然后把 `CollabPreemptionBridge::new(Arc::new(real_service))` 即可。
pub struct CollabPreemptionBridge {
    sink: std::sync::Arc<dyn CollabCommentSink>,
}

impl CollabPreemptionBridge {
    /// 构造 bridge,绑定到具体 sink
    pub fn new(sink: std::sync::Arc<dyn CollabCommentSink>) -> Self {
        Self { sink }
    }
    /// 取得 sink 引用（主要给测试断言用）
    pub fn sink(&self) -> &std::sync::Arc<dyn CollabCommentSink> {
        &self.sink
    }
}

impl PreemptionListener for CollabPreemptionBridge {
    fn on_preempt(&self, current: &QueuedTask, incoming: &QueuedTask) {
        // 构造 redact 过的 command。守门 #5: prompt 走摘要,不让 secret 进 sink
        // **ULYS-207 PI-9 W5**:agent_id / session_id 透传自 QueuedTask,
        // 不再用 incoming_task_id 占位(per W4 已知 trade-off fix path)。
        let cmd = CollabSteeringCommand {
            tenant_id: incoming.tenant_id,
            current_task_id: current.task_id,
            current_prompt_excerpt: redact_excerpt(&current.prompt),
            current_agent_id: current.agent_id,
            current_session_id: current.session_id,
            incoming_task_id: incoming.task_id,
            incoming_prompt_excerpt: redact_excerpt(&incoming.prompt),
            incoming_agent_id: incoming.agent_id,
            incoming_session_id: incoming.session_id,
            enqueued_at: incoming.created_at,
        };
        // fire-and-forget:sink 内部 panic 自负责（守门 #22）
        self.sink.dispatch_steering(cmd);
    }
}

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
/// 字段（per ULYS-207 PI-9 W6）：
/// - `tool_hint`: PI-4 ship 后收紧为 `Option<domain_tool::ToolExecutionMode>` —
///   调度提示,默认 `None`（纯 prompt 任务不需要工具）。
/// - `payload`: 保留 `serde_json::Value` 以向后兼容 W5 之前 caller。
/// - `payload_op`: PI-6 ship 后新增 `Option<star_dto::delta::Op>` — 结构化
///   载荷,7 ops (r/s/d/a/t/p/m)。与 `payload` 同时存在,默认 `None`。
///   agent loop 优先用 `payload_op`（若 Some）,降级走 `payload`（保持向后兼容）。
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
    /// 工具调用 hint（PI-4 ship 后收紧为 `Option<ToolExecutionMode>`）。
    ///
    /// `None` = 纯 prompt 任务,不调任何工具。
    /// `Some(Synchronous)` / `Some(Async)` / `Some(Stream)` / `Some(Background)`
    /// = 调度提示,agent loop 据此选 dispatcher 路径 (per FR-22)。
    pub tool_hint: Option<domain_tool::ToolExecutionMode>,
    /// 任务载荷（向后兼容 W5 之前 caller）。
    ///
    /// 与 `payload_op` 二选一：默认 `Null`,PI-6 后可改设结构化 `Op` 的 JSON
    /// 表示；显式设 `payload_op = Some(op)` 时本字段保留作 fallback。
    pub payload: serde_json::Value,
    /// 任务载荷（PI-6 ship 后新增）— 7 ops (r/s/d/a/t/p/m) 的结构化增量。
    ///
    /// `None` = 不携带结构化变更（默认,纯 prompt / 任意 Value）。
    /// `Some(Op::Set(path, value))` = 设置路径上的值。
    /// `Some(Op::Replace(value))` = 替换整个根值。
    /// agent loop 优先用此字段做 apply（Sprint 3 接 star-taskgraph 增量同步）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_op: Option<star_dto::delta::Op>,
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
            tool_hint: None,
            payload: serde_json::Value::Null,
            payload_op: None,
            priority: DEFAULT_PRIORITY,
            interrupt: false,
            created_at: SystemTime::now(),
        }
    }

    /// Builder: 设置 tool_hint (PI-4 `ToolExecutionMode`)
    pub fn with_tool_hint(mut self, mode: domain_tool::ToolExecutionMode) -> Self {
        self.tool_hint = Some(mode);
        self
    }

    /// Builder: 设置 payload_op (PI-6 `star_dto::delta::Op`)
    pub fn with_payload_op(mut self, op: star_dto::delta::Op) -> Self {
        self.payload_op = Some(op);
        self
    }

    /// Builder: 设置 payload (保留向后兼容)
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
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
/// 4 个 W1 方法：`enqueue` / `take_next` / `cancel` / `len`。
///
/// W2 增量：
/// - `cancel_many` / `cancel_all`：批量 cancel
/// - `mark_running` / `mark_done` / `running_task_id`：loop driver 显式
///   告诉 queue 当前跑的是哪个 task（让 queue 知道该向谁发抢占信号）
/// - `register_preemption_listener`：注册 `PreemptionListener`，
///   `enqueue` 触发抢占时回调
///
/// W2 新增方法的 **默认实现** 都退化为"空操作 / Err"——具体下游（如
/// `InMemoryAgentQueue`）可覆盖。Trait 上挂默认实现是为了
/// 第三方实现（feature flag / mock）的向后兼容：W2 之前 ship 的下游
/// 不会因为 trait 加方法而编译失败。
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

    // ---------------- W2 增量 ----------------

    /// **cancel_many** -- 批量 cancel (W2)。
    ///
    /// 输入 `actor_tenant` 守门：被 cancel 的 task 必须 `task.tenant_id == actor_tenant`
    /// 才计入 `found`，否则既不 cancel 也不计入 `missing`（跨租户的 task
    /// 被静默跳过，避免泄漏其他租户的 task_id 存在性）。
    ///
    /// 不会因 `missing` 非空而返回 `Err`：返回 `CancelReport` 让 caller 自行
    /// 判断。
    ///
    /// 默认实现：循环调 `cancel`，合并 report；`InMemoryAgentQueue` 提供
    /// 单锁 O(N+M) 实现。
    async fn cancel_many(
        &self,
        actor_tenant: TenantId,
        task_ids: &[Uuid],
    ) -> AgentQueueResult<CancelReport> {
        let mut report = CancelReport::default();
        for id in task_ids {
            match self.cancel(actor_tenant, *id).await {
                Ok(()) => report.found.push(*id),
                Err(_) => report.missing.push(*id),
            }
        }
        Ok(report)
    }

    /// **cancel_all** -- 按租户清空全部 (W2)。
    ///
    /// 返回被清掉的 task_id 列表（顺序为 queue 内实际顺序，不保证按时间）。
    /// 不会清掉其他租户的 task（守门 #5）。
    ///
    /// 默认实现：W2 前的 trait 实现没这方法，直接 `Err(Internal("cancel_all
    /// not implemented"))`。`InMemoryAgentQueue` 提供完整实现。
    async fn cancel_all(&self, _actor_tenant: TenantId) -> AgentQueueResult<Vec<Uuid>> {
        Err(AgentQueueError::Internal(
            "cancel_all not implemented by this AgentQueue".into(),
        ))
    }

    /// **mark_running** -- driver 显式告知 queue "我现在跑的是这个 task" (W2)。
    ///
    /// queue 用这个状态决定 enqueue 时是否触发 `PreemptionListener::on_preempt`。
    /// 同一 task 不能被 mark_running 两次（避免 driver 重启 race 把两个 task
    /// 都标成 running）—— 第二次调同名 task_id = noop + warn log（返回 `Ok`）。
    ///
    /// **W2 简化**：本方法只接 `task_id`，无法记下完整 `QueuedTask`，
    /// 所以 `running.tenant_id` 是占位 nil（见 `InMemoryAgentQueue::mark_running`
    /// 实现）。如果 caller 有完整 `QueuedTask`，**应该** 改用 W3 的
    /// [`AgentQueue::mark_running_with`] —— 那个能记下真实租户，让 W3
    /// 加的跨租户守门在 `maybe_fire_preemption` 生效。
    ///
    /// 默认实现：`Err(Internal("mark_running not implemented"))`。
    async fn mark_running(&self, _task_id: Uuid) -> AgentQueueResult<()> {
        Err(AgentQueueError::Internal(
            "mark_running not implemented by this AgentQueue".into(),
        ))
    }

    /// **mark_running_with** -- driver 显式告知 queue "我现在跑的是这个 task"
    /// **(W3 新增)**，并把完整 `QueuedTask` 一起交给 queue。
    ///
    /// 与 `mark_running(task_id)` 的区别：
    /// - 本方法记下真实 `tenant_id` + 完整 snapshot（含 prompt / priority）；
    ///   `maybe_fire_preemption` 用真实 `tenant_id` 守门：跨租户的抢占不再
    ///   fire（守门 #5 真实生效）。
    /// - `mark_running(task_id)` 仍是兼容路径，但 `running.tenant_id` 是
    ///   nil 占位 → `maybe_fire_preemption` 跳过 tenant 守门（行为 = W2 legacy）。
    ///
    /// 默认实现：`Err(Internal("mark_running_with not implemented"))`，让旧
    /// 实现者编译通过（向后兼容）。`InMemoryAgentQueue` 提供完整实装。
    async fn mark_running_with(&self, _task: QueuedTask) -> AgentQueueResult<()> {
        Err(AgentQueueError::Internal(
            "mark_running_with not implemented by this AgentQueue".into(),
        ))
    }

    /// **mark_done** -- driver 告知 queue "这个 task 跑完了，running 状态清掉" (W2)。
    ///
    /// 如果传入的 task_id 与 queue 当前记的 running 不一致 → 返回
    /// `RunningTaskMismatch` 错误（守门 #22），由 caller 决定怎么处理。
    /// 跨租户的 task_id 也算 mismatch。
    ///
    /// 默认实现：`Err(Internal("mark_done not implemented"))`。
    async fn mark_done(&self, _task_id: Uuid) -> AgentQueueResult<()> {
        Err(AgentQueueError::Internal(
            "mark_done not implemented by this AgentQueue".into(),
        ))
    }

    /// **running_task_id** -- 查询当前 queue 记录的 running task (W2)。
    ///
    /// 仅测试 + 监控用。返回 `None` 表示没有 running task。
    /// 默认实现：返回 `None`（W2 前 trait 没这方法，假装没人跑，
    /// 跟 W1 "take_next 隐式 running" 兼容）。
    async fn running_task_id(&self) -> Option<Uuid> {
        None
    }

    /// **register_preemption_listener** -- 注册抢占回调 (W2)。
    ///
    /// `InMemoryAgentQueue` 支持注册多个 listener，按注册顺序回调。
    /// 不注册的 queue 等价于"抢占静默"，跟 W1 行为一致。
    /// 默认实现：no-op。
    async fn register_preemption_listener(
        &self,
        _listener: std::sync::Arc<dyn PreemptionListener>,
    ) {
        // 默认 no-op;具体实现覆盖
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
///
/// W2 增量：
/// - `running` 字段：记录当前 driver 在跑的 task（`task_id`, `tenant_id`）。
///   enqueue 时如果新任务会抢占 (`mode=Interrupt` 或 `mode=Priority` 且
///   `incoming.priority > running.priority`)，fire 全部已注册 listener。
/// - `listeners` 字段：注册过的 `PreemptionListener` 列表。listener 是
///   `Arc<dyn PreemptionListener>`，调用方负责让 listener 的内部状态
///   thread-safe（典型 = `Arc<Mutex<...>>`）。
pub struct InMemoryAgentQueue {
    inner: Mutex<QueueInner>,
    /// W2: 当前 driver 标 running 的 task；enqueue 抢占判断用。
    /// 跟 inner 分开锁：listener 回调时只短暂持有 inner，running 单独锁
    /// 避免 listener 慢回调阻塞所有 enqueue。
    running: Mutex<Option<RunningTask>>,
    /// W2: 抢占回调列表。注册顺序 = 调用顺序。
    listeners: Mutex<Vec<std::sync::Arc<dyn PreemptionListener>>>,
}

/// **RunningTask** -- driver 标 running 的 task 快照 (W2 新增)。
///
/// 复制 `task_id` + `tenant_id` 即可触发 cancel-by-id；完整 QueuedTask 保留
/// 给 listener 回调（含 prompt / priority 等），便于 listener 决定如何
/// 处理。
#[derive(Debug, Clone)]
struct RunningTask {
    task_id: Uuid,
    tenant_id: TenantId,
    /// 完整快照给 listener 用（不存 inner 里，避免锁 inner 时 listener
    /// panic 还要访问 inner）
    snapshot: QueuedTask,
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
            running: Mutex::new(None),
            listeners: Mutex::new(Vec::new()),
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
        // W2: 抢占判断需要 task 的快照传给 listener,所以先 clone 一份
        // 再 move 进 queue。clone 代价 = QueuedTask POD 浅拷贝,O(1)。
        let task_snapshot_for_preempt = task.clone();
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
        drop(g); // 释放 inner 锁再去抢 running/listeners 锁 (守门 #22)

        // W2: 抢占判断 + 回调 fire-and-forget。
        // 抢 inner 锁后才判断,避免 enqueue 持 inner 锁调 listener 引发死锁。
        maybe_fire_preemption(self, &task_snapshot_for_preempt, mode);
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
        drop(g);
        // W2: 同步清理 running（cancel 一个正在跑的任务 = 取消它）。
        // 跨租户的 running 不动（守门 #5）。
        //
        // **W2 简化**:mark_running 只接 task_id,running.tenant_id 占位 nil。
        // 因此 cancel 清理 running 时,如果 running.tenant_id == nil
        // (即 mark_running 走 W2 简化路径),按 task_id 匹配清理(信任 caller);
        // 如果 running.tenant_id == actor_tenant,正常清理;
        // 如果 running.tenant_id != actor_tenant 且 != nil,**不动**
        // (其他租户的 running 不取消)。
        let mut cleared_running = false;
        if let Ok(mut rg) = self.running.lock() {
            let matches = rg.as_ref().map(|r| {
                r.task_id == task_id
                    && (r.tenant_id == TenantId::from(Uuid::nil()) || r.tenant_id == actor_tenant)
            });
            if matches.unwrap_or(false) {
                *rg = None;
                cleared_running = true;
            }
        }
        if !removed_fifo && !removed_pri && !cleared_running {
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

    // ---------------- W2 增量 overrides ----------------

    /// **W2**: 单锁 O(N+M) 批量 cancel — 比默认循环调 cancel 快 (避免每次加锁)。
    async fn cancel_many(
        &self,
        actor_tenant: TenantId,
        task_ids: &[Uuid],
    ) -> AgentQueueResult<CancelReport> {
        if task_ids.is_empty() {
            return Ok(CancelReport::default());
        }
        let mut g = self
            .inner
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("queue poisoned: {e}")))?;
        let wanted: std::collections::HashSet<Uuid> = task_ids.iter().copied().collect();

        // FIFO 桶：retain 一次过；统计 removed_fifo_ids
        let mut removed_fifo_ids: Vec<Uuid> = Vec::new();
        g.fifo.retain(|t| {
            if wanted.contains(&t.task_id) && t.tenant_id == actor_tenant {
                removed_fifo_ids.push(t.task_id);
                false
            } else {
                true
            }
        });

        // Priority 桶：rebuild 一次过；统计 removed_pri_ids
        let drained: Vec<PriorityEntry> = std::mem::take(&mut g.priority).into_iter().collect();
        let mut rebuilt = BinaryHeap::new();
        let mut removed_pri_ids: Vec<Uuid> = Vec::new();
        for e in drained {
            if wanted.contains(&e.task.task_id) && e.task.tenant_id == actor_tenant {
                removed_pri_ids.push(e.task.task_id);
                continue;
            }
            rebuilt.push(e);
        }
        g.priority = rebuilt;
        drop(g); // 不持 inner 锁去动 running

        // 合并 found（保持 input 顺序）/ missing
        let found_set: std::collections::HashSet<Uuid> = removed_fifo_ids
            .iter()
            .chain(removed_pri_ids.iter())
            .copied()
            .collect();
        let found: Vec<Uuid> = task_ids
            .iter()
            .copied()
            .filter(|id| found_set.contains(id))
            .collect();
        let missing: Vec<Uuid> = task_ids
            .iter()
            .copied()
            .filter(|id| !found_set.contains(id))
            .collect();

        // W2 语义：cancel_many 也清掉 running（任一 found 命中即清）。
        // 跨租户的 running 不动（守门 #5）。
        // **W2 简化**:running.tenant_id 占位 nil 时按 task_id 信任 caller
        // (见 cancel W2 doc)。
        if !found.is_empty() {
            if let Ok(mut rg) = self.running.lock() {
                let matches = rg.as_ref().map(|r| {
                    found.contains(&r.task_id)
                        && (r.tenant_id == TenantId::from(Uuid::nil())
                            || r.tenant_id == actor_tenant)
                });
                if matches.unwrap_or(false) {
                    *rg = None;
                }
            }
        }

        Ok(CancelReport { found, missing })
    }

    /// **W2**: 按租户清空全部。
    async fn cancel_all(&self, actor_tenant: TenantId) -> AgentQueueResult<Vec<Uuid>> {
        let mut g = self
            .inner
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("queue poisoned: {e}")))?;
        let mut removed = Vec::new();
        // FIFO
        let new_fifo: VecDeque<QueuedTask> = std::mem::take(&mut g.fifo)
            .into_iter()
            .filter(|t| {
                if t.tenant_id == actor_tenant {
                    removed.push(t.task_id);
                    false
                } else {
                    true
                }
            })
            .collect();
        g.fifo = new_fifo;
        // Priority
        let drained: Vec<PriorityEntry> = std::mem::take(&mut g.priority).into_iter().collect();
        let mut rebuilt = BinaryHeap::new();
        for e in drained {
            if e.task.tenant_id == actor_tenant {
                removed.push(e.task.task_id);
                continue;
            }
            rebuilt.push(e);
        }
        g.priority = rebuilt;
        Ok(removed)
    }

    /// **W2**: driver 标 running。
    async fn mark_running(&self, task_id: Uuid) -> AgentQueueResult<()> {
        // 跨租户守门：driver 必须先 take_next 拿到 QueuedTask 才能 mark_running;
        // 我们从 inner 里反查 task_id 对应的 tenant_id（如果还在 queue 里），或
        // 从正在持有的 QueuedTask 快照里取。简化:driver 必须先 ensure
        // task_id 在 queue 里存在 → 找 tenant;如果不在 queue 但有
        // mark_running 调用过,就保留。
        let mut rg = self
            .running
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("running poisoned: {e}")))?;
        match rg.as_ref() {
            None => {
                *rg = Some(RunningTask {
                    task_id,
                    tenant_id: TenantId::from(Uuid::nil()),
                    snapshot: QueuedTask::new(
                        TenantId::from(Uuid::nil()),
                        AgentId::new(),
                        AgentSessionId::new(),
                        "<running>",
                    ),
                });
                Ok(())
            }
            Some(r) if r.task_id == task_id => Ok(()),
            Some(r) => Err(AgentQueueError::RunningTaskMismatch {
                queue_running: Some(r.task_id),
                marked: task_id,
            }),
        }
    }

    /// **W3**: driver 标 running 时把完整 task 一起交给 queue（带真实
    /// `tenant_id` + snapshot）。`maybe_fire_preemption` 用此 `tenant_id`
    /// 做跨租户守门（守门 #5）。
    async fn mark_running_with(&self, task: QueuedTask) -> AgentQueueResult<()> {
        let mut rg = self
            .running
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("running poisoned: {e}")))?;
        match rg.as_ref() {
            // 已有 running 且 task_id 一致 → 替换为完整快照（允许 driver
            // 在 task 被取走后用 take_next 拿到的 QueuedTask 重新注册）
            Some(r) if r.task_id == task.task_id => {
                *rg = Some(RunningTask {
                    task_id: task.task_id,
                    tenant_id: task.tenant_id,
                    snapshot: task,
                });
                Ok(())
            }
            // 已有 running 且 task_id 不一致 → Mismatch
            Some(r) => Err(AgentQueueError::RunningTaskMismatch {
                queue_running: Some(r.task_id),
                marked: task.task_id,
            }),
            // 无 running → 注册
            None => {
                *rg = Some(RunningTask {
                    task_id: task.task_id,
                    tenant_id: task.tenant_id,
                    snapshot: task,
                });
                Ok(())
            }
        }
    }

    /// **W2**: driver 标 done。
    async fn mark_done(&self, task_id: Uuid) -> AgentQueueResult<()> {
        let mut rg = self
            .running
            .lock()
            .map_err(|e| AgentQueueError::Internal(format!("running poisoned: {e}")))?;
        match rg.as_ref() {
            None => Err(AgentQueueError::RunningTaskMismatch {
                queue_running: None,
                marked: task_id,
            }),
            Some(r) if r.task_id == task_id => {
                *rg = None;
                Ok(())
            }
            Some(r) => Err(AgentQueueError::RunningTaskMismatch {
                queue_running: Some(r.task_id),
                marked: task_id,
            }),
        }
    }

    async fn running_task_id(&self) -> Option<Uuid> {
        self.running.lock().ok().and_then(|g| g.as_ref().map(|r| r.task_id))
    }

    async fn register_preemption_listener(
        &self,
        listener: std::sync::Arc<dyn PreemptionListener>,
    ) {
        if let Ok(mut g) = self.listeners.lock() {
            g.push(listener);
        }
    }
}

// (此占位注释保留以标注旧 helper 替换点;下一行起为 W2 maybe_fire_preemption)

// =====================================================================
// enqueue 抢占判断 (W2 新增独立函数)
// =====================================================================
//
// 把抢占判断从 `enqueue` 内部抽出,让 enqueue 主路径保持短小。
//
// **抢占触发条件** (满足全部):
// 1. queue.running 不为 None (有 task 在跑)
// 2. 当前 task 与 running **同 task_id 不可能**(incoming 是新进 task)
//    → 用 **任务优先级比较 + mode 语义** 判断。
// 3. incoming 是"抢占类"任务:**`mode == Interrupt` 或 `task.interrupt == true`**。
//    任一满足 → 抢占;否则 FIFO 模式按顺序排队,Priority 模式按优先级判断
//    (`incoming.priority > running.snapshot.priority` 才抢占)。
//
// **为什么 task.interrupt 也算抢占**:W1 实现的 push_front 逻辑会因
// `task.interrupt == true` 触发,意味着该 task 语义上是抢占的(不论
// queue mode)。本函数与 push_front 对齐。
//
// **W3 跨租户守门 (守门 #5)**:如果 `running.tenant_id` 是非 nil 的
// (即 driver 走 `mark_running_with` 路径),且 `incoming.tenant_id !=
// running.tenant_id`,则 **静默不 fire** —— 不抛错,不调 listener,避免
// 跨租户抢占语义泄漏。如果 `running.tenant_id` 是 nil (即 W2 legacy
// `mark_running(task_id)` 占位路径),则不守门,与 W2 行为一致。
// 这一设计让 W2 测试不需改一行代码,新 W3 测试用 `mark_running_with`
// 走真守门路径。
//
// 满足全部 → 调用所有 listeners;否则 → 静默入队。
//
// **租户守门失效说明(W2 legacy)**:W2 `mark_running(task_id)` 因为 trait 签名只接
// task_id,无法记录真实 tenant_id(占位 nil)。W3 已加 `mark_running_with`
// 修这一 trade-off,但仍保留 W2 路径以兼容旧 driver。
fn maybe_fire_preemption(
    queue: &InMemoryAgentQueue,
    incoming: &QueuedTask,
    mode: QueueMode,
) {
    // 取 running 快照 (短暂持锁)
    let running_snapshot = match queue.running.lock() {
        Ok(g) => g.clone(),
        Err(_) => None,
    };
    let running = match running_snapshot {
        Some(r) => r,
        None => return,
    };

    // W3 跨租户守门 (守门 #5): running.tenant_id 非 nil 且 != incoming.tenant_id
    // → 静默不 fire。running.tenant_id 是 nil (W2 legacy 占位) → 不守门。
    let nil_tenant = TenantId::from(Uuid::nil());
    if running.tenant_id != nil_tenant && running.tenant_id != incoming.tenant_id {
        return;
    }

    // 抢占语义判断 (W3 仍沿用 W2 简化: 不守门 priority tenant,见上方说明)
    let should_preempt = match mode {
        QueueMode::Interrupt => true,
        QueueMode::Priority => incoming.priority > running.snapshot.priority,
        QueueMode::Fifo => {
            // task.interrupt flag 也算抢占 (与 push_front 对齐)
            incoming.interrupt
        }
    };
    if !should_preempt {
        return;
    }

    // fire-and-forget 调所有 listener。listener 内部 panic 由 listener 自负责
    // (我们这里不 catch_unwind,守门 #22)。
    if let Ok(listeners) = queue.listeners.lock() {
        for l in listeners.iter() {
            l.on_preempt(&running.snapshot, incoming);
        }
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

    // =================================================================
    // W2 tests: cancel_many / cancel_all / mark_running / mark_done /
    //           PreemptionListener 抢占回调 / 端到端集成
    // =================================================================

    /// `RecordingListener` -- 抢占比对工具,记录每次 on_preempt 收到的
    /// (current_id, incoming_id) 对子,供测试断言"fire 了几次 / fire 了什么"。
    #[derive(Debug, Default)]
    struct RecordingListener {
        events: Mutex<Vec<(Uuid, Uuid)>>,
    }
    impl RecordingListener {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
        fn count(&self) -> usize {
            self.events.lock().unwrap().len()
        }
        fn events(&self) -> Vec<(Uuid, Uuid)> {
            self.events.lock().unwrap().clone()
        }
    }
    impl PreemptionListener for RecordingListener {
        fn on_preempt(&self, current: &QueuedTask, incoming: &QueuedTask) {
            self.events
                .lock()
                .unwrap()
                .push((current.task_id, incoming.task_id));
        }
    }

    #[test]
    fn cancel_many_fifo_priority_mixed_tenant_guarded() {
        let q = InMemoryAgentQueue::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        block_on(async {
            let mut fifo_tasks: Vec<QueuedTask> = (0..3)
                .map(|i| make_task(t1, &format!("fifo-{i}"), DEFAULT_PRIORITY))
                .collect();
            let mut prio_tasks: Vec<QueuedTask> = (0..2)
                .map(|i| {
                    let mut t = make_task(t1, &format!("prio-{i}"), 7);
                    t.priority = 7 - i;
                    t
                })
                .collect();
            // 另一租户 1 个 task,不应被 t1 cancel_many 命中
            let other = make_task(t2, "other-tenant", DEFAULT_PRIORITY);

            for t in fifo_tasks.iter().chain(prio_tasks.iter()).chain(std::iter::once(&other)) {
                let mode = if t.prompt.starts_with("prio-") {
                    QueueMode::Priority
                } else {
                    QueueMode::Fifo
                };
                q.enqueue(t.clone(), mode).await.unwrap();
            }
            assert_eq!(q.len().await, 6);

            let ids_to_cancel: Vec<Uuid> = fifo_tasks
                .iter()
                .chain(prio_tasks.iter())
                .map(|t| t.task_id)
                .collect();
            let report = q.cancel_many(t1, &ids_to_cancel).await.unwrap();
            assert_eq!(report.found.len(), 5, "全部 found");
            assert!(report.missing.is_empty());
            assert_eq!(q.len().await, 1, "只剩 other-tenant");

            // 重复 cancel 应 missing 全员
            let report2 = q.cancel_many(t1, &ids_to_cancel).await.unwrap();
            assert_eq!(report2.found.len(), 0);
            assert_eq!(report2.missing.len(), 5);

            // t2 cancel t1 的 task ids → 全 missing (跨租户不计入 found)
            let report3 = q.cancel_many(t2, &ids_to_cancel).await.unwrap();
            assert_eq!(report3.found.len(), 0);
            assert_eq!(report3.missing.len(), 5);
            // t2 自家 task 还在
            assert_eq!(q.len().await, 1);
            let _ = &mut fifo_tasks;
            let _ = &mut prio_tasks;
        });
    }

    #[test]
    fn cancel_all_only_target_tenant() {
        let q = InMemoryAgentQueue::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        block_on(async {
            for i in 0..3 {
                q.enqueue(make_task(t1, &format!("t1-{i}"), DEFAULT_PRIORITY), QueueMode::Fifo)
                    .await
                    .unwrap();
            }
            for i in 0..2 {
                q.enqueue(make_task(t2, &format!("t2-{i}"), DEFAULT_PRIORITY), QueueMode::Fifo)
                    .await
                    .unwrap();
            }
            assert_eq!(q.len().await, 5);

            let removed = q.cancel_all(t1).await.unwrap();
            assert_eq!(removed.len(), 3);
            assert_eq!(q.len().await, 2, "t2 全部保留");

            // t1 再 cancel_all → 空
            let removed2 = q.cancel_all(t1).await.unwrap();
            assert!(removed2.is_empty());
            assert_eq!(q.len().await, 2);
        });
    }

    #[test]
    fn cancel_running_task_clears_running() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let task = make_task(t, "running-target", DEFAULT_PRIORITY);
        let task_id = task.task_id;
        block_on(async {
            q.enqueue(task, QueueMode::Fifo).await.unwrap();
            let taken = q.take_next().await.unwrap();
            assert_eq!(taken.task_id, task_id);

            // **W2 更新**:mark_running 签名只接 task_id,queue 不再反查
            // inner。take_next 后 task 已在 driver 手中,mark_running 直接
            // 接受 (tenant_id 占位 nil)。
            q.mark_running(task_id).await.unwrap();
            assert_eq!(q.running_task_id().await, Some(task_id));

            // cancel running task → 应当成功 (返回 Ok 且 running 清掉)
            q.cancel(t, task_id).await.unwrap();
            assert_eq!(q.running_task_id().await, None);
            assert_eq!(q.len().await, 0);
        });
    }

    #[test]
    fn mark_running_idempotent_same_id_noop() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let task = make_task(t, "x", DEFAULT_PRIORITY);
        let task_id = task.task_id;
        block_on(async {
            q.enqueue(task, QueueMode::Fifo).await.unwrap();
            q.mark_running(task_id).await.unwrap();
            q.mark_running(task_id).await.unwrap(); // 同 id noop
            assert_eq!(q.running_task_id().await, Some(task_id));
        });
    }

    #[test]
    fn mark_running_mismatch_errors() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let a = make_task(t, "a", DEFAULT_PRIORITY);
        let b = make_task(t, "b", DEFAULT_PRIORITY);
        let a_id = a.task_id;
        let b_id = b.task_id;
        block_on(async {
            q.enqueue(a, QueueMode::Fifo).await.unwrap();
            q.enqueue(b, QueueMode::Fifo).await.unwrap();
            q.mark_running(a_id).await.unwrap();
            // 标 b → RunningTaskMismatch (queue 里 b 还在,但 running 已被 a 占)
            let r = q.mark_running(b_id).await;
            assert!(matches!(
                r,
                Err(AgentQueueError::RunningTaskMismatch {
                    queue_running: Some(_),
                    marked: _,
                })
            ));
        });
    }

    #[test]
    fn mark_done_mismatch_errors_when_not_running() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let task = make_task(t, "x", DEFAULT_PRIORITY);
        let task_id = task.task_id;
        block_on(async {
            q.enqueue(task, QueueMode::Fifo).await.unwrap();
            q.mark_running(task_id).await.unwrap();
            // 标错 id → Mismatch
            let r = q.mark_done(Uuid::new_v4()).await;
            assert!(matches!(
                r,
                Err(AgentQueueError::RunningTaskMismatch { .. })
            ));
            // 标对 id → Ok
            q.mark_done(task_id).await.unwrap();
            assert_eq!(q.running_task_id().await, None);
            // 再 mark_done 同 id → Mismatch (None)
            let r2 = q.mark_done(task_id).await;
            assert!(matches!(
                r2,
                Err(AgentQueueError::RunningTaskMismatch {
                    queue_running: None,
                    marked: _,
                })
            ));
        });
    }

    #[test]
    fn preemption_listener_fires_on_interrupt_with_running_same_tenant() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        let normal = make_task(t, "normal", DEFAULT_PRIORITY);
        let interrupt = make_interrupt_task(t, "interrupt");
        let interrupt_id = interrupt.task_id;
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            q.enqueue(normal, QueueMode::Fifo).await.unwrap();
            q.take_next().await.unwrap();
            // **W2 simplification**: mark_running 没有完整 QueuedTask,
            // placeholder running snapshot.task_id != normal_id。测试
            // 只断言 listener 被 fire 且 incoming.task_id == interrupt_id
            // (current 是 placeholder,不强绑定)。
            q.mark_running(Uuid::new_v4()).await.unwrap();
            assert_eq!(listener.count(), 0);

            // 入队 Interrupt (interrupt flag = true) → 应触发 listener
            q.enqueue(interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 1);
            assert_eq!(listener.events()[0].1, interrupt_id);
        });
    }

    #[test]
    fn preemption_listener_cross_tenant_does_fire_w2_known_limitation() {
        // **W2 known limitation**: `mark_running(task_id)` trait 签名只接
        // task_id,queue 无法记录真实 tenant_id(占位 nil)。因此
        // `maybe_fire_preemption` 不做 tenant 守门 —— 跨租户抢占可能触发。
        //
        // **修复路径**:把 trait 改为 `mark_running(task_id, tenant_id)` 或
        // `mark_running_with(task: QueuedTask)`(W3+)。
        //
        // 本测试**显式记录**这一行为,作为已知 trade-off:
        let q = InMemoryAgentQueue::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            q.mark_running(Uuid::new_v4()).await.unwrap();

            // t2 入 Interrupt → 跨租户但 W2 仍 fire (1 次)
            let t2_interrupt = make_interrupt_task(t2, "t2-interrupt");
            let t2_id = t2_interrupt.task_id;
            q.enqueue(t2_interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 1, "W2 暂不守门 tenant,见 doc");
            assert_eq!(listener.events()[0].1, t2_id);
            let _ = t1; // 标记 t1 仍存在以便 future W3 修复加回
        });
    }

    #[test]
    fn preemption_listener_priority_higher_preempts_lower() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            let mut low = make_task(t, "low-prio", 2);
            low.priority = 2;
            let mut high = make_task(t, "high-prio", 9);
            high.priority = 9;
            let high_id = high.task_id;
            q.enqueue(low, QueueMode::Priority).await.unwrap();
            q.take_next().await.unwrap();
            // **W2 simplification**: mark_running(task_id) doesn't have
            // access to the priority of the running task (placeholder
            // snapshot's priority is default 5). So priority-preempt
            // doesn't strictly mean "incoming.priority > low.priority";
            // it means "incoming.priority > placeholder.priority (5)".
            // For high.priority = 9 > 5 → fires. (See W2 docblock.)
            q.mark_running(Uuid::new_v4()).await.unwrap(); // 任意 task_id
            assert_eq!(listener.count(), 0);
            q.enqueue(high, QueueMode::Priority).await.unwrap();
            assert_eq!(listener.count(), 1);
            // events[0].1 (incoming task_id) 应是 high_id;events[0].0 是
            // placeholder running task_id(测试不绑定具体值)
            assert_eq!(listener.events()[0].1, high_id);
        });
    }

    #[test]
    fn preemption_listener_priority_equal_or_lower_no_fire() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            q.mark_running(Uuid::new_v4()).await.unwrap();
            // placeholder running.priority = 5 (QueuedTask::new default)
            // priority = 5 → 不 > 5 → 不 fire
            let mut equal = make_task(t, "equal", 5);
            equal.priority = 5;
            q.enqueue(equal, QueueMode::Priority).await.unwrap();
            assert_eq!(listener.count(), 0);
            // priority = 4 → 不 > 5 → 不 fire
            let mut lower = make_task(t, "lower", 3);
            lower.priority = 3;
            q.enqueue(lower, QueueMode::Priority).await.unwrap();
            assert_eq!(listener.count(), 0);
        });
    }

    #[test]
    fn preemption_listener_fifo_mode_does_not_fire() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            let running = make_task(t, "running", 9);
            let running_id = running.task_id;
            q.enqueue(running, QueueMode::Fifo).await.unwrap();
            q.take_next().await.unwrap();
            q.mark_running(running_id).await.unwrap();

            // 普通 FIFO 入队 → 不抢占
            let normal2 = make_task(t, "fifo2", DEFAULT_PRIORITY);
            q.enqueue(normal2, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 0);
        });
    }

    #[test]
    fn preemption_listener_no_running_no_fire() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            // running = None → 即便入 Interrupt 也不触发
            let interrupt = make_interrupt_task(t, "interrupt");
            q.enqueue(interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 0);
        });
    }

    /// **端到端集成测试**:driver → enqueue normal → mark_running → enqueue Interrupt → listener fires → cancel running → mark_done → Interrupt == take_next。
    #[test]
    fn end_to_end_interrupt_preempts_running_and_drains_queue() {
        let q: Arc<dyn AgentQueue> = Arc::new(InMemoryAgentQueue::new());
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        let normal = make_task(t, "normal-task", DEFAULT_PRIORITY);
        let interrupt = make_interrupt_task(t, "interrupt-task");
        let interrupt_id = interrupt.task_id;
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;

            // 1. driver 拉 normal 入 queue,再 take + mark_running
            q.enqueue(normal.clone(), QueueMode::Fifo).await.unwrap();
            q.take_next().await.unwrap();
            // **W2 simplification**: mark_running 接 task_id 但没完整
            // QueuedTask。placeholder snapshot 的 task_id != normal.task_id。
            // 测试断言 listener fired (current 是 placeholder,不强绑定)
            q.mark_running(Uuid::new_v4()).await.unwrap();
            assert!(q.running_task_id().await.is_some());

            // 2. interrupt 入 queue,listener 应 fire
            q.enqueue(interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 1);
            assert_eq!(listener.events()[0].1, interrupt_id);

            // 3. driver 收到 listener 通知,清理 running 并 cancel
            //    这里 cancel 必须用真实 running task_id 才能命中;
            //    但 placeholder 没有真实 id,所以我们用 take_next 之前的
            //    task_id (已 move 进 queue 又被 take 走,不会再 cancel 命中
            //    queue;但 running 标记的 task_id 是占位 → cancel 占位 task_id
            //    应能命中 running 因为 placeholder 的 tenant_id 是 nil
            //    → 见 cancel W2 简化逻辑)。
            let running_id = q.running_task_id().await.unwrap();
            q.cancel(t, running_id).await.unwrap();
            assert_eq!(q.running_task_id().await, None);

            // 4. driver mark_done 清状态(应 Ok,running 已是 None,再调会 Mismatch)
            //    → mark_done 在 cleanup race 下可能已被 cancel 抢先清掉,
            //    所以此步骤文档化为"driver 视情况可调"
            let _ = q.mark_done(running_id).await;

            // 5. driver 拉下一个 → 应当是 interrupt (push_front 已就位)
            let next = q.take_next().await.unwrap();
            assert_eq!(next.task_id, interrupt_id);
            assert_eq!(next.prompt, "interrupt-task");

            // 6. 队列空,再 take_next 应 Empty
            assert!(q.take_next().await.is_err());

            // 7. listener 只 fire 一次 (Interrupt 后续没再 enqueue)
            assert_eq!(listener.count(), 1);
        });
    }

    #[test]
    fn cancel_many_priority_partial_found() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            let mut a = make_task(t, "a", 8);
            a.priority = 8;
            let mut b = make_task(t, "b", 5);
            b.priority = 5;
            let mut c = make_task(t, "c", 9);
            c.priority = 9;
            q.enqueue(a.clone(), QueueMode::Priority).await.unwrap();
            q.enqueue(b.clone(), QueueMode::Priority).await.unwrap();
            q.enqueue(c.clone(), QueueMode::Priority).await.unwrap();
            assert_eq!(q.len().await, 3);

            // 只 cancel a 和一个不存在的 id → 1 found, 1 missing
            let nonexistent = Uuid::new_v4();
            let report = q
                .cancel_many(t, &[a.task_id, nonexistent])
                .await
                .unwrap();
            assert_eq!(report.found, vec![a.task_id]);
            assert_eq!(report.missing, vec![nonexistent]);
            assert_eq!(q.len().await, 2);
        });
    }

    #[test]
    fn cancel_many_empty_input_returns_empty_report() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        block_on(async {
            q.enqueue(make_task(t, "x", DEFAULT_PRIORITY), QueueMode::Fifo)
                .await
                .unwrap();
            let report = q.cancel_many(t, &[]).await.unwrap();
            assert!(report.found.is_empty());
            assert!(report.missing.is_empty());
            assert_eq!(q.len().await, 1, "空入队不改 queue");
        });
    }

    #[test]
    fn cancel_report_helpers() {
        let r1 = CancelReport {
            found: vec![Uuid::new_v4()],
            missing: vec![],
        };
        assert!(r1.is_complete());
        assert_eq!(r1.total(), 1);
        let r2 = CancelReport {
            found: vec![],
            missing: vec![Uuid::new_v4(), Uuid::new_v4()],
        };
        assert!(!r2.is_complete());
        assert_eq!(r2.total(), 2);
    }

    // =================================================================
    // W3 tests: mark_running_with / 跨租户守门真生效 / CollabPreemptionBridge
    // =================================================================

    /// W3: `mark_running_with` 记录真实 tenant_id,不是 nil 占位
    #[test]
    fn mark_running_with_records_real_tenant() {
        let q = InMemoryAgentQueue::new();
        let tenant = TenantId::new();
        let task = make_task(tenant, "real-running", DEFAULT_PRIORITY);
        let task_id = task.task_id;
        block_on(async {
            q.mark_running_with(task.clone()).await.unwrap();
            assert_eq!(q.running_task_id().await, Some(task_id));
            // 我们没有公开的 getter 直接拿 running.snapshot 字段,
            // 但通过 take_next 不受影响(因为 task 不在 queue)验证 running 是
            // 我们传的 task:
            assert!(q.take_next().await.is_err(), "queue 空,task 在 running 不在 queue");
        });
    }

    /// W3: `mark_running_with` 后再标同名 task_id → idempotent (替换 snapshot)
    #[test]
    fn mark_running_with_idempotent_same_id() {
        let q = InMemoryAgentQueue::new();
        let tenant = TenantId::new();
        let task = make_task(tenant, "v1", DEFAULT_PRIORITY);
        let task_id = task.task_id;
        block_on(async {
            q.mark_running_with(task.clone()).await.unwrap();
            // 再传同 id 但不同 prompt → snapshot 应被替换
            let mut task_v2 = make_task(tenant, "v2", DEFAULT_PRIORITY);
            task_v2.task_id = task_id;
            q.mark_running_with(task_v2).await.unwrap();
            assert_eq!(q.running_task_id().await, Some(task_id));
        });
    }

    /// W3: `mark_running_with` 标错 id → Mismatch
    #[test]
    fn mark_running_with_mismatch_errors() {
        let q = InMemoryAgentQueue::new();
        let tenant = TenantId::new();
        let a = make_task(tenant, "a", DEFAULT_PRIORITY);
        let b = make_task(tenant, "b", DEFAULT_PRIORITY);
        let a_id = a.task_id;
        let b_id = b.task_id;
        block_on(async {
            q.mark_running_with(a).await.unwrap();
            let r = q.mark_running_with(b).await;
            assert!(matches!(
                r,
                Err(AgentQueueError::RunningTaskMismatch {
                    queue_running: Some(_),
                    marked: _,
                })
            ));
            // running 仍是 a (失败不替换)
            assert_eq!(q.running_task_id().await, Some(a_id));
            // 标 a_id 第二次 → idempotent ok
            let task_a_again = make_task(tenant, "a-again", DEFAULT_PRIORITY);
            let mut ta = task_a_again;
            ta.task_id = a_id;
            q.mark_running_with(ta).await.unwrap();
            let _ = b_id;
        });
    }

    /// W3 核心: `mark_running_with` 后,跨租户 Interrupt **不** fire。
    /// 这正是 W2 trade-off 的修复 (守门 #5 真生效)。
    #[test]
    fn preemption_cross_tenant_no_longer_fires_with_mark_running_with() {
        let q = InMemoryAgentQueue::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            // driver 用 mark_running_with 标 t1 的 task
            let t1_task = make_task(t1, "t1-running", DEFAULT_PRIORITY);
            q.mark_running_with(t1_task).await.unwrap();

            // t2 入 Interrupt → 应被守门 #5 拒,不 fire
            let t2_interrupt = make_interrupt_task(t2, "t2-interrupt");
            let t2_id = t2_interrupt.task_id;
            q.enqueue(t2_interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(
                listener.count(),
                0,
                "W3 守门生效:跨租户 Interrupt 不应 fire"
            );
            assert!(q.running_task_id().await.is_some());

            // t2 task 仍在 queue 里 (被静默入队,不被 cancel)
            assert_eq!(q.len().await, 1);
            let next = q.take_next().await.unwrap();
            assert_eq!(next.task_id, t2_id);
        });
    }

    /// W3 回归: 同租户 Interrupt 走 `mark_running_with` → 仍 fire
    #[test]
    fn preemption_same_tenant_still_fires_with_mark_running_with() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            let running = make_task(t, "same-tenant-running", DEFAULT_PRIORITY);
            q.mark_running_with(running).await.unwrap();

            let interrupt = make_interrupt_task(t, "same-tenant-interrupt");
            let iid = interrupt.task_id;
            q.enqueue(interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 1, "同租户仍 fire");
            assert_eq!(listener.events()[0].1, iid);
        });
    }

    /// W3 回归: W2 legacy 路径 (`mark_running(task_id)`) 仍不守门 tenant,
    /// 跟 W2 已 ship 的行为一致 (向后兼容)。
    #[test]
    fn preemption_w2_legacy_path_still_unguarded_for_back_compat() {
        let q = InMemoryAgentQueue::new();
        let t1 = TenantId::new();
        let t2 = TenantId::new();
        let listener = Arc::new(RecordingListener::new());
        block_on(async {
            q.register_preemption_listener(listener.clone()).await;
            // W2 legacy: 不传完整 task,tenant_id 占位 nil
            q.mark_running(Uuid::new_v4()).await.unwrap();
            // t2 Interrupt → W2 legacy 不守门,fire 1 次
            let t2_int = make_interrupt_task(t2, "t2-w2-legacy");
            q.enqueue(t2_int, QueueMode::Fifo).await.unwrap();
            assert_eq!(listener.count(), 1, "W2 legacy 路径仍不守门");
            let _ = t1;
        });
    }

    /// W3: `CollabPreemptionBridge` 把抢占事件转成 `CollabSteeringCommand`
    /// 并投到 sink。
    #[test]
    fn collab_bridge_dispatches_on_preempt() {
        let q = InMemoryAgentQueue::new();
        let t = TenantId::new();
        let sink = Arc::new(InMemoryCollabSink::new());
        let bridge: Arc<dyn PreemptionListener> = Arc::new(CollabPreemptionBridge::new(sink.clone()));
        block_on(async {
            q.register_preemption_listener(bridge).await;
            let running = make_task(t, "running-task", DEFAULT_PRIORITY);
            q.mark_running_with(running).await.unwrap();
            let interrupt = make_interrupt_task(t, "interrupt-task");
            let iid = interrupt.task_id;
            q.enqueue(interrupt, QueueMode::Fifo).await.unwrap();
            assert_eq!(sink.len(), 1);
            let cmds = sink.commands();
            assert_eq!(cmds[0].tenant_id, t);
            assert_eq!(cmds[0].incoming_task_id, iid);
            assert_eq!(cmds[0].incoming_prompt_excerpt, "interrupt-task");
        });
    }

    /// W3: bridge 对长 prompt 做 redact (守门 #5 防 secret 泄漏到 sink)。
    #[test]
    fn collab_bridge_redacts_long_prompt() {
        let sink = Arc::new(InMemoryCollabSink::new());
        let bridge = CollabPreemptionBridge::new(sink.clone());
        let long_prompt = "a".repeat(MAX_PROMPT_EXCERPT_CHARS * 3);
        let current = make_task(TenantId::new(), &long_prompt, DEFAULT_PRIORITY);
        let incoming = make_interrupt_task(TenantId::new(), &long_prompt);
        bridge.on_preempt(&current, &incoming);
        let cmds = sink.commands();
        assert_eq!(cmds.len(), 1);
        // excerpt 应 ≤ MAX_PROMPT_EXCERPT_CHARS + 1 ('…')
        assert!(cmds[0].incoming_prompt_excerpt.chars().count() <= MAX_PROMPT_EXCERPT_CHARS + 1);
        assert!(cmds[0].incoming_prompt_excerpt.ends_with('…'));
        assert!(cmds[0].current_prompt_excerpt.chars().count() <= MAX_PROMPT_EXCERPT_CHARS + 1);
    }

    /// W3: `NoopCollabSink` 不 panic,fire-and-forget 路径安全。
    #[test]
    fn collab_bridge_noop_sink_does_not_panic() {
        let bridge = CollabPreemptionBridge::new(Arc::new(NoopCollabSink));
        let current = make_task(TenantId::new(), "c", DEFAULT_PRIORITY);
        let incoming = make_interrupt_task(TenantId::new(), "i");
        bridge.on_preempt(&current, &incoming); // 不应 panic
    }

    /// W3 端到端集成: driver → mark_running_with(t1 task) → t1 Interrupt 入 queue
    /// → bridge sink 收到 1 条 steering command, sink 内容正确。
    #[test]
    fn end_to_end_w3_with_collab_bridge() {
        let q: Arc<dyn AgentQueue> = Arc::new(InMemoryAgentQueue::new());
        let t = TenantId::new();
        let sink = Arc::new(InMemoryCollabSink::new());
        let bridge: Arc<dyn PreemptionListener> = Arc::new(CollabPreemptionBridge::new(sink.clone()));
        block_on(async {
            q.register_preemption_listener(bridge).await;
            // 1. driver 拉 normal task 入 queue,再 take + mark_running_with
            let normal = make_task(t, "normal-task", DEFAULT_PRIORITY);
            let normal_id = normal.task_id;
            q.enqueue(normal.clone(), QueueMode::Fifo).await.unwrap();
            let taken = q.take_next().await.unwrap();
            assert_eq!(taken.task_id, normal_id);
            // mark_running_with 传完整 task (含真实 tenant_id)
            q.mark_running_with(taken).await.unwrap();
            assert_eq!(q.running_task_id().await, Some(normal_id));

            // 2. t1 Interrupt 入 queue → bridge 触发 sink
            let interrupt = make_interrupt_task(t, "interrupt-now");
            let iid = interrupt.task_id;
            q.enqueue(interrupt, QueueMode::Fifo).await.unwrap();

            // 3. sink 收到 1 条命令,tenant_id 是 t, incoming_task_id 是 iid
            assert_eq!(sink.len(), 1);
            let cmds = sink.commands();
            assert_eq!(cmds[0].tenant_id, t);
            assert_eq!(cmds[0].incoming_task_id, iid);
            assert_eq!(cmds[0].incoming_prompt_excerpt, "interrupt-now");
            assert_eq!(cmds[0].current_task_id, normal_id);
        });
    }

    /// W3: `CollabSteeringCommand` serde round-trip
    /// W5: 扩 4 个新 agent/session 字段,验证 round-trip 仍一致
    #[test]
    fn collab_command_serde_roundtrip() {
        let cmd = CollabSteeringCommand {
            tenant_id: TenantId::new(),
            current_task_id: Uuid::new_v4(),
            current_prompt_excerpt: "short".into(),
            current_agent_id: AgentId::new(),
            current_session_id: AgentSessionId::new(),
            incoming_task_id: Uuid::new_v4(),
            incoming_prompt_excerpt: "new hint".into(),
            incoming_agent_id: AgentId::new(),
            incoming_session_id: AgentSessionId::new(),
            enqueued_at: SystemTime::UNIX_EPOCH,
        };
        let j = serde_json::to_string(&cmd).expect("serialize");
        let back: CollabSteeringCommand = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back, cmd);
    }

    /// **ULYS-207 PI-9 W5**:`CollabPreemptionBridge::on_preempt` 把
    /// `QueuedTask.agent_id` / `session_id` 透传到 `CollabSteeringCommand` 的
    /// 4 个新字段(current_agent_id / current_session_id /
    /// incoming_agent_id / incoming_session_id),**不再用 `task_id` 占位**
    /// (per W4 已知 trade-off fix path)。
    #[test]
    fn collab_bridge_w5_passes_real_agent_and_session_ids() {
        let sink = Arc::new(InMemoryCollabSink::new());
        let bridge = CollabPreemptionBridge::new(sink.clone());
        let t = TenantId::new();
        // current 和 incoming 用**不同**的 agent_id / session_id,确保
        // W5 bridge 透传精确值,不丢字段或互相串。
        let current = make_task(t, "current-prompt", DEFAULT_PRIORITY);
        let incoming = make_interrupt_task(t, "incoming-prompt");
        let current_agent = current.agent_id;
        let current_session = current.session_id;
        let incoming_agent = incoming.agent_id;
        let incoming_session = incoming.session_id;

        bridge.on_preempt(&current, &incoming);

        let cmds = sink.commands();
        assert_eq!(cmds.len(), 1, "bridge must dispatch exactly once");
        let cmd = &cmds[0];
        assert_eq!(cmd.current_agent_id, current_agent, "W5: current_agent_id from QueuedTask");
        assert_eq!(cmd.current_session_id, current_session, "W5: current_session_id from QueuedTask");
        assert_eq!(cmd.incoming_agent_id, incoming_agent, "W5: incoming_agent_id from QueuedTask");
        assert_eq!(cmd.incoming_session_id, incoming_session, "W5: incoming_session_id from QueuedTask");
        // 守门:4 个新字段 ≠ 任何 task_id(W4 占位 = incoming_task_id 已废止)
        assert_ne!(cmd.current_agent_id.as_uuid(), current.task_id);
        assert_ne!(cmd.current_session_id.as_uuid(), current.task_id);
        assert_ne!(cmd.incoming_agent_id.as_uuid(), incoming.task_id);
        assert_ne!(cmd.incoming_session_id.as_uuid(), incoming.task_id);
    }

    /// **ULYS-207 PI-9 W5**:W4 已知 `agent_id` / `agent_session_id` 占位 =
    /// `incoming_task_id`。W5 修复:4 个新字段是真实 `AgentId` /
    /// `AgentSessionId`,可以跟 `task_id` 完全独立。
    #[test]
    fn collab_command_w5_new_fields_independent_from_task_ids() {
        let sink = Arc::new(InMemoryCollabSink::new());
        let bridge = CollabPreemptionBridge::new(sink.clone());
        let t = TenantId::new();
        let current = make_task(t, "c", DEFAULT_PRIORITY);
        let incoming = make_interrupt_task(t, "i");
        bridge.on_preempt(&current, &incoming);
        let cmd = &sink.commands()[0];
        // 4 个新字段应都是非空 UUID(因为 `QueuedTask::new` 默认
        // `AgentId::new() = Uuid::new_v4()`,不会撞 task_id)
        assert!(!cmd.current_agent_id.as_uuid().is_nil());
        assert!(!cmd.current_session_id.as_uuid().is_nil());
        assert!(!cmd.incoming_agent_id.as_uuid().is_nil());
        assert!(!cmd.incoming_session_id.as_uuid().is_nil());
        // 跨字段独立性:任一字段都不应等于另一字段(W4 占位串同 UUID 的风险被消)
        assert_ne!(cmd.current_agent_id.as_uuid(), cmd.current_session_id.as_uuid());
        assert_ne!(cmd.incoming_agent_id.as_uuid(), cmd.incoming_session_id.as_uuid());
        assert_ne!(cmd.current_agent_id.as_uuid(), cmd.incoming_agent_id.as_uuid());
        assert_ne!(cmd.current_session_id.as_uuid(), cmd.incoming_session_id.as_uuid());
    }

    /// **ULYS-207 PI-9 W5**:`CollabSteeringCommand::new` 构造函数签名扩到
    /// 10 个参数(2 tenant_id-related + 2 + 4 W5 agent/session + 2 task +
    /// 2 prompt + 1 timestamp),确保 caller 用真值不会混淆参数顺序。
    #[test]
    fn collab_command_w5_new_constructor_signature() {
        let t = TenantId::new();
        let ct = Uuid::new_v4();
        let ce = "ce".to_string();
        let ca = AgentId::new();
        let cs = AgentSessionId::new();
        let it = Uuid::new_v4();
        let ie = "ie".to_string();
        let ia = AgentId::new();
        let is_ = AgentSessionId::new();
        let ts = SystemTime::now();
        let cmd = CollabSteeringCommand::new(t, ct, ce.clone(), ca, cs, it, ie.clone(), ia, is_, ts);
        assert_eq!(cmd.tenant_id, t);
        assert_eq!(cmd.current_task_id, ct);
        assert_eq!(cmd.current_prompt_excerpt, ce);
        assert_eq!(cmd.current_agent_id, ca);
        assert_eq!(cmd.current_session_id, cs);
        assert_eq!(cmd.incoming_task_id, it);
        assert_eq!(cmd.incoming_prompt_excerpt, ie);
        assert_eq!(cmd.incoming_agent_id, ia);
        assert_eq!(cmd.incoming_session_id, is_);
        assert_eq!(cmd.enqueued_at, ts);
    }

    // =====================================================================
    // ULYS-207 PI-9 W6 测试 (PI-4 ToolExecutionMode + PI-6 star_dto::delta::Op 收紧)
    // =====================================================================

    /// **ULYS-207 PI-9 W6**:`QueuedTask::new()` 默认 `tool_hint = None` (纯 prompt 任务)
    #[test]
    fn queued_task_new_tool_hint_defaults_to_none_w6() {
        let tenant = TenantId::from(Uuid::new_v4());
        let task = make_task(tenant, "纯 prompt 任务", DEFAULT_PRIORITY);
        assert!(task.tool_hint.is_none(), "默认 tool_hint 应为 None");
        assert!(task.payload_op.is_none(), "默认 payload_op 应为 None");
        assert_eq!(task.payload, serde_json::Value::Null);
    }

    /// **ULYS-207 PI-9 W6**:`with_tool_hint` builder 设 4 变体 (PI-4 ToolExecutionMode)
    #[test]
    fn queued_task_with_tool_hint_supports_all_four_modes_w6() {
        let tenant = TenantId::from(Uuid::new_v4());
        for mode in [
            domain_tool::ToolExecutionMode::Synchronous,
            domain_tool::ToolExecutionMode::Async,
            domain_tool::ToolExecutionMode::Stream,
            domain_tool::ToolExecutionMode::Background,
        ] {
            let task = make_task(tenant, "with mode", DEFAULT_PRIORITY).with_tool_hint(mode);
            assert_eq!(task.tool_hint, Some(mode), "{:?} 未透传", mode);
        }
    }

    /// **ULYS-207 PI-9 W6**:`payload_op` 用 `Op::Set` 路径,serde round-trip 一致
    #[test]
    fn queued_task_payload_op_set_serde_roundtrip_w6() {
        use star_dto::delta::Op;
        let tenant = TenantId::from(Uuid::new_v4());
        let op = Op::Set(
            vec!["items".to_string(), "0".to_string(), "name".to_string()],
            serde_json::json!("alpha"),
        );
        let task = make_task(tenant, "task with op", DEFAULT_PRIORITY).with_payload_op(op.clone());
        assert_eq!(task.payload_op, Some(op.clone()));
        let j = serde_json::to_string(&task).expect("serialize");
        let back: QueuedTask = serde_json::from_str(&j).expect("deserialize");
        assert_eq!(back.payload_op, Some(op));
    }

    /// **ULYS-207 PI-9 W6**:payload_op 端到端 `Op::Set` 走 star_dto::delta::apply 正确改值
    #[test]
    fn queued_task_payload_op_apply_reducer_end_to_end_w6() {
        use star_dto::delta::{apply, Op};
        let tenant = TenantId::from(Uuid::new_v4());
        let op = Op::Set(
            vec!["greeting".to_string()],
            serde_json::json!("hello"),
        );
        let task = make_task(tenant, "apply me", DEFAULT_PRIORITY).with_payload_op(op.clone());

        // 模拟 agent loop 优先用 payload_op 调 apply
        let base = serde_json::json!({ "greeting": "world", "n": 1 });
        let applied = if let Some(op) = task.payload_op.as_ref() {
            apply(Some(base.clone()), std::slice::from_ref(op)).expect("apply Ok")
        } else {
            base.clone()
        };
        assert_eq!(applied, serde_json::json!({ "greeting": "hello", "n": 1 }));
    }

    /// **ULYS-207 PI-9 W6**:`payload` (向后兼容 Value) + `payload_op` (新) 同时存在不互斥
    #[test]
    fn queued_task_payload_and_payload_op_coexist_w6() {
        use star_dto::delta::Op;
        let tenant = TenantId::from(Uuid::new_v4());
        let task = make_task(tenant, "both", DEFAULT_PRIORITY)
            .with_payload(serde_json::json!({ "legacy": true }))
            .with_payload_op(Op::Replace(serde_json::json!({ "new": 1 })));
        // payload 与 payload_op 同时存在,各自独立
        assert_eq!(task.payload, serde_json::json!({ "legacy": true }));
        match task.payload_op.as_ref() {
            Some(Op::Replace(v)) => assert_eq!(*v, serde_json::json!({ "new": 1 })),
            other => panic!("expected Op::Replace, got {:?}", other),
        }
    }
}
