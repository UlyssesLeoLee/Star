//! `CanvasMultiUserAudit` 域模型 (per DD-AGENT §4.14 + DD-001 C-26, 9 字段, 0 业务方法).
//!
//! Transaction append-only (per 守门 #13 b 物理删除禁止), SCD Type 2 (per 守门 #13 c), 100% RLS 13 类 (per 守门 #13 a).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (留阶段 2 任务 2.3).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A12 多人编辑 审计 域模型 (per DD-AGENT §4.14 + DD-001 C-26, 9 字段).
///
/// Transaction append-only (per 守门 #13 b 物理删除禁止), SCD Type 2 (per 守门 #13 c).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.3 落地业务方法 (record_audit / query_audit_trail).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasMultiUserAudit {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 操作者用户 ID.
    pub actor_user_id: Uuid,
    /// 操作动作 (e.g. "create_element" / "update_element" / "delete_element" / "post_comment" / "resolve_comment").
    pub action: String,
    /// 操作目标 ID.
    pub target_id: Uuid,
    /// 操作目标类型 (e.g. "element" / "comment" / "permission").
    pub target_type: String,
    /// 操作 payload (before/after snapshot).
    pub payload: serde_json::Value,
    /// SCD Type 2 版本号.
    pub version: i32,
    /// 创建时间 (UTC).
    pub created_at: DateTime<Utc>,
}

impl Default for CanvasMultiUserAudit {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            actor_user_id: Uuid::nil(),
            action: String::new(),
            target_id: Uuid::nil(),
            target_type: String::new(),
            payload: serde_json::Value::Null,
            version: 1,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_multi_user_audit_default() {
        let a = CanvasMultiUserAudit::default();
        assert_eq!(a.version, 1);
        assert_eq!(a.action, "");
    }
}
