//! Project 域实体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::value_object::{
    AgentPolicyId, NotificationSchemeId, PermissionSchemeId, ProjectId, ProjectPolicyId,
    ProjectStatus, ProjectTemplateId, ProjectTemplateType, TenantId, WorkflowId, WorkspaceId,
};

/// **Project 聚合根**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub template_type: ProjectTemplateType,
    pub status: ProjectStatus,
    pub workflow_id: Option<WorkflowId>,
    pub permission_scheme_id: Option<PermissionSchemeId>,
    pub notification_scheme_id: Option<NotificationSchemeId>,
    pub agent_policy_id: Option<AgentPolicyId>,
    pub lead_user_id: Option<uuid::Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl Project {
    pub const FIELD_COUNT: usize = 16;
    pub fn is_active(&self) -> bool {
        self.status == ProjectStatus::Active
    }
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}

/// **ProjectTemplate**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: ProjectTemplateId,
    pub tenant_id: TenantId,
    pub name: String,
    pub template_type: ProjectTemplateType,
    pub default_settings: serde_json::Value,
    pub built_in: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl ProjectTemplate {
    pub const FIELD_COUNT: usize = 9;
}

/// **ProjectPolicy**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPolicy {
    pub id: ProjectPolicyId,
    pub project_id: ProjectId,
    pub agent_policy: serde_json::Value,
    pub worktree_policy: serde_json::Value,
    pub validation_policy: serde_json::Value,
    pub context_policy: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

impl ProjectPolicy {
    pub const FIELD_COUNT: usize = 9;
    pub fn bump_version(&mut self) {
        self.version = self.version.saturating_add(1);
        self.updated_at = Utc::now();
    }
}
