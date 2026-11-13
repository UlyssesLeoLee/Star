/*
@cypher schema=1 source_sha256=edb30e1d03a5c8790409f76698a2f18f48c320f0838982c555ef8476dc7543d2
MERGE (self:File {path:"crates/sandboxd/src/error.rs"})
@endcypher
*/
//! `sandboxd` 错误模型 (per DD-SANDBOX-002 §2.1)
//!
//! 6-field 错误响应 (复用 star-mcp 模式): kind / message / field / details / status / trace_id。
//! 错误枚举跟 DD §2.1 对齐, 追加 2 个变体: [`SandboxdError::InvalidPolicy`] (策略非法 fail-fast,
//! per DD §3.1 时序点 1) + [`SandboxdError::CapacityExceeded`] (session 池容量拒绝, per FR-1.2)。

use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

/// sandboxd 领域错误 (per DD-SANDBOX-002 §2.1)
#[derive(Debug, Error)]
pub enum SandboxdError {
    /// session 不存在 (RunCommand / DestroySession / StreamLogs 时)
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),

    /// session 已存在 (幂等创建冲突)
    #[error("session already exists: {0}")]
    SessionAlreadyExists(Uuid),

    /// resource 维限制超限
    #[error("resource limit exceeded: {limit_type} = {value}")]
    ResourceLimitExceeded {
        /// 限制类型 (memory / cpu / processes)
        limit_type: String,
        /// 限制值
        value: u64,
    },

    /// network 维策略违规
    #[error("network policy violation: {detail}")]
    NetworkViolation {
        /// 违规详情 (域名 / IP)
        detail: String,
    },

    /// fs 维策略违规
    #[error("fs policy violation: {path}")]
    FsViolation {
        /// 违规路径
        path: String,
    },

    /// capability 维策略违规
    #[error("capability policy violation: {cap}")]
    CapabilityViolation {
        /// 违规 capability / privilege 名
        cap: String,
    },

    /// 平台不支持 (三平台之外)
    #[error("platform not supported: {0}")]
    PlatformNotSupported(String),

    /// audit log 写失败 (per FR-7.2 fail-closed 触发)
    #[error("audit log write failed: {0}")]
    AuditLogWriteFailed(String),

    /// 策略非法 (per DD §3.1 时序点 1 fail-fast, 追加变体)
    #[error("invalid policy: {0}")]
    InvalidPolicy(String),

    /// session 池容量满 (per FR-1.2 max_concurrent_sessions, 追加变体)
    #[error("session capacity exceeded: max = {max}")]
    CapacityExceeded {
        /// 池容量上限
        max: usize,
    },

    /// 内部错误 (兜底)
    #[error("internal error: {0}")]
    Internal(String),
}

impl SandboxdError {
    /// 错误 kind (6-field 模型字段 1, per DD §2.1 注释)
    pub fn kind(&self) -> &'static str {
        match self {
            SandboxdError::SessionNotFound(_) => "session_not_found",
            SandboxdError::SessionAlreadyExists(_) => "session_already_exists",
            SandboxdError::ResourceLimitExceeded { .. } => "resource_exceeded",
            SandboxdError::NetworkViolation { .. } => "network_violation",
            SandboxdError::FsViolation { .. } => "fs_violation",
            SandboxdError::CapabilityViolation { .. } => "capability_violation",
            SandboxdError::PlatformNotSupported(_) => "platform_not_supported",
            SandboxdError::AuditLogWriteFailed(_) => "audit_write_failed",
            SandboxdError::InvalidPolicy(_) => "invalid_policy",
            SandboxdError::CapacityExceeded { .. } => "capacity_exceeded",
            SandboxdError::Internal(_) => "internal",
        }
    }

    /// HTTP-like status (6-field 模型字段 5)
    pub fn status(&self) -> u16 {
        match self {
            SandboxdError::SessionNotFound(_) => 404,
            SandboxdError::SessionAlreadyExists(_) => 409,
            SandboxdError::ResourceLimitExceeded { .. } => 429,
            SandboxdError::NetworkViolation { .. }
            | SandboxdError::FsViolation { .. }
            | SandboxdError::CapabilityViolation { .. } => 403,
            SandboxdError::PlatformNotSupported(_) => 501,
            SandboxdError::AuditLogWriteFailed(_) => 503,
            SandboxdError::InvalidPolicy(_) => 422,
            SandboxdError::CapacityExceeded { .. } => 429,
            SandboxdError::Internal(_) => 500,
        }
    }

    /// 转 6-field JSON 响应 (per DD §2.1 `SandboxdErrorResponse`)
    pub fn to_response(&self, trace_id: Uuid) -> SandboxdErrorResponse {
        SandboxdErrorResponse {
            kind: self.kind().to_string(),
            message: self.to_string(),
            field: None,
            details: Value::Null,
            status: self.status(),
            trace_id,
        }
    }
}

/// 6-field 错误响应 (per DD-SANDBOX-002 §2.1, 跟 star-mcp 一致)
#[derive(Debug, serde::Serialize)]
pub struct SandboxdErrorResponse {
    /// 错误类别 (session_not_found / resource_exceeded / ...)
    pub kind: String,
    /// 人类可读信息
    pub message: String,
    /// 具体字段 (无则 None)
    pub field: Option<String>,
    /// 结构化详情
    pub details: Value,
    /// HTTP-like status
    pub status: u16,
    /// 跟 ActorContext.trace_id 对齐 (per 守门 #5 不携带敏感内容)
    pub trace_id: Uuid,
}

/// 转 tonic gRPC status (server 层映射, per DD §2.2)
impl From<SandboxdError> for tonic::Status {
    fn from(e: SandboxdError) -> Self {
        match &e {
            SandboxdError::SessionNotFound(id) => {
                tonic::Status::not_found(format!("session not found: {id}"))
            }
            SandboxdError::SessionAlreadyExists(id) => {
                tonic::Status::already_exists(format!("session already exists: {id}"))
            }
            SandboxdError::ResourceLimitExceeded { .. } => {
                tonic::Status::resource_exhausted(e.to_string())
            }
            SandboxdError::NetworkViolation { .. }
            | SandboxdError::FsViolation { .. }
            | SandboxdError::CapabilityViolation { .. } => {
                tonic::Status::permission_denied(e.to_string())
            }
            SandboxdError::PlatformNotSupported(_) => tonic::Status::unimplemented(e.to_string()),
            SandboxdError::AuditLogWriteFailed(_) => tonic::Status::unavailable(e.to_string()),
            SandboxdError::InvalidPolicy(_) => tonic::Status::invalid_argument(e.to_string()),
            SandboxdError::CapacityExceeded { .. } => {
                tonic::Status::resource_exhausted(e.to_string())
            }
            SandboxdError::Internal(_) => tonic::Status::internal(e.to_string()),
        }
    }
}
