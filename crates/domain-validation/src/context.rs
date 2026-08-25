//! 调用方上下文(`ActorContext`)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, ProjectId, TenantId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub device_id: Option<uuid::Uuid>,
    pub roles: Vec<String>,
    pub project_ids: Vec<ProjectId>,
}

impl ActorContext {
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            roles: Vec::new(),
            project_ids: Vec::new(),
        }
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(roles::TENANT_ADMIN)
    }
    pub fn is_developer(&self) -> bool {
        self.has_role(roles::DEVELOPER)
    }
    pub fn is_service_internal(&self) -> bool {
        self.has_role(roles::SERVICE_INTERNAL)
    }
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_ids.push(project_id);
        self
    }
}
