//! Project 端口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Project, ProjectPolicy, ProjectTemplate};
use crate::error::ProjectError;
use crate::value_object::{
    ProjectId, ProjectTemplateType, TenantId, WorkspaceId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectCommand {
    pub tenant_id: TenantId,
    pub workspace_id: WorkspaceId,
    pub project_key: String,
    pub name: String,
    pub description: Option<String>,
    pub template_type: ProjectTemplateType,
    pub lead_user_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectCommand {
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub lead_user_id: Option<Option<uuid::Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveProjectCommand {
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectPolicyCommand {
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub agent_policy: Option<serde_json::Value>,
    pub worktree_policy: Option<serde_json::Value>,
    pub validation_policy: Option<serde_json::Value>,
    pub context_policy: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProjectQuery {
    pub tenant_id: TenantId,
    pub workspace_id: Option<WorkspaceId>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListProjectQuery {
    fn default() -> Self {
        Self {
            tenant_id: UserId.new(),
            workspace_id: None,
            limit: 50,
            offset: 0,
        }
    }
}

#[async_trait]
pub trait ProjectCommandPort: Send + Sync {
    async fn create_project(
        &self,
        cmd: CreateProjectCommand,
        actor: ActorContext,
    ) -> Result<Project, ProjectError>;
    async fn update_project(
        &self,
        cmd: UpdateProjectCommand,
        actor: ActorContext,
    ) -> Result<Project, ProjectError>;
    async fn archive_project(
        &self,
        cmd: ArchiveProjectCommand,
        actor: ActorContext,
    ) -> Result<Project, ProjectError>;
    async fn update_project_policy(
        &self,
        cmd: UpdateProjectPolicyCommand,
        actor: ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;
}

#[async_trait]
pub trait ProjectQueryPort: Send + Sync {
    async fn get_by_id(
        &self,
        id: ProjectId,
        viewer: ActorContext,
    ) -> Result<Project, ProjectError>;
    async fn get_by_key(
        &self,
        tenant_id: TenantId,
        workspace_id: WorkspaceId,
        project_key: &str,
        viewer: ActorContext,
    ) -> Result<Project, ProjectError>;
    async fn list_projects(
        &self,
        q: ListProjectQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Project>, ProjectError>;
    async fn list_templates(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<Vec<ProjectTemplate>, ProjectError>;
    async fn get_project_policy(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<ProjectPolicy, ProjectError>;
}
