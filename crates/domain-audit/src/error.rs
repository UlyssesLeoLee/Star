//! Audit 域错误

use crate::value_object::AuditEventId;

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("audit event not found: {0}")]
    NotFound(AuditEventId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AuditError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "AUDIT_NOT_FOUND",
            Self::InvalidState(_) => "AUDIT_INVALID_STATE",
            Self::PermissionDenied => "AUDIT_PERMISSION_DENIED",
            Self::Conflict(_) => "AUDIT_CONFLICT",
            Self::Internal(_) => "AUDIT_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for AuditError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for AuditError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}
