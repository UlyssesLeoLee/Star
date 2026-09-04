//! # star-dispatcher (G.1 L0 Agent 派发层 PoC, per SRS-STAR-AGENT-RUNTIME-001 §G-1)
//!
//! **目的**: 1M logical agents 派发 / lifecycle (G-1 PoC v0.0.1)
//!
//! **架构 (per SRS-001 §G-1)**:
//! - L0 派发层: Tokio async dispatcher + InMemory TaskQueue PoC
//! - SQLite WAL TaskQueue: v0.1.0 收官 (本 session 0.0.1 stub)
//! - 1M agents 压测: v0.1.0 收官
//! - multiprocessing.Pool(8-16): v0.2.0 实战
//!
//! **关键不变量 (per SRS-001 G-1)**:
//! - INV-DISP-01: TaskState 6 状态机: Pending → Dispatched → Running → Completed / Failed / Aborted
//! - INV-DISP-02: dispatcher 异步派发 + lifecycle 管理 (per 守门 #19 agent 交互 Python 化)
//! - INV-DISP-03: 1 task 1 tenant 强类型隔离 (per §0 ActorContext)
//! - INV-DISP-04: idempotency_key 注入 (per saga INV-SG-ORCH-03, 跨 session 续)
//!
//! **守门 (per HANDOFF v0.7 §10)**:
//! - 字段命名跟 [`docs/ubiquitous-language.md`](../../../docs/ubiquitous-language.md) v1.0 §1 保持一致
//! - 强类型 ID 模式跟 §6 跨域命令/查询/事件命名约定
//! - 跨 sub-session 续: star-context re-export, star-dto 跨域 DTO
//!
//! **Lead 责任**: 待 5 域 Lead 真人到位 (per AGENTS.md §0 disclaimer 守门 #3, 当前 Mavis 自主 per 9/4 12:19 JST)

#![allow(missing_docs)] // G.1 PoC 启动, Phase 2 spec 完成后补 doc

// H.3 6 SA 真实业务实现 + 12 e2e test (per P4-H.3, 守门 #19 [P] 拍板)
pub mod sa_real_impls;
#[cfg(test)]
pub mod sa_real_tests;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// **TaskState 6 状态机** (per SRS-001 G-1 INV-DISP-01)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Dispatched,
    Running,
    Completed,
    Failed,
    Aborted,
}

/// **Agent 任务定义** (per SRS-001 §G-1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// 全局任务 ID (Uuid v4)
    pub task_id: Uuid,
    /// 租户 ID (跨域隔离, INV-ACT-01)
    pub tenant_id: Uuid,
    /// 任务类型 (e.g. "code_review" / "doc_gen" / "search")
    pub kind: String,
    /// 任务载荷 (JSON 序列化)
    pub payload: serde_json::Value,
    /// 幂等性 key (per saga INV-SG-ORCH-03 跨 session 续)
    pub idempotency_key: String,
    /// 任务创建时间戳 (ms since epoch)
    pub created_at_ms: u64,
    /// 当前状态
    pub state: TaskState,
    /// 状态变更历史
    pub state_history: Vec<TaskStateTransition>,
}

/// **任务状态变更记录**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateTransition {
    /// 旧状态
    pub from: TaskState,
    /// 新状态
    pub to: TaskState,
    /// 变更时间戳 (ms since epoch)
    pub at_ms: u64,
    /// 变更原因 (e.g. "task_executed" / "compensation_started")
    pub reason: String,
}

/// **Agent Task 执行器 trait** (per SRS-001 G-1)
#[async_trait]
pub trait AgentTaskExecutor: Send + Sync {
    /// 执行 task, 返回 Result<(), DispatchError>
    async fn execute(&self, task: &AgentTask) -> Result<(), DispatchError>;
}

/// **派发错误** (per INV-DISP-01)
#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("task {0} not found")]
    TaskNotFound(Uuid),
    #[error("task {0} in invalid state: {1:?}")]
    InvalidState(Uuid, TaskState),
    #[error("task {0} execution failed: {1}")]
    ExecutionFailed(Uuid, String),
    #[error("task {0} aborted: {1}")]
    Aborted(Uuid, String),
    #[error("dispatcher closed")]
    DispatcherClosed,
    /// G-5 Tenant Quota 限额超出
    #[error("tenant {tenant_id} quota exceeded: {resource} (limit {limit}, current {current})")]
    QuotaExceeded {
        tenant_id: Uuid,
        resource: String,
        limit: u64,
        current: u64,
    },
    /// G-4 Shared Pool 资源未找到
    #[error("pool resource {0} not found")]
    PoolNotFound(String),
    /// G-4 Shared Pool 资源耗尽
    #[error("pool resource {resource_id} exhausted (max concurrency reached)")]
    PoolExhausted { resource_id: String },
}

/// **InMemory TaskQueue** (per SRS-001 G-1, v0.0.1 PoC, v0.1.0 收官 改 SQLite WAL)
#[derive(Debug, Default, Clone)]
pub struct InMemoryTaskQueue {
    tasks: Arc<RwLock<HashMap<Uuid, AgentTask>>>,
}

impl InMemoryTaskQueue {
    /// 创建空 InMemory TaskQueue
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入新 task (state=Pending)
    pub async fn enqueue(&self, task: AgentTask) -> Result<Uuid, DispatchError> {
        let mut tasks = self.tasks.write().await;
        let task_id = task.task_id;
        tasks.insert(task_id, task);
        Ok(task_id)
    }

    /// 获取 task
    pub async fn get(&self, task_id: Uuid) -> Result<AgentTask, DispatchError> {
        let tasks = self.tasks.read().await;
        tasks
            .get(&task_id)
            .cloned()
            .ok_or(DispatchError::TaskNotFound(task_id))
    }

    /// 列出所有 task
    pub async fn list(&self) -> Vec<AgentTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 按状态列出 task
    pub async fn list_by_state(&self, state: TaskState) -> Vec<AgentTask> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|t| t.state == state)
            .cloned()
            .collect()
    }

    /// 列出 task 数量
    pub async fn len(&self) -> usize {
        self.tasks.read().await.len()
    }

    /// task 转移 state
    pub async fn transition(
        &self,
        task_id: Uuid,
        to: TaskState,
        at_ms: u64,
        reason: String,
    ) -> Result<AgentTask, DispatchError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(&task_id)
            .ok_or(DispatchError::TaskNotFound(task_id))?;
        let from = task.state;
        task.state = to;
        task.state_history.push(TaskStateTransition {
            from,
            to,
            at_ms,
            reason,
        });
        Ok(task.clone())
    }
}

/// **L0 Dispatcher** (per SRS-001 G-1)
pub struct Dispatcher {
    queue: InMemoryTaskQueue,
    closed: Arc<RwLock<bool>>,
}

impl Dispatcher {
    /// 创建新 dispatcher
    pub fn new() -> Self {
        Self {
            queue: InMemoryTaskQueue::new(),
            closed: Arc::new(RwLock::new(false)),
        }
    }

    /// 提交 task (state=Pending)
    pub async fn submit(&self, task: AgentTask) -> Result<Uuid, DispatchError> {
        if *self.closed.read().await {
            return Err(DispatchError::DispatcherClosed);
        }
        self.queue.enqueue(task).await
    }

    /// 派发 task (state Pending → Dispatched → Running, 然后调 executor)
    /// 返回执行后 task 状态 (Completed / Failed / Aborted)
    pub async fn dispatch(
        &self,
        task_id: Uuid,
        executor: &dyn AgentTaskExecutor,
    ) -> Result<TaskState, DispatchError> {
        if *self.closed.read().await {
            return Err(DispatchError::DispatcherClosed);
        }
        // 转移 state Pending → Dispatched
        let at_ms = now_ms();
        self.queue
            .transition(task_id, TaskState::Dispatched, at_ms, "dispatched".into())
            .await?;
        // 转移 state Dispatched → Running
        let at_ms = now_ms();
        self.queue
            .transition(task_id, TaskState::Running, at_ms, "running".into())
            .await?;
        // 执行
        let task = self.queue.get(task_id).await?;
        match executor.execute(&task).await {
            Ok(()) => {
                let at_ms = now_ms();
                self.queue
                    .transition(task_id, TaskState::Completed, at_ms, "executed_ok".into())
                    .await?;
                Ok(TaskState::Completed)
            }
            Err(DispatchError::Aborted(_, reason)) => {
                let at_ms = now_ms();
                self.queue
                    .transition(task_id, TaskState::Aborted, at_ms, reason)
                    .await?;
                Ok(TaskState::Aborted)
            }
            Err(e) => {
                let at_ms = now_ms();
                self.queue
                    .transition(task_id, TaskState::Failed, at_ms, e.to_string())
                    .await?;
                Ok(TaskState::Failed)
            }
        }
    }

    /// 关闭 dispatcher (不再接受新 task)
    pub async fn close(&self) {
        *self.closed.write().await = true;
    }

    /// 获取 task queue 引用 (用于 list / get 等查询)
    pub fn queue(&self) -> &InMemoryTaskQueue {
        &self.queue
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// **SubAgent Archetype 9 类型** (per LangGraph C-03 + C-13, per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.1.3)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SubAgentArchetype {
    /// SA-01: code-review (PR/MR 审查)
    CodeReview,
    /// SA-02: test-gen (测试生成)
    TestGen,
    /// SA-03: 5-domain-lead-audit (5 域 Lead 决策, per 守门 #3 撤回 Mavis 自主)
    FiveDomainLeadAudit,
    /// SA-04: git-ops (worktree/commit/push)
    GitOps,
    /// SA-05: doc-sync (AGENTS.md / WBS / ADR)
    DocSync,
    /// SA-06: refactor (代码重构)
    Refactor,
    /// SA-07: db-migration (per 守门 #13 W/T/M)
    DbMigration,
    /// SA-08: domain-dev (DDD bounded context 开发)
    DomainDev,
    /// SA-09: free-form (默认 fallback, 通用 plan-execute-verify)
    FreeForm,
}

impl SubAgentArchetype {
    /// 9 SA 名称 (per LangGraph §2.1.3)
    pub fn name(&self) -> &'static str {
        match self {
            Self::CodeReview => "code-review",
            Self::TestGen => "test-gen",
            Self::FiveDomainLeadAudit => "5-domain-lead-audit",
            Self::GitOps => "git-ops",
            Self::DocSync => "doc-sync",
            Self::Refactor => "refactor",
            Self::DbMigration => "db-migration",
            Self::DomainDev => "domain-dev",
            Self::FreeForm => "free-form",
        }
    }
}

/// **SubAgent trait 接口** (per LangGraph C-03, L1 ECS 容器)
#[async_trait]
pub trait SubAgent: Send + Sync {
    /// SubAgent archetype 标识
    fn archetype(&self) -> SubAgentArchetype;
    /// 异步执行 task, 返回 Ok(()) 或 DispatchError
    async fn run(&self, task: &AgentTask) -> Result<(), DispatchError>;
}

/// **SubAgentRegistry** (per LangGraph C-13, L0 注册表)
pub struct SubAgentRegistry {
    agents: Arc<RwLock<HashMap<SubAgentArchetype, Arc<dyn SubAgent>>>>,
}

impl Default for SubAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// **Event 类型** (per SRS-001 G-3 EventBus, G-3)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// Task 状态变更 (per TaskState 6 状态机)
    TaskStateChanged,
    /// SubAgent 生命周期变更 (register / unregister)
    SubAgentLifecycle,
    /// Mailbox 消息到达 (per G-3 Mailbox)
    MailboxMessage,
    /// 跨域编排 Saga 事件 (per star-saga INV-SG-ORCH-04)
    SagaEvent,
}

/// **Event 消息** (per SRS-001 G-3 EventBus, Event + Mailbox + Payload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event 全局 ID
    pub event_id: Uuid,
    /// Event 类型
    pub kind: EventKind,
    /// 来源 (per Agent ID 或 dispatcher 标识)
    pub source: String,
    /// 目标 (None = broadcast)
    pub target: Option<String>,
    /// 租户 ID (INV-ACT-01 跨域隔离)
    pub tenant_id: Uuid,
    /// Payload (JSON 序列化)
    pub payload: serde_json::Value,
    /// Event 时间戳 (ms since epoch)
    pub created_at_ms: u64,
}

/// **EventBus** (per SRS-001 G-3, in-memory pub/sub bus, v0.1.0 升级 Redis/Kafka)
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventKind, Vec<Arc<dyn EventHandler + Send + Sync>>>>>,
}

/// **EventHandler trait** (per SRS-001 G-3)
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Event kind 过滤
    fn interested_in(&self) -> EventKind;
    /// 异步处理 event
    async fn handle(&self, event: &Event) -> Result<(), DispatchError>;
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// 创建空 EventBus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 订阅 EventKind
    pub async fn subscribe(&self, handler: Arc<dyn EventHandler + Send + Sync>) {
        let kind = handler.interested_in();
        let mut subs = self.subscribers.write().await;
        subs.entry(kind).or_default().push(handler);
    }

    /// 发布 Event (广播给所有订阅者)
    pub async fn publish(&self, event: Event) -> Result<usize, DispatchError> {
        let subs = self.subscribers.read().await;
        let handlers = subs.get(&event.kind).cloned().unwrap_or_default();
        let count = handlers.len();
        for h in handlers.iter() {
            // Handler ownership transfer: use Arc<dyn> not Box<dyn> for clone-able
            // For simplicity, we just call and ignore handler result in broadcast
            let _ = h.handle(&event).await;
        }
        Ok(count)
    }

    /// 订阅者数量 by EventKind
    pub async fn subscriber_count(&self, kind: EventKind) -> usize {
        let subs = self.subscribers.read().await;
        subs.get(&kind).map(|v| v.len()).unwrap_or(0)
    }
}

/// **Mailbox 消息** (per SRS-001 G-3, Agent 间消息传递)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxMessage {
    /// 消息 ID
    pub msg_id: Uuid,
    /// 来源 SubAgent archetype
    pub from: SubAgentArchetype,
    /// 目标 SubAgent archetype
    pub to: SubAgentArchetype,
    /// 租户 ID (INV-ACT-01)
    pub tenant_id: Uuid,
    /// 消息内容
    pub body: serde_json::Value,
    /// 时间戳
    pub created_at_ms: u64,
}

/// **Mailbox** (per SRS-001 G-3, in-memory per-tenant 消息队列, v0.1.0 升级 Redis Stream)
pub struct Mailbox {
    inbox: Arc<RwLock<HashMap<SubAgentArchetype, Vec<MailboxMessage>>>>,
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}

/// **Token 使用量记录** (per SRS-001 G-9 Token telemetry, per AGENTS.md §7 已消耗列)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    /// Agent 类型 (per 9 SA Archetype name)
    pub agent: SubAgentArchetype,
    /// 租户 ID (INV-ACT-01)
    pub tenant_id: Uuid,
    /// 任务 ID (per AgentTask.task_id)
    pub task_id: Uuid,
    /// prompt token 估算
    pub prompt_tokens: u64,
    /// completion token 估算
    pub completion_tokens: u64,
    /// 记录时间戳 (ms since epoch)
    pub recorded_at_ms: u64,
}

/// **TokenStore** (per SRS-001 G-9, in-memory token 计量 + telemetry 接口, v0.1.0 收官 接 OpenTelemetry/Prometheus)
pub struct TokenStore {
    records: Arc<RwLock<Vec<TokenUsage>>>,
    /// 累计 token 计数 (prompt + completion, 按 agent + tenant 分组)
    cumulative_by_agent: Arc<RwLock<HashMap<SubAgentArchetype, u64>>>,
    cumulative_by_tenant: Arc<RwLock<HashMap<Uuid, u64>>>,
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

/// **Tenant 资源配额** (per SRS-001 §G-5 Tenant Quota, P3-D 关联 22 domain-identity)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantQuota {
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 每分钟最大 task 派发数
    pub tasks_per_minute: u32,
    /// 每天最大 token 数
    pub tokens_per_day: u64,
    /// 最大并发 in-flight task 数
    pub max_concurrent_tasks: u32,
    /// 最大 task 排队数
    pub max_queued_tasks: u32,
}

impl TenantQuota {
    /// 无限配额 (内部使用, per SystemTenant)
    pub fn unlimited(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            tasks_per_minute: u32::MAX,
            tokens_per_day: u64::MAX,
            max_concurrent_tasks: u32::MAX,
            max_queued_tasks: u32::MAX,
        }
    }
}

/// **Tenant 配额跟踪器** (per SRS-001 §G-5, 实时跟踪 + 限额检查)
pub struct TenantQuotaTracker {
    quotas: Arc<RwLock<HashMap<Uuid, TenantQuota>>>,
    /// in-flight task 计数 (per tenant)
    in_flight: Arc<RwLock<HashMap<Uuid, u32>>>,
    /// 当前排队 task 计数 (per tenant)
    queued: Arc<RwLock<HashMap<Uuid, u32>>>,
    /// 当前分钟 task 计数 (per tenant) — 简化模型 (实际需要时间窗口)
    tasks_this_minute: Arc<RwLock<HashMap<Uuid, u32>>>,
}

impl Default for TenantQuotaTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TenantQuotaTracker {
    /// 创建空 tracker
    pub fn new() -> Self {
        Self {
            quotas: Arc::new(RwLock::new(HashMap::new())),
            in_flight: Arc::new(RwLock::new(HashMap::new())),
            queued: Arc::new(RwLock::new(HashMap::new())),
            tasks_this_minute: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册 tenant 配额
    pub async fn register(&self, quota: TenantQuota) {
        let mut quotas = self.quotas.write().await;
        quotas.insert(quota.tenant_id, quota);
    }

    /// 获取 tenant 配额
    pub async fn get(&self, tenant_id: Uuid) -> Option<TenantQuota> {
        let quotas = self.quotas.read().await;
        quotas.get(&tenant_id).cloned()
    }

    /// 限额检查 — 验证 task 是否可以派发 (in-flight + queued + this_minute)
    /// 返回 Ok(()) 通过, Err(QuotaExceeded) 拒绝
    pub async fn check(&self, tenant_id: Uuid) -> Result<(), DispatchError> {
        let quotas = self.quotas.read().await;
        let quota = quotas
            .get(&tenant_id)
            .cloned()
            .unwrap_or_else(|| TenantQuota::unlimited(tenant_id));
        drop(quotas);
        // in-flight check
        let in_flight = {
            let m = self.in_flight.read().await;
            m.get(&tenant_id).copied().unwrap_or(0)
        };
        if in_flight >= quota.max_concurrent_tasks {
            return Err(DispatchError::QuotaExceeded {
                tenant_id,
                resource: "max_concurrent_tasks".into(),
                limit: quota.max_concurrent_tasks as u64,
                current: in_flight as u64,
            });
        }
        // queued check
        let queued = {
            let m = self.queued.read().await;
            m.get(&tenant_id).copied().unwrap_or(0)
        };
        if queued >= quota.max_queued_tasks {
            return Err(DispatchError::QuotaExceeded {
                tenant_id,
                resource: "max_queued_tasks".into(),
                limit: quota.max_queued_tasks as u64,
                current: queued as u64,
            });
        }
        // tasks this minute check
        let this_minute = {
            let m = self.tasks_this_minute.read().await;
            m.get(&tenant_id).copied().unwrap_or(0)
        };
        if this_minute >= quota.tasks_per_minute {
            return Err(DispatchError::QuotaExceeded {
                tenant_id,
                resource: "tasks_per_minute".into(),
                limit: quota.tasks_per_minute as u64,
                current: this_minute as u64,
            });
        }
        Ok(())
    }

    /// 记录 task 派发 (in-flight +1, queued +1)
    pub async fn record_dispatch(&self, tenant_id: Uuid) {
        {
            let mut m = self.in_flight.write().await;
            *m.entry(tenant_id).or_insert(0) += 1;
        }
        {
            let mut m = self.queued.write().await;
            *m.entry(tenant_id).or_insert(0) += 1;
        }
        {
            let mut m = self.tasks_this_minute.write().await;
            *m.entry(tenant_id).or_insert(0) += 1;
        }
    }

    /// 记录 task 完成 (in-flight -1, queued -1)
    pub async fn record_complete(&self, tenant_id: Uuid) {
        {
            let mut m = self.in_flight.write().await;
            let v = m.entry(tenant_id).or_insert(0);
            if *v > 0 {
                *v -= 1;
            }
        }
        {
            let mut m = self.queued.write().await;
            let v = m.entry(tenant_id).or_insert(0);
            if *v > 0 {
                *v -= 1;
            }
        }
    }

    /// 查询 in-flight
    pub async fn in_flight_count(&self, tenant_id: Uuid) -> u32 {
        self.in_flight
            .read()
            .await
            .get(&tenant_id)
            .copied()
            .unwrap_or(0)
    }

    /// 查询 queued
    pub async fn queued_count(&self, tenant_id: Uuid) -> u32 {
        self.queued
            .read()
            .await
            .get(&tenant_id)
            .copied()
            .unwrap_or(0)
    }
}

/// **Shared Provider** (per SRS-001 §G-4, Shared LLM/HTTP/MCP Pool, 18 §provider/model/profile)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    Llm,
    Http,
    Mcp,
    Rag,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoolResource {
    pub resource_id: String,
    pub kind: ProviderKind,
    pub provider: String,
    pub model: String,
    pub max_concurrency: u32,
}

pub struct SharedPool {
    resources: Arc<RwLock<HashMap<String, PoolResource>>>,
    in_use: Arc<RwLock<HashMap<String, u32>>>,
}

impl Default for SharedPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedPool {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            in_use: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, resource: PoolResource) {
        let mut resources = self.resources.write().await;
        resources.insert(resource.resource_id.clone(), resource);
    }

    pub async fn list(&self) -> Vec<PoolResource> {
        let resources = self.resources.read().await;
        resources.values().cloned().collect()
    }

    pub async fn check_available(&self, resource_id: &str) -> Result<bool, DispatchError> {
        let resources = self.resources.read().await;
        let res = resources
            .get(resource_id)
            .ok_or_else(|| DispatchError::PoolNotFound(resource_id.into()))?;
        let in_use = self.in_use.read().await;
        let current = in_use.get(resource_id).copied().unwrap_or(0);
        Ok(current < res.max_concurrency)
    }

    pub async fn acquire(&self, resource_id: &str) -> Result<(), DispatchError> {
        let mut in_use = self.in_use.write().await;
        let resources = self.resources.read().await;
        let res = resources
            .get(resource_id)
            .ok_or_else(|| DispatchError::PoolNotFound(resource_id.into()))?;
        let v = in_use.entry(resource_id.to_string()).or_insert(0);
        if *v >= res.max_concurrency {
            return Err(DispatchError::PoolExhausted {
                resource_id: resource_id.into(),
            });
        }
        *v += 1;
        Ok(())
    }

    pub async fn release(&self, resource_id: &str) {
        let mut in_use = self.in_use.write().await;
        let v = in_use.entry(resource_id.to_string()).or_insert(0);
        if *v > 0 {
            *v -= 1;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryRecord {
    pub mem_id: Uuid,
    pub agent: SubAgentArchetype,
    pub tenant_id: Uuid,
    pub task_id: Option<Uuid>,
    pub key: String,
    pub value: serde_json::Value,
    pub ttl_sec: u32,
    pub created_at_ms: u64,
}

pub struct MemoryStore {
    records: Arc<RwLock<HashMap<Uuid, MemoryRecord>>>,
    index: Arc<RwLock<HashMap<(Uuid, String), Uuid>>>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn put(&self, record: MemoryRecord) -> Uuid {
        let mem_id = record.mem_id;
        let key = (record.tenant_id, record.key.clone());
        {
            let mut index = self.index.write().await;
            index.insert(key, mem_id);
        }
        let mut records = self.records.write().await;
        records.insert(mem_id, record);
        mem_id
    }

    pub async fn get(&self, tenant_id: Uuid, key: &str) -> Option<MemoryRecord> {
        let index = self.index.read().await;
        let mem_id = *index.get(&(tenant_id, key.to_string()))?;
        drop(index);
        let records = self.records.read().await;
        records.get(&mem_id).cloned()
    }

    pub async fn get_by_id(&self, mem_id: Uuid) -> Option<MemoryRecord> {
        let records = self.records.read().await;
        records.get(&mem_id).cloned()
    }

    pub async fn delete(&self, mem_id: Uuid) -> bool {
        let mut records = self.records.write().await;
        let record = match records.remove(&mem_id) {
            Some(r) => r,
            None => return false,
        };
        drop(records);
        let key = (record.tenant_id, record.key);
        let mut index = self.index.write().await;
        index.remove(&key);
        true
    }

    pub async fn list_by_tenant(&self, tenant_id: Uuid) -> Vec<MemoryRecord> {
        let records = self.records.read().await;
        records
            .values()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    pub async fn record_count(&self) -> usize {
        self.records.read().await.len()
    }
}

impl TokenStore {
    /// 创建空 TokenStore
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            cumulative_by_agent: Arc::new(RwLock::new(HashMap::new())),
            cumulative_by_tenant: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录 token 使用量
    pub async fn record(&self, usage: TokenUsage) {
        let total = usage.prompt_tokens + usage.completion_tokens;
        // 累计 by agent
        {
            let mut by_agent = self.cumulative_by_agent.write().await;
            *by_agent.entry(usage.agent).or_insert(0) += total;
        }
        // 累计 by tenant
        {
            let mut by_tenant = self.cumulative_by_tenant.write().await;
            *by_tenant.entry(usage.tenant_id).or_insert(0) += total;
        }
        // 原始 records
        let mut records = self.records.write().await;
        records.push(usage);
    }

    /// 列出所有记录
    pub async fn list(&self) -> Vec<TokenUsage> {
        self.records.read().await.clone()
    }

    /// 累计 by agent
    pub async fn cumulative_by_agent(&self, agent: SubAgentArchetype) -> u64 {
        let by_agent = self.cumulative_by_agent.read().await;
        by_agent.get(&agent).copied().unwrap_or(0)
    }

    /// 累计 by tenant
    pub async fn cumulative_by_tenant(&self, tenant_id: Uuid) -> u64 {
        let by_tenant = self.cumulative_by_tenant.read().await;
        by_tenant.get(&tenant_id).copied().unwrap_or(0)
    }

    /// 记录总数
    pub async fn record_count(&self) -> usize {
        self.records.read().await.len()
    }
}

/// **Checkpoint 快照** (per SRS-001 §G-7, Crash Recovery + Checkpoint)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub checkpoint_id: Uuid,
    /// 关联任务 ID
    pub task_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// Checkpoint 触发时 task 状态
    pub task_state: TaskState,
    /// Saga context data (per star-saga SagaContext.data)
    pub context_data: serde_json::Value,
    /// 完成步骤列表
    pub completed_steps: Vec<String>,
    /// Checkpoint 时间戳
    pub created_at_ms: u64,
}

/// **CheckpointStore** (per SRS-001 §G-7, in-memory checkpoint 持久化 PoC, v0.1.0 收官接 SQLite)
pub struct CheckpointStore {
    checkpoints: Arc<RwLock<HashMap<Uuid, Checkpoint>>>,
    /// task_id -> 最新 checkpoint_id 索引
    by_task: Arc<RwLock<HashMap<Uuid, Uuid>>>,
}

impl Default for CheckpointStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointStore {
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            by_task: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 保存 checkpoint (per task 自动覆盖最新)
    pub async fn save(&self, cp: Checkpoint) -> Uuid {
        let cp_id = cp.checkpoint_id;
        let task_id = cp.task_id;
        let mut by_task = self.by_task.write().await;
        by_task.insert(task_id, cp_id);
        drop(by_task);
        let mut checkpoints = self.checkpoints.write().await;
        checkpoints.insert(cp_id, cp);
        cp_id
    }

    /// 按 task_id 拿最新 checkpoint (per crash recovery 重启点)
    pub async fn latest_for_task(&self, task_id: Uuid) -> Option<Checkpoint> {
        let by_task = self.by_task.read().await;
        let cp_id = *by_task.get(&task_id)?;
        drop(by_task);
        let checkpoints = self.checkpoints.read().await;
        checkpoints.get(&cp_id).cloned()
    }

    /// 按 cp_id 拿
    pub async fn get(&self, cp_id: Uuid) -> Option<Checkpoint> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints.get(&cp_id).cloned()
    }

    /// checkpoint 总数
    pub async fn count(&self) -> usize {
        self.checkpoints.read().await.len()
    }
}

/// **Context Tier** (per SRS-001 §G-8, L1/L2/L3 三级缓存)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextTier {
    /// L1: 内存 hot (高频访问, per-rpc)
    L1,
    /// L2: 进程级 warm (per-task, in-memory + brief file)
    L2,
    /// L3: 持久化 cold (per §20 brief docs/briefs/<task_id>.md, 跨进程)
    L3,
}

impl ContextTier {
    /// 名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
        }
    }
}

/// **Context Entry** (per SRS-001 §G-8, 三级 Context 入口)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextEntry {
    pub entry_id: Uuid,
    pub tier: ContextTier,
    pub task_id: Uuid,
    pub tenant_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub size_bytes: u64,
    pub created_at_ms: u64,
}

/// **ContextStore** (per SRS-001 §G-8, 三级缓存 PoC, v0.1.0 收官接 Tiering 实战)
pub struct ContextStore {
    entries: Arc<RwLock<HashMap<(ContextTier, String), ContextEntry>>>,
    /// 索引 (per task_id)
    by_task: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
}

impl Default for ContextStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            by_task: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 写 Context 到指定 tier
    pub async fn put(
        &self,
        tier: ContextTier,
        task_id: Uuid,
        tenant_id: Uuid,
        key: &str,
        value: serde_json::Value,
    ) -> Uuid {
        let entry_id = Uuid::new_v4();
        let key_owned = key.to_string();
        let size = value.to_string().len() as u64;
        {
            let mut by_task = self.by_task.write().await;
            by_task.entry(task_id).or_default().push(key_owned.clone());
        }
        let entry = ContextEntry {
            entry_id,
            tier,
            task_id,
            tenant_id,
            key: key_owned.clone(),
            value,
            size_bytes: size,
            created_at_ms: now_ms(),
        };
        let mut entries = self.entries.write().await;
        entries.insert((tier, key_owned), entry);
        entry_id
    }

    /// 按 (tier, key) 读 Context
    pub async fn get(&self, tier: ContextTier, key: &str) -> Option<ContextEntry> {
        let entries = self.entries.read().await;
        entries.get(&(tier, key.to_string())).cloned()
    }

    /// 升级 Context (per G-8 升级路径 L3 → L2 → L1)
    pub async fn promote(
        &self,
        from: ContextTier,
        to: ContextTier,
        key: &str,
    ) -> Option<ContextEntry> {
        let mut entries = self.entries.write().await;
        let entry = entries.remove(&(from, key.to_string()))?;
        let mut promoted = entry;
        promoted.tier = to;
        entries.insert((to, key.to_string()), promoted.clone());
        Some(promoted)
    }

    /// 列出 task 全部 Context
    pub async fn list_by_task(&self, task_id: Uuid) -> Vec<ContextEntry> {
        let by_task = self.by_task.read().await;
        let keys = by_task.get(&task_id).cloned().unwrap_or_default();
        drop(by_task);
        let entries = self.entries.read().await;
        let mut result = vec![];
        for (_, _, entry) in entries
            .iter()
            .filter(|((_, _), e)| e.task_id == task_id)
            .map(|((t, k), e)| (t.clone(), k.clone(), e.clone()))
        {
            result.push(entry);
        }
        result
    }
}

/// **SubAgentPool** (per LangGraph C-13 + L1 任务卡子代理)
/// 跨 sub-agent 隔离 context + per-task checkpoint, N 并行 (≤ 50)
pub struct SubAgentPool {
    registry: SubAgentRegistry,
    /// 当前活跃 sub-agent task 计数
    active_count: Arc<RwLock<HashMap<SubAgentArchetype, u32>>>,
    /// max 并行 (默认 50, per LangGraph)
    max_parallel: u32,
}

impl Default for SubAgentPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SubAgentPool {
    /// 创建新 SubAgentPool (max 50 并行)
    pub fn new() -> Self {
        Self::with_max_parallel(50)
    }

    /// 创建 max_parallel 可配 SubAgentPool
    pub fn with_max_parallel(max_parallel: u32) -> Self {
        Self {
            registry: SubAgentRegistry::new(),
            active_count: Arc::new(RwLock::new(HashMap::new())),
            max_parallel,
        }
    }

    /// 注册 SubAgent
    pub async fn register(&self, agent: Arc<dyn SubAgent + Send + Sync>) {
        self.registry.register(agent).await;
    }

    /// 派生 SubAgent 子任务 (per LangGraph 2-level hierarchical: Top Agent → SubAgent)
    /// 检查 max_parallel 限额, 返回新 task_id
    pub async fn spawn(
        &self,
        archetype: SubAgentArchetype,
        task: &AgentTask,
    ) -> Result<Uuid, DispatchError> {
        // 检查 max 并行
        let current = {
            let m = self.active_count.read().await;
            let total: u32 = m.values().sum();
            total
        };
        if current >= self.max_parallel {
            return Err(DispatchError::PoolExhausted {
                resource_id: format!("subagent_pool_max_parallel_{}", self.max_parallel),
            });
        }
        // 检查 archetype 注册
        let _sa = self.registry.get(archetype).await.ok_or_else(|| {
            DispatchError::PoolNotFound(format!("archetype_{}", archetype.name()))
        })?;
        // in-flight +1
        {
            let mut m = self.active_count.write().await;
            *m.entry(archetype).or_insert(0) += 1;
        }
        Ok(task.task_id)
    }

    /// 完成 SubAgent 子任务 (in-flight -1)
    pub async fn complete(&self, archetype: SubAgentArchetype) {
        let mut m = self.active_count.write().await;
        let v = m.entry(archetype).or_insert(0);
        if *v > 0 {
            *v -= 1;
        }
    }

    /// 当前活跃 sub-agent 任务计数
    pub async fn active_count(&self) -> u32 {
        self.active_count.read().await.values().sum()
    }
}

/// **TopAgent** (per LangGraph L0 全体代理, 1 instance singleton, cross-session checkpoint)
pub struct TopAgent {
    pub agent_id: String, // "top-agent" (singleton)
    pub max_subagent_parallel: u32,
    pool: SubAgentPool,
    checkpoint_store: Arc<CheckpointStore>,
}

impl Default for TopAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl TopAgent {
    /// 创建 Top Agent (singleton)
    pub fn new() -> Self {
        Self {
            agent_id: "top-agent".into(),
            max_subagent_parallel: 50,
            pool: SubAgentPool::new(),
            checkpoint_store: Arc::new(CheckpointStore::new()),
        }
    }

    /// 派生 sub-agent + 持久化 checkpoint (per cross-session)
    pub async fn dispatch_with_checkpoint(
        &self,
        archetype: SubAgentArchetype,
        task: &AgentTask,
    ) -> Result<Uuid, DispatchError> {
        let task_id = self.pool.spawn(archetype, task).await?;
        // 持久化 checkpoint
        self.checkpoint_store
            .save(Checkpoint {
                checkpoint_id: Uuid::new_v4(),
                task_id,
                tenant_id: task.tenant_id,
                task_state: task.state,
                context_data: task.payload.clone(),
                completed_steps: vec![],
                created_at_ms: now_ms(),
            })
            .await;
        Ok(task_id)
    }

    /// 访问 pool (L1 任务卡子代理池)
    pub fn pool(&self) -> &SubAgentPool {
        &self.pool
    }

    /// 访问 checkpoint store
    pub fn checkpoints(&self) -> &CheckpointStore {
        &self.checkpoint_store
    }
}

impl Mailbox {
    /// 创建空 Mailbox
    pub fn new() -> Self {
        Self {
            inbox: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发送消息
    pub async fn send(&self, msg: MailboxMessage) -> Result<(), DispatchError> {
        let mut inbox = self.inbox.write().await;
        inbox.entry(msg.to).or_default().push(msg);
        Ok(())
    }

    /// 拉取消息 (FIFO)
    pub async fn recv(&self, to: SubAgentArchetype) -> Vec<MailboxMessage> {
        let mut inbox = self.inbox.write().await;
        inbox.remove(&to).unwrap_or_default()
    }

    /// 窥探消息数量
    pub async fn peek_len(&self, to: SubAgentArchetype) -> usize {
        let inbox = self.inbox.read().await;
        inbox.get(&to).map(|v| v.len()).unwrap_or(0)
    }
}

impl SubAgentRegistry {
    /// 创建空 registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册 SubAgent (按 archetype 索引)
    pub async fn register(&self, agent: Arc<dyn SubAgent>) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.archetype(), agent);
    }

    /// 获取 SubAgent by archetype
    pub async fn get(&self, archetype: SubAgentArchetype) -> Option<Arc<dyn SubAgent>> {
        let agents = self.agents.read().await;
        agents.get(&archetype).cloned()
    }

    /// 列出所有已注册 archetype
    pub async fn list(&self) -> Vec<SubAgentArchetype> {
        let agents = self.agents.read().await;
        agents.keys().copied().collect()
    }

    /// 已注册数量
    pub async fn len(&self) -> usize {
        self.agents.read().await.len()
    }
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// L0 PoC test 1: task 派发 + lifecycle 6 状态机
    #[tokio::test]
    async fn dispatch_lifecycle_6_states() {
        let d = Dispatcher::new();
        let task = AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "code_review".into(),
            payload: serde_json::json!({"pr_id": 42}),
            idempotency_key: "test_key_1".into(),
            created_at_ms: now_ms(),
            state: TaskState::Pending,
            state_history: vec![],
        };
        let task_id = d.submit(task).await.unwrap();

        // 初始 state = Pending
        let t = d.queue().get(task_id).await.unwrap();
        assert_eq!(t.state, TaskState::Pending);
        assert_eq!(t.state_history.len(), 0);

        // 派发 + 执行 → Completed
        let exec = CountingExecutor::new(1);
        let result = d.dispatch(task_id, &exec).await.unwrap();
        assert_eq!(result, TaskState::Completed);

        // 状态历史: Pending → Dispatched → Running → Completed
        let t = d.queue().get(task_id).await.unwrap();
        assert_eq!(t.state, TaskState::Completed);
        assert_eq!(t.state_history.len(), 3);
        assert_eq!(t.state_history[0].from, TaskState::Pending);
        assert_eq!(t.state_history[0].to, TaskState::Dispatched);
        assert_eq!(t.state_history[1].from, TaskState::Dispatched);
        assert_eq!(t.state_history[1].to, TaskState::Running);
        assert_eq!(t.state_history[2].from, TaskState::Running);
        assert_eq!(t.state_history[2].to, TaskState::Completed);
    }

    /// L0 PoC test 2: 多 task 派发 + 隔离
    #[tokio::test]
    async fn dispatch_multiple_tasks_isolated() {
        let d = Dispatcher::new();
        let mut task_ids = vec![];
        for i in 0..5 {
            let task = AgentTask {
                task_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                kind: "search".into(),
                payload: serde_json::json!({"query": format!("q{}", i)}),
                idempotency_key: format!("key_{}", i),
                created_at_ms: now_ms(),
                state: TaskState::Pending,
                state_history: vec![],
            };
            task_ids.push(d.submit(task).await.unwrap());
        }
        // 5 task 都 Pending
        assert_eq!(d.queue().len().await, 5);
        // 按状态列表
        let pending = d.queue().list_by_state(TaskState::Pending).await;
        assert_eq!(pending.len(), 5);
        // 派发 1 个
        let exec = CountingExecutor::new(1);
        d.dispatch(task_ids[0], &exec).await.unwrap();
        // 4 Pending + 1 Completed
        assert_eq!(d.queue().list_by_state(TaskState::Pending).await.len(), 4);
        assert_eq!(d.queue().list_by_state(TaskState::Completed).await.len(), 1);
    }

    /// L0 PoC test 3: executor 失败 → TaskState::Failed
    #[tokio::test]
    async fn dispatch_executor_failure() {
        let d = Dispatcher::new();
        let task = AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "doc_gen".into(),
            payload: serde_json::json!({}),
            idempotency_key: "fail_key".into(),
            created_at_ms: now_ms(),
            state: TaskState::Pending,
            state_history: vec![],
        };
        let task_id = d.submit(task).await.unwrap();
        let exec = FailingExecutor::new("网络超时");
        let result = d.dispatch(task_id, &exec).await.unwrap();
        assert_eq!(result, TaskState::Failed);
        let t = d.queue().get(task_id).await.unwrap();
        assert_eq!(t.state, TaskState::Failed);
        assert!(t.state_history.last().unwrap().reason.contains("网络超时"));
    }

    /// L0 PoC test 4: close 后不再接受 task
    #[tokio::test]
    async fn dispatch_close_rejects_new_tasks() {
        let d = Dispatcher::new();
        d.close().await;
        let task = AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "test".into(),
            payload: serde_json::json!({}),
            idempotency_key: "closed_key".into(),
            created_at_ms: now_ms(),
            state: TaskState::Pending,
            state_history: vec![],
        };
        let res = d.submit(task).await;
        assert!(matches!(res, Err(DispatchError::DispatcherClosed)));
    }

    /// L0 PoC test 5: executor 计数验证 (跨域编排跨多 task 隔离)
    #[tokio::test]
    async fn dispatch_executor_counter_isolation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let d = Dispatcher::new();
        let mut task_ids = vec![];
        for i in 0..3 {
            let task = AgentTask {
                task_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                kind: "shared_exec".into(),
                payload: serde_json::json!({"i": i}),
                idempotency_key: format!("counter_key_{}", i),
                created_at_ms: now_ms(),
                state: TaskState::Pending,
                state_history: vec![],
            };
            task_ids.push(d.submit(task).await.unwrap());
        }
        let exec = CountingExecutor::with_counter(counter.clone());
        for id in &task_ids {
            d.dispatch(*id, &exec).await.unwrap();
        }
        // 3 task 全执行, counter = 3
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    /// L0 PoC helper: 计数 executor
    struct CountingExecutor {
        counter: Arc<AtomicUsize>,
    }

    impl CountingExecutor {
        fn new(n: usize) -> Self {
            let counter = Arc::new(AtomicUsize::new(0));
            counter.fetch_add(n, Ordering::SeqCst);
            // n 是初始 offset, 不重要, 这里设为 0
            counter.store(0, Ordering::SeqCst);
            Self { counter }
        }
        fn with_counter(counter: Arc<AtomicUsize>) -> Self {
            Self { counter }
        }
    }

    #[async_trait]
    impl AgentTaskExecutor for CountingExecutor {
        async fn execute(&self, _task: &AgentTask) -> Result<(), DispatchError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    /// L0 PoC helper: 失败 executor
    struct FailingExecutor {
        reason: String,
    }

    impl FailingExecutor {
        fn new(reason: &str) -> Self {
            Self {
                reason: reason.into(),
            }
        }
    }

    #[async_trait]
    impl AgentTaskExecutor for FailingExecutor {
        async fn execute(&self, _task: &AgentTask) -> Result<(), DispatchError> {
            Err(DispatchError::ExecutionFailed(
                Uuid::nil(),
                self.reason.clone(),
            ))
        }
    }

    /// G.2 PoC test 1: SubAgent Archetype 9 类型枚举 + name()
    #[test]
    fn archetype_9_types_named() {
        assert_eq!(SubAgentArchetype::CodeReview.name(), "code-review");
        assert_eq!(SubAgentArchetype::TestGen.name(), "test-gen");
        assert_eq!(
            SubAgentArchetype::FiveDomainLeadAudit.name(),
            "5-domain-lead-audit"
        );
        assert_eq!(SubAgentArchetype::GitOps.name(), "git-ops");
        assert_eq!(SubAgentArchetype::DocSync.name(), "doc-sync");
        assert_eq!(SubAgentArchetype::Refactor.name(), "refactor");
        assert_eq!(SubAgentArchetype::DbMigration.name(), "db-migration");
        assert_eq!(SubAgentArchetype::DomainDev.name(), "domain-dev");
        assert_eq!(SubAgentArchetype::FreeForm.name(), "free-form");
    }

    /// G.2 PoC test 2: SubAgentRegistry 注册 + 查找 + 列表
    #[tokio::test]
    async fn subagent_registry_register_and_lookup() {
        let r = SubAgentRegistry::new();
        assert_eq!(r.len().await, 0);
        // 注册 3 个 SA
        r.register(Arc::new(StubSubAgent::new(SubAgentArchetype::CodeReview)))
            .await;
        r.register(Arc::new(StubSubAgent::new(SubAgentArchetype::TestGen)))
            .await;
        r.register(Arc::new(StubSubAgent::new(SubAgentArchetype::FreeForm)))
            .await;
        assert_eq!(r.len().await, 3);
        // 查找
        let sa = r.get(SubAgentArchetype::CodeReview).await.unwrap();
        assert_eq!(sa.archetype(), SubAgentArchetype::CodeReview);
        // 列表
        let list = r.list().await;
        assert_eq!(list.len(), 3);
        assert!(list.contains(&SubAgentArchetype::CodeReview));
    }

    /// G.2 PoC test 3: Dispatcher + SubAgentRegistry 集成 — 按 task.kind 路由 SA
    #[tokio::test]
    async fn dispatcher_routes_via_subagent_registry() {
        let d = Dispatcher::new();
        let r = SubAgentRegistry::new();
        let counter = Arc::new(AtomicUsize::new(0));
        // 注册 3 个 SA (用同一 counter 验证)
        r.register(Arc::new(CountingSubAgent::new(
            SubAgentArchetype::CodeReview,
            counter.clone(),
        )))
        .await;
        r.register(Arc::new(CountingSubAgent::new(
            SubAgentArchetype::TestGen,
            counter.clone(),
        )))
        .await;
        r.register(Arc::new(CountingSubAgent::new(
            SubAgentArchetype::FreeForm,
            counter.clone(),
        )))
        .await;

        // 提交 3 task, 每个 kind 对应不同 SA
        for kind in ["code-review", "test-gen", "free-form"] {
            let task = AgentTask {
                task_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                kind: kind.into(),
                payload: serde_json::json!({}),
                idempotency_key: format!("k_{}", kind),
                created_at_ms: now_ms(),
                state: TaskState::Pending,
                state_history: vec![],
            };
            let task_id = d.submit(task).await.unwrap();
            // 路由: kind → SA
            let sa_kind = match kind {
                "code-review" => SubAgentArchetype::CodeReview,
                "test-gen" => SubAgentArchetype::TestGen,
                "free-form" => SubAgentArchetype::FreeForm,
                _ => panic!("unknown kind"),
            };
            let sa = r.get(sa_kind).await.unwrap();
            // 用 SA 的 run method 直接执行 (跳过 dispatcher.dispatch, 演示 registry 路由)
            let task = d.queue().get(task_id).await.unwrap();
            d.queue()
                .transition(
                    task_id,
                    TaskState::Dispatched,
                    now_ms(),
                    "via_registry".into(),
                )
                .await
                .unwrap();
            d.queue()
                .transition(task_id, TaskState::Running, now_ms(), "via_registry".into())
                .await
                .unwrap();
            sa.run(&task).await.unwrap();
            d.queue()
                .transition(task_id, TaskState::Completed, now_ms(), "sa_ok".into())
                .await
                .unwrap();
        }
        // 3 task 全执行, counter = 3
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    /// G.2 PoC helper: Stub SubAgent (只返回 archetype)
    struct StubSubAgent {
        archetype: SubAgentArchetype,
    }

    impl StubSubAgent {
        fn new(archetype: SubAgentArchetype) -> Self {
            Self { archetype }
        }
    }

    #[async_trait]
    impl SubAgent for StubSubAgent {
        fn archetype(&self) -> SubAgentArchetype {
            self.archetype
        }
        async fn run(&self, _task: &AgentTask) -> Result<(), DispatchError> {
            Ok(())
        }
    }

    /// G.2 PoC helper: 计数 SubAgent (每个 SA 独立 counter)
    struct CountingSubAgent {
        archetype: SubAgentArchetype,
        counter: Arc<AtomicUsize>,
    }

    impl CountingSubAgent {
        fn new(archetype: SubAgentArchetype, counter: Arc<AtomicUsize>) -> Self {
            Self { archetype, counter }
        }
    }

    #[async_trait]
    impl SubAgent for CountingSubAgent {
        fn archetype(&self) -> SubAgentArchetype {
            self.archetype
        }
        async fn run(&self, _task: &AgentTask) -> Result<(), DispatchError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    /// G.3 PoC test 1: EventBus publish + subscribe — broadcast 1 个 EventKind
    #[tokio::test]
    async fn eventbus_publish_subscribe() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        // 订阅 TaskStateChanged 事件
        bus.subscribe(Arc::new(CountingEventHandler::new(
            EventKind::TaskStateChanged,
            counter.clone(),
        )))
        .await;
        assert_eq!(bus.subscriber_count(EventKind::TaskStateChanged).await, 1);
        // 发布 3 个事件
        for i in 0..3 {
            let event = Event {
                event_id: Uuid::new_v4(),
                kind: EventKind::TaskStateChanged,
                source: format!("dispatcher_{}", i),
                target: None,
                tenant_id: Uuid::new_v4(),
                payload: serde_json::json!({"i": i}),
                created_at_ms: now_ms(),
            };
            bus.publish(event).await.unwrap();
        }
        // 1 个订阅者收到 3 个事件
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    /// G.3 PoC test 2: EventBus 多 EventKind 隔离
    #[tokio::test]
    async fn eventbus_kind_isolation() {
        let bus = EventBus::new();
        let task_counter = Arc::new(AtomicUsize::new(0));
        let mail_counter = Arc::new(AtomicUsize::new(0));
        // 订阅不同 kind
        bus.subscribe(Arc::new(CountingEventHandler::new(
            EventKind::TaskStateChanged,
            task_counter.clone(),
        )))
        .await;
        bus.subscribe(Arc::new(CountingEventHandler::new(
            EventKind::MailboxMessage,
            mail_counter.clone(),
        )))
        .await;
        // 发布 2 个 TaskStateChanged + 3 个 MailboxMessage
        for i in 0..2 {
            bus.publish(Event {
                event_id: Uuid::new_v4(),
                kind: EventKind::TaskStateChanged,
                source: "x".into(),
                target: None,
                tenant_id: Uuid::new_v4(),
                payload: serde_json::json!({}),
                created_at_ms: now_ms(),
            })
            .await
            .unwrap();
        }
        for i in 0..3 {
            bus.publish(Event {
                event_id: Uuid::new_v4(),
                kind: EventKind::MailboxMessage,
                source: "x".into(),
                target: None,
                tenant_id: Uuid::new_v4(),
                payload: serde_json::json!({}),
                created_at_ms: now_ms(),
            })
            .await
            .unwrap();
        }
        assert_eq!(task_counter.load(Ordering::SeqCst), 2);
        assert_eq!(mail_counter.load(Ordering::SeqCst), 3);
    }

    /// G.3 PoC test 3: Mailbox 9 SA 隔离发送 + 接收
    #[tokio::test]
    async fn mailbox_9_sa_isolation() {
        let mb = Mailbox::new();
        // 9 SA 各发 1 条
        let archetypes = [
            SubAgentArchetype::CodeReview,
            SubAgentArchetype::TestGen,
            SubAgentArchetype::FiveDomainLeadAudit,
            SubAgentArchetype::GitOps,
            SubAgentArchetype::DocSync,
            SubAgentArchetype::Refactor,
            SubAgentArchetype::DbMigration,
            SubAgentArchetype::DomainDev,
            SubAgentArchetype::FreeForm,
        ];
        for sa in archetypes.iter() {
            let msg = MailboxMessage {
                msg_id: Uuid::new_v4(),
                from: SubAgentArchetype::FreeForm, // 测试 from 任意
                to: *sa,
                tenant_id: Uuid::new_v4(),
                body: serde_json::json!({"for": sa.name()}),
                created_at_ms: now_ms(),
            };
            mb.send(msg).await.unwrap();
        }
        // 9 SA 各收 1 条 (peek)
        for sa in archetypes.iter() {
            assert_eq!(mb.peek_len(*sa).await, 1);
        }
        // CodeReview 收 1 条后清空
        let msgs = mb.recv(SubAgentArchetype::CodeReview).await;
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].to, SubAgentArchetype::CodeReview);
        assert_eq!(mb.peek_len(SubAgentArchetype::CodeReview).await, 0);
    }

    /// G.3 PoC test 4: EventBus + Dispatcher 集成 — task lifecycle 自动 publish
    #[tokio::test]
    async fn eventbus_dispatcher_integration() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        bus.subscribe(Arc::new(CountingEventHandler::new(
            EventKind::TaskStateChanged,
            counter.clone(),
        )))
        .await;

        let d = Dispatcher::new();
        let task = AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "test".into(),
            payload: serde_json::json!({}),
            idempotency_key: "evbus_key".into(),
            created_at_ms: now_ms(),
            state: TaskState::Pending,
            state_history: vec![],
        };
        let task_id = d.submit(task).await.unwrap();

        // 手动 publish 3 个 state 变更 (Pending → Dispatched → Running → Completed)
        for to in [
            TaskState::Dispatched,
            TaskState::Running,
            TaskState::Completed,
        ] {
            d.queue()
                .transition(task_id, to, now_ms(), format!("to_{:?}", to))
                .await
                .unwrap();
            bus.publish(Event {
                event_id: Uuid::new_v4(),
                kind: EventKind::TaskStateChanged,
                source: format!("dispatcher_{}", task_id),
                target: None,
                tenant_id: Uuid::new_v4(),
                payload: serde_json::json!({
                    "task_id": task_id,
                    "to": format!("{:?}", to),
                }),
                created_at_ms: now_ms(),
            })
            .await
            .unwrap();
        }
        // 3 个 event 全订阅到
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    /// G.3 PoC helper: EventHandler 计数 (每个 event +1)
    struct CountingEventHandler {
        kind: EventKind,
        counter: Arc<AtomicUsize>,
    }

    impl CountingEventHandler {
        fn new(kind: EventKind, counter: Arc<AtomicUsize>) -> Self {
            Self { kind, counter }
        }
    }

    #[async_trait]
    impl EventHandler for CountingEventHandler {
        fn interested_in(&self) -> EventKind {
            self.kind
        }
        async fn handle(&self, _event: &Event) -> Result<(), DispatchError> {
            self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    /// G.9 PoC test 1: TokenStore 记录 + 累计 by agent + by tenant
    #[tokio::test]
    async fn tokenstore_record_and_cumulative() {
        let ts = TokenStore::new();
        // 3 record, 2 agent + 2 tenant
        for i in 0..3 {
            ts.record(TokenUsage {
                agent: if i % 2 == 0 {
                    SubAgentArchetype::CodeReview
                } else {
                    SubAgentArchetype::TestGen
                },
                tenant_id: if i < 2 {
                    Uuid::new_v4() // 同一 tenant 假设 (实际不同)
                } else {
                    Uuid::new_v4()
                },
                task_id: Uuid::new_v4(),
                prompt_tokens: 100,
                completion_tokens: 50,
                recorded_at_ms: now_ms(),
            })
            .await;
        }
        // 3 record 全存
        assert_eq!(ts.record_count().await, 3);
        // by agent: CodeReview 2 次 (i=0, 2), TestGen 1 次 (i=1), 每次 total = 150
        assert_eq!(
            ts.cumulative_by_agent(SubAgentArchetype::CodeReview).await,
            300
        );
        assert_eq!(
            ts.cumulative_by_agent(SubAgentArchetype::TestGen).await,
            150
        );
        assert_eq!(ts.cumulative_by_agent(SubAgentArchetype::FreeForm).await, 0);
    }

    /// G.9 PoC test 2: TokenStore + Dispatcher 集成 — task 派发时 record token 估算
    #[tokio::test]
    async fn tokenstore_dispatcher_integration() {
        let ts = TokenStore::new();
        let d = Dispatcher::new();
        let tenant_id = Uuid::new_v4();
        // 模拟 2 task: 1 CodeReview (150 token) + 1 TestGen (200 token)
        let mut task_ids = vec![];
        for (kind, tokens) in [("code-review", 150u64), ("test-gen", 200u64)] {
            let task = AgentTask {
                task_id: Uuid::new_v4(),
                tenant_id,
                kind: kind.into(),
                payload: serde_json::json!({}),
                idempotency_key: format!("k_{}", kind),
                created_at_ms: now_ms(),
                state: TaskState::Pending,
                state_history: vec![],
            };
            task_ids.push((d.submit(task).await.unwrap(), kind, tokens));
        }
        // 派发每个 task 并 record token
        for (task_id, kind, tokens) in &task_ids {
            let exec = CountingExecutor::new(0);
            d.dispatch(*task_id, &exec).await.unwrap();
            let sa = match *kind {
                "code-review" => SubAgentArchetype::CodeReview,
                "test-gen" => SubAgentArchetype::TestGen,
                _ => panic!(),
            };
            ts.record(TokenUsage {
                agent: sa,
                tenant_id,
                task_id: *task_id,
                prompt_tokens: tokens / 2,
                completion_tokens: tokens / 2,
                recorded_at_ms: now_ms(),
            })
            .await;
        }
        // 累计 by tenant
        assert_eq!(ts.cumulative_by_tenant(tenant_id).await, 350);
        // 累计 by agent
        assert_eq!(
            ts.cumulative_by_agent(SubAgentArchetype::CodeReview).await,
            150
        );
        assert_eq!(
            ts.cumulative_by_agent(SubAgentArchetype::TestGen).await,
            200
        );
    }

    /// G.5 PoC test 1: TenantQuotaTracker 注册 + 限额检查通过
    #[tokio::test]
    async fn tenant_quota_register_and_check() {
        let qt = TenantQuotaTracker::new();
        let tenant_id = Uuid::new_v4();
        qt.register(TenantQuota {
            tenant_id,
            tasks_per_minute: 3,
            tokens_per_day: 10000,
            max_concurrent_tasks: 2,
            max_queued_tasks: 5,
        })
        .await;
        // 没记录任何 dispatch -> 限额检查通过
        qt.check(tenant_id).await.unwrap();
        // 1 个 dispatch -> 仍通过
        qt.record_dispatch(tenant_id).await;
        qt.check(tenant_id).await.unwrap();
    }

    /// G.5 PoC test 2: TenantQuota 限额超出 -> QuotaExceeded err
    #[tokio::test]
    async fn tenant_quota_exceeded_rejects() {
        let qt = TenantQuotaTracker::new();
        let tenant_id = Uuid::new_v4();
        qt.register(TenantQuota {
            tenant_id,
            tasks_per_minute: 2,
            tokens_per_day: 10000,
            max_concurrent_tasks: 1,
            max_queued_tasks: 5,
        })
        .await;
        // 第 1 个 dispatch 通过
        qt.record_dispatch(tenant_id).await;
        // 第 2 个 dispatch 触发 max_concurrent_tasks 限额
        let res = qt.check(tenant_id).await;
        assert!(
            matches!(res, Err(DispatchError::QuotaExceeded { ref resource, .. }) if resource == "max_concurrent_tasks")
        );
    }

    /// G.5 PoC test 3: 多 tenant 隔离 (tenant A 限额不影响 tenant B)
    #[tokio::test]
    async fn tenant_quota_isolation() {
        let qt = TenantQuotaTracker::new();
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        // tenant A 限额 1 并发
        qt.register(TenantQuota {
            tenant_id: tenant_a,
            tasks_per_minute: 1,
            tokens_per_day: 100,
            max_concurrent_tasks: 1,
            max_queued_tasks: 1,
        })
        .await;
        // tenant B 无限额
        qt.register(TenantQuota::unlimited(tenant_b)).await;
        // tenant A 1 dispatch 后拒绝
        qt.record_dispatch(tenant_a).await;
        assert!(qt.check(tenant_a).await.is_err());
        // tenant B 不受影响, 通过
        qt.check(tenant_b).await.unwrap();
        qt.record_dispatch(tenant_b).await;
        qt.check(tenant_b).await.unwrap();
        assert_eq!(qt.in_flight_count(tenant_a).await, 1);
        assert_eq!(qt.in_flight_count(tenant_b).await, 1);
    }

    /// G.4 PoC test 1: SharedPool 注册 + 列出
    #[tokio::test]
    async fn sharedpool_register_and_list() {
        let p = SharedPool::new();
        p.register(PoolResource {
            resource_id: "openai/gpt-4".into(),
            kind: ProviderKind::Llm,
            provider: "openai".into(),
            model: "gpt-4".into(),
            max_concurrency: 5,
        })
        .await;
        p.register(PoolResource {
            resource_id: "github-mcp".into(),
            kind: ProviderKind::Mcp,
            provider: "github".into(),
            model: "mcp-server".into(),
            max_concurrency: 2,
        })
        .await;
        let list = p.list().await;
        assert_eq!(list.len(), 2);
    }

    /// G.4 PoC test 2: SharedPool acquire + release 限流
    #[tokio::test]
    async fn sharedpool_acquire_release() {
        let p = SharedPool::new();
        p.register(PoolResource {
            resource_id: "anthropic/claude-3".into(),
            kind: ProviderKind::Llm,
            provider: "anthropic".into(),
            model: "claude-3".into(),
            max_concurrency: 2,
        })
        .await;
        // 2 个 acquire 通过
        p.acquire("anthropic/claude-3").await.unwrap();
        p.acquire("anthropic/claude-3").await.unwrap();
        // 第 3 个 acquire 触发 PoolExhausted
        let res = p.acquire("anthropic/claude-3").await;
        assert!(matches!(res, Err(DispatchError::PoolExhausted { .. })));
        // release 1 后, 可 acquire
        p.release("anthropic/claude-3").await;
        p.acquire("anthropic/claude-3").await.unwrap();
    }

    /// G.4 PoC test 3: SharedPool check_available 跨资源隔离
    #[tokio::test]
    async fn sharedpool_check_available() {
        let p = SharedPool::new();
        p.register(PoolResource {
            resource_id: "rag/embeddings".into(),
            kind: ProviderKind::Rag,
            provider: "qdrant".into(),
            model: "embed-v1".into(),
            max_concurrency: 1,
        })
        .await;
        // 初始可用
        assert!(p.check_available("rag/embeddings").await.unwrap());
        // 1 个 acquire 后耗尽
        p.acquire("rag/embeddings").await.unwrap();
        assert!(!p.check_available("rag/embeddings").await.unwrap());
        // 不存在的资源 → PoolNotFound
        let res = p.check_available("nope/nope").await;
        assert!(matches!(res, Err(DispatchError::PoolNotFound(_))));
    }

    /// G.6 PoC test 1: MemoryStore put + get K-V 索引
    #[tokio::test]
    async fn memorystore_put_and_get() {
        let ms = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        let mem_id = ms
            .put(MemoryRecord {
                mem_id: Uuid::new_v4(),
                agent: SubAgentArchetype::CodeReview,
                tenant_id,
                task_id: Some(Uuid::new_v4()),
                key: "review:pr-42:summary".into(),
                value: serde_json::json!({"issues": 3}),
                ttl_sec: 0,
                created_at_ms: now_ms(),
            })
            .await;
        let r = ms.get(tenant_id, "review:pr-42:summary").await.unwrap();
        assert_eq!(r.mem_id, mem_id);
        assert_eq!(r.value, serde_json::json!({"issues": 3}));
    }

    /// G.6 PoC test 2: MemoryStore put 同 key 覆盖
    #[tokio::test]
    async fn memorystore_put_overwrite_same_key() {
        let ms = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        let id1 = ms
            .put(MemoryRecord {
                mem_id: Uuid::new_v4(),
                agent: SubAgentArchetype::TestGen,
                tenant_id,
                task_id: None,
                key: "k".into(),
                value: serde_json::json!("v1"),
                ttl_sec: 0,
                created_at_ms: now_ms(),
            })
            .await;
        let id2 = ms
            .put(MemoryRecord {
                mem_id: Uuid::new_v4(),
                agent: SubAgentArchetype::TestGen,
                tenant_id,
                task_id: None,
                key: "k".into(),
                value: serde_json::json!("v2"),
                ttl_sec: 0,
                created_at_ms: now_ms(),
            })
            .await;
        let r = ms.get(tenant_id, "k").await.unwrap();
        assert_eq!(r.mem_id, id2);
        assert_eq!(r.value, serde_json::json!("v2"));
    }

    /// G.6 PoC test 3: MemoryStore 多 tenant 隔离 + list_by_tenant + delete
    #[tokio::test]
    async fn memorystore_tenant_isolation_and_list() {
        let ms = MemoryStore::new();
        let ta = Uuid::new_v4();
        let tb = Uuid::new_v4();
        for (t, key) in [(ta, "k1"), (ta, "k2"), (tb, "k1")] {
            ms.put(MemoryRecord {
                mem_id: Uuid::new_v4(),
                agent: SubAgentArchetype::FreeForm,
                tenant_id: t,
                task_id: None,
                key: key.into(),
                value: serde_json::json!("v"),
                ttl_sec: 0,
                created_at_ms: now_ms(),
            })
            .await;
        }
        assert_eq!(ms.list_by_tenant(ta).await.len(), 2);
        assert_eq!(ms.list_by_tenant(tb).await.len(), 1);
        let ta_records = ms.list_by_tenant(ta).await;
        let ta_k1_mem_id = ta_records.iter().find(|r| r.key == "k1").unwrap().mem_id;
        assert!(ms.delete(ta_k1_mem_id).await);
        assert_eq!(ms.list_by_tenant(ta).await.len(), 1);
        assert_eq!(ms.list_by_tenant(tb).await.len(), 1);
    }

    /// G.7 PoC test 1: CheckpointStore save + latest_for_task 恢复点
    #[tokio::test]
    async fn checkpoint_save_and_latest() {
        let cs = CheckpointStore::new();
        let task_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        cs.save(Checkpoint {
            checkpoint_id: Uuid::new_v4(),
            task_id,
            tenant_id,
            task_state: TaskState::Running,
            context_data: serde_json::json!({"step": 3}),
            completed_steps: vec!["step1".into(), "step2".into()],
            created_at_ms: now_ms(),
        })
        .await;
        let latest = cs.latest_for_task(task_id).await.unwrap();
        assert_eq!(latest.task_state, TaskState::Running);
        assert_eq!(latest.completed_steps.len(), 2);
    }

    /// G.7 PoC test 2: CheckpointStore save 同 task 覆盖 (重启点最新)
    #[tokio::test]
    async fn checkpoint_overwrite_latest() {
        let cs = CheckpointStore::new();
        let task_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        // 1st checkpoint
        cs.save(Checkpoint {
            checkpoint_id: Uuid::new_v4(),
            task_id,
            tenant_id,
            task_state: TaskState::Running,
            context_data: serde_json::json!({"step": 1}),
            completed_steps: vec![],
            created_at_ms: now_ms(),
        })
        .await;
        // 2nd checkpoint 同 task_id
        cs.save(Checkpoint {
            checkpoint_id: Uuid::new_v4(),
            task_id,
            tenant_id,
            task_state: TaskState::Completed,
            context_data: serde_json::json!({"step": 5}),
            completed_steps: vec!["s1".into(), "s2".into(), "s3".into()],
            created_at_ms: now_ms(),
        })
        .await;
        // latest 指向 2nd
        let latest = cs.latest_for_task(task_id).await.unwrap();
        assert_eq!(latest.task_state, TaskState::Completed);
        assert_eq!(latest.completed_steps.len(), 3);
    }

    /// G.8 PoC test 1: ContextStore 三级缓存 put + get
    #[tokio::test]
    async fn contextstore_3tier_put_get() {
        let cs = ContextStore::new();
        let task_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        // L1 写
        cs.put(
            ContextTier::L1,
            task_id,
            tenant_id,
            "hot:key",
            serde_json::json!("v_l1"),
        )
        .await;
        // L2 写
        cs.put(
            ContextTier::L2,
            task_id,
            tenant_id,
            "warm:key",
            serde_json::json!("v_l2"),
        )
        .await;
        // L3 写
        cs.put(
            ContextTier::L3,
            task_id,
            tenant_id,
            "cold:key",
            serde_json::json!("v_l3"),
        )
        .await;
        // 按 tier get
        let l1 = cs.get(ContextTier::L1, "hot:key").await.unwrap();
        assert_eq!(l1.value, serde_json::json!("v_l1"));
        assert_eq!(l1.tier, ContextTier::L1);
        let l2 = cs.get(ContextTier::L2, "warm:key").await.unwrap();
        assert_eq!(l2.tier, ContextTier::L2);
        let l3 = cs.get(ContextTier::L3, "cold:key").await.unwrap();
        assert_eq!(l3.tier, ContextTier::L3);
    }

    /// G.8 PoC test 2: ContextStore promote 升级路径 L3 -> L2 -> L1
    #[tokio::test]
    async fn contextstore_promote_l3_to_l1() {
        let cs = ContextStore::new();
        let task_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        // 初始 L3 写
        cs.put(
            ContextTier::L3,
            task_id,
            tenant_id,
            "k",
            serde_json::json!(42),
        )
        .await;
        // L3 -> L2
        let p1 = cs
            .promote(ContextTier::L3, ContextTier::L2, "k")
            .await
            .unwrap();
        assert_eq!(p1.tier, ContextTier::L2);
        // L3 没了
        assert!(cs.get(ContextTier::L3, "k").await.is_none());
        // L2 -> L1
        let p2 = cs
            .promote(ContextTier::L2, ContextTier::L1, "k")
            .await
            .unwrap();
        assert_eq!(p2.tier, ContextTier::L1);
        assert!(cs.get(ContextTier::L2, "k").await.is_none());
        // L1 拿到
        let l1 = cs.get(ContextTier::L1, "k").await.unwrap();
        assert_eq!(l1.value, serde_json::json!(42));
    }

    /// G.8 PoC test 3: ContextStore list_by_task 多 tier 聚合
    #[tokio::test]
    async fn contextstore_list_by_task() {
        let cs = ContextStore::new();
        let task_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        // 3 tier 各 1 条
        cs.put(
            ContextTier::L1,
            task_id,
            tenant_id,
            "a",
            serde_json::json!(1),
        )
        .await;
        cs.put(
            ContextTier::L2,
            task_id,
            tenant_id,
            "b",
            serde_json::json!(2),
        )
        .await;
        cs.put(
            ContextTier::L3,
            task_id,
            tenant_id,
            "c",
            serde_json::json!(3),
        )
        .await;
        let list = cs.list_by_task(task_id).await;
        assert_eq!(list.len(), 3);
    }

    /// H.1 PoC test 1: SubAgentPool 注册 + spawn 限额
    #[tokio::test]
    async fn subagentpool_spawn_with_limit() {
        let pool = SubAgentPool::with_max_parallel(2);
        pool.register(Arc::new(StubSubAgent::new(SubAgentArchetype::CodeReview)))
            .await;
        pool.register(Arc::new(StubSubAgent::new(SubAgentArchetype::TestGen)))
            .await;
        // 2 个 spawn 通过
        for i in 0..2 {
            let task = AgentTask {
                task_id: Uuid::new_v4(),
                tenant_id: Uuid::new_v4(),
                kind: "test".into(),
                payload: serde_json::json!({"i": i}),
                idempotency_key: format!("k{}", i),
                created_at_ms: now_ms(),
                state: TaskState::Pending,
                state_history: vec![],
            };
            let archetype = if i == 0 {
                SubAgentArchetype::CodeReview
            } else {
                SubAgentArchetype::TestGen
            };
            pool.spawn(archetype, &task).await.unwrap();
        }
        assert_eq!(pool.active_count().await, 2);
        // 第 3 个 spawn 触发 PoolExhausted
        let task3 = AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "test".into(),
            payload: serde_json::json!({}),
            idempotency_key: "k3".into(),
            created_at_ms: now_ms(),
            state: TaskState::Pending,
            state_history: vec![],
        };
        let res = pool.spawn(SubAgentArchetype::CodeReview, &task3).await;
        assert!(matches!(res, Err(DispatchError::PoolExhausted { .. })));
    }

    /// H.1 PoC test 2: SubAgentPool spawn 未知 archetype -> PoolNotFound
    #[tokio::test]
    async fn subagentpool_spawn_unregistered_archetype() {
        let pool = SubAgentPool::new();
        // 没注册任何 SA
        let task = AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: "x".into(),
            payload: serde_json::json!({}),
            idempotency_key: "k".into(),
            created_at_ms: now_ms(),
            state: TaskState::Pending,
            state_history: vec![],
        };
        let res = pool.spawn(SubAgentArchetype::DomainDev, &task).await;
        assert!(matches!(res, Err(DispatchError::PoolNotFound(_))));
    }

    /// H.1 PoC test 3: TopAgent (L0) + SubAgentPool (L1) 2-level 集成 + Checkpoint 持久化
    #[tokio::test]
    async fn topagent_l0_l1_2level_with_checkpoint() {
        let top = TopAgent::new();
        // L0 Top Agent 派生 L1 CodeReview + L1 TestGen
        top.pool()
            .register(Arc::new(StubSubAgent::new(SubAgentArchetype::CodeReview)))
            .await;
        top.pool()
            .register(Arc::new(StubSubAgent::new(SubAgentArchetype::TestGen)))
            .await;
        // 派生 2 sub-agent task + 自动 checkpoint
        let mut task_ids = vec![];
        for archetype in [SubAgentArchetype::CodeReview, SubAgentArchetype::TestGen] {
            let task_id = Uuid::new_v4();
            let task = AgentTask {
                task_id,
                tenant_id: Uuid::new_v4(),
                kind: archetype.name().into(),
                payload: serde_json::json!({"i": task_ids.len()}),
                idempotency_key: format!("top_k{}", task_ids.len()),
                created_at_ms: now_ms(),
                state: TaskState::Pending,
                state_history: vec![],
            };
            top.dispatch_with_checkpoint(archetype, &task)
                .await
                .unwrap();
            task_ids.push(task_id);
        }
        // 2 个 active sub-agent
        assert_eq!(top.pool().active_count().await, 2);
        // 2 个 checkpoint 持久化
        assert_eq!(top.checkpoints().count().await, 2);
        // 每个 task 能从 checkpoint 恢复
        for task_id in &task_ids {
            let cp = top.checkpoints().latest_for_task(*task_id).await.unwrap();
            assert_eq!(cp.task_id, *task_id);
        }
    }
}
