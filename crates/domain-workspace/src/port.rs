//! Workspace 端口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::context::ActorContext;
use crate::entity::{Workspace, WorkspaceMember};
use crate::error::WorkspaceError;
use crate::value_object::{TenantId, UserId, WorkspaceId, WorkspaceRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceCommand {
    pub tenant_id: TenantId,
    pub workspace_key: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspaceCommand {
    pub workspace_id: WorkspaceId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemberCommand {
    pub workspace_id: WorkspaceId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role: WorkspaceRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveMemberCommand {
    pub workspace_id: WorkspaceId,
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorkspaceQuery {
    pub tenant_id: TenantId,
    pub limit: u32,
    pub offset: u32,
}

impl Default for ListWorkspaceQuery {
    fn default() -> Self {
        Self {
            tenant_id: TenantId::new(),
            limit: 50,
            offset: 0,
        }
    }
}

#[async_trait]
pub trait WorkspaceCommandPort: Send + Sync {
    async fn create_workspace(
        &self,
        cmd: CreateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    async fn update_workspace(
        &self,
        cmd: UpdateWorkspaceCommand,
        actor: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    async fn add_member(
        &self,
        cmd: AddMemberCommand,
        actor: ActorContext,
    ) -> Result<WorkspaceMember, WorkspaceError>;
    async fn remove_member(
        &self,
        cmd: RemoveMemberCommand,
        actor: ActorContext,
    ) -> Result<(), WorkspaceError>;
}

#[async_trait]
pub trait WorkspaceQueryPort: Send + Sync {
    async fn get_by_id(
        &self,
        id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    async fn get_by_key(
        &self,
        tenant_id: TenantId,
        workspace_key: &str,
        viewer: ActorContext,
    ) -> Result<Workspace, WorkspaceError>;
    async fn list_workspaces(
        &self,
        q: ListWorkspaceQuery,
        viewer: ActorContext,
    ) -> Result<Vec<Workspace>, WorkspaceError>;
    async fn list_members(
        &self,
        workspace_id: WorkspaceId,
        viewer: ActorContext,
    ) -> Result<Vec<WorkspaceMember>, WorkspaceError>;
}
