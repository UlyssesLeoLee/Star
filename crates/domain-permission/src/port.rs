//! Permission 端口

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::context::ActorContext;
use crate::entity::{Permission, PermissionScheme, Role};
use crate::error::PermissionError;
use crate::value_object::{PermissionSchemeId, ProjectId, RoleId, TenantId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleCommand {
    pub tenant_id: TenantId,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleCommand {
    pub role_id: RoleId,
    pub tenant_id: TenantId,
    pub expected_version: u32,
    pub permissions: Option<Vec<String>>,
    pub description: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePermissionSchemeCommand {
    pub project_id: ProjectId,
    pub tenant_id: TenantId,
    pub name: String,
    pub default_role: String,
    pub role_permissions: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckPermissionQuery {
    pub role_id: RoleId,
    pub permission: String,
}

#[async_trait]
pub trait PermissionCommandPort: Send + Sync {
    async fn create_role(
        &self,
        cmd: CreateRoleCommand,
        actor: ActorContext,
    ) -> Result<Role, PermissionError>;
    async fn update_role(
        &self,
        cmd: UpdateRoleCommand,
        actor: ActorContext,
    ) -> Result<Role, PermissionError>;
    async fn delete_role(
        &self,
        role_id: RoleId,
        actor: ActorContext,
    ) -> Result<(), PermissionError>;
    async fn create_scheme(
        &self,
        cmd: CreatePermissionSchemeCommand,
        actor: ActorContext,
    ) -> Result<PermissionScheme, PermissionError>;
}

#[async_trait]
pub trait PermissionQueryPort: Send + Sync {
    async fn get_role(
        &self,
        id: RoleId,
        viewer: ActorContext,
    ) -> Result<Role, PermissionError>;
    async fn list_roles(
        &self,
        tenant_id: TenantId,
        viewer: ActorContext,
    ) -> Result<Vec<Role>, PermissionError>;
    async fn get_scheme(
        &self,
        id: PermissionSchemeId,
        viewer: ActorContext,
    ) -> Result<PermissionScheme, PermissionError>;
    async fn get_scheme_by_project(
        &self,
        project_id: ProjectId,
        viewer: ActorContext,
    ) -> Result<PermissionScheme, PermissionError>;
    /// 检查角色是否具备权限
    async fn check_permission(
        &self,
        q: CheckPermissionQuery,
        viewer: ActorContext,
    ) -> Result<bool, PermissionError>;
    async fn list_permissions(&self) -> Result<Vec<Permission>, PermissionError>;
}
