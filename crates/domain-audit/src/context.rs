//! 调用方上下文(占位)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, TenantId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub user_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub device_id: Option<uuid::Uuid>,
    pub roles: Vec<String>,
}

impl ActorContext {
    pub fn new(user_id: uuid::Uuid, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            roles: Vec::new(),
        }
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    pub fn is_auditor(&self) -> bool {
        self.has_role(roles::TENANT_AUDITOR) || self.has_role(roles::TENANT_ADMIN)
    }
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
}
