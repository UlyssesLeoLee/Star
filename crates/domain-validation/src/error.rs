//! Validation 域错误(`ValidationError`)

use crate::value_object::ValidationId;

/// **Validation 域错误**(5 变体,对应 spec §8 VL-001 ~ VL-007 系列)
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("validation not found: {0}")]
    NotFound(ValidationId),
    #[error("invalid state: {0}")]
    InvalidState(String),
    #[error("permission denied")]
    PermissionDenied,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invariant violated: {0}")]
    InvariantViolated(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ValidationError {
    pub fn code(&self) -> &'static str {
        match self {
            // 沿用 spec §8 错误码
            Self::NotFound(_) => "VL_NOT_FOUND", // VL-001
            Self::InvalidState(_) => "VL_INVALID_STATE", // VL-002/003/005
            Self::PermissionDenied => "VL_PERMISSION_DENIED", // VL-006
            Self::Conflict(_) => "VL_CONFLICT",  // VL-007
            Self::InvariantViolated(_) => "VL_INVARIANT_VIOLATED", // VL-004
            Self::Internal(_) => "VL_INTERNAL",
        }
    }
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}
