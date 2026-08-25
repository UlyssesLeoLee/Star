//! 调用方上下文(占位)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, ProjectId, TenantId, WorkspaceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub user_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub device_id: Option<uuid::Uuid>,
    pub project_ids: Vec<ProjectId>,
    pub workspace_ids: Vec<WorkspaceId>,
    pub roles: Vec<String>,
}

impl ActorContext {
    pub fn new(user_id: uuid::Uuid, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            project_ids: Vec::new(),
            workspace_ids: Vec::new(),
            roles: Vec::new(),
        }
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    pub fn is_project_admin(&self) -> bool {
        self.has_role(roles::PROJECT_ADMIN)
    }
    pub fn is_member_of(&self, project_id: ProjectId) -> bool {
        self.is_project_admin() || self.project_ids.contains(&project_id)
    }
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        if !self.project_ids.contains(&project_id) {
            self.project_ids.push(project_id);
        }
        self
    }
}
