//! 调用方上下文(占位)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, RoleId, TenantId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub user_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub device_id: Option<uuid::Uuid>,
    pub role_ids: Vec<RoleId>,
    pub roles: Vec<String>,
}

impl ActorContext {
    pub fn new(user_id: uuid::Uuid, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            role_ids: Vec::new(),
            roles: Vec::new(),
        }
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(roles::TENANT_ADMIN)
    }
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
    pub fn with_role_id(mut self, rid: RoleId) -> Self {
        self.role_ids.push(rid);
        self
    }
}
