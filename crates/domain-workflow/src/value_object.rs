//! Workflow 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.5 (`workflow` schema)
//! - `docs/specs/domain-workflow-spec.md` §2 (实体清单) / §3 (基本类型)
//!
//! 集中放置强类型 ID、State 类别、枚举等。
//!
//! **system_default 三态**(基本设计 §7.2,REQ-WF-001):
//! - `TODO` (Initial)
//! - `IN_PROGRESS` (Intermediate)
//! - `DONE` (Terminal)
//! 任何自定义 Workflow 必须包含这三个 State(INV-WF-06)。

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID(UUID newtype)
// =====================================================================

define_uuid_id!(WorkflowId);
define_uuid_id!(StateId);
define_uuid_id!(TransitionId);

/// 标准 Project ID(本 crate 引用,跨域 ID 不再依赖 domain-project,
/// 因为本阶段所有 domain-* 严格无跨 crate 依赖,采用强类型 ID newtype)
define_uuid_id!(ProjectId);

/// 强类型 Tenant ID(避免依赖 domain-tenant)
define_uuid_id!(TenantId);

/// 强类型 User ID
define_uuid_id!(UserId);

// =====================================================================
// 枚举:StateCategory
// =====================================================================

/// **State 类别**(`workflow.state.category` 列)
///
/// 来源: docs/data-design.md §4.5 (`ck_state_category`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StateCategory {
    /// 初始状态(Workflow 入口,唯一)
    Initial,
    /// 中间状态(可多次往返)
    Intermediate,
    /// 终态(WorkItem 终止,不可再迁移)
    Terminal,
}

impl Default for StateCategory {
    fn default() -> Self {
        Self::Intermediate
    }
}

impl std::fmt::Display for StateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Initial => "INITIAL",
            Self::Intermediate => "INTERMEDIATE",
            Self::Terminal => "TERMINAL",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 标准角色
// =====================================================================

/// Workflow 相关标准角色常量
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 平台运营
    pub const PLATFORM_OPERATOR: &str = "platform_operator";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读观察者
    pub const VIEWER: &str = "viewer";
}
