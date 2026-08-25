//! Project 域错误

use crate::value_object::ProjectId;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("project not found: {0}")]
    NotFound(ProjectId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ProjectError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "PROJECT_NOT_FOUND",
            Self::InvalidState(_) => "PROJECT_INVALID_STATE",
            Self::PermissionDenied => "PROJECT_PERMISSION_DENIED",
            Self::Conflict(_) => "PROJECT_CONFLICT",
            Self::Internal(_) => "PROJECT_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for ProjectError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for ProjectError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}
