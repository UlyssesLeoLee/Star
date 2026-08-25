//! Permission 域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    PermissionId, PermissionSchemeId, PermissionScope, ProjectId, RoleId, TenantId,
};

/// **Role**(租户内角色)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: RoleId,
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub built_in: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl Role {
    pub const FIELD_COUNT: usize = 10;
    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| p == perm)
    }
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

/// **Permission**(权限码)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: PermissionId,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub scope: PermissionScope,
    pub created_at: DateTime<Utc>,
}

impl Permission {
    pub const FIELD_COUNT: usize = 6;
}

/// **PermissionScheme**(Project 内角色 → 权限映射)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScheme {
    pub id: PermissionSchemeId,
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub name: String,
    /// role_name -> permissions 的映射
    pub role_permissions: std::collections::HashMap<String, Vec<String>>,
    pub default_role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl PermissionScheme {
    pub const FIELD_COUNT: usize = 9;
    pub fn grants(&self, role: &str, perm: &str) -> bool {
        self.role_permissions
            .get(role)
            .map_or(false, |perms| perms.iter().any(|p| p == perm))
    }
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}
