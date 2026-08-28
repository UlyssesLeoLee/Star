//! Star Dashboard Engine (精简实装 v0.1)
//!
//! - 12-grid 布局 (Tailwind 标准)
//! - 10 Gadget 类型
//! - Wallboard 全屏模式
//! - 共享 / 权限
//! - 订阅 + 邮件
//!
//! Phase 2 接: react-grid-layout (前端) + 报告引擎 (后端)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// =====================================================================
// 1. value_object
// =====================================================================

/// Gadget 类型 (10 种)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GadgetType {
    IssueStats,       // 按 status/type/priority 统计
    Burndown,         // 接 domain-report
    Velocity,         // 接 domain-report
    MyWork,           // assigned to me
    RecentActivity,   // 接 domain-audit
    DueSoon,          // 按 due 排序
    JqlTable,         // 自定义 JQL 结果
    Markdown,         // 富文本
    Iframe,           // 嵌入 (Confluence/Notion)
    Clock,            // Sprint 倒计时
}

impl GadgetType {
    pub fn all() -> &'static [GadgetType] {
        &[
            Self::IssueStats, Self::Burndown, Self::Velocity, Self::MyWork,
            Self::RecentActivity, Self::DueSoon, Self::JqlTable,
            Self::Markdown, Self::Iframe, Self::Clock,
        ]
    }

    pub fn default_size(&self) -> GadgetSize {
        match self {
            Self::IssueStats | Self::Burndown | Self::Velocity => GadgetSize { w: 3, h: 2 },
            Self::MyWork | Self::RecentActivity | Self::DueSoon => GadgetSize { w: 2, h: 2 },
            Self::JqlTable => GadgetSize { w: 4, h: 2 },
            Self::Markdown | Self::Iframe => GadgetSize { w: 3, h: 1 },
            Self::Clock => GadgetSize { w: 1, h: 1 },
        }
    }
}

/// Gadget 尺寸 (12-grid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetSize {
    pub w: u8, // 1-12
    pub h: u8, // 1-4
}

/// Dashboard 共享作用域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DashboardScope {
    Personal, // 个人
    Team,     // 团队 (走 domain-permission)
    Project,  // 项目
    Global,   // 全公司 (admin only)
}

// =====================================================================
// 2. entity
// =====================================================================

/// Gadget 实例
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gadget {
    pub id: Uuid,
    pub gadget_type: GadgetType,
    pub title: String,
    pub position: GadgetPosition,
    pub size: GadgetSize,
    pub config: serde_json::Value, // 类型相关配置
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetPosition {
    pub x: u8, // 0-11
    pub y: u8, // 0-N
}

/// Dashboard 聚合根
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub scope: DashboardScope,
    pub owner_id: Uuid,
    pub tenant_id: Uuid,
    pub gadgets: Vec<Gadget>,
    pub wallboard_mode: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Dashboard {
    pub fn new(name: impl Into<String>, scope: DashboardScope, owner_id: Uuid, tenant_id: Uuid) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            scope,
            owner_id,
            tenant_id,
            gadgets: Vec::new(),
            wallboard_mode: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_gadget(&mut self, gadget: Gadget) -> Result<(), DashboardError> {
        if self.gadgets.len() >= 20 {
            return Err(DashboardError::TooManyGadgets(20));
        }
        self.gadgets.push(gadget);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn remove_gadget(&mut self, id: Uuid) -> Result<(), DashboardError> {
        let before = self.gadgets.len();
        self.gadgets.retain(|g| g.id != id);
        if self.gadgets.len() == before {
            return Err(DashboardError::GadgetNotFound(id));
        }
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    pub fn enable_wallboard(&mut self) {
        self.wallboard_mode = true;
        self.updated_at = chrono::Utc::now();
    }
}

// =====================================================================
// 3. error
// =====================================================================

#[derive(Debug, Error, Clone, PartialEq)]
pub enum DashboardError {
    #[error("too many gadgets: max {0}")]
    TooManyGadgets(usize),
    #[error("gadget not found: {0}")]
    GadgetNotFound(Uuid),
    #[error("invalid gadget position: x={0} (max 11), y={1}")]
    InvalidPosition(u8, u8),
    #[error("permission denied: actor {actor} cannot {action} {scope:?}")]
    PermissionDenied { actor: String, action: String, scope: DashboardScope },
}

// =====================================================================
// 4. service
// =====================================================================

pub struct DashboardService;

impl DashboardService {
    pub fn new() -> Self { Self }

    /// 创建仪表板 (按 scope 校验权限)
    pub fn create(
        &self,
        name: impl Into<String>,
        scope: DashboardScope,
        actor_id: Uuid,
        is_admin: bool,
        tenant_id: Uuid,
    ) -> Result<Dashboard, DashboardError> {
        match scope {
            DashboardScope::Global if !is_admin => {
                return Err(DashboardError::PermissionDenied {
                    actor: actor_id.to_string(),
                    action: "create".into(),
                    scope,
                });
            }
            _ => {}
        }
        Ok(Dashboard::new(name, scope, actor_id, tenant_id))
    }

    /// 添加 gadget (自动分配默认位置, 后续可拖拽)
    pub fn add_gadget_at(
        dashboard: &mut Dashboard,
        gadget_type: GadgetType,
        title: impl Into<String>,
    ) -> Result<&Gadget, DashboardError> {
        let size = gadget_type.default_size();
        // 找最底部 y=0 的位置
        let y = dashboard.gadgets.iter().map(|g| g.position.y).max().unwrap_or(0);
        let gadget = Gadget {
            id: Uuid::new_v4(),
            gadget_type,
            title: title.into(),
            position: GadgetPosition { x: 0, y: y + 1 },
            size,
            config: serde_json::json!({}),
        };
        dashboard.add_gadget(gadget)?;
        Ok(dashboard.gadgets.last().unwrap())
    }
}

impl Default for DashboardService {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gadget_type_all_count() {
        assert_eq!(GadgetType::all().len(), 10);
    }

    #[test]
    fn test_gadget_default_size() {
        assert_eq!(GadgetType::Clock.default_size(), GadgetSize { w: 1, h: 1 });
        assert_eq!(GadgetType::JqlTable.default_size(), GadgetSize { w: 4, h: 2 });
    }

    #[test]
    fn test_dashboard_create_personal() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let d = svc.create("My Dashboard", DashboardScope::Personal, actor, false, tenant).unwrap();
        assert_eq!(d.name, "My Dashboard");
        assert_eq!(d.scope, DashboardScope::Personal);
    }

    #[test]
    fn test_dashboard_global_requires_admin() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let r = svc.create("Company Dashboard", DashboardScope::Global, actor, false, tenant);
        assert!(matches!(r, Err(DashboardError::PermissionDenied { .. })));
        let r2 = svc.create("Company Dashboard", DashboardScope::Global, actor, true, tenant);
        assert!(r2.is_ok());
    }

    #[test]
    fn test_add_gadget() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let mut d = svc.create("Test", DashboardScope::Personal, actor, false, tenant).unwrap();
        DashboardService::add_gadget_at(&mut d, GadgetType::Burndown, "Sprint Burndown").unwrap();
        assert_eq!(d.gadgets.len(), 1);
        assert_eq!(d.gadgets[0].gadget_type, GadgetType::Burndown);
    }

    #[test]
    fn test_remove_gadget() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let mut d = svc.create("Test", DashboardScope::Personal, actor, false, tenant).unwrap();
        DashboardService::add_gadget_at(&mut d, GadgetType::Clock, "Sprint Clock").unwrap();
        let id = d.gadgets[0].id;
        d.remove_gadget(id).unwrap();
        assert_eq!(d.gadgets.len(), 0);
    }

    #[test]
    fn test_wallboard_mode() {
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let mut d = Dashboard::new("TV", DashboardScope::Team, actor, tenant);
        assert!(!d.wallboard_mode);
        d.enable_wallboard();
        assert!(d.wallboard_mode);
    }
}
