//! Permission 域错误

use crate::value_object::RoleId;

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("role not found: {0}")]
    NotFound(RoleId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl PermissionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "PERMISSION_NOT_FOUND",
            Self::InvalidState(_) => "PERMISSION_INVALID_STATE",
            Self::PermissionDenied => "PERMISSION_PERMISSION_DENIED",
            Self::Conflict(_) => "PERMISSION_CONFLICT",
            Self::Internal(_) => "PERMISSION_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for PermissionError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for PermissionError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}
