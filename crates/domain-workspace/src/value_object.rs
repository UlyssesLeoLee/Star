//! Workspace 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(WorkspaceId);
define_uuid_id!(WorkspaceMemberId);
define_uuid_id!(TenantId);
define_uuid_id!(UserId);
define_uuid_id!(ProjectId);

/// **Workspace 成员角色**(`workspace_member.role` 列)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkspaceRole {
    /// Workspace 管理员
    Admin,
    /// 成员
    Member,
    /// 访客(只读)
    Guest,
}

impl Default for WorkspaceRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Admin => "ADMIN",
            Self::Member => "MEMBER",
            Self::Guest => "GUEST",
        };
        f.write_str(s)
    }
}

pub mod roles {
    pub const WORKSPACE_ADMIN: &str = "workspace_admin";
    pub const WORKSPACE_MEMBER: &str = "workspace_member";
}
