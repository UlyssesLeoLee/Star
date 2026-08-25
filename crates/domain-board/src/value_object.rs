//! Board 域值对象(Value Objects)
//!
//! 来源:
//! - `docs/data-design.md` §4.6 (`board` schema)
//! - `docs/specs/domain-board-spec.md` §2 (实体清单) / §3 (基本类型)

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

// =====================================================================
// 强类型 ID
// =====================================================================

define_uuid_id!(BoardId);
define_uuid_id!(ColumnId);
define_uuid_id!(SwimlaneId);
define_uuid_id!(ProjectId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);

/// 引用外部域的 State ID(用于 Column.state_id 引用;不强类型依赖 domain-workflow)
define_uuid_id!(StateId);

// =====================================================================
// 枚举:BoardType
// =====================================================================

/// **Board 类型**
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BoardType {
    /// Kanban 看板
    Kanban,
    /// Scrum 板(Sprint 关联)
    Scrum,
}

impl Default for BoardType {
    fn default() -> Self {
        Self::Kanban
    }
}

impl std::fmt::Display for BoardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Kanban => "KANBAN",
            Self::Scrum => "SCRUM",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 枚举:GroupByField(Swimlane group_by 字段)
// =====================================================================

/// **Swimlane group_by 字段**(`swimlane.group_by_field` 列)
///
/// 来源: docs/specs/domain-board-spec.md §2 (Swimlane 实体) / §8 (B-004 错误码)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GroupByField {
    /// 按分配人分组
    Assignee,
    /// 按标签分组
    Label,
    /// 按 Epic 分组
    Epic,
}

impl Default for GroupByField {
    fn default() -> Self {
        Self::Assignee
    }
}

impl std::fmt::Display for GroupByField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Assignee => "ASSIGNEE",
            Self::Label => "LABEL",
            Self::Epic => "EPIC",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 标准角色
// =====================================================================

/// Board 相关标准角色常量
pub mod roles {
    /// 租户管理员
    pub const TENANT_ADMIN: &str = "tenant_admin";
    /// 项目管理员
    pub const PROJECT_ADMIN: &str = "project_admin";
    /// 开发者
    pub const DEVELOPER: &str = "developer";
    /// 只读观察者
    pub const VIEWER: &str = "viewer";
}
