//! Permission 域值对象

use serde::{Deserialize, Serialize};

use crate::define_uuid_id;

define_uuid_id!(RoleId);
define_uuid_id!(PermissionId);
define_uuid_id!(PermissionSchemeId);
define_uuid_id!(TenantId);
define_uuid_id!(ProjectId);

/// **Permission 作用域**(`permission.scope`)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionScope {
    /// Project 内
    Project,
    /// Workspace 内
    Workspace,
    /// Tenant 全局
    Tenant,
}

impl Default for PermissionScope {
    fn default() -> Self {
        Self::Project
    }
}

impl std::fmt::Display for PermissionScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Project => "PROJECT",
            Self::Workspace => "WORKSPACE",
            Self::Tenant => "TENANT",
        };
        f.write_str(s)
    }
}

/// **权限码标准常量**(常用权限的全局字符串)
pub mod perm_codes {
    // WorkItem
    pub const WORKITEM_READ: &str = "workitem:read";
    pub const WORKITEM_CREATE: &str = "workitem:create";
    pub const WORKITEM_UPDATE: &str = "workitem:update";
    pub const WORKITEM_DELETE: &str = "workitem:delete";
    // Worktree
    pub const WORKTREE_READ: &str = "worktree:read";
    pub const WORKTREE_CREATE: &str = "worktree:create";
    pub const WORKTREE_MERGE: &str = "worktree:merge";
    // Project
    pub const PROJECT_ADMIN: &str = "project:admin";
    // Tenant
    pub const TENANT_ADMIN: &str = "tenant:admin";
}

pub mod roles {
    pub const TENANT_ADMIN: &str = "tenant_admin";
    pub const PROJECT_ADMIN: &str = "project_admin";
}
