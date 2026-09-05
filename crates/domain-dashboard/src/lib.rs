//! Star Dashboard Engine (精简实装 v0.1)
//!
//! - 12-grid 布局 (Tailwind 标准)
//! - 10 Gadget 类型
//! - Wallboard 全屏模式
//! - 共享 / 权限
//! - 订阅 + 邮件
//!
//! Phase 2 接: react-grid-layout (前端) + 报告引擎 (后端)

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
    /// 按 status/type/priority 统计
    IssueStats,
    /// 接 domain-report
    Burndown,
    /// 接 domain-report
    Velocity,
    /// assigned to me
    MyWork,
    /// 接 domain-audit
    RecentActivity,
    /// 按 due 排序
    DueSoon,
    /// 自定义 JQL 结果
    JqlTable,
    /// 富文本
    Markdown,
    /// 嵌入 (Confluence/Notion)
    Iframe,
    /// Sprint 倒计时
    Clock,
}

impl GadgetType {
    /// 全部 Gadget 类型
    pub fn all() -> &'static [GadgetType] {
        &[
            Self::IssueStats,
            Self::Burndown,
            Self::Velocity,
            Self::MyWork,
            Self::RecentActivity,
            Self::DueSoon,
            Self::JqlTable,
            Self::Markdown,
            Self::Iframe,
            Self::Clock,
        ]
    }

    /// 该类型的默认尺寸
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
    /// 宽度 (1-12)
    pub w: u8,
    /// 高度 (1-4)
    pub h: u8,
}

/// Dashboard 共享作用域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DashboardScope {
    /// 个人
    Personal,
    /// 团队 (走 domain-permission)
    Team,
    /// 项目
    Project,
    /// 全公司 (admin only)
    Global,
}

// =====================================================================
// 2. entity
// =====================================================================

/// Gadget 实例
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Gadget {
    /// Gadget ID
    pub id: Uuid,
    /// Gadget 类型
    pub gadget_type: GadgetType,
    /// 标题
    pub title: String,
    /// 网格位置
    pub position: GadgetPosition,
    /// 尺寸
    pub size: GadgetSize,
    /// 类型相关配置
    pub config: serde_json::Value,
}

/// Gadget 网格位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GadgetPosition {
    /// X 坐标 (0-11)
    pub x: u8,
    /// Y 坐标 (0-N)
    pub y: u8,
}

/// Dashboard 聚合根
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dashboard {
    /// Dashboard ID
    pub id: Uuid,
    /// 名称
    pub name: String,
    /// 共享作用域
    pub scope: DashboardScope,
    /// 所有者用户 ID
    pub owner_id: Uuid,
    /// 租户 ID
    pub tenant_id: Uuid,
    /// Gadget 列表
    pub gadgets: Vec<Gadget>,
    /// 是否 wallboard 全屏模式
    pub wallboard_mode: bool,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Dashboard {
    /// 构造 Dashboard
    pub fn new(
        name: impl Into<String>,
        scope: DashboardScope,
        owner_id: Uuid,
        tenant_id: Uuid,
    ) -> Self {
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

    /// 添加 gadget (上限 20)
    pub fn add_gadget(&mut self, gadget: Gadget) -> Result<(), DashboardError> {
        if self.gadgets.len() >= 20 {
            return Err(DashboardError::TooManyGadgets(20));
        }
        self.gadgets.push(gadget);
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// 移除 gadget
    pub fn remove_gadget(&mut self, id: Uuid) -> Result<(), DashboardError> {
        let before = self.gadgets.len();
        self.gadgets.retain(|g| g.id != id);
        if self.gadgets.len() == before {
            return Err(DashboardError::GadgetNotFound(id));
        }
        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// 开启 wallboard 全屏模式
    pub fn enable_wallboard(&mut self) {
        self.wallboard_mode = true;
        self.updated_at = chrono::Utc::now();
    }
}

// =====================================================================
// 3. error
// =====================================================================

/// Dashboard 领域错误
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DashboardError {
    /// Gadget 数量超限
    #[error("too many gadgets: max {0}")]
    TooManyGadgets(usize),
    /// Gadget 未找到
    #[error("gadget not found: {0}")]
    GadgetNotFound(Uuid),
    /// 非法网格位置
    #[error("invalid gadget position: x={0} (max 11), y={1}")]
    InvalidPosition(u8, u8),
    /// 权限不足
    #[error("permission denied: actor {actor} cannot {action} {scope:?}")]
    PermissionDenied {
        /// 操作者
        actor: String,
        /// 尝试的操作
        action: String,
        /// 目标作用域
        scope: DashboardScope,
    },
}

// =====================================================================
// 4. service
// =====================================================================

/// Dashboard 服务
pub struct DashboardService;

impl DashboardService {
    /// 构造服务
    pub fn new() -> Self {
        Self
    }

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
        let y = dashboard
            .gadgets
            .iter()
            .map(|g| g.position.y)
            .max()
            .unwrap_or(0);
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
    fn default() -> Self {
        Self::new()
    }
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
        assert_eq!(
            GadgetType::JqlTable.default_size(),
            GadgetSize { w: 4, h: 2 }
        );
    }

    #[test]
    fn test_dashboard_create_personal() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let d = svc
            .create(
                "My Dashboard",
                DashboardScope::Personal,
                actor,
                false,
                tenant,
            )
            .unwrap();
        assert_eq!(d.name, "My Dashboard");
        assert_eq!(d.scope, DashboardScope::Personal);
    }

    #[test]
    fn test_dashboard_global_requires_admin() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let r = svc.create(
            "Company Dashboard",
            DashboardScope::Global,
            actor,
            false,
            tenant,
        );
        assert!(matches!(r, Err(DashboardError::PermissionDenied { .. })));
        let r2 = svc.create(
            "Company Dashboard",
            DashboardScope::Global,
            actor,
            true,
            tenant,
        );
        assert!(r2.is_ok());
    }

    #[test]
    fn test_add_gadget() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let mut d = svc
            .create("Test", DashboardScope::Personal, actor, false, tenant)
            .unwrap();
        DashboardService::add_gadget_at(&mut d, GadgetType::Burndown, "Sprint Burndown").unwrap();
        assert_eq!(d.gadgets.len(), 1);
        assert_eq!(d.gadgets[0].gadget_type, GadgetType::Burndown);
    }

    #[test]
    fn test_remove_gadget() {
        let svc = DashboardService::new();
        let actor = Uuid::new_v4();
        let tenant = Uuid::new_v4();
        let mut d = svc
            .create("Test", DashboardScope::Personal, actor, false, tenant)
            .unwrap();
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
