//! Tenant 域错误(`TenantError`)
//!
//! 来源: docs/api-design.md §8(错误码),docs/specs/domain-tenant-spec.md §6
//!
//! **5 标准变体**(与骨架阶段承诺锁定):
//! - `NotFound` — 资源不存在
//! - `InvalidState` — 违反不变量 / 状态机非法迁移
//! - `PermissionDenied` — 跨租户访问 / 角色不足
//! - `Conflict` — 唯一键冲突 / 乐观锁失败
//! - `Internal` — 内部错误(IO / 序列化 / 事件总线等)
//!
//! Phase 3 由 `crates/api` 实现 `Into<ApiError>` 完成 HTTP 状态码映射。

use crate::value_object::TenantId;

/// **Tenant 域错误**(5 变体)
#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    /// 资源不存在
    #[error("tenant not found: {0}")]
    NotFound(TenantId),

    /// 违反不变量或状态机非法迁移(INV-TEN-01~02)
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
}

impl TenantError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "TENANT_NOT_FOUND",
            Self::InvalidState(_) => "TENANT_INVALID_STATE",
            Self::PermissionDenied => "TENANT_PERMISSION_DENIED",
            Self::Conflict(_) => "TENANT_CONFLICT",
            Self::Internal(_) => "TENANT_INTERNAL",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for TenantError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for TenantError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}
