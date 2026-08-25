//! Workspace 域错误

use crate::value_object::WorkspaceId;

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("workspace not found: {0}")]
    NotFound(WorkspaceId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl WorkspaceError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "WORKSPACE_NOT_FOUND",
            Self::InvalidState(_) => "WORKSPACE_INVALID_STATE",
            Self::PermissionDenied => "WORKSPACE_PERMISSION_DENIED",
            Self::Conflict(_) => "WORKSPACE_CONFLICT",
            Self::Internal(_) => "WORKSPACE_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for WorkspaceError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for WorkspaceError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}
