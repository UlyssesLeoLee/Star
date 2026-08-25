//! 调用方上下文(`ActorContext`)
//!
//! **Phase 2 现状**:本 crate 自带最小 `ActorContext`(本 crate 内调用方使用)。
//! **Phase 3 计划**:由 `domain-identity` 颁发统一 `ActorContext` 取代本 crate 内定义,
//! 以避免 `domain-*` 之间的循环依赖。

use serde::{Deserialize, Serialize};

use crate::value_object::{roles, ProjectId, TenantId, UserId};

/// **Actor 上下文**(命令/查询端口的 `actor` / `viewer` 参数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID(强类型)
    pub user_id: UserId,
    /// 当前租户 ID
    pub tenant_id: TenantId,
    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<uuid::Uuid>,
    /// 当前用户角色
    pub roles: Vec<String>,
    /// 当前 Project IDs(用于 Project Policy 校验)
    pub project_ids: Vec<ProjectId>,
}

impl ActorContext {
    /// 构造一个新的 `ActorContext`(测试 / 默认场景便捷构造)。
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

    /// 添加角色(测试 / 构造场景使用)。
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// 添加 Project(测试 / 构造场景使用)。
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_ids.push(project_id);
        self
    }

    /// 当前用户是否可访问指定 Project。
    pub fn can_access_project(&self, project_id: ProjectId) -> bool {
        self.is_tenant_admin() || self.project_ids.contains(&project_id)
    }
}
