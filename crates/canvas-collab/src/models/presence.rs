//! `PresenceCursor` 域模型 (per DD-AGENT §4.14, 8 字段, 0 业务方法).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (留阶段 2 任务 2.3).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A12 多人编辑 实时光标 域模型 (per DD-AGENT §4.14, 8 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.3 落地业务方法 (update_cursor / heartbeat).
///
/// 注: 不用 `Eq` 因 `f32` 不实现 `Eq` (NaN != NaN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresenceCursor {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 用户 ID.
    pub user_id: Uuid,
    /// 用户名 (display name).
    pub user_name: String,
    /// 光标 X 坐标.
    pub x: f32,
    /// 光标 Y 坐标.
    pub y: f32,
    /// 光标颜色 (e.g. "#FF5733", 12-color 调色板 color-blind 友好).
    pub color: String,
    /// 最近心跳时间 (30s heartbeat TTL, per NFR-AGENT-MU-CONS-01).
    pub last_seen: DateTime<Utc>,
}

impl Default for PresenceCursor {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            user_id: Uuid::nil(),
            user_name: String::new(),
            x: 0.0,
            y: 0.0,
            color: "#000000".to_string(),
            last_seen: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_cursor_default() {
        let cur = PresenceCursor::default();
        assert_eq!(cur.id, Uuid::nil());
        assert_eq!(cur.color, "#000000");
    }
}
