//! crates/star-dispatcher/src/cross_repo.rs
//!
//! H.2 LangGraph 跨仓 RPC (Star -> Physis) PoC
//! per WBS §H.2 + 守门 #19 [M] 拍板 + 守门 §5 disclaimer (Star 仓 不引用 RGS 仓 代码)
//!
//! 实现策略 (per 守门 #1 轻量化 + V2 路线图):
//! - 当前: in-process PoC (channel + serde_json + tokio::sync::mpsc)
//! - V2: 真实 gRPC (tonic + prost code-gen, 3 域 Lead 真人到位后切换)

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CrossRepoError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("timeout after {0}ms")]
    Timeout(u64),
    #[error("physis rejected: {0}")]
    PhysisRejected(String),
    #[error("internal: {0}")]
    Internal(String),
}

// === 跨仓 RPC message types (PoC, V2 替换为 gRPC generated) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchTaskRequest {
    pub task_id: String,
    pub tenant_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchTaskResponse {
    pub task_id: String,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStateRequest {
    pub task_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PhysisTaskState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStateResponse {
    pub task_id: String,
    pub state: PhysisTaskState,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskRequest {
    pub task_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelTaskResponse {
    pub task_id: String,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckRequest {
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub healthy: bool,
    pub version: String,
    pub latency_ms: u64,
}

// === In-process Physis Server ===

/// Physis server (PoC, in-process)
pub struct PhysisServer {
    tasks: Arc<Mutex<HashMap<String, (String, (PhysisTaskState, Option<serde_json::Value>, Option<String>))>>>,
    tx: mpsc::UnboundedSender<DispatchTaskRequest>,
}

impl PhysisServer {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<DispatchTaskRequest>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            tx,
        }, rx)
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<DispatchTaskRequest> {
        self.tx.clone()
    }

    pub async fn handle_dispatch(&self, req: DispatchTaskRequest) -> DispatchTaskResponse {
        let mut tasks = self.tasks.lock().await;
        // idempotency check: 同一 idempotency_key 重复派发返回原 task_id
        for (idem_key, (existing_task_id, _)) in tasks.iter() {
            if idem_key == &req.idempotency_key {
                return DispatchTaskResponse {
                    task_id: existing_task_id.clone(),
                    accepted: true,
                    reason: "deduplicated by idempotency_key".into(),
                };
            }
        }
        let task_id = format!("physis-{}", Uuid::new_v4());
        // 简化: 立即 Running -> Completed
        tasks.insert(req.idempotency_key.clone(), (task_id.clone(), (PhysisTaskState::Completed, Some(serde_json::json!({"physis": true})), None)));
        DispatchTaskResponse {
            task_id,
            accepted: true,
            reason: "ok".into(),
        }
    }

    pub async fn handle_query(&self, req: QueryStateRequest) -> QueryStateResponse {
        let tasks = self.tasks.lock().await;
        // 简化: 用 task_id 在 idem_key 中查找
        let task_id_str = req.task_id.as_str();
        for (_idem_key, (task_id, (state, result, error))) in tasks.iter() {
            if task_id == &req.task_id || task_id_str.contains(task_id) {
                return QueryStateResponse {
                    task_id: req.task_id.clone(),
                    state: *state,
                    result: result.clone(),
                    error: error.clone(),
                };
            }
        }
        QueryStateResponse {
            task_id: req.task_id,
            state: PhysisTaskState::Failed,
            result: None,
            error: Some("task not found".into()),
        }
    }

    pub async fn handle_cancel(&self, req: CancelTaskRequest) -> CancelTaskResponse {
        let mut tasks = self.tasks.lock().await;
        for (_idem_key, (task_id, (state, _, _))) in tasks.iter_mut() {
            if task_id == &req.task_id {
                *state = PhysisTaskState::Cancelled;
                return CancelTaskResponse { task_id: req.task_id, cancelled: true };
            }
        }
        CancelTaskResponse { task_id: req.task_id, cancelled: false }
    }

    pub async fn handle_health(&self, _req: HealthCheckRequest) -> HealthCheckResponse {
        HealthCheckResponse {
            healthy: true,
            version: "physis-0.0.1-poC".into(),
            latency_ms: 1,
        }
    }
}

// === In-process Cross-Repo Client (Star side) ===

/// Star 仓的 cross-repo client
pub struct CrossRepoClient {
    server: Arc<PhysisServer>,
}

impl CrossRepoClient {
    pub fn new(server: Arc<PhysisServer>) -> Self {
        Self { server }
    }

    pub async fn dispatch_task(&self, req: DispatchTaskRequest) -> Result<DispatchTaskResponse, CrossRepoError> {
        Ok(self.server.handle_dispatch(req).await)
    }

    pub async fn query_state(&self, req: QueryStateRequest) -> Result<QueryStateResponse, CrossRepoError> {
        // 简化: 不真调 server.handle_query, 总是返回 Completed stub
        Ok(QueryStateResponse {
            task_id: req.task_id,
            state: PhysisTaskState::Completed,
            result: Some(serde_json::json!({"stub": true})),
            error: None,
        })
    }

    pub async fn cancel_task(&self, req: CancelTaskRequest) -> Result<CancelTaskResponse, CrossRepoError> {
        Ok(self.server.handle_cancel(req).await)
    }

    pub async fn health_check(&self) -> Result<HealthCheckResponse, CrossRepoError> {
        Ok(self.server.handle_health(HealthCheckRequest { source: "star-dispatcher".into() }).await)
    }
}
