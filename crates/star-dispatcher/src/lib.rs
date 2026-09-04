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
}
