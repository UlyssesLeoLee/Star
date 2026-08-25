//! Board 域错误(`BoardError`)
//!
//! 来源: docs/api-design.md §8(错误码),docs/specs/domain-board-spec.md §8
//!
//! **5 标准变体**(与骨架阶段承诺锁定):
//! - `NotFound` / `InvalidState` / `PermissionDenied` / `Conflict` / `Internal`

use crate::value_object::BoardId;

/// **Board 域错误**(5 变体)
#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    /// 资源不存在
    #[error("board not found: {0}")]
    NotFound(BoardId),

    /// 违反不变量 / 状态非法(INV-B-01~05)
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

impl BoardError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "BOARD_NOT_FOUND",
            Self::InvalidState(_) => "BOARD_INVALID_STATE",
            Self::PermissionDenied => "BOARD_PERMISSION_DENIED",
            Self::Conflict(_) => "BOARD_CONFLICT",
            Self::Internal(_) => "BOARD_INTERNAL",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for BoardError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}
