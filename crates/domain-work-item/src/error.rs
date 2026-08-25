//! WorkItem 域错误(`WorkItemError`)
//!
//! 来源: docs/api-design.md §8(错误码),docs/specs/domain-work-item-spec.md §6
//!
//! **5 标准变体**(保持骨架阶段承诺的稳定契约):
//! - `NotFound` — 资源不存在
//! - `InvalidState` — 违反不变量 / 状态机非法迁移
//! - `PermissionDenied` — 跨租户访问 / 角色不足
//! - `Conflict` — 唯一键冲突 / 乐观锁失败
//! - `Internal` — 内部错误(IO / 序列化 / 事件总线等)
//!
//! Phase 3 由 `crates/api` 实现 `Into<ApiError>` 完成 HTTP 状态码映射。

use uuid::Uuid;

use crate::value_object::WorkItemId;

/// **WorkItem 域错误**(5 变体,与骨架阶段承诺锁定)
#[derive(Debug, thiserror::Error)]
pub enum WorkItemError {
    /// 资源不存在(WorkItem / Requirement / AC / BusinessGoal 等)。
    #[error("work item not found: {0}")]
    NotFound(WorkItemId),

    /// 违反不变量或状态机非法迁移(INV-WI-01~09 / REQ-WF-001)。
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// 权限不足(跨租户访问 / 角色不足 / 缺 Project 成员资格)。
    #[error("permission denied")]
    PermissionDenied,

    /// 唯一键冲突 / 乐观锁失败 / 重复操作。
    #[error("conflict: {0}")]
    Conflict(String),

    /// 内部错误(IO / 序列化 / 事件总线 / DB 连接等)。
    #[error("internal error: {0}")]
    Internal(String),
}

impl WorkItemError {
    /// 错误码字符串(供 `crates/api` 映射 HTTP 状态码)。
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "WORK_ITEM_NOT_FOUND",
            Self::InvalidState(_) => "WORK_ITEM_INVALID_STATE",
            Self::PermissionDenied => "WORK_ITEM_PERMISSION_DENIED",
            Self::Conflict(_) => "WORK_ITEM_CONFLICT",
            Self::Internal(_) => "WORK_ITEM_INTERNAL",
        }
    }

    /// 是否为 5xx 错误(用于 `api` 层映射 HTTP 500)。
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_))
    }
}

// =====================================================================
// From impl:常见外部错误 → WorkItemError
// =====================================================================

impl From<uuid::Error> for WorkItemError {
    fn from(e: uuid::Error) -> Self {
        Self::Internal(format!("uuid error: {e}"))
    }
}

impl From<tokio::sync::mpsc::error::SendError<()>> for WorkItemError {
    fn from(e: tokio::sync::mpsc::error::SendError<()>) -> Self {
        Self::Internal(format!("event channel send error: {e}"))
    }
}

// 注:`From<Uuid> for WorkItemId` 等强类型 ID 的转换由 `value_object::define_uuid_id!` 宏提供。
// 这里不再重复实现,避免 E0119 冲突。
const _: fn() = || {
    // 强制让编译器感知 Uuid 已使用(避免 unused 警告)
    let _ = Uuid::nil();
};
