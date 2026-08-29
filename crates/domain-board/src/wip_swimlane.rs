//! Star Board — WIP 限制 + 泳道 + Saved View (wt-w9-wip 扩展)
//!
//! - WIP 限制: 列满时拒绝 transition + 告警
//! - 泳道: 按 assignee / epic / label / priority 分组
//! - Saved View: 用户保存的视图 (复用 ui-3pane-arch.md §1.3 Cmd+1/2/3/4 视图族)

#![warn(missing_docs)]
#![warn(rust_2018_idiorms)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. WIP 限制
// =====================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WipLimit {
    pub column_id: String,
    pub max_items: u32,
    pub current_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WipAction {
    Allow,
    Warn,
    Block,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwimlaneGroupBy {
    Assignee,
    Epic,
    Label,
    Priority,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Swimlane {
    pub group_by: SwimlaneGroupBy,
    pub custom_field: Option<String>, // for Custom
    pub collapsed: bool,
}

impl Swimlane {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedView {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub tenant_id: Uuid,
    pub board_id: Uuid,
    pub layout: ViewLayout,
    pub filters: ViewFilters,
    pub density: ViewDensity,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewLayout {
    Board,    // Cmd+1
    Timeline, // Cmd+2
    List,     // Cmd+3
    Overview, // Cmd+4
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ViewFilters {
    pub assignee: Option<Uuid>,
    pub epic: Option<Uuid>,
    pub label: Option<String>,
    pub priority: Option<String>,
    pub due_within_days: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewDensity {
    Compact,     // 默认: 14px 字体
    Comfortable, // 16px 字体
    Focus,       // 18px 字体
}

impl SavedView {
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

pub struct BoardService;

impl BoardService {
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
