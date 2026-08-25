//! Identity 域错误(`IdentityError`)
//!
//! **5 标准变体**(与骨架阶段承诺锁定)

use crate::value_object::UserId;

/// **Identity 域错误**
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    /// 资源不存在
    #[error("user not found: {0}")]
    NotFound(UserId),

    /// 违反不变量(INV-IDN-01~02)
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,

    /// 唯一键冲突 / 乐观锁失败 / 重复操作
    #[error("conflict: {0}")]
    Conflict(String),

    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

impl IdentityError {
    /// 错误码字符串
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "IDENTITY_NOT_FOUND",
            Self::InvalidState(_) => "IDENTITY_INVALID_STATE",
            Self::PermissionDenied => "IDENTITY_PERMISSION_DENIED",
            Self::Conflict(_) => "IDENTITY_CONFLICT",
            Self::Internal(_) => "IDENTITY_INTERNAL",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for IdentityError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for IdentityError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}
