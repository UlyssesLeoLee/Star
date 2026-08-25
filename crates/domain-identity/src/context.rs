//! 调用方上下文(`ActorContext`)
//!
//! **Phase 2 现状**:本 crate 自带 `ActorContext` 的"完整版"实现(5 字段),
//! 因为本 crate 是其他 5 个横切 crate 的"颁发源"(Phase 3 整合时统一引用本 crate)。
//! **Phase 2 限制**:为避免跨 crate 依赖,本 crate 仍使用本 crate 内的占位 ID
//! (UUID 而非 domain-tenant 颁发的 TenantId),与 §23.2 一致。

use serde::{Deserialize, Serialize};

use crate::value_object::{DeviceId, ProjectId, RoleId, TenantId, UserId, roles};

/// **Actor 上下文**(命令/查询端口的 `actor` / `viewer` 参数)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    /// 当前用户 ID
    pub user_id: UserId,
    /// 当前租户 ID
    pub tenant_id: TenantId,
    /// 当前设备 ID(Local Runtime 三重绑定,§23.2)
    pub device_id: Option<DeviceId>,
    /// 当前用户已加入的 Project IDs
    pub project_ids: Vec<ProjectId>,
    /// 当前用户角色 IDs(关联 `Role.id`)
    pub role_ids: Vec<RoleId>,
    /// 当前用户角色字符串(`tenant_admin` / `developer` 等,冗余便于快速判断)
    pub roles: Vec<String>,
}

impl ActorContext {
    /// 构造一个新的 `ActorContext`
    pub fn new(user_id: UserId, tenant_id: TenantId) -> Self {
        Self {
            user_id,
            tenant_id,
            device_id: None,
            project_ids: Vec::new(),
            role_ids: Vec::new(),
            roles: Vec::new(),
        }
    }

    /// 是否具备指定角色字符串
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// 是否具备指定角色 ID
    pub fn has_role_id(&self, role_id: RoleId) -> bool {
        self.role_ids.contains(&role_id)
    }

    /// 是否为租户管理员
    pub fn is_tenant_admin(&self) -> bool {
        self.has_role(roles::TENANT_ADMIN)
    }

    /// 是否为服务账户
    pub fn is_service_account(&self) -> bool {
        self.has_role(roles::SERVICE_ACCOUNT)
    }

    /// 是否为 `project_id` 的成员
    pub fn is_member_of(&self, project_id: ProjectId) -> bool {
        self.is_tenant_admin() || self.project_ids.contains(&project_id)
    }

    /// 添加角色字符串
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// 添加角色 ID
    pub fn with_role_id(mut self, role_id: RoleId) -> Self {
        self.role_ids.push(role_id);
        self
    }

    /// 添加 Project 成员资格
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        if !self.project_ids.contains(&project_id) {
            self.project_ids.push(project_id);
        }
        self
    }

    /// 绑定设备
    pub fn with_device(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }
}
