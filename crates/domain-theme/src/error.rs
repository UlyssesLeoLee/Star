//! 主题系统错误类型

use thiserror::Error;

/// 主题系统错误类型
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ThemeError {
    /// 主题未找到
    #[error("theme not found: {0}")]
    NotFound(String),

    /// theme id 冲突
    #[error("theme id 冲突: {id} 已存在")]
    DuplicateId {
        /// 冲突的 theme id
        id: String,
    },

    /// theme 字段不完整
    #[error("theme 字段不完整: 缺 {0}")]
    IncompleteDefinition(String),

    /// 无效 hex 颜色值
    #[error("无效 hex 颜色: {0}")]
    InvalidHex(String),

    /// 无效 px 间距值(仅允许 0-512)
    #[error("无效 px 间距: {0} (仅允许 0-512)")]
    InvalidSpacing(u32),

    /// 权限拒绝
    #[error("权限拒绝: {actor} 不能修改 {scope:?} 作用域")]
    PermissionDenied {
        /// 执行操作的 actor 标识
        actor: String,
        /// 被拒绝修改的作用域
        scope: String,
    },

    /// 存储层错误
    #[error("storage error: {0}")]
    Storage(String),

    /// 序列化错误
    #[error("serialization error: {0}")]
    Serialization(String),
}
