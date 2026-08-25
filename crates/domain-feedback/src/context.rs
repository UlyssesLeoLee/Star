//! 调用方上下文(`ActorContext`)

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, ProjectId, TenantId, UserId};

/// **Actor 调用上下文**(INV-FB-06 必带 tenant_id,跨 tenant 拒绝)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID
    pub user_id: UserId,
    /// 当前租户 ID(13 类对象必带,§6.1,REQ-SEC-001)
    pub tenant_id: TenantId,
    /// 当前设备 ID
    pub device_id: Option<uuid::Uuid>,
    /// 当前 Project IDs(用于 Project Policy 校验)
    pub project_ids: Vec<ProjectId>,
    /// 当前用户角色
    pub roles: Vec<String>,
    /// 当前用户是否为 AI Agent 触发的会话(INV-FB-07)
    pub is_agent_session: bool,
}

impl ActorContext {
    /// 创建 ActorContext(默认非 AI)
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            project_ids: Vec::new(),
            roles: Vec::new(),
            is_agent_session: false,
        }
    }

    /// 是否包含指定 role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 是否 tenant admin
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(roles::TENANT_ADMIN)
    }

    /// 链式添加 role
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// 链式添加 project
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_ids.push(project_id);
        self
    }

    /// 链式标记为 AI Agent 会话(INV-FB-07)
    pub fn with_agent_session(mut self, is_agent: bool) -> Self {
        self.is_agent_session = is_agent;
        self
    }
}
