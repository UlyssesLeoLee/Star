//! Development 域错误(`DevelopmentError`)
//!
//! 来源: docs/api-design.md §8,docs/specs/domain-development-spec.md §8
//!
//! **错误码**:
//! - `D-001` Execution / ChangeSet 不存在
//! - `D-002` ChangeSet 已 commit,不可修改
//! - `D-003` Risk Signal kind 不在 8 种类型中
//! - `D-004` Object Storage Key 缺 tenant_id 前缀
//! - `D-005` AISelfClaim 未走 Validation Chain(VAL-001)
//! - `D-006` Symbol refresh 与 Repository 不属同 Tenant
//!
//! **7 个标准变体**:
//! - `NotFound` / `InvalidState` / `PermissionDenied` / `Conflict` / `Internal`
//! - `ValidationRequired`(D-005,VAL-001)
//! - `InvalidRiskSignalKind`(D-003)
//! - `InvalidObjectStorageKey`(D-004)

use crate::value_object::{ChangeSetId, ExecutionId};

/// **Development 域错误**
#[derive(Debug, thiserror::Error)]
pub enum DevelopmentError {
    /// 资源不存在
    #[error("execution not found: {0}")]
    ExecutionNotFound(ExecutionId),

    /// ChangeSet 不存在
    #[error("change_set not found: {0}")]
    ChangeSetNotFound(ChangeSetId),

    /// 违反不变量或状态机非法迁移
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// 权限不足(跨租户访问 / 角色不足)
    #[error("permission denied")]
    PermissionDenied,

    /// 唯一键冲突 / 乐观锁失败 / 重复操作
    #[error("conflict: {0}")]
    Conflict(String),

    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),

    /// D-005:AISelfClaim Risk Signal 未走 Validation Chain
    #[error("D-005: AISelfClaim RiskSignal 必须先通过 Validation Chain (VAL-001)")]
    ValidationRequired,

    /// D-003:Risk Signal kind 不在 8 种基本类型中
    #[error("D-003: Risk Signal kind '{0}' 不在 8 种基本类型中")]
    InvalidRiskSignalKind(String),

    /// D-004:Object Storage Key 缺 tenant_id 前缀
    #[error("D-004: Object Storage Key 必须以 tenant_id 为第一段: {0}")]
    InvalidObjectStorageKey(String),

    /// D-006:跨租户访问 Repository
    #[error("D-006: Repository {repository_id} 不属于 Tenant {tenant_id}")]
    CrossTenantRepositoryAccess {
        /// Repository ID
        repository_id: uuid::Uuid,
        /// 期望的 Tenant ID
        tenant_id: uuid::Uuid,
    },
}

impl DevelopmentError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::ExecutionNotFound(_) => "D-001",
            Self::ChangeSetNotFound(_) => "D-001",
            Self::InvalidState(_) => "DX_INVALID_STATE",
            Self::PermissionDenied => "SEC-007",
            Self::Conflict(_) => "DX_CONFLICT",
            Self::Internal(_) => "DX_INTERNAL",
            Self::ValidationRequired => "D-005",
            Self::InvalidRiskSignalKind(_) => "D-003",
            Self::InvalidObjectStorageKey(_) => "D-004",
            Self::CrossTenantRepositoryAccess { .. } => "D-006",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }

    /// 默认 HTTP 状态码
    pub fn http_status(&self) -> u16 {
        match self {
            Self::ExecutionNotFound(_) | Self::ChangeSetNotFound(_) => 404,
            Self::PermissionDenied => 403,
            Self::Conflict(_) => 409,
            Self::ValidationRequired => 409,
            Self::InvalidState(_) | Self::InvalidRiskSignalKind(_) => 422,
            Self::InvalidObjectStorageKey(_) => 422,
            Self::CrossTenantRepositoryAccess { .. } => 403,
            Self::Internal(_) => 500,
        }
    }
}

impl From<uuid::Error> for DevelopmentError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}
