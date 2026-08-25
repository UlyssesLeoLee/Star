//! Comment 域错误(`CommentError`)

use crate::value_object::CommentId;

/// **Comment 域错误**(5 变体)
#[derive(Debug, thiserror::Error)]
pub enum CommentError {
    #[error("comment not found: {0}")]
    NotFound(CommentId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl CommentError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "CMT_NOT_FOUND",
            Self::InvalidState(_) => "CMT_INVALID_STATE",
            Self::PermissionDenied => "CMT_PERMISSION_DENIED",
            Self::Conflict(_) => "CMT_CONFLICT",
            Self::Internal(_) => "CMT_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}
