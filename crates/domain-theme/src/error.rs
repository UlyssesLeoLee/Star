//! 主题系统错误类型

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ThemeError {
    #[error("theme not found: {0}")]
    NotFound(String),

    #[error("theme id 冲突: {id} 已存在")]
    DuplicateId { id: String },

    #[error("theme 字段不完整: 缺 {0}")]
    IncompleteDefinition(String),

    #[error("无效 hex 颜色: {0}")]
    InvalidHex(String),

    #[error("无效 px 间距: {0} (仅允许 0-512)")]
    InvalidSpacing(u32),

    #[error("权限拒绝: {actor} 不能修改 {scope:?} 作用域")]
    PermissionDenied { actor: String, scope: String },

    #[error("storage error: {0}")]
    Storage(String),

    #[error("serialization error: {0}")]
    Serialization(String),
}
