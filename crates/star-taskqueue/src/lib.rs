// SPDX-License-Identifier: MIT OR Apache-2.0
//! `star-taskqueue` — L0 SQLite WAL Task Queue (Phase G.1, per SRS-001 §G-1)
//!
//! **目的**: 1M logical agents 的 L0 任务派发持久化层 (G.1)
//!
//! **架构 (per SRS-001 G-1)**:
//! - 6 状态机: Pending → Dispatched → Running → Completed / Failed / Aborted (INV-DISP-01)
//! - SQLite WAL mode (写并发 + 读不阻塞)
//! - tokio async via `spawn_blocking` (rusqlite 是 sync)
//!
//! **守门 #13 W 派生 (per 2026-09-01 18:30 JST 拍板)**:
//! - task_queue 是 W (短 TTL 作業中, completed 任务定期 expire / 物理删除)
//! - 物理删除 OK, タイマー失効 (e.g. completed > 24h 后清理)
//! - 不进 RLS 13 類 (W 不强制 RLS)
//! - 不进 SCD (M 才走 SCD Type 2)
//!
//! **Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 反转 + 9/5 10:43 JST 内推)**:
//! - TaskQueue schema 决策由 Mavis 临时代签, 真人到位后追溯签字
//! - author = `Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手`

#![allow(missing_docs)] // G.1 PoC 启动, Phase 2 spec 完成后补 doc

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

mod sqlite_backend;

/// 6 状态机 (per INV-DISP-01)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskState {
    /// 任务入队, 等待派发
    Pending,
    /// 已派发到执行者, 未开始
    Dispatched,
    /// 正在执行
    Running,
    /// 成功完成
    Completed,
    /// 执行失败 (e.g. timeout / exception)
    Failed,
    /// 主动中止 (e.g. 用户取消 / quota 超限)
    Aborted,
}

/// Agent 任务 (per SRS-001 G-1, 跟 star-dispatcher 兼容)
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
    /// 尝试次数 (e.g. retry)
    pub attempt: u32,
}

/// 任务状态变更记录
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

/// 任务队列错误
#[derive(Debug, Error)]
pub enum TaskQueueError {
    /// 任务未找到
    #[error("task {0} not found")]
    TaskNotFound(Uuid),
    /// 任务状态非法
    #[error("task {0} in invalid state: {1:?}")]
    InvalidState(Uuid, TaskState),
    /// SQLite 错误
    #[error("sqlite: {0}")]
    Sqlite(String),
    /// JSON 序列化/反序列化失败
    #[error("json: {0}")]
    Json(String),
    /// IO 错误
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// 队列已关闭
    #[error("queue closed")]
    QueueClosed,
}

/// 任务队列 trait (per SRS-001 G-1, 4 核心操作)
#[async_trait]
pub trait TaskQueue: Send + Sync {
    /// 入队新 task (state=Pending)
    async fn enqueue(&self, task: AgentTask) -> Result<Uuid, TaskQueueError>;
    /// 按 FIFO 顺序取出一个 Pending 任务, 标记为 Dispatched
    async fn dequeue(&self) -> Result<Option<AgentTask>, TaskQueueError>;
    /// 确认任务完成 (Completed)
    async fn ack(&self, task_id: Uuid) -> Result<AgentTask, TaskQueueError>;
    /// 任务失败 (Failed, 可 retry)
    async fn fail(&self, task_id: Uuid, reason: String) -> Result<AgentTask, TaskQueueError>;
    /// 按 ID 查询任务
    async fn get(&self, task_id: Uuid) -> Result<AgentTask, TaskQueueError>;
    /// 列出指定状态的任务
    async fn list_by_state(
        &self,
        state: TaskState,
        limit: u32,
    ) -> Result<Vec<AgentTask>, TaskQueueError>;
    /// 队列长度
    async fn len(&self) -> Result<u64, TaskQueueError>;
    /// 删除 completed/aborted 任务 (物理删除, per守门 #13 a W 派生)
    async fn cleanup_completed(&self, older_than_ms: u64) -> Result<u64, TaskQueueError>;
}

/// SQLite WAL 任务队列 (per SRS-001 G-1)
pub struct SqliteTaskQueue {
    inner: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteTaskQueue {
    /// 创建新 SQLite TaskQueue (WAL mode, sync)
    pub fn new(path: &str) -> Result<Self, TaskQueueError> {
        let conn =
            rusqlite::Connection::open(path).map_err(|e| TaskQueueError::Sqlite(e.to_string()))?;
        // 启用 WAL 模式 (写并发 + 读不阻塞)
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| TaskQueueError::Sqlite(e.to_string()))?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| TaskQueueError::Sqlite(e.to_string()))?;
        sqlite_backend::init_schema(&conn).map_err(|e| TaskQueueError::Sqlite(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    /// 内存 SQLite (per unit test 优先)
    pub fn in_memory() -> Result<Self, TaskQueueError> {
        Self::new(":memory:")
    }
}

#[async_trait]
impl TaskQueue for SqliteTaskQueue {
    async fn enqueue(&self, task: AgentTask) -> Result<Uuid, TaskQueueError> {
        let conn = self.inner.clone();
        let task_id = task.task_id;
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::insert_task(&conn, &task)
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))?;
        Ok(task_id)
    }

    async fn dequeue(&self) -> Result<Option<AgentTask>, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::dequeue_pending(&conn)
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }

    async fn ack(&self, task_id: Uuid) -> Result<AgentTask, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::transition_state(
                &conn,
                task_id,
                TaskState::Dispatched,
                TaskState::Completed,
                "ack",
            )
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }

    async fn fail(&self, task_id: Uuid, reason: String) -> Result<AgentTask, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::transition_state(
                &conn,
                task_id,
                TaskState::Dispatched,
                TaskState::Failed,
                &reason,
            )
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }

    async fn get(&self, task_id: Uuid) -> Result<AgentTask, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::get_task(&conn, task_id)
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }

    async fn list_by_state(
        &self,
        state: TaskState,
        limit: u32,
    ) -> Result<Vec<AgentTask>, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::list_by_state(&conn, state, limit)
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }

    async fn len(&self) -> Result<u64, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::count_all(&conn)
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }

    async fn cleanup_completed(&self, older_than_ms: u64) -> Result<u64, TaskQueueError> {
        let conn = self.inner.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.blocking_lock();
            sqlite_backend::cleanup_completed(&conn, older_than_ms)
        })
        .await
        .map_err(|e| TaskQueueError::Sqlite(format!("join: {e}")))?
        .map_err(|e: rusqlite::Error| TaskQueueError::Sqlite(e.to_string()))
    }
}

#[allow(unused_imports)]
use sqlite_backend as _;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(kind: &str) -> AgentTask {
        AgentTask {
            task_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            kind: kind.into(),
            payload: serde_json::json!({"k": "v"}),
            idempotency_key: Uuid::new_v4().to_string(),
            created_at_ms: 1_700_000_000_000,
            state: TaskState::Pending,
            state_history: vec![],
            attempt: 0,
        }
    }

    #[tokio::test]
    async fn enqueue_dequeue_ack() {
        let q = SqliteTaskQueue::in_memory().unwrap();
        let task = make_task("code_review");
        let id = q.enqueue(task.clone()).await.unwrap();
        assert_eq!(id, task.task_id);

        // FIFO
        let t = q.dequeue().await.unwrap().expect("should have 1 task");
        assert_eq!(t.task_id, task.task_id);
        assert_eq!(t.state, TaskState::Dispatched);

        // ack -> Completed
        let acked = q.ack(task.task_id).await.unwrap();
        assert_eq!(acked.state, TaskState::Completed);
    }

    #[tokio::test]
    async fn enqueue_fail_retry() {
        let q = SqliteTaskQueue::in_memory().unwrap();
        let task = make_task("search");
        q.enqueue(task.clone()).await.unwrap();
        let t = q.dequeue().await.unwrap().unwrap();
        assert_eq!(t.state, TaskState::Dispatched);
        let failed = q.fail(task.task_id, "network error".into()).await.unwrap();
        assert_eq!(failed.state, TaskState::Failed);
    }

    #[tokio::test]
    async fn list_by_state_count() {
        let q = SqliteTaskQueue::in_memory().unwrap();
        for _ in 0..5 {
            q.enqueue(make_task("k")).await.unwrap();
        }
        assert_eq!(q.len().await.unwrap(), 5);
        assert_eq!(
            q.list_by_state(TaskState::Pending, 100)
                .await
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            q.list_by_state(TaskState::Completed, 100)
                .await
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn cleanup_removes_completed() {
        let q = SqliteTaskQueue::in_memory().unwrap();
        let task = make_task("k");
        q.enqueue(task.clone()).await.unwrap();
        q.dequeue().await.unwrap();
        q.ack(task.task_id).await.unwrap();
        assert_eq!(q.len().await.unwrap(), 1);
        // older_than_ms = task.created_at_ms + 1 -> 删除
        let removed = q.cleanup_completed(task.created_at_ms + 1).await.unwrap();
        assert_eq!(removed, 1);
        assert_eq!(q.len().await.unwrap(), 0);
    }
}
