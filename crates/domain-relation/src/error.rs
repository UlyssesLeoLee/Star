//! Relation 域错误(`RelationError`)

use crate::value_object::RelationId;

/// **Relation 域错误**(5 变体)
#[derive(Debug, thiserror::Error)]
pub enum RelationError {
    #[error("relation not found: {0}")]
    NotFound(RelationId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl RelationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "REL_NOT_FOUND",
            Self::InvalidState(_) => "REL_INVALID_STATE",
            Self::PermissionDenied => "REL_PERMISSION_DENIED",
            Self::Conflict(_) => "REL_CONFLICT",
            Self::Internal(_) => "REL_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}
