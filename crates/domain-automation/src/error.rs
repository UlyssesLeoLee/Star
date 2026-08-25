//! Automation 域错误(`AutomationError`)
//!
//! 来源: docs/api-design.md §8(错误码),docs/specs/domain-automation-spec.md §8
//!
//! **9 标准变体**:
//! - `NotFound` — 资源不存在(AU-001)
//! - `InvalidEventType` — Trigger event_type 未知(AU-002)
//! - `ResourceNotFound` — Action 引用资源不存在(AU-003)
//! - `CyclicRule` — 循环规则(AU-004)
//! - `ProtectedAction` — Rule 尝试 Protected 动作(AU-005,如 pr:merge)
//! - `RuleDisabled` — 规则已禁用
//! - `PermissionDenied` — 跨租户 / 角色不足
//! - `Conflict` — 乐观锁失败 / 名称 UNIQUE 冲突
//! - `RateLimited` — 触发频率超限(INV-AUTO-08)
//! - `Internal` — 内部错误
//!
//! Phase 3 由 `crates/api` 实现 `Into<ApiError>` 完成 HTTP 状态码映射。

use crate::value_object::RuleId;

/// **Automation 域错误**
#[derive(Debug, thiserror::Error)]
pub enum AutomationError {
    /// 资源不存在(AU-001 / 404)
    #[error("automation rule not found: {0}")]
    NotFound(RuleId),

    /// Trigger event_type 不在已知列表(AU-002 / 422)
    #[error("invalid trigger event_type: {0}")]
    InvalidEventType(String),

    /// Action 引用资源不存在(AU-003 / 422)
    #[error("action references non-existent resource: {0}")]
    ResourceNotFound(String),

    /// 循环规则(AU-004 / 409)
    #[error("cyclic rule detected: {0}")]
    CyclicRule(String),

    /// Protected 动作禁止 Rule(AU-005 / 403)
    #[error("protected action forbidden for rules: {0}")]
    ProtectedAction(String),

    /// 规则已禁用
    #[error("rule is disabled: {0}")]
    RuleDisabled(RuleId),

    /// 权限不足
    #[error("permission denied")]
    PermissionDenied,

    /// 唯一键冲突 / 乐观锁失败 / 重复操作
    #[error("conflict: {0}")]
    Conflict(String),

    /// 触发频率超限(INV-AUTO-08)
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// 内部错误
    #[error("internal error: {0}")]
    Internal(String),
}

impl AutomationError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码)
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "AU-001",
            Self::InvalidEventType(_) => "AU-002",
            Self::ResourceNotFound(_) => "AU-003",
            Self::CyclicRule(_) => "AU-004",
            Self::ProtectedAction(_) => "AU-005",
            Self::RuleDisabled(_) => "AU_RULE_DISABLED",
            Self::PermissionDenied => "AU_PERMISSION_DENIED",
            Self::Conflict(_) => "AU_CONFLICT",
            Self::RateLimited(_) => "AU_RATE_LIMITED",
            Self::Internal(_) => "AU_INTERNAL",
        }
    }

    /// 是否为 5xx 错误
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

impl From<uuid::Error> for AutomationError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<serde_json::Error> for AutomationError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal(format!("serde_json error: {e}"))
    }
}
