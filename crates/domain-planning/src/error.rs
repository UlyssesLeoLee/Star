//! Planning 域错误(`PlanningError`)

use crate::value_object::SprintId;

/// **Planning 域错误**(5 变体)
#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    /// 资源不存在
    #[error("sprint not found: {0}")]
    NotFound(SprintId),
    /// 违反不变量
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,
    /// 唯一键冲突 / 乐观锁失败 / 重复
    #[error("conflict: {0}")]
    Conflict(String),
    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

impl PlanningError {
    /// 错误码字符串
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "PL_NOT_FOUND",
            Self::InvalidState(_) => "PL_INVALID_STATE",
            Self::PermissionDenied => "PL_PERMISSION_DENIED",
            Self::Conflict(_) => "PL_CONFLICT",
            Self::Internal(_) => "PL_INTERNAL",
        }
    }
    /// 是否为 5xx
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}
