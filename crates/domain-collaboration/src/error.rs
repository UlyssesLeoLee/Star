//! Collaboration 域错误(`CollaborationError`)
//!
//! 来源: docs/specs/domain-collaboration-spec.md §8(错误码) +
//! docs/api-design.md §8(错误码)。
//!
//! **7 标准变体**(在骨架阶段 5 变体基础上扩展 2 个协作场景特化变体):
//! - `NotFound` — 资源不存在(Session / Participant / Channel / Cursor)
//! - `InvalidState` — 违反不变量 / 状态机非法迁移(INV-CB-01~06)
//! - `PermissionDenied` — 跨租户访问 / 角色不足
//! - `Conflict` — 唯一键冲突 / Subscription 重复 / 乐观锁失败
//! - `RateLimited` — Subscription 超过 100/Connection(spec §10 CB-002,api-design §4.2)
//! - `Timeout` — 心跳 60s 未到 / WebSocket ping 60s 未回 pong(spec §8 CB-004)
//! - `Internal` — 内部错误(IO / 序列化 / 事件总线等)
//!
//! Phase 3 由 `crates/api` 实现 `Into<ApiError>` 完成 HTTP / WS 状态码映射。

use crate::value_object::SessionId;

/// **Collaboration 域错误**(7 变体)
#[derive(Debug, thiserror::Error)]
pub enum CollaborationError {
    /// 资源不存在
    #[error("collaboration resource not found: {0}")]
    NotFound(SessionId),

    /// 违反不变量或状态机非法迁移(INV-CB-01~06)
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// 权限不足(跨租户访问 / 角色不足)
    #[error("permission denied")]
    PermissionDenied,

    /// 唯一键冲突 / 乐观锁失败 / 重复操作
    #[error("conflict: {0}")]
    Conflict(String),

    /// Subscription 超过 100/Connection(spec §8 CB-002,api-design §4.2)
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// 心跳 60s 未到 / WebSocket ping 60s 未回 pong(spec §8 CB-004)
    #[error("timeout: {0}")]
    Timeout(String),

    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

impl CollaborationError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP / WS 状态码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "CB_NOT_FOUND",
            Self::InvalidState(_) => "CB_INVALID_STATE",
            Self::PermissionDenied => "CB_PERMISSION_DENIED",
            Self::Conflict(_) => "CB_CONFLICT",
            Self::RateLimited(_) => "CB-002",
            Self::Timeout(_) => "CB-004",
            Self::Internal(_) => "CB_INTERNAL",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_) | Self::Timeout(_))
    }
}

impl From<uuid::Error> for CollaborationError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}
