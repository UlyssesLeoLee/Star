//! Star Board — WIP 限制 + 泳道 + Saved View (wt-w9-wip 扩展)
//!
//! - WIP 限制: 列满时拒绝 transition + 告警
//! - 泳道: 按 assignee / epic / label / priority 分组
//! - Saved View: 用户保存的视图 (复用 ui-3pane-arch.md §1.3 Cmd+1/2/3/4 视图族)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. WIP 限制
// =====================================================================

/// WIP (在制品) 限制
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WipLimit {
    /// 列 ID
    pub column_id: String,
    /// 最大允许数量
    pub max_items: u32,
    /// 当前数量
    pub current_count: u32,
}

/// WIP 限制检查结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WipAction {
    /// 允许
    Allow,
    /// 告警但允许
    Warn,
    /// 拒绝
    Block,
}

/// WIP 限制检查器
pub struct WipGuard;

impl WipGuard {
    /// 检查是否允许 transition 到指定列
    pub fn check(limit: &WipLimit) -> WipAction {
        if limit.current_count < limit.max_items {
            WipAction::Allow
        } else if limit.current_count == limit.max_items {
            WipAction::Warn
        } else {
            WipAction::Block
        }
    }

    /// 拖入新工作项 (WIP 计数 +1)
    pub fn add(limit: &mut WipLimit) -> WipAction {
        limit.current_count += 1;
        Self::check(limit)
    }

    /// 拖出工作项 (WIP 计数 -1)
    pub fn remove(limit: &mut WipLimit) {
        if limit.current_count > 0 {
            limit.current_count -= 1;
        }
    }
}

// =====================================================================
// 2. 泳道
// =====================================================================

/// 泳道分组维度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwimlaneGroupBy {
    /// 按经办人分组
    Assignee,
    /// 按史诗分组
    Epic,
    /// 按标签分组
    Label,
    /// 按优先级分组
    Priority,
    /// 按自定义字段分组
    Custom,
}

/// 泳道
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swimlane {
    /// 分组维度
    pub group_by: SwimlaneGroupBy,
    /// 自定义字段名 (仅 Custom 分组时使用)
    pub custom_field: Option<String>,
    /// 是否折叠
    pub collapsed: bool,
}

impl Swimlane {
    /// 构造泳道
    pub fn new(group_by: SwimlaneGroupBy) -> Self {
        Self {
            group_by,
            custom_field: None,
            collapsed: false,
        }
    }
}

// =====================================================================
// 3. Saved View
// =====================================================================

/// 用户保存的视图
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedView {
    /// 视图 ID
    pub id: Uuid,
    /// 视图名称
    pub name: String,
    /// 所有者用户 ID
    pub owner_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// 所属看板 ID
    pub board_id: Uuid,
    /// 布局
    pub layout: ViewLayout,
    /// 过滤条件
    pub filters: ViewFilters,
    /// 显示密度
    pub density: ViewDensity,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 视图布局 (对应 Cmd+1/2/3/4 视图族)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewLayout {
    /// 看板视图 (Cmd+1)
    Board,
    /// 时间线视图 (Cmd+2)
    Timeline,
    /// 列表视图 (Cmd+3)
    List,
    /// 概览视图 (Cmd+4)
    Overview,
}

/// 视图过滤条件
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ViewFilters {
    /// 按经办人过滤
    pub assignee: Option<Uuid>,
    /// 按史诗过滤
    pub epic: Option<Uuid>,
    /// 按标签过滤
    pub label: Option<String>,
    /// 按优先级过滤
    pub priority: Option<String>,
    /// 按到期天数过滤
    pub due_within_days: Option<u32>,
}

/// 显示密度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewDensity {
    /// 紧凑 (默认: 14px 字体)
    Compact,
    /// 舒适 (16px 字体)
    Comfortable,
    /// 聚焦 (18px 字体)
    Focus,
}

impl SavedView {
    /// 构造 saved view
    pub fn new(
        name: impl Into<String>,
        owner_id: Uuid,
        tenant_id: Uuid,
        board_id: Uuid,
        layout: ViewLayout,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            owner_id,
            tenant_id,
            board_id,
            layout,
            filters: ViewFilters::default(),
            density: ViewDensity::Comfortable,
            created_at: now,
            updated_at: now,
        }
    }

    /// 视图对应的键盘快捷键
    pub fn shortcut(&self) -> &'static str {
        match self.layout {
            ViewLayout::Board => "Cmd+1",
            ViewLayout::Timeline => "Cmd+2",
            ViewLayout::List => "Cmd+3",
            ViewLayout::Overview => "Cmd+4",
        }
    }
}

// =====================================================================
// 4. BoardService (WIP + 泳道 + Saved View 聚合)
// =====================================================================

/// 看板服务 (WIP + 泳道 + Saved View 聚合)
pub struct BoardService;

impl BoardService {
    /// 构造服务
    pub fn new() -> Self {
        Self
    }

    /// 创建 WIP 限制
    pub fn create_wip_limit(column_id: impl Into<String>, max: u32) -> WipLimit {
        WipLimit {
            column_id: column_id.into(),
            max_items: max,
            current_count: 0,
        }
    }

    /// 创建泳道
    pub fn create_swimlane(group_by: SwimlaneGroupBy) -> Swimlane {
        Swimlane::new(group_by)
    }

    /// 创建 saved view
    pub fn create_saved_view(
        name: impl Into<String>,
        owner_id: Uuid,
        tenant_id: Uuid,
        board_id: Uuid,
        layout: ViewLayout,
    ) -> SavedView {
        SavedView::new(name, owner_id, tenant_id, board_id, layout)
    }
}

impl Default for BoardService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wip_allow_when_under() {
        let mut l = WipLimit {
            column_id: "todo".into(),
            max_items: 5,
            current_count: 3,
        };
        let a = WipGuard::add(&mut l);
        assert_eq!(a, WipAction::Allow);
        assert_eq!(l.current_count, 4);
    }

    #[test]
    fn test_wip_warn_at_limit() {
        let mut l = WipLimit {
            column_id: "doing".into(),
            max_items: 5,
            current_count: 4,
        };
        let a = WipGuard::add(&mut l);
        assert_eq!(a, WipAction::Warn);
    }

    #[test]
    fn test_wip_block_over_limit() {
        let mut l = WipLimit {
            column_id: "done".into(),
            max_items: 5,
            current_count: 5,
        };
        let a = WipGuard::add(&mut l);
        assert_eq!(a, WipAction::Block);
    }

    #[test]
    fn test_wip_remove_decrement() {
        let mut l = WipLimit {
            column_id: "todo".into(),
            max_items: 5,
            current_count: 3,
        };
        WipGuard::remove(&mut l);
        assert_eq!(l.current_count, 2);
        WipGuard::remove(&mut l);
        WipGuard::remove(&mut l);
        WipGuard::remove(&mut l); // 0 时不递减
        assert_eq!(l.current_count, 0);
    }

    #[test]
    fn test_swimlane_new() {
        let s = Swimlane::new(SwimlaneGroupBy::Assignee);
        assert_eq!(s.group_by, SwimlaneGroupBy::Assignee);
        assert!(!s.collapsed);
    }

    #[test]
    fn test_saved_view_shortcut() {
        let v = SavedView::new(
            "My Board",
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ViewLayout::Board,
        );
        assert_eq!(v.shortcut(), "Cmd+1");
        let v2 = SavedView::new(
            "Timeline",
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ViewLayout::Timeline,
        );
        assert_eq!(v2.shortcut(), "Cmd+2");
    }

    #[test]
    fn test_saved_view_default_filters() {
        let v = SavedView::new(
            "Test",
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ViewLayout::List,
        );
        assert!(v.filters.assignee.is_none());
        assert!(v.filters.epic.is_none());
    }
}
