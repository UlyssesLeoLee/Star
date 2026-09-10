//! `CanvasElementBackend` 域模型 (per DD-AGENT §4.14 + DD-001 C-25, 12 字段).
//!
//! 仅 struct 字段定义 + Default impl + derive, 0 业务方法 (留阶段 2 任务 2.3).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A12 画布元素 后端模型 (per DD-AGENT §4.14 + DD-001 C-25, 12 字段).
///
/// 阶段 1 基础 仅字段定义, 阶段 2 任务 2.3 落地业务方法 (create_element / update_element / delete_element).
///
/// 注: 不用 `Eq` 因 `f32` 不实现 `Eq` (NaN != NaN).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasElementBackend {
    /// 全局唯一 ID (v4 UUID).
    pub id: Uuid,
    /// 所属画布 ID.
    pub canvas_id: Uuid,
    /// 元素类型 (e.g. "sticky" / "frame" / "connector" / "shape" / "text" / "image").
    pub kind: String,
    /// 画布 X 坐标.
    pub x: f32,
    /// 画布 Y 坐标.
    pub y: f32,
    /// 元素宽度.
    pub width: f32,
    /// 元素高度.
    pub height: f32,
    /// 元素旋转角度 (弧度).
    pub rotation: f32,
    /// Z-order 索引.
    pub z_index: i32,
    /// 任意 JSON content (类型特定数据).
    pub content: serde_json::Value,
    /// 锁状态.
    pub locked: bool,
    /// 隐藏状态.
    pub hidden: bool,
    /// 乐观锁版本号.
    pub version: i32,
}

impl Default for CanvasElementBackend {
    fn default() -> Self {
        // 0 业务逻辑, 仅字段预填
        Self {
            id: Uuid::nil(),
            canvas_id: Uuid::nil(),
            kind: String::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            z_index: 0,
            content: serde_json::Value::Null,
            locked: false,
            hidden: false,
            version: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_element_backend_default() {
        let elem = CanvasElementBackend::default();
        assert_eq!(elem.id, Uuid::nil());
        assert_eq!(elem.canvas_id, Uuid::nil());
        assert_eq!(elem.z_index, 0);
        assert_eq!(elem.version, 1);
    }

    #[test]
    fn test_canvas_element_backend_clone() {
        let elem1 = CanvasElementBackend::default();
        let elem2 = elem1.clone();
        assert_eq!(elem1, elem2);
    }
}
