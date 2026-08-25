//! Relation 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(RelationId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);
define_uuid_id!(WorkItemId);

// =====================================================================
// 枚举:RelationType(spec §2, INV-R-06)
// =====================================================================

/// **Relation 类型**
///
/// 来源: docs/data-design.md §4.8 / spec §2
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationType {
    /// A blocks B(A 阻塞 B)
    Blocks,
    /// A blocked_by B(A 被 B 阻塞)
    BlockedBy,
    /// A relates_to B(A 与 B 关联)
    RelatesTo,
    /// A duplicates B(A 重复 B)
    Duplicates,
    /// A clones B(A 克隆 B)
    Clones,
}

impl Default for RelationType {
    fn default() -> Self {
        Self::RelatesTo
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Blocks => "BLOCKS",
            Self::BlockedBy => "BLOCKED_BY",
            Self::RelatesTo => "RELATES_TO",
            Self::Duplicates => "DUPLICATES",
            Self::Clones => "CLONES",
        };
        f.write_str(s)
    }
}

// =====================================================================
// 角色
// =====================================================================

pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const PROJECT_ADMIN: &str = "project_admin";
    pub const DEVELOPER: &str = "developer";
    pub const VIEWER: &str = "viewer";
}
