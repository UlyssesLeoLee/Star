//! Planning 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID
// =====================================================================

define_uuid_id!(SprintId);
define_uuid_id!(BacklogId);
define_uuid_id!(RoadmapId);
define_uuid_id!(MilestoneId);
define_uuid_id!(BurndownSnapshotId);
define_uuid_id!(ProjectId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 枚举:SprintState(状态机)
// =====================================================================

/// **Sprint 状态**(`planning.sprint.state` 列,CHECK 约束白名单)
///
/// 来源: docs/data-design.md §4.7 (Sprint 状态机) / spec §3 INV-PL-01
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SprintState {
    /// 规划中(尚未开始)
    Planning,
    /// 活跃(已启动)
    Active,
    /// 已关闭(不可再变更)
    Closed,
}

impl Default for SprintState {
    fn default() -> Self {
        Self::Planning
    }
}

impl std::fmt::Display for SprintState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Planning => "PLANNING",
            Self::Active => "ACTIVE",
            Self::Closed => "CLOSED",
        };
        f.write_str(s)
    }
}

impl SprintState {
    /// 是否允许迁移到目标状态(INV-PL-01)
    pub fn can_transition_to(self, target: Self) -> bool {
        use SprintState::*;
        if self == target {
            return true; // 幂等
        }
        match (self, target) {
            (Planning, Active) => true,   // start_sprint
            (Active, Closed) => true,     // close_sprint
            _ => false,                   // 不可逆
        }
    }
}

// =====================================================================
// 枚举:CloseMoveTarget(关闭 Sprint 时未完成 WorkItem 去向)
// =====================================================================

/// **Sprint 关闭时未完成 WorkItem 移动目标**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloseMoveTarget {
    /// 移回 Backlog
    Backlog,
    /// 移至下一个 Sprint
    NextSprint,
}

impl Default for CloseMoveTarget {
    fn default() -> Self {
        Self::Backlog
    }
}

// =====================================================================
// 标准角色
// =====================================================================

pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
    pub const VIEWER: &str = "viewer";
}
