//! 调用方上下文(占位)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, TenantId, WorkspaceId};

/// **Actor 上下文**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 用户 ID
    pub user_id: uuid::Uuid,
    /// 租户 ID
    pub tenant_id: TenantId,
    /// 设备 ID
    pub device_id: Option<uuid::Uuid>,
    /// 角色字符串
    pub roles: Vec<String>,
    /// 成员 Workspace IDs
    pub workspace_ids: Vec<WorkspaceId>,
}

impl ActorContext {
    pub fn new(user_id: uuid::Uuid, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            roles: Vec::new(),
            workspace_ids: Vec::new(),
        }
    }
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
    pub fn is_workspace_admin(&self) -> bool {
        self.has_role(roles::WORKSPACE_ADMIN)
    }
    pub fn is_member_of(&self, ws: WorkspaceId) -> bool {
        self.workspace_ids.contains(&ws)
    }
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
    pub fn with_workspace(mut self, ws: WorkspaceId) -> Self {
        if !self.workspace_ids.contains(&ws) {
            self.workspace_ids.push(ws);
        }
        self
    }
}
