//! `CanvasPermission` 域模型 (per DD-AGENT §4.14 + A12.7, 7 字段, 0 业务方法).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (留阶段 2 任务 2.3).
//!
//! 3 级权限: view / comment / edit, BFF middleware 强制 (per 守门 #5 + 9/1 13:03+13:05 JST envoy 偏好).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 3 级权限枚举 (per A12.7, BFF middleware 强制).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// 只读 (查看画布 + 评论).
    View,
    /// 评论 (可发评论, 不可改元素).
    Comment,
    /// 编辑 (完整修改权限).
    Edit,
}

/// A12 画布权限 域模型 (per DD-AGENT §4.14 + A12.7, 7 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.3 落地业务方法 (grant_permission / revoke_permission).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasPermission {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 授权用户 ID.
    pub user_id: Uuid,
    /// 权限等级.
    pub level: PermissionLevel,
    /// 授权人 ID.
    pub granted_by: Uuid,
    /// 授权时间 (UTC).
    pub granted_at: DateTime<Utc>,
    /// 过期时间 (None = 永久).
    pub expires_at: Option<DateTime<Utc>>,
}

impl Default for CanvasPermission {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            user_id: Uuid::nil(),
            level: PermissionLevel::View,
            granted_by: Uuid::nil(),
            granted_at: Utc::now(),
            expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_permission_default() {
        let p = CanvasPermission::default();
        assert_eq!(p.level, PermissionLevel::View);
    }
}
