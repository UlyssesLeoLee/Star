//! 调用方上下文(`ActorContext`)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, ProjectId, TenantId, UserId};

/// **Actor 上下文**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID
    pub user_id: UserId,
    /// 当前租户 ID
    pub tenant_id: TenantId,
    /// 当前设备 ID
    pub device_id: Option<uuid::Uuid>,
    /// 当前用户角色
    pub roles: Vec<String>,
    /// 当前 Project IDs
    pub project_ids: Vec<ProjectId>,
}

impl ActorContext {
    /// 构造一个新的 `ActorContext`。
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            roles: Vec::new(),
            project_ids: Vec::new(),
        }
    }

    /// 是否具备指定角色。
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 是否为租户管理员。
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(roles::TENANT_ADMIN)
    }

    /// 添加角色
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// 添加 Project
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_ids.push(project_id);
        self
    }
}
