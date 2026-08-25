//! Feedback 域错误(`FeedbackError`)
//!
//! 7 条错误码(FB-001~FB-007,spec §8)

use crate::value_object::FeedbackId;

/// **Feedback 域错误**(7 变体,FB-001~FB-007)
#[derive(Debug, thiserror::Error)]
pub enum FeedbackError {
    /// FB-001: 404 — Feedback 不存在
    #[error("feedback not found: {0}")]
    NotFound(FeedbackId),

    /// FB-002: 409 — 非法 6 状态迁移
    #[error("invalid state transition: {0}")]
    InvalidState(String),

    /// FB-003: 422 — Target 不可解析
    #[error("target unresolvable: {0}")]
    TargetUnresolvable(String),

    /// FB-004: 422 — APPLIED 之后尝试 update
    #[error("feedback is read-only after APPLIED")]
    ReadOnly,

    /// FB-005: 409 — 删除非 OPEN 状态 Feedback
    #[error("only OPEN feedback can be deleted (FB-005)")]
    NotDeletable,

    /// FB-006: 422 — Supersede 缺少 successor_id
    #[error("supersede requires successor_id (FB-006)")]
    MissingSuccessor,

    /// FB-007: 422 — Feedback Target 跨 Worktree
    #[error("cross-worktree feedback target forbidden (FB-007)")]
    CrossWorktree,

    /// 鉴权/授权拒绝
    #[error("permission denied")]
    PermissionDenied,

    /// 冲突(版本/唯一性)
    #[error("conflict: {0}")]
    Conflict(String),

    /// 内部错误
    #[error("internal: {0}")]
    Internal(String),
}

impl FeedbackError {
    /// 错误码(与 API design §8.3.3 FB- 系列对齐)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "FB_NOT_FOUND",
            Self::InvalidState(_) => "FB_INVALID_STATE_TRANSITION",
            Self::TargetUnresolvable(_) => "FB_TARGET_UNRESOLVABLE",
            Self::ReadOnly => "FB_READ_ONLY",
            Self::NotDeletable => "FB_NOT_DELETABLE",
            Self::MissingSuccessor => "FB_MISSING_SUCCESSOR",
            Self::CrossWorktree => "FB_CROSS_WORKTREE",
            Self::PermissionDenied => "FB_PERMISSION_DENIED",
            Self::Conflict(_) => "FB_CONFLICT",
            Self::Internal(_) => "FB_INTERNAL",
        }
    }

    /// 是否服务端错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}
