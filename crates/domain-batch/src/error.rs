//! Batch 域错误 (per BATCH-REQ-001 §8 错误码 BA-001~016 + SEC-001/002/007)
//!
//! 16 错误码:
/*
| 错误码 | HTTP | 触发条件 |
|---|---|---|
| `SEC-001/002/007` | 401/403/403 | 鉴权类 |
| `BA-001` | 404 | Task 不存在 |
| `BA-002` | 404 | Run 不存在 |
| `BA-003` | 404 | Node 不存在 |
| `BA-004` | 404 | NodeType 不存在 |
| `BA-005` | 422 | DAG JSON schema 校验失败 |
| `BA-006` | 422 | DAG 拓扑有环 (per INV-BA-03) |
| `BA-007` | 422 | NodeType config_schema 校验失败 |
| `BA-008` | 422 | cron 表达式非法 |
| `BA-009` | 403 | 节点类型未审批 (per INV-BA-05) |
| `BA-010` | 409 | Task 已存在同名 (SCD Type 2 同名不同 version) |
| `BA-011` | 409 | Run 已在 running 状态 (per F-022 重入保护) |
| `BA-012` | 408 | 节点执行超时 (per F-023 + INV-BA-07) |
| `BA-013` | 500 | 节点执行失败 (per F-022 重试用尽) |
| `BA-014` | 500 | Worker lease 丢失 (per ADR-0030 + INV-BA-09) |
| `BA-015` | 503 | batch 引擎过载 (per NFR-002 50 worker / 500 节点/秒限流) |
| `BA-016` | 500 | DB 写入失败 (走 `batch_event` 记录失败) |
*/
//! 注: SEC-* 系列由 `star-context` 提供, BA-* 系列在本 crate 定义.

use thiserror::Error;

use crate::{NodeId, NodeTypeId, RunId, TaskId};

/// Batch 域错误 (BA-001~016, 完整 16 错误码, per BATCH-REQ-001 §8)
#[derive(Debug, Error)]
pub enum BatchError {
    /// `SEC-002` 403 跨 tenant/跨域拒绝 (per INV-BA-01 + INV-BA-10)
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// `SEC-007` 401 鉴权失败
    #[error("unauthenticated")]
    Unauthenticated,
    /// `BA-001` 404 Task 不存在
    #[error("task not found: {0}")]
    TaskNotFound(TaskId),
    /// `BA-002` 404 Run 不存在
    #[error("run not found: {0}")]
    RunNotFound(RunId),
    /// `BA-003` 404 Node 不存在
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
    /// `BA-004` 404 NodeType 不存在
    #[error("node type not found: {0}")]
    NodeTypeNotFound(NodeTypeId),
    /// `BA-005` 422 DAG JSON schema 校验失败
    #[error("invalid DAG schema: {0}")]
    InvalidDagSchema(String),
    /// `BA-006` 422 DAG 拓扑有环 (per INV-BA-03)
    #[error("DAG topology has cycle: {0}")]
    DagCycle(String),
    /// `BA-007` 422 NodeType config_schema 校验失败
    #[error("invalid node type config: {0}")]
    InvalidNodeTypeConfig(String),
    /// `BA-008` 422 cron 表达式非法
    #[error("invalid cron expression: {0}")]
    InvalidCron(String),
    /// `BA-009` 403 节点类型未审批 (per INV-BA-05)
    #[error("node type not approved: {0}")]
    NodeTypeNotApproved(NodeTypeId),
    /// `BA-010` 409 Task 已存在同名 (SCD Type 2 同名不同 version)
    #[error("task name conflict: {0}")]
    TaskNameConflict(String),
    /// `BA-011` 409 Run 已在 running 状态 (per F-022 重入保护)
    #[error("run already running: {0}")]
    RunAlreadyRunning(RunId),
    /// `BA-012` 408 节点执行超时 (per F-023 + INV-BA-07)
    #[error("node execution timeout: {0}")]
    NodeTimeout(NodeId),
    /// `BA-013` 500 节点执行失败 (per F-022 重试用尽)
    #[error("node execution failed: {0}")]
    NodeExecutionFailed(String),
    /// `BA-014` 500 Worker lease 丢失 (per ADR-0030 + INV-BA-09)
    #[error("worker lease lost: {0}")]
    WorkerLeaseLost(String),
    /// `BA-015` 503 batch 引擎过载 (per NFR-002 50 worker / 500 节点/秒限流)
    #[error("batch engine overloaded")]
    EngineOverloaded,
    /// `BA-016` 500 DB 写入失败 (走 `batch_event` 记录失败)
    #[error("database error: {0}")]
    Database(String),
    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

/// Batch 错误码字符串 (供 API/日志用)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchErrorCode {
    /// SEC-001/002/007
    PermissionDenied,
    /// BA-001
    TaskNotFound,
    /// BA-002
    RunNotFound,
    /// BA-003
    NodeNotFound,
    /// BA-004
    NodeTypeNotFound,
    /// BA-005
    InvalidDagSchema,
    /// BA-006
    DagCycle,
    /// BA-007
    InvalidNodeTypeConfig,
    /// BA-008
    InvalidCron,
    /// BA-009
    NodeTypeNotApproved,
    /// BA-010
    TaskNameConflict,
    /// BA-011
    RunAlreadyRunning,
    /// BA-012
    NodeTimeout,
    /// BA-013
    NodeExecutionFailed,
    /// BA-014
    WorkerLeaseLost,
    /// BA-015
    EngineOverloaded,
    /// BA-016
    Database,
    /// 内部
    Internal,
}

impl BatchError {
    /// 错误码字符串 (per `error.code()` API 约定, 对齐 domain-automation)
    pub fn code(&self) -> &'static str {
        match self {
            Self::PermissionDenied(_) => "BATCH_PERMISSION_DENIED",
            Self::Unauthenticated => "BATCH_UNAUTHENTICATED",
            Self::TaskNotFound(_) => "BATCH_TASK_NOT_FOUND",
            Self::RunNotFound(_) => "BATCH_RUN_NOT_FOUND",
            Self::NodeNotFound(_) => "BATCH_NODE_NOT_FOUND",
            Self::NodeTypeNotFound(_) => "BATCH_NODE_TYPE_NOT_FOUND",
            Self::InvalidDagSchema(_) => "BATCH_INVALID_DAG_SCHEMA",
            Self::DagCycle(_) => "BATCH_DAG_CYCLE",
            Self::InvalidNodeTypeConfig(_) => "BATCH_INVALID_NODE_TYPE_CONFIG",
            Self::InvalidCron(_) => "BATCH_INVALID_CRON",
            Self::NodeTypeNotApproved(_) => "BATCH_NODE_TYPE_NOT_APPROVED",
            Self::TaskNameConflict(_) => "BATCH_TASK_NAME_CONFLICT",
            Self::RunAlreadyRunning(_) => "BATCH_RUN_ALREADY_RUNNING",
            Self::NodeTimeout(_) => "BATCH_NODE_TIMEOUT",
            Self::NodeExecutionFailed(_) => "BATCH_NODE_EXECUTION_FAILED",
            Self::WorkerLeaseLost(_) => "BATCH_WORKER_LEASE_LOST",
            Self::EngineOverloaded => "BATCH_ENGINE_OVERLOADED",
            Self::Database(_) => "BATCH_DATABASE_ERROR",
            Self::Internal(_) => "BATCH_INTERNAL",
        }
    }

    /// 是否 5xx 服务端错误
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::NodeExecutionFailed(_)
                | Self::WorkerLeaseLost(_)
                | Self::EngineOverloaded
                | Self::Database(_)
                | Self::Internal(_)
        )
    }

    /// HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            Self::PermissionDenied(_) => 403,
            Self::Unauthenticated => 401,
            Self::TaskNotFound(_)
            | Self::RunNotFound(_)
            | Self::NodeNotFound(_)
            | Self::NodeTypeNotFound(_) => 404,
            Self::InvalidDagSchema(_)
            | Self::DagCycle(_)
            | Self::InvalidNodeTypeConfig(_)
            | Self::InvalidCron(_) => 422,
            Self::NodeTypeNotApproved(_) => 403,
            Self::TaskNameConflict(_) | Self::RunAlreadyRunning(_) => 409,
            Self::NodeTimeout(_) => 408,
            Self::NodeExecutionFailed(_)
            | Self::WorkerLeaseLost(_)
            | Self::Database(_)
            | Self::Internal(_) => 500,
            Self::EngineOverloaded => 503,
        }
    }
}

impl From<uuid::Error> for BatchError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<serde_json::Error> for BatchError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal(format!("json error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_for_task_not_found() {
        let err = BatchError::TaskNotFound(TaskId::new());
        assert_eq!(err.code(), "BATCH_TASK_NOT_FOUND");
        assert_eq!(err.http_status(), 404);
        assert!(!err.is_server_error());
    }

    #[test]
    fn code_for_dag_cycle() {
        let err = BatchError::DagCycle("node_a -> node_b -> node_a".to_string());
        assert_eq!(err.code(), "BATCH_DAG_CYCLE");
        assert_eq!(err.http_status(), 422);
    }

    #[test]
    fn code_for_engine_overloaded() {
        let err = BatchError::EngineOverloaded;
        assert_eq!(err.code(), "BATCH_ENGINE_OVERLOADED");
        assert_eq!(err.http_status(), 503);
        assert!(err.is_server_error());
    }
}
