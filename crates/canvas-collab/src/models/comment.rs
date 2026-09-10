//! `CanvasComment` 域模型 (per DD-AGENT §4.14, 9 字段, 0 业务方法).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (留阶段 2 任务 2.3).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A12 画布评论 域模型 (per DD-AGENT §4.14, 9 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.3 落地业务方法 (post_comment / resolve_comment).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasComment {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 评论线程 ID (跟 comment_pin 关联).
    pub thread_id: Uuid,
    /// 回复评论的父评论 ID (None = 顶层评论).
    pub parent_id: Option<Uuid>,
    /// 评论作者 ID.
    pub author_id: Uuid,
    /// 评论内容.
    pub content: String,
    /// @ 提醒 用户 ID 列表 (per A12.5).
    pub mentioned_user_ids: Vec<Uuid>,
    /// 是否已解决.
    pub resolved: bool,
    /// 创建时间 (UTC).
    pub created_at: DateTime<Utc>,
    /// 更新时间 (UTC).
    pub updated_at: DateTime<Utc>,
}

impl Default for CanvasComment {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            thread_id: Uuid::nil(),
            parent_id: None,
            author_id: Uuid::nil(),
            content: String::new(),
            mentioned_user_ids: Vec::new(),
            resolved: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_comment_default() {
        let c = CanvasComment::default();
        assert_eq!(c.id, Uuid::nil());
        assert!(!c.resolved);
        assert!(c.mentioned_user_ids.is_empty());
    }
}
